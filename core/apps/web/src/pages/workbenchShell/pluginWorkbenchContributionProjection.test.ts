import { describe, expect, it } from "vitest";
import type { PluginExtensionRegistry } from "@ctx/types";
import {
  isWorkbenchPluginTemplateId,
  parseWorkbenchPluginTemplateId,
  projectPluginWorkbenchDeclarativeContributions,
  projectPluginWorkbenchContributions,
  projectWorkbenchDeclarativeContributionProjection,
  projectWorkbenchContributionProjection,
  toWorkbenchPluginTemplateId,
} from "./pluginWorkbenchContributionProjection";

describe("pluginWorkbenchContributionProjection", () => {
  it("projects plugin UI surfaces into source-labeled Workbench candidates", () => {
    const registry: PluginExtensionRegistry = {
      revision: 7,
      ui_surfaces: [
        {
          plugin_id: "zeta.tools",
          plugin_name: "Zeta Tools",
          plugin_version: "0.2.0",
          plugin_path: "/plugins/zeta/ctx-plugin.json",
          plugin_revision: "rev-z",
          contribution: {
            id: "review",
            name: "Review Panel",
            surface: "panel",
            description: "Shows review context.",
            entrypoint: "main",
            contexts: ["review", "review", "  change-set  ", ""],
          },
        },
        {
          plugin_id: "alpha.tools",
          plugin_name: "Alpha Tools",
          plugin_version: "0.1.0",
          plugin_path: "/plugins/alpha/ctx-plugin.json",
          plugin_revision: "rev-a",
          contribution: {
            id: "status",
            name: "Status Strip",
            surface: "status_bar",
          },
        },
      ],
    };

    expect(projectPluginWorkbenchContributions(registry)).toEqual([
      {
        id: "plugin:zeta.tools/review",
        contributionId: "review",
        title: "Review Panel",
        description: "Shows review context.",
        surface: "panel",
        contexts: ["review", "change-set"],
        entrypoint: "main",
        source: {
          kind: "plugin",
          pluginId: "zeta.tools",
          pluginName: "Zeta Tools",
          pluginVersion: "0.2.0",
          pluginPath: "/plugins/zeta/ctx-plugin.json",
          pluginRevision: "rev-z",
        },
        compatibility: { kind: "compatible" },
      },
      {
        id: "plugin:alpha.tools/status",
        contributionId: "status",
        title: "Status Strip",
        description: null,
        surface: "status_bar",
        contexts: [],
        entrypoint: null,
        source: {
          kind: "plugin",
          pluginId: "alpha.tools",
          pluginName: "Alpha Tools",
          pluginVersion: "0.1.0",
          pluginPath: "/plugins/alpha/ctx-plugin.json",
          pluginRevision: "rev-a",
        },
        compatibility: { kind: "compatible" },
      },
    ]);
  });

  it("filters malformed records and marks non-Workbench surfaces as unsupported", () => {
    const registry: PluginExtensionRegistry = {
      revision: 1,
      ui_surfaces: [
        {
          plugin_id: "broken.tools",
          plugin_name: "Broken Tools",
          plugin_version: "0.1.0",
          plugin_path: "/plugins/broken/ctx-plugin.json",
          contribution: {
            id: "broken",
            name: "   ",
            surface: "panel",
          },
        },
        {
          plugin_id: "palette.tools",
          plugin_name: "Palette Tools",
          plugin_version: "0.1.0",
          plugin_path: "/plugins/palette/ctx-plugin.json",
          contribution: {
            id: "search",
            name: "Search Command",
            surface: "command_palette",
          },
        },
      ],
    };

    expect(projectPluginWorkbenchContributions(registry)).toEqual([
      expect.objectContaining({
        id: "plugin:palette.tools/search",
        compatibility: { kind: "unsupported_surface", surface: "command_palette" },
      }),
    ]);
  });

  it("projects declarative Workbench buckets as source-labeled inert candidates", () => {
    const registry = {
      revision: 9,
      templates: [
        {
          plugin_id: "review.tools",
          plugin_name: "Review Tools",
          plugin_version: "0.3.0",
          plugin_path: "/plugins/review/ctx-plugin.json",
          plugin_revision: "rev-review",
          contribution: {
            id: "summary",
            name: "Review Summary Template",
            title: "Review Summary",
            template: "review",
            contexts: ["review", " review ", ""],
            data_sources: ["change-set/diff-summary", "change-set/diff-summary"],
          },
        },
      ],
      toolbar_actions: [
        {
          plugin_id: "review.tools",
          plugin_name: "Review Tools",
          plugin_version: "0.3.0",
          plugin_path: "/plugins/review/ctx-plugin.json",
          contribution: {
            id: "open",
            name: "Open Review Panel Action",
            title: "Open Review",
            command: "review.open",
            icon: "panel-top",
            contexts: ["review"],
          },
        },
      ],
      artifact_renderers: [
        {
          plugin_id: "review.tools",
          plugin_name: "Review Tools",
          plugin_version: "0.3.0",
          plugin_path: "/plugins/review/ctx-plugin.json",
          contribution: {
            id: "patch",
            name: "Patch Artifact Renderer",
            artifact_types: ["text/x-diff", " text/x-diff "],
            renderer: "host.diff-artifact",
          },
        },
      ],
      card_renderers: [
        {
          plugin_id: "review.tools",
          plugin_name: "Review Tools",
          plugin_version: "0.3.0",
          plugin_path: "/plugins/review/ctx-plugin.json",
          contribution: {
            id: "finding",
            name: "Finding Card Renderer",
            card: "review.finding",
            renderer: "host.review-finding-card",
          },
        },
      ],
      detail_sections: [
        {
          plugin_id: "review.tools",
          plugin_name: "Review Tools",
          plugin_version: "0.3.0",
          plugin_path: "/plugins/review/ctx-plugin.json",
          contribution: {
            id: "diff",
            name: "Diff Detail Section",
            section: "diff-summary",
            renderer: "host.diff-summary-section",
          },
        },
      ],
      review_sections: [
        {
          plugin_id: "review.tools",
          plugin_name: "Review Tools",
          plugin_version: "0.3.0",
          plugin_path: "/plugins/review/ctx-plugin.json",
          contribution: {
            id: "gate",
            name: "Gate Review Section",
            section: "gate-state",
            renderer: "host.gate-state-section",
          },
        },
      ],
    } as unknown as PluginExtensionRegistry;

    const projection = projectWorkbenchDeclarativeContributionProjection({
      loadState: { kind: "ready" },
      registry,
    });

    expect(projection.kind).toBe("ready");
    expect(projection.buckets.templates[0]).toMatchObject({
      id: "templates:review.tools/summary",
      title: "Review Summary",
      template: "review",
      contexts: ["review"],
      dataSources: ["change-set/diff-summary"],
      source: {
        pluginId: "review.tools",
        pluginName: "Review Tools",
        pluginVersion: "0.3.0",
        pluginPath: "/plugins/review/ctx-plugin.json",
        pluginRevision: "rev-review",
        label: "Review Tools 0.3.0",
      },
      compatibility: { kind: "compatible" },
    });
    expect(projection.buckets.toolbarActions[0]).toMatchObject({
      id: "toolbar_actions:review.tools/open",
      title: "Open Review",
      icon: "panel-top",
      intent: { kind: "plugin_command", command: "review.open" },
    });
    expect(projection.buckets.artifactRenderers[0]).toMatchObject({
      artifactTypes: ["text/x-diff"],
      renderer: "host.diff-artifact",
    });
    expect(projection.buckets.cardRenderers[0]).toMatchObject({
      card: "review.finding",
      renderer: "host.review-finding-card",
    });
    expect(projection.buckets.detailSections[0]).toMatchObject({
      section: "diff-summary",
      renderer: "host.diff-summary-section",
    });
    expect(projection.buckets.reviewSections[0]).toMatchObject({
      section: "gate-state",
      renderer: "host.gate-state-section",
    });
    expect(projection.buckets.toolbarActions[0]).toEqual(
      JSON.parse(JSON.stringify(projection.buckets.toolbarActions[0])),
    );
  });

  it("filters malformed declarative records and marks unsafe metadata invalid", () => {
    const registry = {
      revision: 1,
      templates: [
        {
          plugin_id: "broken.tools",
          plugin_name: "Broken Tools",
          plugin_version: "0.1.0",
          plugin_path: "/plugins/broken/ctx-plugin.json",
          contribution: {
            id: "blank-title",
            name: "Blank Title",
            title: "   ",
            template: "review",
          },
        },
        {
          plugin_id: "missing-meta.tools",
          plugin_name: "Missing Metadata Tools",
          plugin_version: " ",
          plugin_path: "/plugins/missing/ctx-plugin.json",
          contribution: {
            id: "review-template",
            name: "Review Template",
            title: "Review",
            template: "review",
          },
        },
      ],
      artifact_renderers: [
        {
          plugin_id: "broken.tools",
          plugin_name: "Broken Tools",
          plugin_version: "0.1.0",
          plugin_path: "/plugins/broken/ctx-plugin.json",
          contribution: {
            id: "missing-artifact-type",
            name: "Missing Artifact Type",
            artifact_types: [],
            renderer: "host.diff-artifact",
          },
        },
      ],
    } as unknown as PluginExtensionRegistry;

    expect(projectPluginWorkbenchDeclarativeContributions(registry)).toEqual([
      expect.objectContaining({
        id: "templates:missing-meta.tools/review-template",
        compatibility: { kind: "invalid", reasons: ["missing_plugin_version"] },
      }),
    ]);
  });

  it("marks unknown declarative template and renderer IDs as unsupported data", () => {
    const registry = {
      revision: 1,
      templates: [
        {
          plugin_id: "review.tools",
          plugin_name: "Review Tools",
          plugin_version: "0.3.0",
          plugin_path: "/plugins/review/ctx-plugin.json",
          contribution: {
            id: "summary",
            name: "Review Summary Template",
            title: "Review Summary",
            template: "custom-review-template",
          },
        },
      ],
      review_sections: [
        {
          plugin_id: "review.tools",
          plugin_name: "Review Tools",
          plugin_version: "0.3.0",
          plugin_path: "/plugins/review/ctx-plugin.json",
          contribution: {
            id: "custom",
            name: "Custom Review Section",
            section: "custom",
            renderer: "plugin.custom-renderer",
          },
        },
      ],
    } as unknown as PluginExtensionRegistry;

    expect(projectPluginWorkbenchDeclarativeContributions(registry)).toEqual([
      expect.objectContaining({
        bucket: "review_sections",
        renderer: "plugin.custom-renderer",
        compatibility: { kind: "unsupported_renderer", renderer: "plugin.custom-renderer" },
      }),
      expect.objectContaining({
        bucket: "templates",
        template: "custom-review-template",
        compatibility: { kind: "unsupported_template", template: "custom-review-template" },
      }),
    ]);
  });

  it("returns safe declarative fallback states while the registry loads or fails", () => {
    expect(projectWorkbenchDeclarativeContributionProjection({ loadState: { kind: "loading" } })).toEqual({
      kind: "loading",
      candidates: [],
      buckets: {
        templates: [],
        toolbarActions: [],
        artifactRenderers: [],
        cardRenderers: [],
        detailSections: [],
        reviewSections: [],
      },
      fallback: { kind: "registry_loading" },
    });

    expect(
      projectWorkbenchDeclarativeContributionProjection({
        loadState: { kind: "error", message: "registry unavailable" },
      }),
    ).toEqual({
      kind: "error",
      message: "registry unavailable",
      candidates: [],
      buckets: {
        templates: [],
        toolbarActions: [],
        artifactRenderers: [],
        cardRenderers: [],
        detailSections: [],
        reviewSections: [],
      },
      fallback: { kind: "registry_error", message: "registry unavailable" },
    });

    expect(projectWorkbenchDeclarativeContributionProjection({ loadState: { kind: "ready" } })).toEqual({
      kind: "empty",
      candidates: [],
      buckets: {
        templates: [],
        toolbarActions: [],
        artifactRenderers: [],
        cardRenderers: [],
        detailSections: [],
        reviewSections: [],
      },
      fallback: null,
    });
  });

  it("resolves loading, empty, error, ready, and fallback projection states", () => {
    expect(projectWorkbenchContributionProjection({ loadState: { kind: "loading" } })).toMatchObject({
      kind: "loading",
      candidates: [],
      activeCandidate: null,
      fallback: null,
      effectiveTemplateId: "classic",
    });

    expect(projectWorkbenchContributionProjection({ loadState: { kind: "ready" } })).toEqual({
      kind: "empty",
      candidates: [],
      activeCandidate: null,
      fallback: null,
      effectiveTemplateId: "classic",
    });

    expect(
      projectWorkbenchContributionProjection({
        loadState: { kind: "error", message: "registry unavailable" },
        activeTemplateId: "plugin:removed.tools/review",
      }),
    ).toMatchObject({
      kind: "error",
      message: "registry unavailable",
      fallback: {
        kind: "unavailable",
        requestedTemplateId: "plugin:removed.tools/review",
        fallbackTemplateId: "classic",
        reason: "error",
      },
      effectiveTemplateId: "classic",
    });
  });

  it("keeps a compatible active plugin template selected as data only", () => {
    const registry: PluginExtensionRegistry = {
      revision: 1,
      ui_surfaces: [
        {
          plugin_id: "review/tools",
          plugin_name: "Review Tools",
          plugin_version: "0.1.0",
          plugin_path: "/plugins/review/ctx-plugin.json",
          contribution: {
            id: "panel/main",
            name: "Review Panel",
            surface: "panel",
            entrypoint: "main",
          },
        },
      ],
    };

    const projection = projectWorkbenchContributionProjection({
      loadState: { kind: "ready" },
      registry,
      activeTemplateId: "plugin:review%2Ftools/panel%2Fmain",
    });

    expect(projection).toMatchObject({
      kind: "ready",
      effectiveTemplateId: "plugin:review%2Ftools/panel%2Fmain",
      fallback: null,
      activeCandidate: {
        id: "plugin:review%2Ftools/panel%2Fmain",
        entrypoint: "main",
      },
    });
  });

  it("falls back when a persisted plugin template no longer exists in the registry", () => {
    const projection = projectWorkbenchContributionProjection({
      loadState: { kind: "ready" },
      registry: { revision: 3, ui_surfaces: [] },
      activeTemplateId: "plugin:removed%2Ftools/review%2Fpanel",
    });

    expect(projection).toEqual({
      kind: "fallback",
      candidates: [],
      activeCandidate: null,
      fallback: {
        kind: "removed_plugin",
        requestedTemplateId: "plugin:removed%2Ftools/review%2Fpanel",
        fallbackTemplateId: "classic",
        pluginId: "removed/tools",
        contributionId: "review/panel",
      },
      effectiveTemplateId: "classic",
    });
  });

  it("falls back when the active plugin surface is not Workbench-compatible", () => {
    const registry: PluginExtensionRegistry = {
      revision: 1,
      ui_surfaces: [
        {
          plugin_id: "settings.tools",
          plugin_name: "Settings Tools",
          plugin_version: "0.1.0",
          plugin_path: "/plugins/settings/ctx-plugin.json",
          contribution: {
            id: "prefs",
            name: "Plugin Preferences",
            surface: "settings",
          },
        },
      ],
    };

    expect(
      projectWorkbenchContributionProjection({
        loadState: { kind: "ready" },
        registry,
        activeTemplateId: "plugin:settings.tools/prefs",
      }),
    ).toMatchObject({
      kind: "fallback",
      fallback: {
        kind: "unavailable",
        reason: "incompatible",
        requestedTemplateId: "plugin:settings.tools/prefs",
      },
      effectiveTemplateId: "classic",
    });
  });

  it("round-trips plugin-qualified Workbench IDs", () => {
    const id = toWorkbenchPluginTemplateId("review/tools", "panel/main");

    expect(id).toBe("plugin:review%2Ftools/panel%2Fmain");
    expect(isWorkbenchPluginTemplateId(id)).toBe(true);
    expect(parseWorkbenchPluginTemplateId(id)).toEqual({
      pluginId: "review/tools",
      contributionId: "panel/main",
    });
  });

  it("rejects malformed plugin-qualified Workbench IDs", () => {
    expect(isWorkbenchPluginTemplateId("plugin:review.tools/panel/extra")).toBe(false);
    expect(isWorkbenchPluginTemplateId("plugin:review.tools")).toBe(false);
    expect(isWorkbenchPluginTemplateId("plugin:%E0%A4%A/panel")).toBe(false);
  });
});
