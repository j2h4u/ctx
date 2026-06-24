# Providers

ctx imports existing agent history through provider adapters. Each adapter must
make a narrow, testable claim about the source format it reads and the event
fields it indexes.

## Supported Providers

The current CLI imports local history for:

- Codex sessions under `~/.codex/sessions` when present;
- Pi sessions when a supported local transcript format is available.

Use `ctx sources` for the truth on the current machine:

```bash
ctx sources
ctx sources --json
```

If a provider is detected but unsupported, ctx should report that status rather
than parsing unknown private files.

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
