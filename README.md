# ctx

Search your agent history.

ctx makes local agent session history searchable.

It indexes existing provider transcripts into a local SQLite store so future
agents can find prior discussions, commands, attempts, files, and notes that
may explain earlier decisions instead of starting from zero. The first user is
an agent calling the CLI, with humans using the same commands when they need to
inspect the store.

## Scope

The current CLI scope is deliberately narrow:

- local agent history indexing under `~/.ctx`;
- read-only discovery of provider history locations;
- explicit imports into SQLite;
- search over indexed sessions and events;
- deterministic context bundles with citations back to source records;
- JSON output for agent-facing commands;
- no writes into source repositories during setup or import;
- no account, service, UI server, shell integration, or background process.

ctx does not infer conclusions for you. `ctx context` returns selected matches,
metadata, snippets, and citations. An agent may use that material to write its
own synthesis, but ctx itself is a retrieval tool.

## What ctx Is Not

- Not an autonomous memory system.
- Not an LLM summarizer.
- Not a git or GitHub wrapper.
- Not a browser UI product.
- Not a remote sync service in this release.

## Install Or Run

Build from this checkout:

```bash
cargo build -p ctx
cargo install --path crates/ctx-cli
```

Run from source while developing:

```bash
cargo run -p ctx -- status
cargo run -p ctx -- search "retry handling"
```

## Quick Start

Create the local store and discover provider history:

```bash
ctx setup
ctx status
ctx sources
```

Index local history explicitly:

```bash
ctx import
ctx import --provider codex
ctx import --path ~/.codex/sessions
```

Search and inspect results:

```bash
ctx list
ctx search "checkout retry"
ctx show <session-or-record-id>
ctx context "checkout retry"
```

Use JSON for agent workflows:

```bash
ctx sources --json
ctx search "sqlite migration" --json
ctx context "sqlite migration" --json
```

## Public CLI

The current CLI command surface is:

```text
ctx setup
ctx status
ctx import
ctx sources
ctx list
ctx show <session-or-record-id>
ctx search <query>
ctx context <query>
ctx doctor
ctx validate
```

Agent-facing commands support `--json` where structured output is useful:

```text
ctx status --json
ctx sources --json
ctx import --json
ctx list --json
ctx show <session-or-record-id> --json
ctx search <query> --json
ctx context <query> --json
ctx doctor --json
```

## Data Model

ctx indexes provider history as sessions and events. An event may be a user
message, assistant message, tool call, command, command output preview, file
reference, lifecycle marker, or provider-specific metadata.

Search results include stable IDs for `ctx show`, provider names, timestamps,
repository or working-directory metadata when known, snippets, match reasons,
and source citations. Raw provider transcript files stay in provider-owned
locations such as `~/.codex/sessions`; ctx stores the searchable metadata and
text it needs in SQLite under `~/.ctx`.

See [docs/storage.md](docs/storage.md) for exact storage and privacy behavior.

## Docs

- [Getting started](docs/getting-started.md)
- [CLI reference](docs/cli-reference.md)
- [Storage and privacy](docs/storage.md)
- [Providers](docs/providers.md)
- [Search and context](docs/search.md)
- [Agent usage](docs/agent-usage.md)
- [Troubleshooting](docs/troubleshooting.md)

## Design Principles

- Prefer explicit imports over ambient collection.
- Keep raw provider ownership clear.
- Preserve citations so agents can verify retrieved material.
- Keep output deterministic for the same database, query, and options.
- Treat the local ctx data root as private developer history.
