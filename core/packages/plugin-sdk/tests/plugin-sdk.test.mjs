import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

import acpProviderPlugin from "../dist/examples/acp-provider-plugin.js";
import reviewPanelCommandPlugin, {
  deferredReviewPluginContributions,
  reviewImporterActions,
} from "../dist/examples/review-panel-command-plugin.js";
import {
  assertValidCtxPlugin,
  command,
  defineCtxPlugin,
  formatPluginDiagnostics,
  validateCtxPlugin,
  validateCtxPluginSet,
} from "../dist/src/index.js";

test("review panel and command example compiles and validates", () => {
  const result = validateCtxPlugin(reviewPanelCommandPlugin);

  assert.equal(result.ok, true, formatPluginDiagnostics(result.diagnostics));
  assert.equal(
    reviewPanelCommandPlugin.contributes.ui_surfaces[0].surface,
    "panel",
  );
  assert.deepEqual(reviewImporterActions.actions, [
    "plugin.command.run",
    "note.attest",
  ]);
  assert.equal(
    deferredReviewPluginContributions.review_sections[0].status,
    "deferred",
  );
  assert.equal(
    deferredReviewPluginContributions.artifact_renderers[0].kind,
    "artifact_renderer",
  );
});

test("ACP provider example compiles and validates against local stdio target", () => {
  assert.equal(validateCtxPlugin(acpProviderPlugin).ok, true);
  assert.deepEqual(acpProviderPlugin.contributes.providers[0].capabilities, [
    "acp.v1",
  ]);
  assert.ok(
    acpProviderPlugin.compatibility.capabilities.includes("acp.v1.local-stdio"),
  );
});

test("ACP provider JSON manifest fixture validates against current SDK rules", async () => {
  const raw = await readFile(
    new URL("../examples/fixtures/acp-provider-plugin.json", import.meta.url),
    "utf8",
  );
  const manifest = JSON.parse(raw);

  assert.equal(validateCtxPlugin(manifest).ok, true);
});

test("invalid plugin definitions return actionable diagnostics", () => {
  const invalid = defineCtxPlugin({
    id: "example.invalid",
    name: "Invalid",
    version: "0.1.0",
    entrypoints: [{ id: "main", command: "node" }],
    contributes: {
      commands: [
        command({
          id: "open",
          title: "Open",
          entrypoint: "missing",
        }),
      ],
    },
  });

  const result = validateCtxPlugin(invalid);

  assert.equal(result.ok, false);
  assert.deepEqual(
    result.diagnostics.map((diagnostic) => diagnostic.code),
    ["command_id_not_namespaced", "unknown_entrypoint"],
  );
  assert.match(
    formatPluginDiagnostics(result.diagnostics),
    /contributes\.commands\[0\]\.id/,
  );
});

test("assertValidCtxPlugin throws diagnostics for invalid fixtures", () => {
  assert.throws(
    () =>
      assertValidCtxPlugin({
        id: "example.empty",
        name: "Empty",
        version: "0.1.0",
      }),
    /empty_plugin_definition/,
  );
});

test("plugin ids and provider ids are hard conflicts across a plugin set", () => {
  const first = defineCtxPlugin({
    id: "example.provider-a",
    name: "Provider A",
    version: "0.1.0",
    contributes: {
      providers: [{ id: "example-agent", name: "Example Agent" }],
    },
  });
  const second = defineCtxPlugin({
    id: "example.provider-a",
    name: "Provider B",
    version: "0.1.0",
    contributes: {
      providers: [{ id: "example-agent", name: "Example Agent B" }],
    },
  });

  const result = validateCtxPluginSet([first, second]);

  assert.equal(result.ok, false);
  assert.deepEqual(
    result.diagnostics.map((diagnostic) => diagnostic.code),
    ["duplicate_plugin_id", "duplicate_provider_id"],
  );
});

test("collector/importer definitions cannot declare direct store writes", () => {
  const result = validateCtxPlugin({
    id: "example.importer",
    name: "Importer",
    version: "0.1.0",
    contributes: {
      collectors: [
        {
          id: "example.importer.collect",
          name: "Collect",
          store_writes: ["agent_work"],
        },
      ],
    },
  });

  assert.equal(result.ok, false);
  assert.ok(
    result.diagnostics.some(
      (diagnostic) => diagnostic.code === "collector_store_write_forbidden",
    ),
  );
  assert.ok(
    result.diagnostics.some(
      (diagnostic) => diagnostic.code === "unknown_manifest_property",
    ),
  );
});

test("malformed JSON-like manifests return diagnostics instead of throwing", () => {
  const result = validateCtxPlugin({
    id: "example.malformed",
    name: "Malformed",
    version: "0.1.0",
    entrypoints: {
      id: "main",
      command: "node",
    },
    contributes: {
      commands: "not-a-list",
      ui_surfaces: [
        {
          id: "panel",
          name: "Panel",
          surface: "floating-window",
          contexts: ["review", ""],
        },
      ],
    },
    compatibility: {
      capabilities: "acp.v1",
    },
  });

  assert.equal(result.ok, false);
  assert.deepEqual(
    result.diagnostics.map((diagnostic) => diagnostic.code),
    [
      "expected_array",
      "expected_array",
      "contribution_id_not_plugin_qualified",
      "expected_string",
      "invalid_ui_surface_kind",
      "expected_array",
    ],
  );
});

test("entrypoint runtime fields are JSON-validated", () => {
  const result = validateCtxPlugin({
    id: "example.entrypoint",
    name: "Entrypoint",
    version: "0.1.0",
    entrypoints: [
      {
        id: "main",
        kind: "thread",
        command: "node",
        args: ["script.js", ""],
        environment: {
          OK: "1",
          BAD: 1,
        },
      },
    ],
  });

  assert.equal(result.ok, false);
  assert.deepEqual(
    result.diagnostics.map((diagnostic) => diagnostic.code),
    ["expected_string", "expected_string", "invalid_entrypoint_kind"],
  );
});

test("deferred contribution buckets are rejected when embedded in manifest", () => {
  const result = validateCtxPlugin({
    id: "example.deferred-in-manifest",
    name: "Deferred In Manifest",
    version: "0.1.0",
    contributes: {
      commands: [{ id: "example.deferred-in-manifest.open", title: "Open" }],
      workbench: {
        templates: [],
      },
      artifact_renderers: [],
    },
  });

  assert.equal(result.ok, false);
  assert.deepEqual(
    result.diagnostics
      .filter((diagnostic) => diagnostic.code === "unknown_manifest_property")
      .map((diagnostic) => diagnostic.path),
    ["contributes.workbench", "contributes.artifact_renderers"],
  );
});
