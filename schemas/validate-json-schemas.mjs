import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import Ajv2020 from "../core/node_modules/ajv/dist/2020.js";

const DRAFT_2020_12_SCHEMA = "https://json-schema.org/draft/2020-12/schema";
const URI_SCHEME_RE = /^[A-Za-z][A-Za-z0-9+.-]*:/u;

export function collectSchemaFiles(files) {
  if (!Array.isArray(files) || files.length === 0) {
    throw new Error("expected schema files as argv");
  }

  const seen = new Set();
  return files.map((file) => {
    const displayFile = String(file || "").trim();
    if (!displayFile) {
      throw new Error("schema file arguments must be non-empty");
    }

    const absoluteFile = path.resolve(displayFile);
    let realFile;
    try {
      realFile = fs.realpathSync(absoluteFile);
    } catch (error) {
      throw new Error(`${displayFile}: schema file is not readable: ${errorMessage(error)}`);
    }
    if (seen.has(realFile)) {
      throw new Error(`duplicate schema file argument: ${displayFile}`);
    }
    seen.add(realFile);

    return {
      displayFile,
      file: realFile,
    };
  });
}

export function loadSchemaEntries(files) {
  const idToEntry = new Map();

  return collectSchemaFiles(files).map((entry) => {
    let schema;
    try {
      schema = JSON.parse(fs.readFileSync(entry.file, "utf8"));
    } catch (error) {
      throw new Error(`${entry.displayFile}: invalid JSON: ${errorMessage(error)}`);
    }

    if (!schema || typeof schema !== "object" || Array.isArray(schema)) {
      throw new Error(`${entry.displayFile}: schema must parse to an object`);
    }
    if (typeof schema.$schema !== "string") {
      throw new Error(`${entry.displayFile}: schema must declare $schema`);
    }
    const schemaUri = normalizeSchemaUriForFile(entry, "$schema", schema.$schema);
    if (schemaUri !== DRAFT_2020_12_SCHEMA) {
      throw new Error(
        `${entry.displayFile}: schema must use ${DRAFT_2020_12_SCHEMA}`,
      );
    }
    if (typeof schema.$id !== "string" || schema.$id.trim() === "") {
      throw new Error(`${entry.displayFile}: schema must declare a non-empty $id`);
    }

    const id = normalizeSchemaUriForFile(entry, "$id", schema.$id);
    if (idToEntry.has(id)) {
      const existing = idToEntry.get(id);
      throw new Error(
        `${entry.displayFile}: duplicate $id ${id} already declared by ${existing.displayFile}`,
      );
    }

    const loaded = {
      ...entry,
      id,
      schema,
    };
    idToEntry.set(id, loaded);
    return loaded;
  });
}

export function validateJsonSchemas(files) {
  const entries = loadSchemaEntries(files);
  assertLocalRelativeRefs(entries);

  const ajv = new Ajv2020({
    allErrors: true,
    strictSchema: true,
    strictTypes: false,
    validateFormats: false,
  });

  for (const entry of entries) {
    if (!ajv.validateSchema(entry.schema)) {
      throw new Error(
        `${entry.displayFile}: invalid JSON Schema:\n${ajv.errorsText(ajv.errors, {
          separator: "\n",
        })}`,
      );
    }

    try {
      ajv.addSchema(entry.schema, entry.id);
    } catch (error) {
      throw new Error(`${entry.displayFile}: failed to load schema: ${errorMessage(error)}`);
    }
  }

  for (const entry of entries) {
    try {
      const validate = ajv.getSchema(entry.id);
      if (typeof validate !== "function") {
        throw new Error(`AJV did not return a validator for ${entry.id}`);
      }
    } catch (error) {
      throw new Error(`${entry.displayFile}: failed to compile schema: ${errorMessage(error)}`);
    }
  }

  return {
    schemaCount: entries.length,
    schemaIds: entries.map((entry) => entry.id).sort(),
  };
}

export function assertLocalRelativeRefs(entries) {
  const byFile = new Map(entries.map((entry) => [entry.file, entry]));

  for (const entry of entries) {
    for (const { pointer, ref } of findRefs(entry.schema)) {
      if (!isLocalRelativeRef(ref)) {
        continue;
      }

      const resolvedFile = resolveLocalRefFile(entry.file, ref);
      const target = byFile.get(resolvedFile);
      if (!target) {
        throw new Error(
          `${entry.displayFile}${pointer}: local $ref ${JSON.stringify(ref)} resolves to ` +
            `${formatPath(resolvedFile)}, which is not one of the loaded schemas`,
        );
      }

      const resolvedId = normalizeSchemaUri(resolveRefAgainstId(entry.id, ref));
      if (resolvedId !== target.id) {
        throw new Error(
          `${entry.displayFile}${pointer}: local $ref ${JSON.stringify(ref)} resolves by $id ` +
            `to ${resolvedId}, but ${target.displayFile} declares ${target.id}`,
        );
      }
    }
  }
}

export function* findRefs(value, pointer = "") {
  if (!value || typeof value !== "object") {
    return;
  }

  if (Array.isArray(value)) {
    for (let index = 0; index < value.length; index += 1) {
      yield* findRefs(value[index], `${pointer}/${index}`);
    }
    return;
  }

  if (typeof value.$ref === "string") {
    yield {
      pointer: `${pointer}/$ref`,
      ref: value.$ref,
    };
  }

  for (const [key, child] of Object.entries(value)) {
    if (key === "$ref") {
      continue;
    }
    yield* findRefs(child, `${pointer}/${escapeJsonPointerSegment(key)}`);
  }
}

export function isLocalRelativeRef(ref) {
  if (typeof ref !== "string") {
    return false;
  }

  const baseRef = ref.split("#", 1)[0];
  return (
    baseRef !== "" &&
    !baseRef.startsWith("/") &&
    !baseRef.startsWith("//") &&
    !URI_SCHEME_RE.test(baseRef)
  );
}

function resolveLocalRefFile(fromFile, ref) {
  const resolvedUrl = new URL(ref, pathToFileURL(fromFile));
  resolvedUrl.hash = "";
  const resolvedPath = fileURLToPath(resolvedUrl);
  return fs.existsSync(resolvedPath) ? fs.realpathSync(resolvedPath) : path.resolve(resolvedPath);
}

function resolveRefAgainstId(id, ref) {
  const resolvedUrl = new URL(ref, id);
  resolvedUrl.hash = "";
  return resolvedUrl.href;
}

function normalizeSchemaUri(uri) {
  let url;
  try {
    url = new URL(uri);
  } catch (error) {
    throw new Error(`invalid absolute schema URI ${JSON.stringify(uri)}: ${errorMessage(error)}`);
  }
  url.hash = "";
  return url.href;
}

function normalizeSchemaUriForFile(entry, field, uri) {
  try {
    return normalizeSchemaUri(uri);
  } catch (error) {
    throw new Error(`${entry.displayFile}: invalid ${field}: ${errorMessage(error)}`);
  }
}

function escapeJsonPointerSegment(segment) {
  return segment.replace(/~/gu, "~0").replace(/\//gu, "~1");
}

function formatPath(file) {
  const relative = path.relative(process.cwd(), file);
  return relative && !relative.startsWith("..") ? relative : file;
}

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}

function isMain() {
  return process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href;
}

if (isMain()) {
  try {
    const result = validateJsonSchemas(process.argv.slice(2));
    console.log(`compiled ${result.schemaCount} JSON schemas with AJV 2020`);
  } catch (error) {
    console.error(errorMessage(error));
    process.exitCode = 1;
  }
}
