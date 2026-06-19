import type {
  ApprovedCtxActionId,
  PluginArtifactRendererContribution,
  PluginContributionRegistration,
  PluginExtensionRegistry,
  PluginWorkbenchCardRendererContribution,
  PluginWorkbenchSectionContribution,
  PluginWorkbenchTemplateContribution,
  PluginWorkbenchToolbarActionContribution,
  PluginUiSurfaceContribution,
  PluginUiSurfaceKind,
} from "@ctx/types";
import type {
  WorkbenchBuiltinTemplateId,
  WorkbenchPluginTemplateId,
  WorkbenchTemplateId,
} from "../../workbench/types";

export type WorkbenchContributionProjectionLoadState =
  | { kind: "loading" }
  | { kind: "error"; message: string }
  | { kind: "ready" };

export type WorkbenchContributionCompatibility =
  | { kind: "compatible" }
  | { kind: "unsupported_surface"; surface: PluginUiSurfaceKind }
  | { kind: "invalid"; reasons: string[] };

export type WorkbenchContributionSource = {
  kind: "plugin";
  pluginId: string;
  pluginName: string;
  pluginVersion: string;
  pluginPath: string;
  pluginRevision: string | null;
};

export type WorkbenchContributionCandidate = {
  id: WorkbenchPluginTemplateId;
  contributionId: string;
  title: string;
  description: string | null;
  surface: PluginUiSurfaceKind;
  contexts: string[];
  entrypoint: string | null;
  source: WorkbenchContributionSource;
  compatibility: WorkbenchContributionCompatibility;
};

export type WorkbenchDeclarativeContributionBucket =
  | "templates"
  | "toolbar_actions"
  | "artifact_renderers"
  | "card_renderers"
  | "detail_sections"
  | "review_sections";

export type WorkbenchDeclarativeContributionCompatibility =
  | { kind: "compatible" }
  | { kind: "unsupported_template"; template: string }
  | { kind: "unsupported_renderer"; renderer: string }
  | { kind: "invalid"; reasons: string[] };

export type WorkbenchDeclarativeContributionBaseCandidate = {
  id: string;
  bucket: WorkbenchDeclarativeContributionBucket;
  contributionId: string;
  name: string;
  title: string;
  description: string | null;
  contexts: string[];
  dataSources: string[];
  source: WorkbenchContributionSource & {
    label: string;
  };
  compatibility: WorkbenchDeclarativeContributionCompatibility;
};

export type WorkbenchTemplateContributionCandidate = WorkbenchDeclarativeContributionBaseCandidate & {
  bucket: "templates";
  template: WorkbenchBuiltinTemplateId | string;
};

export type WorkbenchToolbarActionContributionCandidate = WorkbenchDeclarativeContributionBaseCandidate & {
  bucket: "toolbar_actions";
  icon: string | null;
  intent:
    | { kind: "ctx_action"; action: ApprovedCtxActionId | string }
    | { kind: "plugin_command"; command: string }
    | null;
};

export type WorkbenchArtifactRendererContributionCandidate = WorkbenchDeclarativeContributionBaseCandidate & {
  bucket: "artifact_renderers";
  artifactTypes: string[];
  renderer: string;
};

export type WorkbenchCardRendererContributionCandidate = WorkbenchDeclarativeContributionBaseCandidate & {
  bucket: "card_renderers";
  card: string;
  renderer: string;
};

export type WorkbenchSectionContributionCandidate = WorkbenchDeclarativeContributionBaseCandidate & {
  bucket: "detail_sections" | "review_sections";
  section: string;
  renderer: string;
};

export type WorkbenchDeclarativeContributionCandidate =
  | WorkbenchTemplateContributionCandidate
  | WorkbenchToolbarActionContributionCandidate
  | WorkbenchArtifactRendererContributionCandidate
  | WorkbenchCardRendererContributionCandidate
  | WorkbenchSectionContributionCandidate;

export type WorkbenchDeclarativeContributionBuckets = {
  templates: WorkbenchTemplateContributionCandidate[];
  toolbarActions: WorkbenchToolbarActionContributionCandidate[];
  artifactRenderers: WorkbenchArtifactRendererContributionCandidate[];
  cardRenderers: WorkbenchCardRendererContributionCandidate[];
  detailSections: WorkbenchSectionContributionCandidate[];
  reviewSections: WorkbenchSectionContributionCandidate[];
};

export type WorkbenchDeclarativeContributionProjection =
  | {
      kind: "loading";
      candidates: WorkbenchDeclarativeContributionCandidate[];
      buckets: WorkbenchDeclarativeContributionBuckets;
      fallback: { kind: "registry_loading" };
    }
  | {
      kind: "error";
      message: string;
      candidates: WorkbenchDeclarativeContributionCandidate[];
      buckets: WorkbenchDeclarativeContributionBuckets;
      fallback: { kind: "registry_error"; message: string };
    }
  | {
      kind: "empty";
      candidates: [];
      buckets: WorkbenchDeclarativeContributionBuckets;
      fallback: null;
    }
  | {
      kind: "ready";
      candidates: WorkbenchDeclarativeContributionCandidate[];
      buckets: WorkbenchDeclarativeContributionBuckets;
      fallback: null;
    };

export type WorkbenchContributionFallback =
  | {
      kind: "removed_plugin";
      requestedTemplateId: WorkbenchPluginTemplateId;
      fallbackTemplateId: WorkbenchBuiltinTemplateId;
      pluginId: string;
      contributionId: string;
    }
  | {
      kind: "unavailable";
      requestedTemplateId: WorkbenchPluginTemplateId;
      fallbackTemplateId: WorkbenchBuiltinTemplateId;
      reason: "loading" | "error" | "incompatible";
    };

export type WorkbenchContributionProjection =
  | {
      kind: "loading";
      candidates: WorkbenchContributionCandidate[];
      activeCandidate: null;
      fallback: WorkbenchContributionFallback | null;
      effectiveTemplateId: WorkbenchTemplateId;
    }
  | {
      kind: "error";
      message: string;
      candidates: WorkbenchContributionCandidate[];
      activeCandidate: null;
      fallback: WorkbenchContributionFallback | null;
      effectiveTemplateId: WorkbenchTemplateId;
    }
  | {
      kind: "empty";
      candidates: [];
      activeCandidate: null;
      fallback: null;
      effectiveTemplateId: WorkbenchTemplateId;
    }
  | {
      kind: "ready";
      candidates: WorkbenchContributionCandidate[];
      activeCandidate: WorkbenchContributionCandidate | null;
      fallback: WorkbenchContributionFallback | null;
      effectiveTemplateId: WorkbenchTemplateId;
    }
  | {
      kind: "fallback";
      candidates: WorkbenchContributionCandidate[];
      activeCandidate: null;
      fallback: WorkbenchContributionFallback;
      effectiveTemplateId: WorkbenchBuiltinTemplateId;
    };

export type ProjectWorkbenchContributionProjectionOptions = {
  loadState: WorkbenchContributionProjectionLoadState;
  registry?: PluginExtensionRegistry | null;
  activeTemplateId?: WorkbenchTemplateId | null;
  fallbackTemplateId?: WorkbenchBuiltinTemplateId;
};

export type ProjectWorkbenchDeclarativeContributionProjectionOptions = {
  loadState: WorkbenchContributionProjectionLoadState;
  registry?: PluginExtensionRegistry | null;
};

type PluginDeclarativeExtensionRegistry = PluginExtensionRegistry & {
  templates?: PluginContributionRegistration<PluginWorkbenchTemplateContribution>[];
  toolbar_actions?: PluginContributionRegistration<PluginWorkbenchToolbarActionContribution>[];
  artifact_renderers?: PluginContributionRegistration<PluginArtifactRendererContribution>[];
  card_renderers?: PluginContributionRegistration<PluginWorkbenchCardRendererContribution>[];
  detail_sections?: PluginContributionRegistration<PluginWorkbenchSectionContribution>[];
  review_sections?: PluginContributionRegistration<PluginWorkbenchSectionContribution>[];
};

const WORKBENCH_COMPATIBLE_SURFACES = new Set<PluginUiSurfaceKind>([
  "panel",
  "sidebar",
  "status_bar",
]);

const DEFAULT_FALLBACK_TEMPLATE_ID: WorkbenchBuiltinTemplateId = "classic";

const SUPPORTED_DECLARATIVE_TEMPLATES = new Set<string>(["classic", "kanban", "multipane", "review"]);

const SUPPORTED_DECLARATIVE_RENDERERS = new Set<string>([
  "host.diff-artifact",
  "host.review-finding-card",
  "host.diff-summary-section",
  "host.gate-state-section",
]);

const normalizeText = (value: string | null | undefined): string => String(value ?? "").trim();

const normalizeStringList = (values: readonly string[] | null | undefined): string[] => {
  const seen = new Set<string>();
  const out: string[] = [];
  for (const value of values ?? []) {
    const normalized = normalizeText(value);
    if (!normalized || seen.has(normalized)) continue;
    seen.add(normalized);
    out.push(normalized);
  }
  return out;
};

const candidateSortKey = (entry: WorkbenchContributionCandidate): string =>
  [
    entry.compatibility.kind === "compatible" ? "0" : "1",
    entry.surface,
    entry.title.toLowerCase(),
    entry.source.pluginName.toLowerCase(),
    entry.id.toLowerCase(),
  ].join("\0");

const declarativeCandidateSortKey = (entry: WorkbenchDeclarativeContributionCandidate): string =>
  [
    entry.compatibility.kind === "compatible" ? "0" : "1",
    entry.bucket,
    entry.title.toLowerCase(),
    entry.source.pluginName.toLowerCase(),
    entry.id.toLowerCase(),
  ].join("\0");

const PLUGIN_TEMPLATE_ID_PREFIX = "plugin:";
const PLUGIN_TEMPLATE_ID_SEPARATOR = "/";

const encodePluginTemplateIdPart = (value: string): string => encodeURIComponent(value);

const decodePluginTemplateIdPart = (value: string): string | null => {
  try {
    return decodeURIComponent(value);
  } catch {
    return null;
  }
};

const parsePluginTemplateIdParts = (
  value: WorkbenchTemplateId | string,
): { pluginId: string; contributionId: string } | null => {
  if (!value.startsWith(PLUGIN_TEMPLATE_ID_PREFIX)) return null;

  const body = value.slice(PLUGIN_TEMPLATE_ID_PREFIX.length);
  const separator = body.indexOf(PLUGIN_TEMPLATE_ID_SEPARATOR);
  if (separator <= 0 || separator !== body.lastIndexOf(PLUGIN_TEMPLATE_ID_SEPARATOR) || separator === body.length - 1) {
    return null;
  }

  const pluginId = decodePluginTemplateIdPart(body.slice(0, separator));
  const contributionId = decodePluginTemplateIdPart(body.slice(separator + 1));
  if (!pluginId || !contributionId) return null;

  return { pluginId, contributionId };
};

export const isWorkbenchPluginTemplateId = (value: WorkbenchTemplateId | string): value is WorkbenchPluginTemplateId =>
  parsePluginTemplateIdParts(value) !== null;

export const parseWorkbenchPluginTemplateId = (
  value: WorkbenchPluginTemplateId,
): { pluginId: string; contributionId: string } => {
  const parsed = parsePluginTemplateIdParts(value);
  if (!parsed) throw new Error(`Invalid Workbench plugin template ID: ${value}`);
  return parsed;
};

export const toWorkbenchPluginTemplateId = (
  pluginId: string,
  contributionId: string,
): WorkbenchPluginTemplateId =>
  `${PLUGIN_TEMPLATE_ID_PREFIX}${encodePluginTemplateIdPart(pluginId)}${PLUGIN_TEMPLATE_ID_SEPARATOR}${encodePluginTemplateIdPart(
    contributionId,
  )}`;

export const projectPluginWorkbenchContributions = (
  registry: PluginExtensionRegistry,
): WorkbenchContributionCandidate[] => {
  const candidates: WorkbenchContributionCandidate[] = [];
  for (const registration of registry.ui_surfaces ?? []) {
    const candidate = projectPluginWorkbenchContribution(registration);
    if (candidate) candidates.push(candidate);
  }
  return candidates.sort((left, right) => candidateSortKey(left).localeCompare(candidateSortKey(right)));
};

export const projectPluginWorkbenchDeclarativeContributions = (
  registry: PluginExtensionRegistry,
): WorkbenchDeclarativeContributionCandidate[] => {
  const declarativeRegistry = registry as PluginDeclarativeExtensionRegistry;
  const candidates: WorkbenchDeclarativeContributionCandidate[] = [];

  for (const registration of declarativeRegistry.templates ?? []) {
    const candidate = projectWorkbenchTemplateContribution(registration);
    if (candidate) candidates.push(candidate);
  }
  for (const registration of declarativeRegistry.toolbar_actions ?? []) {
    const candidate = projectWorkbenchToolbarActionContribution(registration);
    if (candidate) candidates.push(candidate);
  }
  for (const registration of declarativeRegistry.artifact_renderers ?? []) {
    const candidate = projectWorkbenchArtifactRendererContribution(registration);
    if (candidate) candidates.push(candidate);
  }
  for (const registration of declarativeRegistry.card_renderers ?? []) {
    const candidate = projectWorkbenchCardRendererContribution(registration);
    if (candidate) candidates.push(candidate);
  }
  for (const registration of declarativeRegistry.detail_sections ?? []) {
    const candidate = projectWorkbenchSectionContribution("detail_sections", registration);
    if (candidate) candidates.push(candidate);
  }
  for (const registration of declarativeRegistry.review_sections ?? []) {
    const candidate = projectWorkbenchSectionContribution("review_sections", registration);
    if (candidate) candidates.push(candidate);
  }

  return candidates.sort((left, right) =>
    declarativeCandidateSortKey(left).localeCompare(declarativeCandidateSortKey(right))
  );
};

const buildDeclarativeContributionBase = (
  bucket: WorkbenchDeclarativeContributionBucket,
  registration: PluginContributionRegistration<{
    id: string;
    name: string;
    description?: string | null;
    contexts?: string[];
    data_sources?: string[];
  }>,
  title: string,
): WorkbenchDeclarativeContributionBaseCandidate | null => {
  const contributionId = normalizeText(registration.contribution.id);
  const pluginId = normalizeText(registration.plugin_id);
  const name = normalizeText(registration.contribution.name);
  const normalizedTitle = normalizeText(title);
  if (!contributionId || !pluginId || !name || !normalizedTitle) return null;

  const pluginName = normalizeText(registration.plugin_name);
  const pluginVersion = normalizeText(registration.plugin_version);
  const reasons: string[] = [];
  if (!pluginName) reasons.push("missing_plugin_name");
  if (!pluginVersion) reasons.push("missing_plugin_version");

  return {
    id: `${bucket}:${encodePluginTemplateIdPart(pluginId)}${PLUGIN_TEMPLATE_ID_SEPARATOR}${encodePluginTemplateIdPart(
      contributionId,
    )}`,
    bucket,
    contributionId,
    name,
    title: normalizedTitle,
    description: normalizeText(registration.contribution.description) || null,
    contexts: normalizeStringList(registration.contribution.contexts),
    dataSources: normalizeStringList(registration.contribution.data_sources),
    source: {
      kind: "plugin",
      pluginId,
      pluginName: pluginName || pluginId,
      pluginVersion,
      pluginPath: normalizeText(registration.plugin_path),
      pluginRevision: normalizeText(registration.plugin_revision) || null,
      label: pluginVersion ? `${pluginName || pluginId} ${pluginVersion}` : pluginName || pluginId,
    },
    compatibility: reasons.length ? { kind: "invalid", reasons } : { kind: "compatible" },
  };
};

const mergeDeclarativeCompatibility = (
  base: WorkbenchDeclarativeContributionCompatibility,
  next: WorkbenchDeclarativeContributionCompatibility,
): WorkbenchDeclarativeContributionCompatibility => {
  if (base.kind === "invalid" || next.kind === "compatible") return base;
  return next;
};

const projectWorkbenchTemplateContribution = (
  registration: PluginContributionRegistration<PluginWorkbenchTemplateContribution>,
): WorkbenchTemplateContributionCandidate | null => {
  const template = normalizeText(registration.contribution.template);
  const base = buildDeclarativeContributionBase("templates", registration, registration.contribution.title);
  if (!base || !template) return null;

  return {
    ...base,
    bucket: "templates",
    template,
    compatibility: mergeDeclarativeCompatibility(
      base.compatibility,
      SUPPORTED_DECLARATIVE_TEMPLATES.has(template)
        ? { kind: "compatible" }
        : { kind: "unsupported_template", template },
    ),
  };
};

const projectWorkbenchToolbarActionContribution = (
  registration: PluginContributionRegistration<PluginWorkbenchToolbarActionContribution>,
): WorkbenchToolbarActionContributionCandidate | null => {
  const base = buildDeclarativeContributionBase("toolbar_actions", registration, registration.contribution.title);
  if (!base) return null;

  const command = normalizeText(registration.contribution.command);
  const action = normalizeText(registration.contribution.action);
  return {
    ...base,
    bucket: "toolbar_actions",
    icon: normalizeText(registration.contribution.icon) || null,
    intent: command
      ? { kind: "plugin_command", command }
      : action
        ? { kind: "ctx_action", action }
        : null,
  };
};

const projectWorkbenchArtifactRendererContribution = (
  registration: PluginContributionRegistration<PluginArtifactRendererContribution>,
): WorkbenchArtifactRendererContributionCandidate | null => {
  const renderer = normalizeText(registration.contribution.renderer);
  const artifactTypes = normalizeStringList(registration.contribution.artifact_types);
  const base = buildDeclarativeContributionBase("artifact_renderers", registration, registration.contribution.name);
  if (!base || !renderer || artifactTypes.length === 0) return null;

  return {
    ...base,
    bucket: "artifact_renderers",
    artifactTypes,
    renderer,
    compatibility: mergeDeclarativeCompatibility(
      base.compatibility,
      SUPPORTED_DECLARATIVE_RENDERERS.has(renderer)
        ? { kind: "compatible" }
        : { kind: "unsupported_renderer", renderer },
    ),
  };
};

const projectWorkbenchCardRendererContribution = (
  registration: PluginContributionRegistration<PluginWorkbenchCardRendererContribution>,
): WorkbenchCardRendererContributionCandidate | null => {
  const card = normalizeText(registration.contribution.card);
  const renderer = normalizeText(registration.contribution.renderer);
  const base = buildDeclarativeContributionBase("card_renderers", registration, registration.contribution.name);
  if (!base || !card || !renderer) return null;

  return {
    ...base,
    bucket: "card_renderers",
    card,
    renderer,
    compatibility: mergeDeclarativeCompatibility(
      base.compatibility,
      SUPPORTED_DECLARATIVE_RENDERERS.has(renderer)
        ? { kind: "compatible" }
        : { kind: "unsupported_renderer", renderer },
    ),
  };
};

const projectWorkbenchSectionContribution = (
  bucket: "detail_sections" | "review_sections",
  registration: PluginContributionRegistration<PluginWorkbenchSectionContribution>,
): WorkbenchSectionContributionCandidate | null => {
  const section = normalizeText(registration.contribution.section);
  const renderer = normalizeText(registration.contribution.renderer);
  const base = buildDeclarativeContributionBase(bucket, registration, registration.contribution.name);
  if (!base || !section || !renderer) return null;

  return {
    ...base,
    bucket,
    section,
    renderer,
    compatibility: mergeDeclarativeCompatibility(
      base.compatibility,
      SUPPORTED_DECLARATIVE_RENDERERS.has(renderer)
        ? { kind: "compatible" }
        : { kind: "unsupported_renderer", renderer },
    ),
  };
};

const emptyDeclarativeContributionBuckets = (): WorkbenchDeclarativeContributionBuckets => ({
  templates: [],
  toolbarActions: [],
  artifactRenderers: [],
  cardRenderers: [],
  detailSections: [],
  reviewSections: [],
});

const bucketDeclarativeContributions = (
  candidates: WorkbenchDeclarativeContributionCandidate[],
): WorkbenchDeclarativeContributionBuckets => {
  const buckets = emptyDeclarativeContributionBuckets();
  for (const candidate of candidates) {
    switch (candidate.bucket) {
      case "templates":
        buckets.templates.push(candidate);
        break;
      case "toolbar_actions":
        buckets.toolbarActions.push(candidate);
        break;
      case "artifact_renderers":
        buckets.artifactRenderers.push(candidate);
        break;
      case "card_renderers":
        buckets.cardRenderers.push(candidate);
        break;
      case "detail_sections":
        buckets.detailSections.push(candidate);
        break;
      case "review_sections":
        buckets.reviewSections.push(candidate);
        break;
    }
  }
  return buckets;
};

export const projectWorkbenchDeclarativeContributionProjection = ({
  loadState,
  registry,
}: ProjectWorkbenchDeclarativeContributionProjectionOptions): WorkbenchDeclarativeContributionProjection => {
  const candidates = registry ? projectPluginWorkbenchDeclarativeContributions(registry) : [];
  const buckets = bucketDeclarativeContributions(candidates);

  if (loadState.kind === "loading") {
    return {
      kind: "loading",
      candidates,
      buckets,
      fallback: { kind: "registry_loading" },
    };
  }

  if (loadState.kind === "error") {
    return {
      kind: "error",
      message: loadState.message,
      candidates,
      buckets,
      fallback: { kind: "registry_error", message: loadState.message },
    };
  }

  if (candidates.length === 0) {
    return {
      kind: "empty",
      candidates: [],
      buckets,
      fallback: null,
    };
  }

  return {
    kind: "ready",
    candidates,
    buckets,
    fallback: null,
  };
};

const projectPluginWorkbenchContribution = (
  registration: PluginContributionRegistration<PluginUiSurfaceContribution>,
): WorkbenchContributionCandidate | null => {
  const contributionId = normalizeText(registration.contribution.id);
  const pluginId = normalizeText(registration.plugin_id);
  const title = normalizeText(registration.contribution.name);
  const surface = registration.contribution.surface;
  if (!contributionId || !pluginId || !title || !surface) return null;

  const reasons: string[] = [];
  if (!normalizeText(registration.plugin_name)) reasons.push("missing_plugin_name");
  if (!normalizeText(registration.plugin_version)) reasons.push("missing_plugin_version");

  const compatibility: WorkbenchContributionCompatibility = reasons.length
    ? { kind: "invalid", reasons }
    : WORKBENCH_COMPATIBLE_SURFACES.has(surface)
      ? { kind: "compatible" }
      : { kind: "unsupported_surface", surface };

  return {
    id: toWorkbenchPluginTemplateId(pluginId, contributionId),
    contributionId,
    title,
    description: normalizeText(registration.contribution.description) || null,
    surface,
    contexts: normalizeStringList(registration.contribution.contexts),
    entrypoint: normalizeText(registration.contribution.entrypoint) || null,
    source: {
      kind: "plugin",
      pluginId,
      pluginName: normalizeText(registration.plugin_name) || pluginId,
      pluginVersion: normalizeText(registration.plugin_version),
      pluginPath: normalizeText(registration.plugin_path),
      pluginRevision: normalizeText(registration.plugin_revision) || null,
    },
    compatibility,
  };
};

export const projectWorkbenchContributionProjection = ({
  loadState,
  registry,
  activeTemplateId = "classic",
  fallbackTemplateId = DEFAULT_FALLBACK_TEMPLATE_ID,
}: ProjectWorkbenchContributionProjectionOptions): WorkbenchContributionProjection => {
  const candidates = registry ? projectPluginWorkbenchContributions(registry) : [];
  const requestedTemplateId = activeTemplateId ?? fallbackTemplateId;
  const pluginTemplateRequested = isWorkbenchPluginTemplateId(requestedTemplateId);

  if (loadState.kind === "loading") {
    const fallback = pluginTemplateRequested
      ? buildUnavailableFallback(requestedTemplateId, fallbackTemplateId, "loading")
      : null;
    return {
      kind: "loading",
      candidates,
      activeCandidate: null,
      fallback,
      effectiveTemplateId: fallback ? fallback.fallbackTemplateId : requestedTemplateId,
    };
  }

  if (loadState.kind === "error") {
    const fallback = pluginTemplateRequested
      ? buildUnavailableFallback(requestedTemplateId, fallbackTemplateId, "error")
      : null;
    return {
      kind: "error",
      message: loadState.message,
      candidates,
      activeCandidate: null,
      fallback,
      effectiveTemplateId: fallback ? fallback.fallbackTemplateId : requestedTemplateId,
    };
  }

  if (candidates.length === 0 && !pluginTemplateRequested) {
    return {
      kind: "empty",
      candidates: [],
      activeCandidate: null,
      fallback: null,
      effectiveTemplateId: requestedTemplateId,
    };
  }

  const activeCandidate = pluginTemplateRequested
    ? candidates.find((candidate) => candidate.id === requestedTemplateId) ?? null
    : null;
  if (!pluginTemplateRequested || activeCandidate?.compatibility.kind === "compatible") {
    return {
      kind: "ready",
      candidates,
      activeCandidate,
      fallback: null,
      effectiveTemplateId: requestedTemplateId,
    };
  }

  const fallback = activeCandidate
    ? buildUnavailableFallback(requestedTemplateId, fallbackTemplateId, "incompatible")
    : buildRemovedPluginFallback(requestedTemplateId, fallbackTemplateId);
  return {
    kind: "fallback",
    candidates,
    activeCandidate: null,
    fallback,
    effectiveTemplateId: fallbackTemplateId,
  };
};

const buildRemovedPluginFallback = (
  requestedTemplateId: WorkbenchPluginTemplateId,
  fallbackTemplateId: WorkbenchBuiltinTemplateId,
): WorkbenchContributionFallback => {
  const { pluginId, contributionId } = parseWorkbenchPluginTemplateId(requestedTemplateId);
  return {
    kind: "removed_plugin",
    requestedTemplateId,
    fallbackTemplateId,
    pluginId,
    contributionId,
  };
};

const buildUnavailableFallback = (
  requestedTemplateId: WorkbenchPluginTemplateId,
  fallbackTemplateId: WorkbenchBuiltinTemplateId,
  reason: "loading" | "error" | "incompatible",
): WorkbenchContributionFallback => ({
  kind: "unavailable",
  requestedTemplateId,
  fallbackTemplateId,
  reason,
});
