# CLI Reference

ctx is a local CLI for indexing and searching agent session history.

## Setup And Health

```bash
ctx setup
ctx status
ctx status --json
ctx doctor
ctx doctor --json
ctx validate
```

- `setup` creates `~/.ctx`, opens or creates the SQLite store, writes
  `config.toml` when needed, discovers known provider history locations, and
  prints next steps.
- `status` prints the ctx root, database path, configured providers, indexed
  counts, and last import status.
- `doctor` performs environment and store checks intended for humans and agents.
- `validate` checks local configuration, database readability, and imported
  source references.

Setup is local. It does not change shell startup files, install repository
integrations, write into source repos, start background processes, call model
APIs, make network calls, or require API keys.

## Sources

```bash
ctx sources
ctx sources --json
```

`sources` lists provider history locations ctx can import or has already
indexed. A source row should include provider, path, detection status, last
import cursor when known, and a clear unsupported reason when ctx can detect a
tool but cannot safely parse its history yet.

## Import

```bash
ctx import
ctx import --all
ctx import --provider codex
ctx import --provider pi
ctx import --path ~/.codex/sessions
ctx import --resume
ctx import --json
```

`import` indexes provider history into the local SQLite store. It is explicit
and safe to re-run. Current adapters rescan sources idempotently and replace or
skip unchanged indexed rows; adapters that expose stable cursors include them in
source metadata. Import records source metadata, sessions, events, searchable
text, bounded previews, cursors when available, and source paths. It reports
files, bytes, sessions, and events processed so long imports are observable.

The current import path is for provider history. It is not a general archive
restore command.

## List And Show

```bash
ctx list
ctx list --json
ctx show <session-or-record-id>
ctx show <session-or-record-id> --json
```

`list` returns indexed sessions or records with IDs, provider, title or first
message preview, timestamp, repository or working directory when known, and
event counts. `show` expands one indexed item and includes event IDs suitable
for citations and follow-up retrieval.

## Search

```bash
ctx search "build failure"
ctx search "sqlite storage" --provider codex
ctx search "retry handling" --repo checkout --since 60d
ctx search "Buildkite failure" --include-subagents --event-type command_output
ctx search --file crates/foo/src/lib.rs
ctx search "token budget" --json
```

`search` queries indexed sessions and events. Results should include title,
snippet, provider, session ID, event ID or sequence, timestamp, repository or
working directory when known, primary or subagent role when known, source path
and cursor when available, match reason, and a stable ID for `ctx show`.

## Context

```bash
ctx context "checkout retry"
ctx context "checkout retry" --max-tokens 6000
ctx context "checkout retry" --provider codex --since 30d
ctx context "checkout retry" --json
```

`context` returns a deterministic retrieval bundle for agents. It includes the
query, filters, selected sessions/events, snippets, command/tool events where
relevant, provider and repository metadata, and citations back to raw sources.

`context` does not call a model or infer decisions. If a provider transcript
already contains a human- or provider-written synopsis, ctx may index and
return it as quoted source material with citation.

## JSON Contract

JSON output is intended for agents and scripts. Fields should remain stable
within the current CLI scope unless a command marks a nested object as
provisional.

Required JSON output paths:

```text
ctx status --json
ctx sources --json
ctx import --json
ctx list --json
ctx show <session-or-record-id> --json
ctx search [query] --json
ctx context <query> --json
ctx doctor --json
```

JSON responses should include machine-readable IDs, provider names, timestamps
in RFC 3339 when available, local path fields only when useful for citation or
diagnostics, and explicit warnings when a source file has moved or disappeared.
