export const CTX_PLUGIN_MANIFEST_SCHEMA_VERSION = 1 as const;

export type NonEmptyString = string;

export type CtxPluginManifest = {
  schema_version?: typeof CTX_PLUGIN_MANIFEST_SCHEMA_VERSION;
  id: NonEmptyString;
  name: NonEmptyString;
  version: NonEmptyString;
  description?: string | null;
  entrypoints?: PluginEntrypoint[];
  contributes?: PluginContributions;
  compatibility?: PluginCompatibility;
};

export type PluginEntrypointKind = "process" | "worker" | "webview";

export type PluginEntrypoint = {
  id: NonEmptyString;
  kind?: PluginEntrypointKind;
  command: NonEmptyString;
  args?: string[];
  cwd?: string | null;
  environment?: Record<string, string>;
};

export type PluginContributions = {
  providers?: AcpProviderContribution[];
  runtimes?: RuntimeContribution[];
  commands?: CommandContribution[];
  collectors?: CollectorContribution[];
  observers?: ObserverContribution[];
  ui_surfaces?: UiSurfaceContribution[];
  templates?: WorkbenchTemplateContribution[];
  toolbar_actions?: WorkbenchToolbarActionContribution[];
  artifact_renderers?: ArtifactRendererContribution[];
  card_renderers?: WorkbenchCardRendererContribution[];
  detail_sections?: WorkbenchSectionContribution[];
  review_sections?: WorkbenchSectionContribution[];
};

export type NamedEntrypointContribution = {
  id: NonEmptyString;
  name: NonEmptyString;
  description?: string | null;
  entrypoint?: string | null;
};

export type AcpProviderContribution = NamedEntrypointContribution & {
  capabilities?: ProviderCapability[];
};

export type ProviderCapability = "acp.v1" | (string & {});

export type RuntimeContribution = NamedEntrypointContribution & {
  capabilities?: string[];
};

export type CommandContribution = {
  id: NonEmptyString;
  title: NonEmptyString;
  description?: string | null;
  category?: string | null;
  entrypoint?: string | null;
};

export type ApprovedCtxActionId =
  | "work.focus"
  | "task.start"
  | "ctx.command.run"
  | "plugin.command.run"
  | "work.export_redact"
  | "artifact.attach"
  | "note.attest"
  | "gate.update"
  | "provider.settings.open"
  | "provider.session.restart";

export type CollectorContribution = NamedEntrypointContribution & {
  events?: string[];
};

export type ImporterContribution = CollectorContribution;

export type ObserverContribution = NamedEntrypointContribution & {
  events?: string[];
};

export type UiSurfaceKind =
  | "panel"
  | "sidebar"
  | "status_bar"
  | "command_palette"
  | "settings";

export type UiSurfaceContribution = NamedEntrypointContribution & {
  surface: UiSurfaceKind;
  contexts?: string[];
};

export type DeclarativeWorkbenchContribution = {
  id: NonEmptyString;
  name: NonEmptyString;
  description?: string | null;
  contexts?: string[];
  data_sources?: string[];
};

export type WorkbenchTemplateContribution = DeclarativeWorkbenchContribution & {
  title: NonEmptyString;
  template: NonEmptyString;
};

export type WorkbenchToolbarActionContribution =
  DeclarativeWorkbenchContribution & {
    title: NonEmptyString;
    command?: NonEmptyString;
    action?: ApprovedCtxActionId;
    icon?: string | null;
  };

export type ArtifactRendererContribution = DeclarativeWorkbenchContribution & {
  artifact_types: NonEmptyString[];
  renderer: NonEmptyString;
};

export type WorkbenchCardRendererContribution =
  DeclarativeWorkbenchContribution & {
    card: NonEmptyString;
    renderer: NonEmptyString;
  };

export type WorkbenchSectionContribution = DeclarativeWorkbenchContribution & {
  section: NonEmptyString;
  renderer: NonEmptyString;
};

export type DeferredContributionKind =
  | "redaction_processor"
  | "export_processor";

export type DeferredContribution<TKind extends DeferredContributionKind> = {
  kind: TKind;
  status: "deferred";
  reason: NonEmptyString;
  contract?: string;
};

export type DeferredContributionCatalog = {
  redaction_processors?: DeferredContribution<"redaction_processor">[];
  export_processors?: DeferredContribution<"export_processor">[];
};

export type PluginCompatibility = {
  min_ctx_version?: string | null;
  capabilities?: string[];
};

export type PluginDefinition = CtxPluginManifest;

export type ApprovedImporterActionRequest = {
  contribution_id: NonEmptyString;
  actions: ApprovedCtxActionId[];
};

export type PluginDiagnosticSeverity = "error" | "warning";

export type PluginDefinitionDiagnostic = {
  severity: PluginDiagnosticSeverity;
  code: string;
  message: string;
  path: string;
};

export type PluginValidationResult =
  | { ok: true; diagnostics: PluginDefinitionDiagnostic[] }
  | { ok: false; diagnostics: PluginDefinitionDiagnostic[] };
