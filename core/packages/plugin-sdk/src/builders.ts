import type {
  AcpProviderContribution,
  ApprovedImporterActionRequest,
  CollectorContribution,
  CommandContribution,
  CtxPluginManifest,
  DeferredContribution,
  DeferredContributionKind,
  ImporterContribution,
  ObserverContribution,
  PluginDefinition,
  PluginEntrypoint,
  RuntimeContribution,
  UiSurfaceContribution,
  DeferredContributionCatalog,
} from "./types.js";

export function defineCtxPlugin<const TPlugin extends PluginDefinition>(
  plugin: TPlugin,
): TPlugin {
  return plugin;
}

export function defineCtxPluginManifest<const TManifest extends CtxPluginManifest>(
  manifest: TManifest,
): TManifest {
  return manifest;
}

export function entrypoint<const TEntrypoint extends PluginEntrypoint>(
  contribution: TEntrypoint,
): TEntrypoint {
  return contribution;
}

export function command<const TCommand extends CommandContribution>(
  contribution: TCommand,
): TCommand {
  return contribution;
}

export function acpProvider<const TProvider extends AcpProviderContribution>(
  contribution: TProvider,
): TProvider & { capabilities: string[] } {
  const capabilities = new Set(contribution.capabilities ?? []);
  capabilities.add("acp.v1");
  return {
    ...contribution,
    capabilities: Array.from(capabilities),
  };
}

export function runtime<const TRuntime extends RuntimeContribution>(
  contribution: TRuntime,
): TRuntime {
  return contribution;
}

export function collector<const TCollector extends CollectorContribution>(
  contribution: TCollector,
): TCollector {
  return contribution;
}

export function importer<const TImporter extends ImporterContribution>(
  contribution: TImporter,
): TImporter {
  return contribution;
}

export function observer<const TObserver extends ObserverContribution>(
  contribution: TObserver,
): TObserver {
  return contribution;
}

export function uiSurface<const TSurface extends UiSurfaceContribution>(
  contribution: TSurface,
): TSurface {
  return contribution;
}

export function reviewPanelSurface<const TSurface extends Omit<UiSurfaceContribution, "surface">>(
  contribution: TSurface,
): TSurface & { surface: "panel" } {
  return {
    ...contribution,
    surface: "panel",
  };
}

export function deferredContribution<const TKind extends DeferredContributionKind>(
  kind: TKind,
  reason: string,
): DeferredContribution<TKind> {
  return {
    kind,
    status: "deferred",
    reason,
    contract: "docs/plugin-contribution-contract.mdx",
  };
}

export function defineDeferredContributions<
  const TDeferred extends DeferredContributionCatalog,
>(deferred: TDeferred): TDeferred {
  return deferred;
}

export function approvedImporterActions<
  const TRequest extends ApprovedImporterActionRequest,
>(request: TRequest): TRequest {
  return request;
}
