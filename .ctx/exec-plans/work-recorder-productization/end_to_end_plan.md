# ctx Work Recorder Productization Plan

Task to drive: `feb64c1c-e58c-40f8-b1e9-1094dca0646e`

Manager/handoff task: `e3ed0449-7e0f-488b-9450-b5165860be2a`

Primary implementation branch:

- `ctxrs/ctx` branch: `work-record`
- local worktree: `/home/daddy/code/ctx-multi-repo-workspace/worktrees/ctx/work-record-product`

Private/hosted implementation repo:

- `ctxrs/ctx-private` primary branch is allowed for this program
- canonical repo: `/home/daddy/code/ctx-multi-repo-workspace/ctx-private`
- create a manual worktree under `/home/daddy/code/ctx-multi-repo-workspace/worktrees/ctx-private/`

ADE repo:

- `ctxrs/ade` is the old ADE. Treat it as frozen unless a task explicitly asks
  for ADE maintenance.
- Do not build new Work Recorder functionality in `ctxrs/ade`.

## Handoff And Branch Discipline

- Do not push to `ctxrs/ctx/main`.
- Use the existing `ctxrs/ctx` branch `work-record` unless you discover a hard
  technical reason to create a child branch. If a child branch is needed, record
  the reason and parent commit in the status file.
- It is OK to push `ctxrs/ctx` `work-record` as the public Work Recorder branch.
- It is OK to push `ctxrs/ctx-private` primary branch for hosted/private work,
  per the user's instruction.
- Before editing `ctx-private`, read its repo instructions and context packs:
  - `/home/daddy/code/ctx-multi-repo-workspace/ctx-private/AGENTS.md`;
  - `.ctx/ctx-pack/agent-basics`;
  - `.ctx/ctx-pack/specs`.
- Preserve or copy this plan into the implementation branch's exec-plan location
  so the final code carries its provenance. Keep an implementation status file
  updated next to the plan.
- Do not mutate production services, publish public releases, or cut over
  production `ctx.rs` APIs unless the user explicitly authorizes that later.

## Background

The product direction changed.

`ctxrs/ctx` is no longer the ADE. It should become the standalone Work Recorder:

> ctx records agent work so it can be attached to PRs, searched later, and
> shared across teams.

The immediate value must be local and passive:

- install ctx;
- run `ctx setup`;
- ctx imports existing local agent history;
- ctx passively records future work using hooks/shims/importers;
- ctx opens a useful local dashboard;
- agents and humans can search, inspect, and attach Work Records to PRs.

Hosted/team functionality is the monetization path, but it must be shaped around
the same data model. Build enough hosted/staging infrastructure to validate sync,
sharing, PR reports, auth/team boundaries, storage, and policy, without cutting
over production `ctx.rs` APIs yet.

## Product Decisions To Preserve

- Public noun: `Work Record`.
- CLI noun: plain `ctx` commands, not `ctx work`.
- Storage default: machine-level local data under `~/.ctx/work-record/`.
- Main local DB: `~/.ctx/work-record/work.sqlite`.
- Blob storage: `~/.ctx/work-record/blobs/`.
- Capture inbox/spool: `~/.ctx/work-record/inbox/*.jsonl`.
- No files in the repo by default.
- No required always-on daemon for capture.
- On-demand dashboard server is fine.
- Optional daemon/service is allowed only for live dashboard/watch/indexing, and
  must be reversible with `ctx uninstall`.
- Git, jj, and GitHub CLI are first-class capture targets.
- Use shims/wrappers for `git`, `jj`, and `gh` initially.
- Do not require repo-level hooks by default.
- PR publication should upsert a separate PR comment by default, not mutate the
  PR description by default.
- Hosted sync must not upload raw transcripts by default. Full transcript sync is
  explicit opt-in.
- Retroactive linking is useful but confidence-labeled; never pretend inferred
  links are facts.
- Work Recorder must be valuable without ADE adoption.
- ADE/composable UI work is not the current product bet.

## Milestones And Gates

Work in these milestones. Do not advance to the next milestone until its review
gate passes or the status file records an explicit accepted deferral.

1. **Foundation contract**
   - schema/type contracts;
   - capture/spool architecture;
   - VCS abstraction;
   - privacy/export matrix;
   - hosted sync contract.
   - Gate: architecture reviewer PASS.
2. **Local product**
   - storage/migrations;
   - setup/import;
   - capture shims/hooks/importers;
   - search/context;
   - dashboard/report;
   - uninstall/doctor.
   - Gate: local product reviewer, security reviewer, UI reviewer PASS.
3. **Hosted staging**
   - Cloudflare Worker;
   - Neon migrations;
   - R2 blobs;
   - team/auth model;
   - sync/report/share API;
   - staging smoke tests.
   - Gate: hosted/API/security reviewer PASS.
4. **CI/release**
   - Buildkite matrix;
   - release dry-run;
   - installers;
   - platform smoke tests;
   - resource-safe scripts.
   - Gate: CI/release reviewer PASS.
5. **Dogfood/final**
   - local setup on this machine;
   - import existing local history;
   - fresh sample run;
   - dashboard screenshots;
   - PR report flow;
   - fresh-agent search/context validation.
   - Gate: final done-ness reviewer PASS.

Status files required:

- `.ctx/exec-plans/work-recorder-productization/implementation_status.md`;
- `.ctx/exec-plans/work-recorder-productization/decision_log.md`;
- `.ctx/exec-plans/work-recorder-productization/risk_register.md`;
- `.ctx/exec-plans/work-recorder-productization/validation_log.md`;
- `.ctx/exec-plans/work-recorder-productization/reviewer_verdicts.md`.

## Concrete Local Data Contract

Use SQLite as the local source of truth. Use append-friendly normalized tables
plus materialized projections for dashboard/search speed.

### Global Rules

- IDs are UUIDv7 where generated by ctx.
- Imported provider IDs are stored separately from ctx IDs.
- SQLite timestamp columns are `INTEGER NOT NULL` Unix milliseconds unless the
  column is explicitly nullable. API/CLI JSON renders timestamps as UTC RFC3339.
- Local table IDs are `TEXT PRIMARY KEY NOT NULL`.
- Foreign keys are enabled and enforced.
- Mutable/syncable tables include:
  - `created_at_ms INTEGER NOT NULL`;
  - `updated_at_ms INTEGER NOT NULL`;
  - `source_id TEXT`;
  - `visibility TEXT NOT NULL DEFAULT 'local_only'`;
  - `fidelity TEXT NOT NULL DEFAULT 'partial'`;
  - `sync_state TEXT NOT NULL DEFAULT 'local_only'`;
  - `sync_version INTEGER NOT NULL DEFAULT 0`;
  - `deleted_at_ms INTEGER`;
  - `metadata_json TEXT NOT NULL DEFAULT '{}'`.
- Append-only event tables do not update rows except for redaction/sync metadata;
  if an event needs correction, write a superseding event.
- All externally visible JSON schemas are versioned.
- Large raw payloads are stored as blob objects. SQLite stores references,
  hashes, byte sizes, media type, redaction state, and preview text.
- Runtime code must not use `unwrap`/`expect`.
- Migrations are one-way, versioned, and tested against empty DB plus seeded
  realistic DB.

### Required Enum Values

Implement enums as validated text values at the Rust/TypeScript boundary and
SQLite `CHECK` constraints where practical.

- `visibility`: `local_only | reportable | sync_metadata | sync_full | withheld`.
- `fidelity`: `full | partial | imported | inferred | summary_only`.
- `sync_state`: `local_only | pending | synced | failed | withheld`.
- `confidence`: `explicit | high | medium | low | unknown`.
- `redaction_state`: `raw | redacted | safe_preview | withheld`.

### Required Index Pattern

Every foreign-key column used in joins must have an index. Minimum indexes:

- all `*_id` foreign-key columns;
- `events(seq)`;
- `events(work_record_id, occurred_at_ms)`;
- `events(session_id, occurred_at_ms)`;
- `sessions(work_record_id)`;
- `sessions(root_session_id)`;
- `runs(work_record_id, started_at_ms)`;
- `work_records(last_activity_at_ms)`;
- `vcs_workspaces(kind, repo_fingerprint)`;
- `pull_requests(provider, owner, repo, number)` unique;
- `sync_outbox(sync_state, updated_at_ms)`;
- FTS indexes listed below.

### Minimum Tables

`capture_sources`

- `id TEXT PRIMARY KEY NOT NULL`;
- `kind TEXT NOT NULL`: `provider_import | provider_hook | shim | direct_cli | dashboard | hosted_sync | manual`;
- `provider TEXT NOT NULL`: `codex | claude | pi | cursor | shell | git | jj | gh | unknown`;
- `machine_id TEXT NOT NULL`;
- `process_id INTEGER`;
- `cwd TEXT`;
- `raw_source_path TEXT`;
- `external_session_id TEXT`;
- `started_at_ms INTEGER NOT NULL`;
- `ended_at_ms INTEGER`;
- `fidelity TEXT NOT NULL`;
- `visibility TEXT NOT NULL DEFAULT 'local_only'`;
- `sync_state TEXT NOT NULL DEFAULT 'local_only'`;
- `sync_version INTEGER NOT NULL DEFAULT 0`;
- `metadata_json TEXT NOT NULL DEFAULT '{}'`.

`work_records`

- `id TEXT PRIMARY KEY NOT NULL`;
- `title TEXT NOT NULL`;
- `summary TEXT`;
- `status TEXT NOT NULL`: `open | active | completed | abandoned | archived`;
- `primary_vcs_workspace_id TEXT REFERENCES vcs_workspaces(id)`;
- `started_at_ms INTEGER`;
- `last_activity_at_ms INTEGER NOT NULL`;
- `completed_at_ms INTEGER`;
- `confidence TEXT NOT NULL DEFAULT 'unknown'`;
- `created_at_ms INTEGER NOT NULL`;
- `updated_at_ms INTEGER NOT NULL`;
- `source_id TEXT REFERENCES capture_sources(id)`;
- `visibility TEXT NOT NULL DEFAULT 'local_only'`;
- `fidelity TEXT NOT NULL DEFAULT 'partial'`;
- `sync_state TEXT NOT NULL DEFAULT 'local_only'`;
- `sync_version INTEGER NOT NULL DEFAULT 0`;
- `deleted_at_ms INTEGER`;
- `metadata_json TEXT NOT NULL DEFAULT '{}'`.

`sessions`

- `id TEXT PRIMARY KEY NOT NULL`;
- `work_record_id TEXT REFERENCES work_records(id)`;
- `parent_session_id TEXT REFERENCES sessions(id)`;
- `root_session_id TEXT REFERENCES sessions(id)`;
- `capture_source_id TEXT REFERENCES capture_sources(id)`;
- `provider TEXT NOT NULL`;
- `external_session_id TEXT`;
- `external_agent_id TEXT`;
- `agent_type TEXT NOT NULL`: `primary | subagent | agent_team_member | reviewer | implementer | unknown`;
- `role_hint TEXT`;
- `is_primary INTEGER NOT NULL DEFAULT 0`;
- `status TEXT NOT NULL`: `started | active | idle | completed | failed | interrupted | imported`;
- `fidelity TEXT NOT NULL`;
- `transcript_blob_id TEXT REFERENCES artifacts(id)`;
- `started_at_ms INTEGER NOT NULL`;
- `ended_at_ms INTEGER`;
- `created_at_ms INTEGER NOT NULL`;
- `updated_at_ms INTEGER NOT NULL`;
- `visibility TEXT NOT NULL DEFAULT 'local_only'`;
- `sync_state TEXT NOT NULL DEFAULT 'local_only'`;
- `sync_version INTEGER NOT NULL DEFAULT 0`;
- `deleted_at_ms INTEGER`;
- `metadata_json TEXT NOT NULL DEFAULT '{}'`.

`session_edges`

- `id TEXT PRIMARY KEY NOT NULL`;
- `from_session_id TEXT NOT NULL REFERENCES sessions(id)`;
- `to_session_id TEXT NOT NULL REFERENCES sessions(id)`;
- `edge_type TEXT NOT NULL`: `parent_child | delegated | reviewed | spawned | resumed_from | imported_related`;
- `confidence TEXT NOT NULL DEFAULT 'unknown'`;
- `source_id TEXT REFERENCES capture_sources(id)`;
- `created_at_ms INTEGER NOT NULL`;
- `updated_at_ms INTEGER NOT NULL`;
- `visibility TEXT NOT NULL DEFAULT 'local_only'`;
- `fidelity TEXT NOT NULL DEFAULT 'partial'`;
- `sync_state TEXT NOT NULL DEFAULT 'local_only'`;
- `sync_version INTEGER NOT NULL DEFAULT 0`;
- `deleted_at_ms INTEGER`;
- `metadata_json TEXT NOT NULL DEFAULT '{}'`.

`runs`

- `id TEXT PRIMARY KEY NOT NULL`;
- `work_record_id TEXT REFERENCES work_records(id)`;
- `session_id TEXT REFERENCES sessions(id)`;
- `run_type TEXT NOT NULL`: `agent_turn | command | tool_call | review | import | evidence | summary`;
- `status TEXT NOT NULL`: `queued | running | succeeded | failed | cancelled | partial`;
- `started_at_ms INTEGER NOT NULL`;
- `ended_at_ms INTEGER`;
- `exit_code INTEGER`;
- `cwd TEXT`;
- `command_preview TEXT`;
- `input_blob_id TEXT REFERENCES artifacts(id)`;
- `output_blob_id TEXT REFERENCES artifacts(id)`;
- `created_at_ms INTEGER NOT NULL`;
- `updated_at_ms INTEGER NOT NULL`;
- `source_id TEXT REFERENCES capture_sources(id)`;
- `visibility TEXT NOT NULL DEFAULT 'local_only'`;
- `fidelity TEXT NOT NULL DEFAULT 'partial'`;
- `sync_state TEXT NOT NULL DEFAULT 'local_only'`;
- `sync_version INTEGER NOT NULL DEFAULT 0`;
- `deleted_at_ms INTEGER`;
- `metadata_json TEXT NOT NULL DEFAULT '{}'`.

`events`

- `id TEXT PRIMARY KEY NOT NULL`;
- `seq INTEGER NOT NULL UNIQUE`;
- `work_record_id TEXT REFERENCES work_records(id)`;
- `session_id TEXT REFERENCES sessions(id)`;
- `run_id TEXT REFERENCES runs(id)`;
- `event_type TEXT NOT NULL`: `message | tool_call | tool_output | command_started | command_output | command_finished | file_touched | vcs_change | pr_link | evidence | artifact | summary | notice`;
- `role TEXT`: `user | assistant | system | tool | unknown`;
- `occurred_at_ms INTEGER NOT NULL`;
- `capture_source_id TEXT REFERENCES capture_sources(id)`;
- `payload_json TEXT NOT NULL DEFAULT '{}'`;
- `payload_blob_id TEXT REFERENCES artifacts(id)`;
- `dedupe_key TEXT`;
- `visibility TEXT NOT NULL DEFAULT 'local_only'`;
- `redaction_state TEXT NOT NULL DEFAULT 'safe_preview'`;
- `fidelity TEXT NOT NULL DEFAULT 'partial'`;
- `sync_state TEXT NOT NULL DEFAULT 'local_only'`;
- `sync_version INTEGER NOT NULL DEFAULT 0`;
- `deleted_at_ms INTEGER`;
- `metadata_json TEXT NOT NULL DEFAULT '{}'`.

`vcs_workspaces`

- `id TEXT PRIMARY KEY NOT NULL`;
- `kind TEXT NOT NULL`: `git | jj`;
- `root_path TEXT NOT NULL`;
- `repo_fingerprint TEXT NOT NULL`;
- `primary_remote_url_normalized TEXT`;
- `host TEXT NOT NULL DEFAULT 'unknown'`: `github | gitlab | bitbucket | local | unknown`;
- `owner TEXT`;
- `name TEXT`;
- `monorepo_subpath TEXT`;
- `created_at_ms INTEGER NOT NULL`;
- `updated_at_ms INTEGER NOT NULL`;
- `source_id TEXT REFERENCES capture_sources(id)`;
- `visibility TEXT NOT NULL DEFAULT 'local_only'`;
- `fidelity TEXT NOT NULL DEFAULT 'partial'`;
- `sync_state TEXT NOT NULL DEFAULT 'local_only'`;
- `sync_version INTEGER NOT NULL DEFAULT 0`;
- `deleted_at_ms INTEGER`;
- `metadata_json TEXT NOT NULL DEFAULT '{}'`;
- `UNIQUE(kind, repo_fingerprint)`.

`vcs_changes`

- `id TEXT PRIMARY KEY NOT NULL`;
- `vcs_workspace_id TEXT NOT NULL REFERENCES vcs_workspaces(id)`;
- `kind TEXT NOT NULL`: `git_commit | git_branch | git_worktree | jj_change | jj_bookmark | patch | working_copy`;
- `change_id TEXT NOT NULL`;
- `parent_change_ids_json TEXT NOT NULL DEFAULT '[]'`;
- `branch_or_bookmark TEXT`;
- `tree_hash TEXT`;
- `author_time_ms INTEGER`;
- `confidence TEXT NOT NULL DEFAULT 'unknown'`;
- `created_at_ms INTEGER NOT NULL`;
- `updated_at_ms INTEGER NOT NULL`;
- `source_id TEXT REFERENCES capture_sources(id)`;
- `visibility TEXT NOT NULL DEFAULT 'local_only'`;
- `fidelity TEXT NOT NULL DEFAULT 'partial'`;
- `sync_state TEXT NOT NULL DEFAULT 'local_only'`;
- `sync_version INTEGER NOT NULL DEFAULT 0`;
- `deleted_at_ms INTEGER`;
- `metadata_json TEXT NOT NULL DEFAULT '{}'`;
- `UNIQUE(vcs_workspace_id, kind, change_id)`.

`pull_requests`

- `id TEXT PRIMARY KEY NOT NULL`;
- `vcs_workspace_id TEXT REFERENCES vcs_workspaces(id)`;
- `provider TEXT NOT NULL`: `github | gitlab | unknown`;
- `url TEXT NOT NULL`;
- `number INTEGER`;
- `owner TEXT`;
- `repo TEXT`;
- `title TEXT`;
- `state TEXT`;
- `head_ref TEXT`;
- `base_ref TEXT`;
- `head_sha TEXT`;
- `confidence TEXT NOT NULL DEFAULT 'unknown'`;
- `link_source TEXT NOT NULL`: `explicit | gh_shim | captured_url | inferred_branch | inferred_commit | manual`;
- `created_at_ms INTEGER NOT NULL`;
- `updated_at_ms INTEGER NOT NULL`;
- `source_id TEXT REFERENCES capture_sources(id)`;
- `visibility TEXT NOT NULL DEFAULT 'local_only'`;
- `fidelity TEXT NOT NULL DEFAULT 'partial'`;
- `sync_state TEXT NOT NULL DEFAULT 'local_only'`;
- `sync_version INTEGER NOT NULL DEFAULT 0`;
- `deleted_at_ms INTEGER`;
- `metadata_json TEXT NOT NULL DEFAULT '{}'`;
- `UNIQUE(provider, owner, repo, number)`.

`work_record_links`

- `id TEXT PRIMARY KEY NOT NULL`;
- `work_record_id TEXT NOT NULL REFERENCES work_records(id)`;
- `target_type TEXT NOT NULL`: `session | run | event | vcs_workspace | vcs_change | pull_request | artifact | evidence`;
- `target_id TEXT NOT NULL`;
- `link_type TEXT NOT NULL`: `produced | touched | references | evidence_for | published_to | likely_related`;
- `confidence TEXT NOT NULL DEFAULT 'unknown'`;
- `source_id TEXT REFERENCES capture_sources(id)`;
- `created_at_ms INTEGER NOT NULL`;
- `updated_at_ms INTEGER NOT NULL`;
- `visibility TEXT NOT NULL DEFAULT 'local_only'`;
- `fidelity TEXT NOT NULL DEFAULT 'partial'`;
- `sync_state TEXT NOT NULL DEFAULT 'local_only'`;
- `sync_version INTEGER NOT NULL DEFAULT 0`;
- `deleted_at_ms INTEGER`;
- `metadata_json TEXT NOT NULL DEFAULT '{}'`;
- `UNIQUE(work_record_id, target_type, target_id, link_type)`.

`artifacts`

- `id TEXT PRIMARY KEY NOT NULL`;
- `kind TEXT NOT NULL`: `transcript | stdout | stderr | screenshot | report | diff | file_snapshot | json | markdown | binary`;
- `blob_hash TEXT NOT NULL`;
- `blob_path TEXT NOT NULL`;
- `byte_size INTEGER NOT NULL`;
- `media_type TEXT`;
- `preview_text TEXT`;
- `redaction_state TEXT NOT NULL DEFAULT 'safe_preview'`;
- `created_at_ms INTEGER NOT NULL`;
- `updated_at_ms INTEGER NOT NULL`;
- `source_id TEXT REFERENCES capture_sources(id)`;
- `visibility TEXT NOT NULL DEFAULT 'local_only'`;
- `fidelity TEXT NOT NULL DEFAULT 'partial'`;
- `sync_state TEXT NOT NULL DEFAULT 'local_only'`;
- `sync_version INTEGER NOT NULL DEFAULT 0`;
- `deleted_at_ms INTEGER`;
- `metadata_json TEXT NOT NULL DEFAULT '{}'`;
- `UNIQUE(blob_hash, kind)`.

`evidence`

- `id TEXT PRIMARY KEY NOT NULL`;
- `work_record_id TEXT NOT NULL REFERENCES work_records(id)`;
- `vcs_change_id TEXT REFERENCES vcs_changes(id)`;
- `kind TEXT NOT NULL`: `test | lint | build | typecheck | screenshot | review | ci | manual`;
- `status TEXT NOT NULL`: `passed | failed | skipped | stale | unknown`;
- `freshness TEXT NOT NULL DEFAULT 'unbound'`: `fresh | probably_fresh | stale | unbound | inferred`;
- `command_run_id TEXT REFERENCES runs(id)`;
- `artifact_id TEXT REFERENCES artifacts(id)`;
- `observed_tree_hash TEXT`;
- `observed_head_sha TEXT`;
- `started_at_ms INTEGER`;
- `ended_at_ms INTEGER`;
- `stale_reason TEXT`;
- `created_at_ms INTEGER NOT NULL`;
- `updated_at_ms INTEGER NOT NULL`;
- `source_id TEXT REFERENCES capture_sources(id)`;
- `visibility TEXT NOT NULL DEFAULT 'local_only'`;
- `fidelity TEXT NOT NULL DEFAULT 'partial'`;
- `sync_state TEXT NOT NULL DEFAULT 'local_only'`;
- `sync_version INTEGER NOT NULL DEFAULT 0`;
- `deleted_at_ms INTEGER`;
- `metadata_json TEXT NOT NULL DEFAULT '{}'`.

`summaries`

- `id TEXT PRIMARY KEY NOT NULL`;
- `work_record_id TEXT REFERENCES work_records(id)`;
- `session_id TEXT REFERENCES sessions(id)`;
- `kind TEXT NOT NULL`: `imported_provider_summary | ctx_generated | agent_supplied | human_note`;
- `model_or_source TEXT`;
- `text TEXT NOT NULL`;
- `citations_json TEXT NOT NULL DEFAULT '[]'`;
- `created_at_ms INTEGER NOT NULL`;
- `updated_at_ms INTEGER NOT NULL`;
- `source_id TEXT REFERENCES capture_sources(id)`;
- `visibility TEXT NOT NULL DEFAULT 'local_only'`;
- `fidelity TEXT NOT NULL DEFAULT 'partial'`;
- `sync_state TEXT NOT NULL DEFAULT 'local_only'`;
- `sync_version INTEGER NOT NULL DEFAULT 0`;
- `deleted_at_ms INTEGER`;
- `metadata_json TEXT NOT NULL DEFAULT '{}'`.

`files_touched`

- `id TEXT PRIMARY KEY NOT NULL`;
- `work_record_id TEXT REFERENCES work_records(id)`;
- `run_id TEXT REFERENCES runs(id)`;
- `event_id TEXT REFERENCES events(id)`;
- `vcs_workspace_id TEXT REFERENCES vcs_workspaces(id)`;
- `path TEXT NOT NULL`;
- `change_kind TEXT`: `read | created | modified | deleted | renamed | unknown`;
- `old_path TEXT`;
- `line_count_delta INTEGER`;
- `confidence TEXT NOT NULL DEFAULT 'unknown'`;
- `created_at_ms INTEGER NOT NULL`;
- `updated_at_ms INTEGER NOT NULL`;
- `source_id TEXT REFERENCES capture_sources(id)`;
- `visibility TEXT NOT NULL DEFAULT 'local_only'`;
- `fidelity TEXT NOT NULL DEFAULT 'partial'`;
- `sync_state TEXT NOT NULL DEFAULT 'local_only'`;
- `sync_version INTEGER NOT NULL DEFAULT 0`;
- `deleted_at_ms INTEGER`;
- `metadata_json TEXT NOT NULL DEFAULT '{}'`.

`tags`

- `id TEXT PRIMARY KEY NOT NULL`;
- `name TEXT NOT NULL UNIQUE`;
- `kind TEXT NOT NULL DEFAULT 'user'`: `user | system | inferred`;
- `created_at_ms INTEGER NOT NULL`;
- `updated_at_ms INTEGER NOT NULL`;
- `metadata_json TEXT NOT NULL DEFAULT '{}'`.

`work_record_tags`

- `work_record_id TEXT NOT NULL REFERENCES work_records(id)`;
- `tag_id TEXT NOT NULL REFERENCES tags(id)`;
- `source_id TEXT REFERENCES capture_sources(id)`;
- `confidence TEXT NOT NULL DEFAULT 'unknown'`;
- `created_at_ms INTEGER NOT NULL`;
- `PRIMARY KEY (work_record_id, tag_id)`.

`record_edges`

- `id TEXT PRIMARY KEY NOT NULL`;
- `from_record_id TEXT NOT NULL REFERENCES work_records(id)`;
- `to_record_id TEXT NOT NULL REFERENCES work_records(id)`;
- `edge_type TEXT NOT NULL`: `continues | duplicates | blocks | related | supersedes | split_from`;
- `confidence TEXT NOT NULL DEFAULT 'unknown'`;
- `source_id TEXT REFERENCES capture_sources(id)`;
- `created_at_ms INTEGER NOT NULL`;
- `updated_at_ms INTEGER NOT NULL`;
- `visibility TEXT NOT NULL DEFAULT 'local_only'`;
- `fidelity TEXT NOT NULL DEFAULT 'partial'`;
- `sync_state TEXT NOT NULL DEFAULT 'local_only'`;
- `sync_version INTEGER NOT NULL DEFAULT 0`;
- `deleted_at_ms INTEGER`;
- `metadata_json TEXT NOT NULL DEFAULT '{}'`.

`sync_aliases`

- `id TEXT PRIMARY KEY NOT NULL`;
- `local_table TEXT NOT NULL`;
- `local_id TEXT NOT NULL`;
- `hosted_id TEXT NOT NULL`;
- `team_id TEXT`;
- `created_at_ms INTEGER NOT NULL`;
- `updated_at_ms INTEGER NOT NULL`;
- `UNIQUE(local_table, local_id, team_id)`;
- `UNIQUE(hosted_id, team_id)`.

`sync_cursors`

- `id TEXT PRIMARY KEY NOT NULL`;
- `team_id TEXT`;
- `device_id TEXT NOT NULL`;
- `stream TEXT NOT NULL`;
- `cursor TEXT NOT NULL`;
- `last_synced_at_ms INTEGER`;
- `created_at_ms INTEGER NOT NULL`;
- `updated_at_ms INTEGER NOT NULL`;
- `UNIQUE(team_id, device_id, stream)`.

`sync_batches`

- `id TEXT PRIMARY KEY NOT NULL`;
- `team_id TEXT`;
- `device_id TEXT NOT NULL`;
- `direction TEXT NOT NULL`: `upload | download`;
- `status TEXT NOT NULL`: `pending | running | succeeded | failed`;
- `started_at_ms INTEGER`;
- `finished_at_ms INTEGER`;
- `row_count INTEGER NOT NULL DEFAULT 0`;
- `error TEXT`;
- `created_at_ms INTEGER NOT NULL`;
- `updated_at_ms INTEGER NOT NULL`;
- `metadata_json TEXT NOT NULL DEFAULT '{}'`.

`sync_outbox`

- `id TEXT PRIMARY KEY NOT NULL`;
- `local_table TEXT NOT NULL`;
- `local_id TEXT NOT NULL`;
- `operation TEXT NOT NULL`: `insert | update | delete | blob_upload`;
- `team_id TEXT`;
- `device_id TEXT NOT NULL`;
- `sync_state TEXT NOT NULL DEFAULT 'pending'`: `pending | synced | failed | withheld`;
- `attempt_count INTEGER NOT NULL DEFAULT 0`;
- `next_attempt_at_ms INTEGER`;
- `last_error TEXT`;
- `payload_json TEXT NOT NULL DEFAULT '{}'`;
- `created_at_ms INTEGER NOT NULL`;
- `updated_at_ms INTEGER NOT NULL`;
- `UNIQUE(local_table, local_id, operation, team_id)`.

`audit_log`

- `id TEXT PRIMARY KEY NOT NULL`;
- `actor_kind TEXT NOT NULL`: `human | agent | system | hosted`;
- `actor_id TEXT`;
- `action TEXT NOT NULL`;
- `target_table TEXT`;
- `target_id TEXT`;
- `occurred_at_ms INTEGER NOT NULL`;
- `source_id TEXT REFERENCES capture_sources(id)`;
- `metadata_json TEXT NOT NULL DEFAULT '{}'`.

### FTS Tables

Create FTS5 tables or equivalent projections for:

- `work_record_search`:
  - `record_id`;
  - `title`;
  - `summary`;
  - `primary_user_text`;
  - `decision_text`;
  - `evidence_text`;
  - `tag_text`;
- `event_search`:
  - `event_id`;
  - `work_record_id`;
  - `session_id`;
  - `role`;
  - `safe_preview_text`;
  - `rank_bucket`;
- `artifact_search`:
  - `artifact_id`;
  - `work_record_id`;
  - `safe_preview_text`.

FTS projections must include only redacted/safe text unless a local-only raw
search mode is explicitly requested.

### Uniqueness And Idempotency

Required unique/dedupe keys:

- provider import: `(provider, external_session_id, external_event_id)`;
- shim command: `(machine_id, process_start_time, pid, cwd, command_hash)`;
- VCS change: `(vcs_workspace_id, kind, change_id)`;
- PR: `(provider, owner, repo, number)`;
- artifact blob: content hash;
- event: source-specific `dedupe_key`.

Imports and capture retries must be idempotent.

### Visibility Values

- `local_only`: never exported or synced.
- `reportable`: may appear in local reports after redaction.
- `sync_metadata`: may sync metadata/safe previews.
- `sync_full`: may sync raw/full content after explicit opt-in.
- `withheld`: stored but hidden from search/report/dashboard by default.

## Concrete Capture Architecture

Capture must work without a daemon.

### Write Path

1. Provider hooks/shims/importers create JSONL envelope events.
2. They append to spool files under `~/.ctx/work-record/inbox/`.
3. A fast importer moves events into SQLite.
4. Dashboard server may run the importer continuously while open.
5. If no dashboard/server is running, the next `ctx` command imports pending
   spool events before serving results.

### Spool Contract

- Directory: `~/.ctx/work-record/inbox/`.
- File naming:
  - `capture-{machine_id}-{pid}-{unix_ms}-{random}.jsonl.tmp`;
  - atomic rename to `.jsonl` after writer closes.
- Each line:
  - `schema_version`;
  - `capture_event_id`;
  - `dedupe_key`;
  - `source`;
  - `occurred_at`;
  - `cwd`;
  - `env_session_hints`;
  - `payload`;
  - `payload_hash`;
  - `fidelity`.
- Writers must never block the underlying command for more than a small bounded
  timeout, for example 50 ms best-effort plus background/async flush if possible.
- If capture fails, write a local capture error event if possible, but never
  change the underlying command exit code.

### SQLite Settings

- WAL mode enabled.
- Busy timeout set.
- Short transactions.
- Importer processes one spool file atomically:
  - claim file by rename to `.processing`;
  - import idempotently;
  - move to `.done` or `.failed`;
  - failed files retain error metadata.
- Corruption recovery:
  - `ctx doctor` can identify failed spool files;
  - `ctx repair` can retry failed imports;
  - raw spool data is not deleted until import succeeds and retention policy
    allows cleanup.

### Shims

Initial shims:

- `git`;
- `jj`;
- `gh`.

Shim behavior:

- find the real binary deterministically;
- execute real command with original args/env/stdin/stdout/stderr;
- preserve exit code exactly;
- capture command metadata, cwd, timing, exit code, and output previews when
  safe;
- capture full output only when configured or when evidence command was wrapped;
- never emit ctx errors into command stdout;
- stderr warnings only in debug mode.

## VCS And Multi-Repo Contract

Use `vcs_workspace`, not `repo`, as the core abstraction.

Git:

- workspace root is `git rev-parse --show-toplevel`;
- identity uses normalized remote URL where available plus root path and initial
  fingerprint;
- commits use SHA;
- branches and worktrees are attributes, not identities.

jj:

- workspace root uses `jj root`;
- changes use jj change IDs;
- bookmarks map to branch-like names;
- Git-backed jj workspaces also link to Git remotes/commits when available.

Multi-repo records:

- a Work Record can link to many `vcs_workspaces`;
- `primary_repo_id` is a UI convenience only;
- cross-repo grouping uses cwd, transcript paths, shim events, file paths, time
  windows, explicit agent hints, and PR/commit links;
- every inferred cross-repo link stores confidence and source.

Edge cases required in fixtures:

- nested Git repos;
- monorepo subdirectory;
- Git worktree;
- jj rewritten change;
- detached HEAD;
- no remote;
- multiple remotes;
- same repo on two machines;
- one Work Record touching two repos.

## Hosted Sync Contract

Local rows must be sync-ready, even if sync is off.

### Device Identity

- `machine_id` stored in `~/.ctx/work-record/device.json`.
- `installation_id` distinct from user/team identity.
- hosted sync uses scoped device tokens.

### Sync Units

- metadata rows sync as JSON patches/batches;
- blobs upload separately to R2 by content hash;
- default sync includes:
  - work record metadata;
  - safe summaries;
  - PR/evidence metadata;
  - redacted previews;
  - artifact metadata;
- default sync excludes:
  - raw transcripts;
  - raw tool output;
  - command env;
  - full stdout/stderr;
  - screenshots unless marked reportable/syncable.

### Cursors And Conflicts

- Sync cursor is per device, per table/stream.
- Use tombstones for deletes.
- Local generated IDs remain stable.
- Server assigns hosted IDs only as aliases.
- Conflict policy:
  - append-only events do not conflict;
  - mutable metadata uses last-writer plus audit trail for low-risk fields;
  - visibility narrowing wins over visibility widening;
  - explicit links beat inferred links.

### Blob Protocol

- client asks Worker for upload intent;
- Worker returns signed/scoped R2 upload URL or direct Worker upload route;
- blob key format:
  - `teams/{team_id}/records/{record_id}/blobs/{sha256}`;
- server records hash, size, media type, visibility, and redaction state;
- downloads require team membership and row-level visibility.

## Agent Context Packet Contract

`ctx context --json` must return a documented packet suitable for future agents.

Minimum shape:

```json
{
  "schema_version": 1,
  "query": "checkout retry",
  "generated_at": "2026-06-22T00:00:00Z",
  "budget": {
    "max_tokens": 12000,
    "estimated_tokens": 4312
  },
  "results": [
    {
      "record_id": "uuid",
      "title": "Fix checkout retry",
      "summary": "short redacted summary",
      "rank": 0.93,
      "why_matched": ["title", "primary_user_message", "failed_command"],
      "citations": [
        {
          "type": "event",
          "id": "uuid",
          "label": "primary user prompt",
          "time": "2026-06-22T00:00:00Z"
        }
      ],
      "evidence": [
        {
          "id": "uuid",
          "kind": "test",
          "status": "passed",
          "freshness": "fresh"
        }
      ],
      "links": {
        "dashboard": "http://127.0.0.1:.../records/uuid",
        "pr": "https://github.com/org/repo/pull/123"
      },
      "visibility": "reportable"
    }
  ],
  "pagination": {
    "cursor": "opaque",
    "has_more": false
  }
}
```

Rules:

- snippets are redacted by default;
- raw payloads require explicit `ctx show --raw` or local dashboard expansion;
- every result explains `why_matched`;
- every claim has a citation ID;
- context output has token budget and truncation metadata.

## Hosted Staging Concrete API

Implement or intentionally stub these routes in `ctx-private` staging.

Base staging host should be isolated from production, for example:

- `https://staging-api.ctx.rs`;
- `https://staging-app.ctx.rs`;
- or equivalent Cloudflare preview URLs recorded in status.

Required Worker routes:

- `GET /healthz`;
- `POST /v1/auth/device/start`;
- `POST /v1/auth/device/complete`;
- `GET /v1/me`;
- `POST /v1/teams`;
- `GET /v1/teams/:team_id`;
- `POST /v1/teams/:team_id/invites`;
- `POST /v1/sync/batches`;
- `GET /v1/sync/cursor`;
- `POST /v1/blobs/upload-intent`;
- `PUT /v1/blobs/:blob_id` or signed upload equivalent;
- `GET /v1/work-records`;
- `GET /v1/work-records/:id`;
- `GET /v1/work-records/:id/report`;
- `POST /v1/work-records/:id/share`;
- `POST /v1/github/webhook`;
- `POST /v1/pr-reports/upsert`;
- `GET /v1/audit-log`.

Required staging bindings:

- Neon database URL secret, staging only;
- R2 bucket binding, staging only;
- queue binding, staging only if async ingest used;
- GitHub app/webhook secret, staging only if implemented;
- session/JWT signing secret, staging only.

Required access roles:

- owner;
- admin;
- member;
- viewer;
- external_report_viewer.

Access-control matrix must cover:

- list records;
- view record metadata;
- view raw transcript;
- view report;
- upload blob;
- download blob;
- share report;
- delete/tombstone record;
- export team data;
- view audit log;
- manage members.

Required Neon migrations:

- users;
- teams;
- memberships;
- invitations;
- devices/installations;
- sync_batches;
- sync_cursors;
- work_records;
- sessions;
- events_metadata;
- evidence;
- artifacts;
- pull_requests;
- blobs;
- report_shares;
- audit_log.

Required staging smoke:

- deploy Worker to staging;
- run migrations against staging Neon;
- create test team/user/device;
- sync one redacted Work Record;
- upload one blob to staging R2;
- fetch report as authorized user;
- fail to fetch as unauthorized user;
- verify production Neon/R2/env are untouched.

## Privacy And Redaction Matrix

Default behavior by surface:

| Data | Local Dashboard | Search Snippet | Local Report | PR Comment | Hosted Metadata Sync | Hosted Full Sync |
| --- | --- | --- | --- | --- | --- | --- |
| record title | visible | visible | visible | visible | sync | sync |
| user prompt | visible | redacted preview | redacted preview | summary only | redacted preview | opt-in |
| assistant transcript | visible | redacted preview | summary only | summary only | redacted preview | opt-in |
| tool output | collapsed | redacted preview | evidence summary | not raw | metadata only | opt-in |
| command args | visible with redaction | redacted | redacted | selected redacted | redacted | opt-in |
| env vars | hidden | hidden | hidden | hidden | never | never |
| screenshots | visible local | metadata | opt-in artifact | opt-in link | metadata | opt-in |
| PR URL | visible | visible | visible | visible | sync | sync |
| evidence result | visible | visible | visible | visible | sync | sync |
| raw blobs | explicit expand | never | never by default | never | never | opt-in |

Required secret fixtures:

- fake OpenAI key;
- fake Anthropic key;
- GitHub token;
- AWS key;
- bearer token in command output;
- `.env` path;
- private Git remote URL with token;
- PR URL;
- secret-like value in transcript;
- secret-like value in screenshot/artifact metadata.

Tests must prove those do not appear in:

- search snippets;
- context packets;
- PR comments;
- hosted logs;
- hosted metadata sync;
- default dashboard screenshots used as artifacts;
- exported reports.

## Evidence Trust Model

Evidence has freshness relative to a VCS change.

Evidence statuses:

- `passed`;
- `failed`;
- `skipped`;
- `unknown`;
- `stale`;
- `not_applicable`.

Freshness:

- `fresh`: evidence observed exact current commit/tree/change ID.
- `probably_fresh`: evidence observed same branch/bookmark and no relevant file
  changes since.
- `stale`: linked change differs from current change.
- `unbound`: no VCS change could be associated.
- `inferred`: associated by heuristic only.

Evidence pages must show:

- command;
- cwd;
- start/end time;
- duration;
- exit code;
- stdout/stderr preview;
- full output blob link if local and allowed;
- observed VCS change;
- current VCS change;
- freshness verdict;
- capture source/fidelity;
- reviewer CTA.

PR report first viewport must include:

- verdict: `ready`, `needs evidence`, `stale evidence`, `failed evidence`, or
  `insufficient capture`;
- PR link and VCS change;
- top 3 changes;
- evidence summary;
- missing/stale evidence callouts;
- link to full local/hosted report;
- no raw transcript by default.

## Dashboard Concrete Acceptance

Dashboard first screen must include:

- capture health card;
- imported history count;
- records over time;
- recent Work Records table;
- active/recent sessions;
- repos/VCS workspaces touched;
- evidence health summary;
- recent PR links/reports;
- search box.

Record detail must include tabs or sections:

- overview;
- timeline;
- sessions;
- commands/tools;
- evidence;
- artifacts;
- PR/commits/changes;
- raw transcript, collapsed and local-only by default.

Required seeded visual fixtures:

- empty new install;
- imported-history rich install;
- single full Work Record with transcript, tool calls, command output, evidence,
  artifact, PR link;
- partial imported provider record;
- failed command/evidence;
- stale evidence;
- unsafe inferred PR link;
- multi-repo record;
- long paths/long title/large output;
- redacted secret fixture.

Required screenshots:

- overview desktop 1440x900;
- overview mobile 390x844;
- record detail desktop;
- PR report desktop;
- transcript/timeline desktop;
- search results desktop;
- empty state;
- partial state;
- stale evidence state;
- redaction fixture state.

Store screenshot paths in the status file and manually inspect them with
`view_image` or Playwright screenshot review.

## Buildkite And Release Matrix

Define concrete Buildkite pipeline steps before claiming CI is wired.

Required platforms:

- Linux x86_64;
- macOS arm64;
- macOS x86_64 if runner available, otherwise documented blocker;
- Windows x86_64;
- FreeBSD x86_64 for CLI if runner available, otherwise release cross-build plus
  documented runner blocker.

Required target triples:

- `x86_64-unknown-linux-gnu`;
- `aarch64-apple-darwin`;
- `x86_64-apple-darwin`;
- `x86_64-pc-windows-msvc`;
- `x86_64-unknown-freebsd`.

Required steps:

- format;
- lint/clippy;
- unit tests;
- integration tests;
- migration tests;
- fixture import tests;
- shim failure-mode tests;
- dashboard web tests;
- Playwright screenshot tests;
- hosted Worker tests;
- hosted migration tests;
- release dry-run;
- installer smoke test;
- uninstall smoke test.

Required artifacts:

- binaries;
- checksums;
- installer scripts;
- dashboard screenshot bundle;
- test timing JSON;
- coverage report if available;
- release manifest JSON.

Resource-safe defaults:

- script detects total memory and CPU count;
- local default Cargo jobs: min(physical cores, max(1, memory_gb / 3));
- CI default Cargo jobs set per runner class;
- use `CARGO_BUILD_JOBS`;
- use `CARGO_TERM_COLOR=always`;
- use `RUSTFLAGS` only when required and documented;
- Bazel local resources capped;
- do not overlap heavy Rust, Bazel, and Playwright jobs on small runners.

If a runner/platform is unavailable, status must include:

- missing runner label;
- attempted command;
- exact blocker;
- proposed Buildkite agent pool change;
- whether release artifacts can still be produced.

## End State

When this program is complete:

1. `/home/daddy/code/ctx-multi-repo-workspace` is effectively a two-repo working
   setup for `ctx` and `ctx-private`; `control-plane` is no longer an active
   development dependency for this product line.
2. `ctxrs/ctx` on branch `work-record` contains a clean standalone open-source
   Work Recorder product.
3. `ctxrs/ctx-private` contains the private hosted/team service implementation,
   staging configuration, internal operational tooling, and ctx.rs site/docs work
   for the Work Recorder product.
4. Local capture works end to end:
   - setup;
   - import existing local agent history;
   - capture new work through passive hooks/shims/importers;
   - capture Git, jj, and gh activity;
   - store transcript/tool/command/artifact/evidence records;
   - search and context retrieval for future agents;
   - dashboard/report pages.
5. Hosted staging works end to end:
   - auth/team model;
   - sync API;
   - Neon schema/migrations;
   - R2 blob storage;
   - Cloudflare Worker/API deployment path;
   - PR report share pages or staging links;
   - redaction/export policy;
   - no production ctx API cutover.
6. Buildkite is wired and green for the shippable products across the required
   platform matrix.
7. The local machine has ctx set up against this implementation, has ingested
   local agent work, and has an opened dashboard that is visually useful.
8. A final done-ness subagent has adversarially verified the plan, tests, docs,
   screenshots, hosted wiring, release artifacts, and remaining deferrals.

## Implementation Tracks

Run these tracks in parallel with bounded ownership. Every track should have an
implementation worker and a separate review worker where it touches code.

### Track 1: Repo/Product Split

Goal: make `ctxrs/ctx` an unambiguous Work Recorder repo.

Tasks:

- Remove remaining ADE/product ambiguity from README/docs/source.
- Remove or quarantine old ADE code from `ctxrs/ctx`; do not leave a confusing
  mixed product.
- Keep only Work Recorder crates, CLI, dashboard/report UI, docs, tests, release
  scripts, and CI.
- Decide crate/package names and module boundaries:
  - core schema/types;
  - local store;
  - capture/import;
  - VCS/PR linking;
  - search/context;
  - dashboard/report server;
  - CLI;
  - hosted sync client.
- Preserve Bazel where useful, but do not carry ADE Bazel complexity forward.
- Update `.gitignore`, release metadata, install scripts, and package metadata.

Review requirements:

- no ADE-only code paths remain in the public Work Recorder repo;
- no old `ctx work` namespace in public docs or CLI unless retained as a hidden
  compatibility alias with explicit tests;
- root README and CLI help describe the same product.

### Track 2: Local Data Model And Storage

Goal: one durable local model that supports local-only usage and later hosted
sync.

Design around these concepts:

- `work_records`;
- `sessions`;
- `session_edges`;
- `runs`;
- `events`;
- `capture_sources`;
- `vcs_workspaces`;
- `vcs_changes`;
- `pull_requests`;
- `files_touched`;
- `evidence`;
- `artifacts`;
- `summaries`;
- `tags`;
- `record_edges`;
- FTS/search indexes.

Requirements:

- SQLite migrations are versioned and tested.
- Large payloads go to blob storage, not inline SQLite rows.
- Every captured row carries source/fidelity/provenance metadata.
- Support parent/child/subagent sessions in one `sessions` table.
- Support Git and jj without Git-only naming in core schema.
- Support many repos per Work Record.
- Support confidence-labeled inferred links.
- Support redaction/export visibility at row/artifact level.
- Support future hosted sync IDs and conflict handling.

Review requirements:

- schema reviewer confirms it supports local, remote-devbox, ephemeral-agent, and
  hosted-sync scenarios;
- migration tests pass;
- storage-size strategy for large tool output is explicit and tested.

### Track 3: Passive Capture

Goal: ctx records useful work without requiring the human to run per-task
commands.

Capture mechanisms:

- provider imports:
  - Codex local session files;
  - Claude Code local transcript/hooks where available;
  - Pi if practical;
  - Cursor/other provider imports only if reliable;
- hooks:
  - Claude/Codex provider hooks for session/subagent lifecycle where supported;
  - record transcript paths and capture fidelity;
- shims/wrappers:
  - `git`;
  - `jj`;
  - `gh`;
- direct CLI:
  - setup/status/doctor/search/context/dashboard/report/publish;
  - rare `attach`/annotation/repair.

Requirements:

- Shims never break the underlying command if ctx capture fails.
- Shims preserve exit code/stdout/stderr.
- Shims log capture errors out-of-band and quietly by default.
- `ctx uninstall` removes shims/hooks/service files cleanly.
- `ctx doctor` validates install, path order, hooks, shims, DB, and writeability.
- Capture works if dashboard/daemon is not running.
- Capture writes to JSONL spool or SQLite safely with locking.
- Setup imports existing history and opens dashboard by default.

Review requirements:

- failure-mode tests for each shim;
- uninstall tests;
- import idempotency tests;
- subagent lifecycle fixtures where providers expose subagent events;
- explicit fidelity labels when provider data is partial.

### Track 4: Search And Agent Access

Goal: future agents are the primary customer of the record.

Commands:

- `ctx search <query> [--json]`;
- `ctx context <query> [--json|--markdown]`;
- `ctx show <record-id> [--json]`;
- `ctx list [--repo ...] [--provider ...] [--since ...] [--json]`;
- `ctx report <record-id> --format markdown|html|json`;

Requirements:

- FTS search over normalized high-signal fields.
- Ranking boosts:
  - explicit record title/notes;
  - primary user messages;
  - manager summaries/decisions;
  - review conclusions;
  - subagent final summaries;
  - subagent internal messages;
  - raw tool output last.
- Filters for repo, provider, primary/subagent, evidence state, PR, time, and
  fidelity.
- JSON output is stable and documented.
- Provide an agent skill/instructions doc for using ctx search/context/report.
- Add benchmarks or scale fixtures for months of local history.

Review requirements:

- agent-API reviewer confirms future agents can answer useful questions without
  scraping UI;
- search benchmark results are recorded;
- no raw secret leakage through search snippets by default.

### Track 5: Dashboard And Work Report UI

Goal: a local browser dashboard that makes recorded work legible.

Use a proven component system or template where useful, such as Tabler, rather
than hand-inventing a rough dashboard.

Dashboard requirements:

- overview of recent work;
- active/newly captured sessions;
- repo/VCS workspace drilldown;
- Work Record detail;
- session transcript/detail;
- command/tool output with truncation/expand controls;
- evidence/test/artifact view;
- PR report view;
- missing/stale evidence view;
- slow/flaky/repeated-failure insights;
- import health and capture health;
- search page.

Work Report requirements:

- first viewport answers:
  - what work is this?
  - what PR/commit/change is related?
  - what changed?
  - what evidence exists?
  - what is stale/missing?
  - what is safe/unsafe to trust?
  - what should a human reviewer do next?
- transcript available but not the first thing by default;
- artifacts/screenshots/links are visible;
- long paths, long outputs, missing data, and partial captures look intentional.

Visual quality gates:

- manual screenshot review is mandatory;
- desktop and mobile screenshots;
- empty/partial/full data states;
- no text overlap;
- no decorative gimmicks;
- sensible spacing and dense developer workflow feel;
- dark/light mode if supported.

Review requirements:

- UI reviewer stores screenshots/artifact paths in the plan directory;
- final done reviewer must inspect screenshots, not just trust tests.

### Track 6: PR Linking And Evidence

Goal: attach Work Records to PRs with useful context, without magic claims.

Requirements:

- Capture explicit `gh pr create` and `gh pr view` activity.
- Infer likely PR links from repo, branch/bookmark, commits, time window, file
  overlap, and captured URLs.
- Store link confidence and evidence source.
- Let user/agent confirm/dismiss inferred links.
- `ctx publish pr <record-id>` upserts a separate PR comment by default.
- Include stable marker in PR comments, for example:
  `<!-- ctx:work-record record_id=... -->`.
- Config to disable automatic PR comments.
- Markdown report includes redacted summary, evidence, artifacts, stale checks,
  and dashboard/local share links.

Review requirements:

- idempotent PR comment tests;
- no PR description mutation by default;
- inferred links never presented as authoritative;
- safe behavior when `gh` is unauthenticated.

### Track 7: Hosted/Team Service In `ctx-private`

Goal: build the hosted shape in parallel so local design does not paint us into a
corner.

Recommended stack:

- Cloudflare Workers for API/auth/webhooks/share pages;
- Neon Postgres for relational hosted data;
- Cloudflare R2 for transcript/artifact/report blobs;
- Cloudflare Queues or equivalent for async ingest/report work;
- Stripe/billing skeleton if already standard in `ctx-private`, otherwise design
  and stub behind feature flags.

Required hosted capabilities:

- users;
- teams/orgs;
- memberships/roles;
- projects/repos;
- devices/installations;
- sync cursors;
- Work Record metadata sync;
- artifact/blob upload;
- redaction/export policy;
- PR report share page;
- GitHub webhook handling or app integration plan;
- audit log;
- retention controls;
- team search API shape;
- staging environment config.

Security/privacy requirements:

- no raw transcripts by default;
- explicit opt-in for full transcript sync;
- local redaction before upload;
- server-side access control tests;
- signed or scoped R2 object access;
- no secrets in logs;
- clear deletion/export path.

Review requirements:

- hosted API contract reviewer;
- Neon migration reviewer;
- Cloudflare/R2 deployment reviewer;
- security reviewer.

### Track 8: Site And Docs

Goal: redo ctx.rs docs/site locally around Work Recorder. Do not publish unless
explicitly told.

Required docs/site pages:

- homepage with current banner/tagline;
- getting started;
- install;
- concepts:
  - Work Records;
  - sessions/subagents;
  - capture fidelity;
  - evidence;
  - PR reports;
  - local storage;
  - hosted sync;
- CLI reference;
- agent usage guide;
- Git/jj/GitHub capture guide;
- privacy/security;
- uninstall;
- hosted/team preview;
- troubleshooting.

Review requirements:

- docs reviewer checks claims against implementation;
- no ADE positioning in ctx docs;
- no hosted claims unless staging path exists or clearly marked preview.

### Track 9: CI, Buildkite, Release, Platforms

Goal: shift-left, resource-safe CI that catches real failures without killing
developer machines.

Required local scripts:

- one fast check command for everyday dev;
- focused Rust test helpers with low jobs;
- Bazel checks if retained;
- web/dashboard checks if web exists;
- migration/schema checks;
- docs/site checks;
- e2e smoke checks;
- release dry-run.

Buildkite requirements:

- Linux;
- macOS;
- Windows;
- FreeBSD if feasible for CLI; if not feasible, document runner/blocker and add
  release cross-build validation;
- artifact upload;
- timing capture;
- flaky-test reporting;
- resource limits for cargo/Bazel jobs.

Release requirements:

- native CLI binaries;
- install scripts:
  - macOS/Linux/FreeBSD: `curl -fsSL https://ctx.rs/install | sh`;
  - Windows: `powershell -ExecutionPolicy ByPass -c "irm https://ctx.rs/install.ps1 | iex"`;
- checksum/signing story;
- rollback/withdraw story;
- update story;
- Windows signing decision documented.

Review requirements:

- CI reviewer verifies Buildkite green evidence;
- release reviewer verifies artifact matrix or documented blocker;
- no merge to `ctx/main`.

### Track 10: Dogfood E2E

Goal: prove the thing with real local agent history and at least one fresh run.

Required:

- install/set up ctx locally from the branch build;
- import existing local agent history from this machine;
- open dashboard in Chrome;
- manually inspect dashboard screenshots;
- run at least one fresh agent-style task in a scratch repo;
- capture Git/jj/gh activity where possible;
- generate evidence;
- generate Work Report;
- create dummy private PR if safe/auth available;
- attach PR report/comment if safe;
- ask a fresh agent to use `ctx search/context` against the recorded work and
  answer questions like:
  - what did agents get stuck on?
  - what commands were slow or failing?
  - what docs would have helped?
  - which PR/report has missing evidence?

Review requirements:

- dogfood report stored in plan directory;
- screenshots stored or artifact-linked;
- dashboard is not blank/sparse for the dogfood data;
- known gaps are fixed or explicitly listed.

## Subagent Program

Use many subagents, but keep write scopes disjoint.

Recommended parallel groups:

1. Product/repo split mapper.
2. Local schema/storage implementer.
3. Capture/import implementer.
4. VCS/PR linking implementer.
5. Search/agent-access implementer.
6. Dashboard/report UI implementer.
7. Hosted Worker/Neon/R2 implementer.
8. Site/docs implementer.
9. CI/Buildkite/release implementer.
10. Dogfood/e2e worker.

Reviewer subagents:

1. Architecture/data model reviewer.
2. Capture fidelity/failure-mode reviewer.
3. Security/privacy reviewer.
4. Hosted/API/access-control reviewer.
5. UI visual reviewer.
6. Agent-access/search reviewer.
7. Docs/claims reviewer.
8. CI/release reviewer.
9. SDLC/process reviewer.
10. Final done-ness reviewer.

Rules:

- Do not delegate the current blocking critical-path action if it must be done
  locally.
- Assign exact file/module ownership for implementation agents.
- Tell workers they are not alone in the codebase and must not revert others.
- Use review agents after each major slice, not only at the end.
- Any significant reviewer finding must be fixed or explicitly recorded as an
  accepted deferral with rationale.
- Keep status files updated after every major slice.

## Build/Resource Policy

This machine has hit memory pressure during broad Rust builds. Implement and use
resource-safe wrappers before running broad checks.

Requirements:

- cap Cargo jobs based on machine memory;
- cap linker parallelism where possible;
- keep sccache or equivalent if available;
- avoid overlapping heavy Rust/Bazel/desktop builds;
- record timings for each CI lane;
- make scripts portable enough that a new machine can run them safely by default.

## Completion Criteria

Do not send a final "done" message until all of these are true:

- `ctxrs/ctx` branch `work-record` builds and tests locally.
- `ctxrs/ctx-private` hosted/staging work builds and tests locally.
- Buildkite is green for the configured matrix, or any missing runner/platform is
  documented as the only remaining external blocker with exact next action.
- Release dry-run/artifact generation passes for shippable CLI targets.
- Bazel checks pass if Bazel remains in the product.
- Install scripts are tested or dry-run tested.
- `ctx setup` works from the branch build on this machine.
- Existing local agent history is imported.
- Dashboard opens in Chrome and is visually inspected with screenshots.
- Search/context commands return useful agent-readable JSON/markdown.
- PR report generation and publish/upsert behavior is tested.
- Git, jj, and gh capture paths are tested.
- Capture failure cannot break underlying git/jj/gh commands.
- Uninstall removes shims/hooks/services and is tested.
- Hosted staging API, Neon migrations, and R2 blob flow are wired and tested.
- Security/privacy review returns PASS.
- Hosted access-control review returns PASS.
- UI visual review returns PASS.
- Docs/claims review returns PASS.
- CI/release review returns PASS.
- SDLC/process review returns PASS.
- Final done-ness subagent returns PASS against this plan.
- Status file includes:
  - commits/branches;
  - validation commands and exact outcomes;
  - Buildkite URLs;
  - staging URLs;
  - dashboard URLs/screenshots;
  - dogfood Work Record IDs;
  - dummy PR URL if created;
  - accepted deferrals, if any;
  - reviewer agent IDs/verdicts.
- Worktrees are clean except for intentional uncommitted local-only artifacts.
- Nothing is pushed to `ctx/main`.

## Manager Monitoring Instructions

After receiving this plan, continue end to end without sending a final message
until completion criteria are satisfied. Use progress updates, subagents, and
reviewers liberally. If genuinely blocked, state the blocker and proposed fix,
then continue with unblocked work.

The supervising task will check in roughly every 30 minutes. Treat those checks
as accountability, not permission to stop.
