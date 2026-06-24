# Search And Context

ctx has two retrieval commands:

- `ctx search` finds matching sessions and events.
- `ctx context` builds a bounded retrieval bundle for an agent.

## Search

Examples:

```bash
ctx search "build failure"
ctx search "sqlite storage" --provider codex
ctx search "retry handling" --repo checkout --since 60d
ctx search "Buildkite failure" --include-subagents --event-type command_output
ctx search --file crates/foo/src/lib.rs
```

A useful result includes:

- title or event label;
- snippet with visible truncation when needed;
- provider;
- session ID;
- event ID or sequence;
- timestamp;
- repository or working directory when known;
- primary or subagent role when known;
- source path and cursor or line when available;
- match reason;
- stable ID for `ctx show`.

## Filters

Search filters should narrow both human output and JSON:

- `--provider <name>`;
- `--repo <name-or-path>`;
- `--since <duration-or-date>`;
- `--event-type <type>`;
- `--file <path>`;
- `--primary-only`;
- `--include-subagents`;
- `--limit <n>`.

## Context

`ctx context` is deterministic retrieval. For the same database, query, filters,
and budget, it should select the same material in the same order.

Context output should include:

- query and filters;
- selected sessions and events;
- snippets and bounded surrounding text;
- command/tool events where relevant;
- provider, date, repository, and working-directory metadata;
- citations back to indexed and raw sources;
- warnings for moved or deleted raw source paths;
- explicit truncation markers when the budget removes material.

It should dedupe repeated hits from the same event or session and respect the
requested token budget.

## Citation Format

Human context output should make citations easy to copy:

```text
[codex session=<session-id> event=<event-id> source=<path> cursor=<cursor>]
```

JSON context output should carry the same pieces as structured fields.
