import {
  CTX_PLUGIN_MANIFEST_SCHEMA_VERSION,
  type PluginDefinition,
  type PluginDefinitionDiagnostic,
  type PluginValidationResult,
} from "./types.js";

type ObjectRecord = Record<string, unknown>;

const ENTRYPOINT_KINDS = new Set(["process", "worker", "webview"]);
const UI_SURFACE_KINDS = new Set([
  "panel",
  "sidebar",
  "status_bar",
  "command_palette",
  "settings",
]);

export class CtxPluginValidationError extends Error {
  readonly diagnostics: PluginDefinitionDiagnostic[];

  constructor(diagnostics: PluginDefinitionDiagnostic[]) {
    super(formatPluginDiagnostics(diagnostics));
    this.name = "CtxPluginValidationError";
    this.diagnostics = diagnostics;
  }
}

export function validateCtxPlugin(plugin: unknown): PluginValidationResult {
  const diagnostics: PluginDefinitionDiagnostic[] = [];
  const manifest = asRecord(plugin, "$", diagnostics);
  if (!manifest) {
    return { ok: false, diagnostics };
  }

  const entrypointIds = new Set<string>();
  const contributionIds = new Set<string>();

  validateManifestHeader(manifest, diagnostics);
  validateAllowedKeys(
    manifest,
    [
      "schema_version",
      "id",
      "name",
      "version",
      "description",
      "entrypoints",
      "contributes",
      "compatibility",
    ],
    "$",
    diagnostics,
  );

  for (const [index, item] of optionalArray(
    manifest.entrypoints,
    "entrypoints",
    diagnostics,
  ).entries()) {
    validateEntrypoint(item, `entrypoints[${index}]`, entrypointIds, diagnostics);
  }

  const contributes = optionalRecord(
    manifest.contributes,
    "contributes",
    diagnostics,
  );
  if (contributes) {
    validateAllowedKeys(
      contributes,
      [
        "providers",
        "runtimes",
        "commands",
        "collectors",
        "observers",
        "ui_surfaces",
      ],
      "contributes",
      diagnostics,
    );
    validateNamedContributionList(
      optionalArray(contributes.providers, "contributes.providers", diagnostics),
      "contributes.providers",
      ["id", "name", "description", "entrypoint", "capabilities"],
      entrypointIds,
      contributionIds,
      diagnostics,
      { stringArrayFields: ["capabilities"] },
    );
    validateNamedContributionList(
      optionalArray(contributes.runtimes, "contributes.runtimes", diagnostics),
      "contributes.runtimes",
      ["id", "name", "description", "entrypoint", "capabilities"],
      entrypointIds,
      contributionIds,
      diagnostics,
      { stringArrayFields: ["capabilities"] },
    );
    validateCommandContributions(
      optionalArray(contributes.commands, "contributes.commands", diagnostics),
      manifest.id,
      entrypointIds,
      contributionIds,
      diagnostics,
    );
    validateNamedContributionList(
      optionalArray(contributes.collectors, "contributes.collectors", diagnostics),
      "contributes.collectors",
      ["id", "name", "description", "entrypoint", "events"],
      entrypointIds,
      contributionIds,
      diagnostics,
      { stringArrayFields: ["events"], validateCollectorBoundary: true },
    );
    validateNamedContributionList(
      optionalArray(contributes.observers, "contributes.observers", diagnostics),
      "contributes.observers",
      ["id", "name", "description", "entrypoint", "events"],
      entrypointIds,
      contributionIds,
      diagnostics,
      { stringArrayFields: ["events"] },
    );
    validateNamedContributionList(
      optionalArray(contributes.ui_surfaces, "contributes.ui_surfaces", diagnostics),
      "contributes.ui_surfaces",
      ["id", "name", "surface", "description", "entrypoint", "contexts"],
      entrypointIds,
      contributionIds,
      diagnostics,
      {
        pluginId: manifest.id,
        requirePluginQualifiedId: true,
        stringArrayFields: ["contexts"],
        validateSurfaceKind: true,
      },
    );
  }

  const compatibility = optionalRecord(
    manifest.compatibility,
    "compatibility",
    diagnostics,
  );
  if (compatibility) {
    validateAllowedKeys(
      compatibility,
      ["min_ctx_version", "capabilities"],
      "compatibility",
      diagnostics,
    );
    validateOptionalNullableString(
      compatibility.min_ctx_version,
      "compatibility.min_ctx_version",
      diagnostics,
    );
    validateOptionalStringArray(
      compatibility.capabilities,
      "compatibility.capabilities",
      diagnostics,
    );
  }

  if (!hasRuntimeShape(manifest)) {
    diagnostics.push({
      severity: "error",
      code: "empty_plugin_definition",
      message:
        "Plugin must declare at least one entrypoint or current-manifest contribution.",
      path: "$",
    });
  }

  return diagnostics.some((diagnostic) => diagnostic.severity === "error")
    ? { ok: false, diagnostics }
    : { ok: true, diagnostics };
}

export function validateCtxPluginSet(
  plugins: readonly unknown[],
): PluginValidationResult {
  const diagnostics: PluginDefinitionDiagnostic[] = [];
  const pluginIds = new Set<string>();
  const providerIds = new Set<string>();

  for (const [pluginIndex, plugin] of plugins.entries()) {
    const pluginResult = validateCtxPlugin(plugin);
    diagnostics.push(
      ...pluginResult.diagnostics.map((diagnostic) => ({
        ...diagnostic,
        path: `plugins[${pluginIndex}].${diagnostic.path}`,
      })),
    );

    const pluginRecord = isRecord(plugin) ? plugin : null;
    trackUnique(
      pluginIds,
      pluginRecord?.id,
      `plugins[${pluginIndex}].id`,
      "duplicate_plugin_id",
      diagnostics,
    );

    const contributes = isRecord(pluginRecord?.contributes)
      ? pluginRecord.contributes
      : null;
    const providers = Array.isArray(contributes?.providers)
      ? contributes.providers
      : [];
    for (const [providerIndex, provider] of providers.entries()) {
      const providerRecord = isRecord(provider) ? provider : null;
      trackUnique(
        providerIds,
        providerRecord?.id,
        `plugins[${pluginIndex}].contributes.providers[${providerIndex}].id`,
        "duplicate_provider_id",
        diagnostics,
      );
    }
  }

  return diagnostics.some((diagnostic) => diagnostic.severity === "error")
    ? { ok: false, diagnostics }
    : { ok: true, diagnostics };
}

export function assertValidCtxPlugin(plugin: unknown): PluginDefinition {
  const result = validateCtxPlugin(plugin);
  if (!result.ok) {
    throw new CtxPluginValidationError(result.diagnostics);
  }
  return plugin as PluginDefinition;
}

export function formatPluginDiagnostics(
  diagnostics: PluginDefinitionDiagnostic[],
): string {
  if (diagnostics.length === 0) {
    return "Plugin definition is valid.";
  }
  return diagnostics
    .map(
      (diagnostic) =>
        `${diagnostic.severity.toUpperCase()} ${diagnostic.code} at ${diagnostic.path}: ${diagnostic.message}`,
    )
    .join("\n");
}

function validateManifestHeader(
  manifest: ObjectRecord,
  diagnostics: PluginDefinitionDiagnostic[],
): void {
  if (
    manifest.schema_version !== undefined &&
    manifest.schema_version !== CTX_PLUGIN_MANIFEST_SCHEMA_VERSION
  ) {
    diagnostics.push({
      severity: "error",
      code: "unsupported_schema_version",
      message: `Expected schema_version ${CTX_PLUGIN_MANIFEST_SCHEMA_VERSION}.`,
      path: "schema_version",
    });
  }

  validateRequiredString(manifest.id, "id", diagnostics);
  validateRequiredString(manifest.name, "name", diagnostics);
  validateRequiredString(manifest.version, "version", diagnostics);
  validateOptionalNullableString(manifest.description, "description", diagnostics);
}

function validateEntrypoint(
  value: unknown,
  path: string,
  entrypointIds: Set<string>,
  diagnostics: PluginDefinitionDiagnostic[],
): void {
  const item = asRecord(value, path, diagnostics);
  if (!item) return;

  validateAllowedKeys(
    item,
    ["id", "kind", "command", "args", "cwd", "environment"],
    path,
    diagnostics,
  );
  validateRequiredString(item.id, `${path}.id`, diagnostics);
  validateRequiredString(item.command, `${path}.command`, diagnostics);
  validateOptionalNullableString(item.cwd, `${path}.cwd`, diagnostics);
  validateOptionalStringArray(item.args, `${path}.args`, diagnostics);
  validateOptionalStringRecord(item.environment, `${path}.environment`, diagnostics);
  if (item.kind !== undefined) {
    if (typeof item.kind !== "string" || !ENTRYPOINT_KINDS.has(item.kind)) {
      diagnostics.push({
        severity: "error",
        code: "invalid_entrypoint_kind",
        message: "Entrypoint kind must be one of process, worker, or webview.",
        path: `${path}.kind`,
      });
    }
  }
  trackUnique(
    entrypointIds,
    item.id,
    `${path}.id`,
    "duplicate_entrypoint_id",
    diagnostics,
  );
}

function validateCommandContributions(
  commands: readonly unknown[],
  pluginId: unknown,
  entrypointIds: ReadonlySet<string>,
  contributionIds: Set<string>,
  diagnostics: PluginDefinitionDiagnostic[],
): void {
  for (const [index, value] of commands.entries()) {
    const path = `contributes.commands[${index}]`;
    const command = asRecord(value, path, diagnostics);
    if (!command) continue;

    validateRequiredString(command.id, `${path}.id`, diagnostics);
    validateRequiredString(command.title, `${path}.title`, diagnostics);
    validateAllowedKeys(
      command,
      ["id", "title", "description", "category", "entrypoint"],
      path,
      diagnostics,
    );
    validateOptionalNullableString(command.description, `${path}.description`, diagnostics);
    validateOptionalNullableString(command.category, `${path}.category`, diagnostics);
    validatePluginQualifiedId(
      command.id,
      pluginId,
      `${path}.id`,
      "command_id_not_namespaced",
      "Command id",
      diagnostics,
    );
    trackUnique(
      contributionIds,
      command.id,
      `${path}.id`,
      "duplicate_contribution_id",
      diagnostics,
    );
    validateEntrypointReference(
      command.entrypoint,
      `${path}.entrypoint`,
      entrypointIds,
      diagnostics,
    );
  }
}

type NamedContributionOptions = {
  pluginId?: unknown;
  requirePluginQualifiedId?: boolean;
  stringArrayFields?: readonly string[];
  validateCollectorBoundary?: boolean;
  validateSurfaceKind?: boolean;
};

function validateNamedContributionList(
  contributions: readonly unknown[],
  pathPrefix: string,
  allowedKeys: readonly string[],
  entrypointIds: ReadonlySet<string>,
  contributionIds: Set<string>,
  diagnostics: PluginDefinitionDiagnostic[],
  options: NamedContributionOptions = {},
): void {
  for (const [index, value] of contributions.entries()) {
    const path = `${pathPrefix}[${index}]`;
    const contribution = asRecord(value, path, diagnostics);
    if (!contribution) continue;

    validateAllowedKeys(contribution, allowedKeys, path, diagnostics);
    validateRequiredString(contribution.id, `${path}.id`, diagnostics);
    validateRequiredString(contribution.name, `${path}.name`, diagnostics);
    validateOptionalNullableString(
      contribution.description,
      `${path}.description`,
      diagnostics,
    );
    if (options.requirePluginQualifiedId) {
      validatePluginQualifiedId(
        contribution.id,
        options.pluginId,
        `${path}.id`,
        "contribution_id_not_plugin_qualified",
        "Contribution id",
        diagnostics,
      );
    }
    trackUnique(
      contributionIds,
      contribution.id,
      `${path}.id`,
      "duplicate_contribution_id",
      diagnostics,
    );
    validateEntrypointReference(
      contribution.entrypoint,
      `${path}.entrypoint`,
      entrypointIds,
      diagnostics,
    );
    for (const field of options.stringArrayFields ?? []) {
      validateOptionalStringArray(
        contribution[field],
        `${path}.${field}`,
        diagnostics,
      );
    }
    if (options.validateSurfaceKind) {
      validateUiSurfaceKind(contribution.surface, `${path}.surface`, diagnostics);
    }
    if (options.validateCollectorBoundary) {
      validateCollectorStoreBoundary(contribution, path, diagnostics);
    }
  }
}

function validateCollectorStoreBoundary(
  contribution: ObjectRecord,
  path: string,
  diagnostics: PluginDefinitionDiagnostic[],
): void {
  if ("writes" in contribution || "store_writes" in contribution) {
    diagnostics.push({
      severity: "error",
      code: "collector_store_write_forbidden",
      message:
        "Collector/importer contributions must request approved ctx actions instead of declaring direct store writes.",
      path,
    });
  }
  if ("allowed_actions" in contribution) {
    diagnostics.push({
      severity: "error",
      code: "importer_actions_not_manifest_fields",
      message:
        "Importer action requests are SDK sidecar values and must not be embedded in the current v1 manifest.",
      path: `${path}.allowed_actions`,
    });
  }
}

function validateUiSurfaceKind(
  value: unknown,
  path: string,
  diagnostics: PluginDefinitionDiagnostic[],
): void {
  if (typeof value !== "string" || !UI_SURFACE_KINDS.has(value)) {
    diagnostics.push({
      severity: "error",
      code: "invalid_ui_surface_kind",
      message:
        "UI surface must be one of panel, sidebar, status_bar, command_palette, or settings.",
      path,
    });
  }
}

function validateAllowedKeys(
  value: ObjectRecord,
  allowedKeys: readonly string[],
  path: string,
  diagnostics: PluginDefinitionDiagnostic[],
): void {
  for (const key of Object.keys(value)) {
    if (!allowedKeys.includes(key)) {
      diagnostics.push({
        severity: "error",
        code: "unknown_manifest_property",
        message: `Property '${key}' is not part of the current v1 plugin manifest schema.`,
        path: `${path}.${key}`,
      });
    }
  }
}

function validateRequiredString(
  value: unknown,
  path: string,
  diagnostics: PluginDefinitionDiagnostic[],
): void {
  if (typeof value !== "string" || value.trim().length === 0) {
    diagnostics.push({
      severity: "error",
      code: "empty_string",
      message: "Expected a non-empty string.",
      path,
    });
  }
}

function validateOptionalNullableString(
  value: unknown,
  path: string,
  diagnostics: PluginDefinitionDiagnostic[],
): void {
  if (value === undefined || value === null) return;
  if (typeof value !== "string") {
    diagnostics.push({
      severity: "error",
      code: "expected_string",
      message: "Expected a string or null.",
      path,
    });
  }
}

function validateOptionalStringArray(
  value: unknown,
  path: string,
  diagnostics: PluginDefinitionDiagnostic[],
): void {
  if (value === undefined) return;
  if (!Array.isArray(value)) {
    diagnostics.push({
      severity: "error",
      code: "expected_array",
      message: "Expected an array of strings.",
      path,
    });
    return;
  }
  for (const [index, item] of value.entries()) {
    if (typeof item !== "string" || item.trim().length === 0) {
      diagnostics.push({
        severity: "error",
        code: "expected_string",
        message: "Expected a non-empty string.",
        path: `${path}[${index}]`,
      });
    }
  }
}

function validateOptionalStringRecord(
  value: unknown,
  path: string,
  diagnostics: PluginDefinitionDiagnostic[],
): void {
  if (value === undefined) return;
  const record = asRecord(value, path, diagnostics);
  if (!record) return;
  for (const [key, item] of Object.entries(record)) {
    if (typeof item !== "string") {
      diagnostics.push({
        severity: "error",
        code: "expected_string",
        message: "Expected a string value.",
        path: `${path}.${key}`,
      });
    }
  }
}

function validateEntrypointReference(
  entrypointId: unknown,
  path: string,
  entrypointIds: ReadonlySet<string>,
  diagnostics: PluginDefinitionDiagnostic[],
): void {
  if (entrypointId === undefined || entrypointId === null) {
    return;
  }
  if (typeof entrypointId !== "string" || entrypointId.trim().length === 0) {
    diagnostics.push({
      severity: "error",
      code: "empty_entrypoint_reference",
      message: "Entrypoint reference must be a non-empty string when provided.",
      path,
    });
    return;
  }
  if (!entrypointIds.has(entrypointId)) {
    diagnostics.push({
      severity: "error",
      code: "unknown_entrypoint",
      message: `Unknown entrypoint '${entrypointId}'.`,
      path,
    });
  }
}

function validatePluginQualifiedId(
  value: unknown,
  pluginId: unknown,
  path: string,
  code: string,
  label: string,
  diagnostics: PluginDefinitionDiagnostic[],
): void {
  if (
    typeof pluginId !== "string" ||
    pluginId.trim().length === 0 ||
    typeof value !== "string" ||
    value.trim().length === 0
  ) {
    return;
  }
  if (!value.startsWith(`${pluginId}.`)) {
    diagnostics.push({
      severity: "error",
      code,
      message: `${label} must be source-qualified with the plugin id prefix '${pluginId}.'.`,
      path,
    });
  }
}

function trackUnique(
  seen: Set<string>,
  value: unknown,
  path: string,
  code: string,
  diagnostics: PluginDefinitionDiagnostic[],
): void {
  if (typeof value !== "string" || value.trim().length === 0) {
    return;
  }
  if (seen.has(value)) {
    diagnostics.push({
      severity: "error",
      code,
      message: `Duplicate id '${value}'.`,
      path,
    });
  }
  seen.add(value);
}

function optionalArray(
  value: unknown,
  path: string,
  diagnostics: PluginDefinitionDiagnostic[],
): readonly unknown[] {
  if (value === undefined) return [];
  if (Array.isArray(value)) return value;
  diagnostics.push({
    severity: "error",
    code: "expected_array",
    message: "Expected an array.",
    path,
  });
  return [];
}

function optionalRecord(
  value: unknown,
  path: string,
  diagnostics: PluginDefinitionDiagnostic[],
): ObjectRecord | null {
  if (value === undefined) return null;
  return asRecord(value, path, diagnostics);
}

function asRecord(
  value: unknown,
  path: string,
  diagnostics: PluginDefinitionDiagnostic[],
): ObjectRecord | null {
  if (isRecord(value)) return value;
  diagnostics.push({
    severity: "error",
    code: "expected_object",
    message: "Expected an object.",
    path,
  });
  return null;
}

function isRecord(value: unknown): value is ObjectRecord {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function hasRuntimeShape(plugin: ObjectRecord): boolean {
  if (Array.isArray(plugin.entrypoints) && plugin.entrypoints.length > 0) {
    return true;
  }
  const contributes = isRecord(plugin.contributes) ? plugin.contributes : null;
  if (!contributes) return false;
  return [
    contributes.providers,
    contributes.runtimes,
    contributes.commands,
    contributes.collectors,
    contributes.observers,
    contributes.ui_surfaces,
  ].some((value) => Array.isArray(value) && value.length > 0);
}
