import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { validateJsonSchemas } from "./validate-json-schemas.mjs";

test("compiles schemas with local relative refs resolved through $id", async () => {
  await withTempSchemas(async (dir) => {
    const defsDir = path.join(dir, "defs");
    await mkdir(defsDir);

    const root = path.join(dir, "root.schema.json");
    const child = path.join(defsDir, "child.schema.json");
    await writeJson(root, {
      $schema: "https://json-schema.org/draft/2020-12/schema",
      $id: "https://schemas.example.test/root.schema.json",
      title: "Root",
      $ref: "./defs/child.schema.json#/$defs/Child",
    });
    await writeJson(child, {
      $schema: "https://json-schema.org/draft/2020-12/schema",
      $id: "https://schemas.example.test/defs/child.schema.json",
      title: "Child",
      $defs: {
        Child: {
          type: "object",
          properties: {
            name: { type: "string" },
          },
        },
      },
    });

    const result = validateJsonSchemas([root, child]);

    assert.equal(result.schemaCount, 2);
  });
});

test("rejects local relative refs that are not loaded", async () => {
  await withTempSchemas(async (dir) => {
    const root = path.join(dir, "root.schema.json");
    await writeJson(root, {
      $schema: "https://json-schema.org/draft/2020-12/schema",
      $id: "https://schemas.example.test/root.schema.json",
      $ref: "./missing.schema.json",
    });

    assert.throws(
      () => validateJsonSchemas([root]),
      /local \$ref "\.\/missing\.schema\.json" resolves to .*missing\.schema\.json, which is not one of the loaded schemas/u,
    );
  });
});

test("rejects local relative refs whose target $id does not match URI resolution", async () => {
  await withTempSchemas(async (dir) => {
    const root = path.join(dir, "root.schema.json");
    const child = path.join(dir, "child.schema.json");
    await writeJson(root, {
      $schema: "https://json-schema.org/draft/2020-12/schema",
      $id: "https://schemas.example.test/root.schema.json",
      $ref: "./child.schema.json",
    });
    await writeJson(child, {
      $schema: "https://json-schema.org/draft/2020-12/schema",
      $id: "https://schemas.example.test/other/child.schema.json",
      type: "object",
    });

    assert.throws(
      () => validateJsonSchemas([root, child]),
      /resolves by \$id to https:\/\/schemas\.example\.test\/child\.schema\.json, but .*child\.schema\.json declares https:\/\/schemas\.example\.test\/other\/child\.schema\.json/u,
    );
  });
});

test("rejects invalid ref fragments during AJV compilation", async () => {
  await withTempSchemas(async (dir) => {
    const root = path.join(dir, "root.schema.json");
    const child = path.join(dir, "child.schema.json");
    await writeJson(root, {
      $schema: "https://json-schema.org/draft/2020-12/schema",
      $id: "https://schemas.example.test/root.schema.json",
      $ref: "./child.schema.json#/$defs/Missing",
    });
    await writeJson(child, {
      $schema: "https://json-schema.org/draft/2020-12/schema",
      $id: "https://schemas.example.test/child.schema.json",
      $defs: {
        Present: { type: "string" },
      },
    });

    assert.throws(
      () => validateJsonSchemas([root, child]),
      /failed to compile schema: can't resolve reference \.\/child\.schema\.json#\/\$defs\/Missing/u,
    );
  });
});

async function withTempSchemas(run) {
  const dir = await mkdtemp(path.join(os.tmpdir(), "ctx-schema-validator-"));
  try {
    await run(dir);
  } finally {
    await rm(dir, { force: true, recursive: true });
  }
}

async function writeJson(file, value) {
  await writeFile(file, `${JSON.stringify(value, null, 2)}\n`);
}
