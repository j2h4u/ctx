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
  export_processors: [
    // @ts-expect-error export_processors only accepts export_processor deferred markers.
    deferredContribution("redaction_processor", "Wrong deferred bucket."),
  ],
});

defineCtxPlugin({
  id: "example.invalid-toolbar-targets",
  name: "Invalid Toolbar Targets",
  version: "0.1.0",
  contributes: {
    toolbar_actions: [
      {
        id: "example.invalid-toolbar-targets.command",
        name: "Null Command",
        title: "Open",
        // @ts-expect-error toolbar action command targets must be omitted or strings.
        command: null,
      },
      {
        id: "example.invalid-toolbar-targets.action",
        name: "Null Action",
        title: "Focus",
        // @ts-expect-error toolbar action action targets must be omitted or approved action ids.
        action: null,
      },
    ],
  },
});
