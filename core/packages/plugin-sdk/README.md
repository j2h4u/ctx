# @ctx/plugin-sdk

Repo-local TypeScript helpers for ctx plugin manifests.

This package is local to the ctx repo today. It is shaped so it can become a
publishable package later, but publishing to npm is not part of local validation.

```ts
import { command, defineCtxPlugin, reviewPanelSurface } from "@ctx/plugin-sdk";

export default defineCtxPlugin({
  id: "example.review",
  name: "Example Review",
  version: "0.1.0",
  contributes: {
    commands: [command({ id: "example.review.open", title: "Open Review" })],
    ui_surfaces: [
      reviewPanelSurface({
        id: "example.review.panel",
        name: "Review",
        contexts: ["review"],
      }),
    ],
    review_sections: [
      {
        id: "example.review.gates",
        name: "Gate State",
        section: "gate-state",
        renderer: "host.gate-state-section",
      },
    ],
  },
});
```

The SDK is manifest-first and intentionally narrow. It types supported
manifest contributions, including host-owned declarative Workbench buckets such
as `templates`, `toolbar_actions`, `artifact_renderers`, `card_renderers`,
`detail_sections`, and `review_sections`.

Deferred markers remain sidecar SDK values, not additional manifest schema
fields. Redaction and export processors are deferred sidecars in this slice.
Arbitrary React/webview execution remains deferred or capability-gated.
