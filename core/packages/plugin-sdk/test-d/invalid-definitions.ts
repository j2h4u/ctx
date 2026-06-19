import {
  command,
  defineDeferredContributions,
  defineCtxPlugin,
  deferredContribution,
  entrypoint,
} from "../src/index.js";

defineCtxPlugin({
  id: "example.invalid",
  name: "Invalid",
  version: "0.1.0",
  contributes: {
    commands: [
      // @ts-expect-error command title is required by the manifest contract.
      command({ id: "example.invalid.missing-title" }),
    ],
  },
});

defineCtxPlugin({
  id: "example.invalid-entrypoint",
  name: "Invalid Entrypoint",
  version: "0.1.0",
  entrypoints: [
    // @ts-expect-error entrypoint command is required by the manifest contract.
    entrypoint({ id: "main" }),
  ],
});

defineDeferredContributions({
  review_sections: [
    // @ts-expect-error review_sections only accepts review_section deferred markers.
    deferredContribution("toolbar_action", "Wrong deferred bucket."),
  ],
});
