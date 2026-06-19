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
  },
});
```

The SDK is manifest-first and intentionally narrow. It types supported
manifest contributions and exposes deferred contribution markers for areas that
are named in the contribution contract but do not yet have runtime execution
semantics. Deferred markers and importer action requests are sidecar SDK values,
not additional manifest schema fields.
