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
  workbench_templates: [
    deferredContribution(
      "workbench_template",
      "Workbench templates are deferred because they are not present in the current v1 manifest schema.",
    ),
  ],
  artifact_renderers: [
    deferredContribution(
      "artifact_renderer",
      "Artifact renderers are deferred because renderer runtime semantics are not present in the current v1 manifest schema.",
    ),
  ],
  card_sections: [
    deferredContribution(
      "card_section",
      "Card section rendering is deferred until host section lifecycle and data binding semantics are fixed.",
    ),
  ],
  detail_sections: [
    deferredContribution(
      "detail_section",
      "Detail section rendering is deferred until host section lifecycle and data binding semantics are fixed.",
    ),
  ],
  review_sections: [
    deferredContribution(
      "review_section",
      "Review section rendering is deferred until the host owns section lifecycle and data binding semantics.",
    ),
  ],
  toolbar_actions: [
    deferredContribution(
      "toolbar_action",
      "Toolbar action placement is deferred until host action provenance and permission prompts are defined.",
    ),
  ],
});

export default reviewPanelCommandPlugin;
