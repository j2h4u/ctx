# Provider Support

Provider support is intentionally conservative. A provider is documented as
locally importable only when the public CLI can read existing local history for
that provider.

## Status Meanings

| Status | Meaning |
| --- | --- |
| `local_import` | The CLI can import an existing local history source for this provider. |
| `local_import_when_supported` | The CLI has an importer for a specific local format, but support depends on that file existing and matching the documented format. |
| `fixture_only` | The repository has sanitized fixture coverage, but the public CLI does not discover or import native local history for that provider. |
| `detected_unsupported` | The CLI can detect something about the provider but intentionally does not import it. |
| `blocked` | No shipped discovery or import path exists. |

## Current Matrix

Machine-readable provider metadata lives in
[provider-support-matrix.json](provider-support-matrix.json). The public truth
is:

| Provider | Status | Public import path |
| --- | --- | --- |
| Codex | `local_import` | `~/.codex/sessions`, `~/.codex/history.jsonl`, or an explicit Codex path. |
| Pi | `local_import_when_supported` | `~/.pi/sessions.jsonl` or an explicit Pi JSONL path. |
| Claude | `fixture_only` | No native local importer in the public CLI. |
| OpenCode | `fixture_only` | No native local importer in the public CLI. |
| Antigravity | `fixture_only` | No native local importer in the public CLI. |
| Gemini | `fixture_only` | No native local importer in the public CLI. |
| Cursor | `fixture_only` | No native local importer in the public CLI. |

## Required Evidence For Promotion

Before a provider moves beyond `fixture_only` or `blocked`, the change needs:

- a documented local source format;
- read-only source discovery or an explicit `--path` contract;
- malformed-input tests;
- idempotent re-import tests;
- source citation fields in search/context output;
- storage and redaction notes for provider-specific sensitive fields;
- docs updates in this file and `provider-support-matrix.json`.
