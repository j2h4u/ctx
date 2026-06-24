# Getting Started

ctx indexes local agent history so agents can search previous sessions before
they repeat work.

## 1. Build The CLI

```bash
cargo build -p ctx
cargo install --path crates/ctx-cli
```

## 2. Create Local Storage

```bash
ctx setup
ctx status
```

Setup creates `~/.ctx`, initializes SQLite, writes configuration, discovers
known provider history locations, and prints example searches. It does not write
to source repositories or require network access.

## 3. See Available Sources

```bash
ctx sources
ctx sources --json
```

Use this to confirm which provider histories were found and which ones need an
explicit path.

## 4. Import History

```bash
ctx import
ctx import --provider codex
ctx import --path ~/.codex/sessions
```

Imports are resumable. Re-running import should continue from stored cursors or
dedupe already indexed sessions.

## 5. Search

```bash
ctx search "failed migration"
ctx search "failed migration" --json
ctx show <session-or-record-id>
```

Use result IDs with `ctx show` when you need the surrounding events.

## 6. Build Agent Context

```bash
ctx context "failed migration" --max-tokens 8000
ctx context "failed migration" --json
```

Context output is a retrieval bundle, not a generated narrative. It is designed
to be pasted into an agent prompt or consumed as JSON by an agent harness.
