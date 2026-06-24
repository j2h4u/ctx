# JSON Contracts

ctx JSON is for local agents and scripts. It can include prompts, command
output previews, local paths, and compatibility field names from the current
store. Treat it as private until a user reviews and redacts it.

All JSON commands currently use `schema_version: 1`.

## Setup

```bash
ctx setup --json
```

Writes local storage and returns:

- `schema_version`;
- `data_root`;
- `database_path`;
- `config_path`;
- `sources`;
- `network_required: false`;
- `repo_writes: false`.

## Status

```bash
ctx status --json
```

Reads local storage state and returns:

- `schema_version`;
- `initialized`;
- `data_root`;
- `database_path`;
- `config_path`;
- `indexed_items`;
- `indexed_sources`;
- `local_only: true`.

## Sources

```bash
ctx sources --json
```

Writes nothing and returns:

- `schema_version`;
- `sources[]`.

Each source includes:

- `provider`;
- `path`;
- `exists`;
- `source_format`;
- `status`;
- `raw_retention`.

## Import

```bash
ctx import --json
```

Writes the local SQLite index and returns:

- `schema_version`;
- `resume`;
- `resume_mode`;
- `totals`;
- `sources[]`.

`totals` and each source row include file, byte, session, event, edge, skipped,
and failed counts. `resume_mode` is currently `idempotent_rescan` when
`--resume` is passed and `normal_scan` otherwise.

## List

```bash
ctx list --json
```

Writes nothing and returns:

- `schema_version`;
- `items[]`.

Items include an opaque `id`, a `kind`, and fields available for that indexed
item. Session rows can include `provider`, `external_session_id`, `agent_type`,
`started_at`, and `ended_at`.

## Show

```bash
ctx show <item-uuid> --json
```

Writes nothing and returns:

- `schema_version`;
- `item`;
- `events[]` for sessions;
- `sessions[]` and `events[]` for compatibility item rows.

`show --json` currently exposes stored row shapes more directly than
`search --json` or `context --json`. Do not treat nested event payload fields as
share-safe or stable unless a future contract says so.

## Search

```bash
ctx search [query] --json
```

Writes nothing and returns:

- `schema_version`;
- `query`;
- `filters`;
- `generated_at`;
- `results[]`;
- `pagination`;
- `truncation`;
- `share_safe: false`.

Each result can include:

- `record_id`, the current opaque item identifier used with `ctx show`;
- `session_id`;
- `event_id`;
- `event_seq`;
- `title`;
- `snippet`;
- `rank`;
- `provider`;
- `timestamp`;
- `cwd`;
- `raw_source_path`;
- `raw_source_exists`;
- `cursor`;
- `why_matched`;
- `citations[]`;
- `links`;
- `visibility`.

`record_id` is a compatibility field name in the current JSON. Agents should
treat it as an opaque item ID, not as a product concept.

## Context

```bash
ctx context <query> --json
```

Writes nothing and returns:

- `schema_version`;
- `query`;
- `filters`;
- `generated_at`;
- `budget`;
- `results[]`;
- `pagination`;
- `truncation`;
- `share_safe: false`.

Each result can include:

- `record_id`, the current opaque item identifier used with `ctx show`;
- `title`;
- `summary`;
- `rank`;
- `why_matched`;
- `citations[]`;
- `links`;
- `visibility`.

`summary` is returned only from indexed source material or bounded local
previews. ctx does not call a model to create it during context rendering.

## Citation Fields

Citations can include:

- `type`;
- `id`;
- `label`;
- `time`;
- `provider`;
- `session_id`;
- `event_seq`;
- `raw_source_path`;
- `raw_source_exists`;
- `cursor`.

`raw_source_exists: false` means indexed text is available but the raw source
was not present at the stored path when checked.

## Doctor And Validate

```bash
ctx doctor --json
ctx validate --json
```

Both commands read local storage and return findings:

- `doctor`: `schema_version`, `ok`, `findings`;
- `validate`: `schema_version`, `valid`, `findings`.

## Compatibility Limits

The current JSON still contains some internal compatibility names. Future
pre-release hardening may rename those fields to more neutral item/source terms.
Until then, agents should rely on documented meanings, not field-name origin.
