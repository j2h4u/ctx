import {
  command,
  approvedImporterActions,
  defineCtxPlugin,
  defineDeferredContributions,
  deferredContribution,
  entrypoint,
  reviewPanelSurface,
} from "../src/index.js";

const reviewPanelCommandPlugin = defineCtxPlugin({
  id: "example.review",
  name: "Example Review Panel",
  version: "0.1.0",
  description: "Example review panel and command plugin manifest.",
  entrypoints: [
    entrypoint({
      id: "main",
      kind: "process",
      command: "node",
      args: ["dist/review-command.js"],
    }),
  ],
  contributes: {
    commands: [
      command({
        id: "example.review.open",
        title: "Open Review Panel",
        category: "Review",
        entrypoint: "main",
      }),
    ],
    collectors: [
      {
        id: "example.review.importer",
        name: "Example Review Importer",
        entrypoint: "main",
        events: ["change-set.updated"],
      },
    ],
    ui_surfaces: [
      reviewPanelSurface({
        id: "example.review.panel",
        name: "Review Panel",
        description: "Review-focused panel surface.",
        entrypoint: "main",
        contexts: ["review", "change-set"],
      }),
    ],
    templates: [
      {
        id: "example.review.summary-template",
        name: "Review Summary Template",
        title: "Review Summary",
        template: "review-summary",
        contexts: ["review"],
        data_sources: ["change-set/diff-summary", "plugin-diagnostics"],
      },
    ],
    toolbar_actions: [
      {
        id: "example.review.open-action",
        name: "Open Review Panel Action",
        title: "Open Review",
        command: "example.review.open",
        contexts: ["review"],
      },
    ],
    artifact_renderers: [
      {
        id: "example.review.patch-artifact-renderer",
        name: "Patch Artifact Renderer",
        artifact_types: ["text/x-diff"],
        renderer: "host.diff-artifact",
        contexts: ["review"],
      },
    ],
    card_renderers: [
      {
        id: "example.review.finding-card-renderer",
        name: "Finding Card Renderer",
        card: "review.finding",
        renderer: "host.review-finding-card",
        data_sources: ["check/gate-state"],
      },
    ],
    detail_sections: [
      {
        id: "example.review.diff-detail-section",
        name: "Diff Detail Section",
        section: "diff-summary",
        renderer: "host.diff-summary-section",
        data_sources: ["change-set/diff-summary"],
      },
    ],
    review_sections: [
      {
        id: "example.review.gate-review-section",
        name: "Gate Review Section",
        section: "gate-state",
        renderer: "host.gate-state-section",
        data_sources: ["check/gate-state"],
      },
    ],
  },
});

export const reviewImporterActions = approvedImporterActions({
  contribution_id: "example.review.importer",
  actions: ["plugin.command.run", "note.attest"],
});

export const deferredReviewPluginContributions = defineDeferredContributions({
  redaction_processors: [
    deferredContribution(
      "redaction_processor",
      "Redaction processor execution is deferred until host redaction provenance and preview semantics are fixed.",
    ),
  ],
  export_processors: [
    deferredContribution(
      "export_processor",
      "Export processor execution is deferred until export permissions and redaction defaults are fixed.",
    ),
  ],
});

export default reviewPanelCommandPlugin;
