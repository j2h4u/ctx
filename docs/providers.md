# Providers

ctx imports existing agent history through provider adapters. Each adapter must
make a narrow, testable claim about the source format it reads and the event
fields it indexes.

## Supported Local Imports

The current CLI imports local history for:

- Codex session JSONL trees under `~/.codex/sessions`;
- Codex `~/.codex/history.jsonl`;
- Pi `~/.pi/sessions.jsonl` when that local file exists and matches the
  supported JSONL format.

Use `ctx sources` for the truth on the current machine:

```bash
ctx sources
ctx sources --json
```

If a provider is not listed by `ctx sources`, the current CLI does not discover
or import that provider's native history.

## Fixture-Only Providers

The repository includes normalized fixtures for Claude, OpenCode,
Antigravity, Gemini, and Cursor provider shapes. Those fixtures are useful for
adapter contracts and tests, but they are not native local importers in the
public CLI.

Do not document one of these providers as locally importable until the CLI can
discover or import that provider's real local history and the provider support
matrix marks the shipped path accordingly.

## Import Rules

Provider imports should be:

- read-only with respect to provider-owned files;
- explicit through `ctx import`;
- safe to interrupt and re-run, using idempotent rescans or provider cursors
  when available;
- idempotent for unchanged source files;
- clear about which fields were indexed and which were left raw-only;
- conservative when a transcript schema is unknown or malformed.

## Fidelity

An imported session may include messages, tool calls, command events, output
previews, file references, parent/child agent relationships, usage metadata, and
lifecycle events. Not every provider exposes every field.

Search and context output must identify the provider and cite the source path or
cursor when available so an agent can verify important details.
