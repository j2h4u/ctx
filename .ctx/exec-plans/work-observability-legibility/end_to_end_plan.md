# Work Observability And Legibility End-To-End Plan

## Context

Task: `feb64c1c-e58c-40f8-b1e9-1094dca0646e`

Canonical worktree:

`/home/daddy/code/ctx-multi-repo-workspace/worktrees/ctx/agent-work-semantics-primary`

Canonical branch:

`ctx/agent-work-semantics-primary`

This plan builds on the Work-first productization pass ending at:

`bf11a2c Record Work productization follow-up`

The previous pass made ctx a credible local Work record foundation:

- Work-first README/docs.
- `ctx setup workspace|scratch|status|uninstall`.
- `ctx work` schema/list/show/import/export/validate/inspect/redaction-preview.
- `ctx work capture command`, `link-pr`, `note`, `recent`.
- user-local `git`/`gh` shim capture.
- local Work graph storage around `ChangeSet` and `Contribution`.
- ADE/web projections over the same graph.

This pass is the first shippable **Work observability and legibility** slice.
It should not try to build every future hosted/control-plane feature. It should
make local Work records actually usable by agents and humans.

## Product Goal

After this pass, a user should believe:

> ctx records useful local Work from coding agents, lets future agents retrieve
> the relevant context, and gives humans a readable Work Report for a task,
> change, or PR.

Agents are the primary customer. Humans are the reviewer/operator customer.

The hero workflows are:

1. Agent asks: "What relevant prior Work exists for this new task?"
2. Human asks: "What happened in this Work record and what evidence exists?"
3. Reviewer asks: "Is the evidence fresh for this PR/change set?"

## Scope Discipline

This plan intentionally cuts the work to one vertical slice.

Required P0 slice:

- durable Work Record identity;
- safe local capture/indexing from existing ctx sessions plus `git`/`gh` command
  capture;
- evidence records with freshness/staleness semantics;
- deterministic summary/context records with citations;
- SQLite FTS search over redacted searchable text;
- stable agent contract through CLI JSON and daemon API;
- Work Report page focused on reviewer legibility;
- tests, reviews, validation, and final done-ness pass.

Optional P1 only if P0 is truly complete:

- MCP tools/resources over the same daemon contract;
- provider-backed LLM summarization;
- Git notes/trailers for external durable entrypoints;
- richer artifact previews;
- semantic/vector search.

Out of scope unless explicitly approved:

- hosted/team sync;
- billing/org administration;
- remote push, PR creation, release;
- replacing SQLite with a graph database;
- arbitrary executable UI/plugin runtime;
- broad namespace migration from `agent-work` compatibility internals.

## External Comparison

Entire's public direction is checkpoint-first:

- commit-linked checkpoints pair code changes with the agent session that
  produced them;
- setup installs git/agent hooks and stores metadata on a separate
  `entire/checkpoints/v1` branch;
- web views show checkpoint detail, sessions, timeline, tool calls, diffs, AI
  attribution, and commit/PR links;
- skills let agents search past work, explain code, and hand off sessions.

Sources reviewed:

- `https://entire.io/`
- `https://github.com/entireio/cli`
- `https://docs.entire.io/web/review-checkpoint`
- `https://docs.entire.io/web/inspect-session`
- `https://entire.io/blog/improving-agentic-search-in-coding-agents`
- `https://github.com/entireio/skills`

Copy:

- low-friction capture: setup once, keep using existing agents;
- durable commit/PR entrypoints;
- many-to-many session/change modeling;
- web detail page with session/timeline/change views;
- agent-invokable search/explain/handoff workflows;
- explicit subagent session support.

Differentiate:

- ctx Work records should be reviewable and evidence-aware, not only historical;
- trust should be visible as source/fidelity/freshness, not implied;
- raw telemetry should remain local by default;
- reports should answer "what evidence exists and is it fresh?" not just "why
  was this written?"

## Core Architecture Decision

Keep SQLite as the local source of truth. Do not add a graph database.

Use relational graph tables:

- `work_records` owns identity and lifecycle;
- link tables connect records to tasks, sessions, runs, change sets, PRs,
  commits, files, artifacts, and evidence;
- event/evidence/summary tables own legibility primitives;
- FTS tables index redacted searchable text;
- JSON columns preserve provider-specific detail;
- daemon APIs are the stable contract;
- CLI/ADE/MCP are clients of the same model.

Raw SQLite is not the agent/user contract. Agents and integrations should use:

- `ctx work ... --json`;
- daemon JSON APIs;
- MCP tools/resources if implemented;
- exported Work bundles.

## Work Record Identity Rules

Make Work Record durable and non-optional.

A Work Record starts when one of these happens:

- ADE creates a task/session;
- CLI setup/capture sees agent-related command activity for a registered
  workspace and no matching Work Record exists;
- `ctx work link-pr` links a PR URL and no matching Work Record exists;
- `ctx work link-commit` links a commit and no matching Work Record exists;
- import/backfill creates a record from historical data.

Grouping rules:

- ADE task is the strongest grouping key.
- A PR URL is a strong grouping key.
- A commit SHA is a strong grouping key.
- A session without task/PR/commit gets an active Work Record.
- CLI-only command capture without task/PR/commit attaches to an ambient branch
  Work Record keyed by `workspace_id + repo_root + branch + head_sha`, then can
  be merged/linked later.
- Subagent sessions attach to the parent Work Record when parent lineage is
  known.
- One Work Record may link many sessions/runs/change sets/commits/PRs.
- One PR/commit should not silently fork multiple Work Records; if duplicates
  exist, the report must flag merge-needed state.

Split/merge rules:

- `ctx work link-pr` should reuse an existing matching Work Record when possible.
- A future `ctx work merge <from> <to>` may merge duplicates; not P0 unless
  duplicate handling requires it.
- No automatic destructive merge. Preserve source links and diagnostics.

## P0 Data Model

### `work_records`

Owns identity and report lifecycle.

Fields:

- `work_id`;
- `workspace_id`;
- title/objective;
- lifecycle:
  - `active`;
  - `waiting`;
  - `blocked`;
  - `ready_for_review`;
  - `merged`;
  - `abandoned`;
- primary repo root;
- primary branch;
- base/head commit where known;
- current diff fingerprint where known;
- trust verdict:
  - `verified`;
  - `stale`;
  - `missing_evidence`;
  - `partial`;
  - `untrusted_local_capture`;
  - `failed`;
- summary freshness:
  - `missing`;
  - `fresh`;
  - `stale`;
  - `partial`;
  - `locked`;
- created/updated timestamps;
- schema version;
- canonical JSON payload for forward compatibility.

### `work_record_links`

Typed edges from a Work Record to existing ctx objects.

Link target kinds:

- task;
- session;
- run;
- change set;
- contribution;
- pull request;
- commit;
- branch;
- worktree;
- artifact;
- evidence;
- summary;
- file;
- external.

Fields:

- `link_id`;
- `work_id`;
- `workspace_id`;
- target kind;
- target ID or JSON ref;
- role:
  - source;
  - result;
  - evidence;
  - context;
  - parent;
  - child;
  - related;
- created/updated timestamps;
- source/fidelity/trust labels.

### `work_events`

Normalized timeline events for legibility and agent retrieval.

Sources:

- session event;
- user/assistant message;
- tool call start/end/output;
- command capture;
- artifact creation;
- diff/change-set update;
- PR link/update;
- evidence/check run;
- summary generation;
- import/export.

Fields:

- `event_id`;
- `work_id`;
- `workspace_id`;
- monotonic sequence;
- source record pointer;
- event type;
- event time;
- actor kind:
  - human;
  - agent;
  - subagent;
  - system;
  - plugin;
- provider/harness/model where known;
- redaction/sensitivity class;
- trust/fidelity/source labels;
- payload JSON;
- redacted/searchable text;
- optional artifact/object refs.

### `work_evidence`

Evidence is an observed claim, not proof by itself.

Kinds:

- command;
- test;
- lint;
- format;
- typecheck;
- build;
- screenshot;
- recording;
- log;
- manual review;
- agent review;
- CI result;
- artifact inspection.

Statuses:

- observed_pass;
- observed_fail;
- skipped;
- unknown;
- stale;

Evidence must record:

- command/tool/argv, redacted;
- cwd;
- environment redaction policy;
- started/finished timestamps;
- exit code;
- output/artifact refs;
- repo root;
- head SHA;
- branch;
- dirty/untracked state;
- dependency fingerprint where practical;
- diff fingerprint at time of evidence;
- freshness against current fingerprint;
- source/fidelity/trust labels.

UI/API wording constraint:

- Prefer "observed command exited 0" over "tests passed" unless the command is
  classified as a known test command.
- Never imply local user-space shim evidence is tamper-proof.

### `work_summaries`

Summaries are derived records, not truth.

Kinds:

- `live_summary`;
- `context_summary`;
- `report_summary`;
- `decision_log`;
- `evidence_summary`.

Fields:

- `summary_id`;
- `work_id`;
- kind;
- audience:
  - agent;
  - human;
  - reviewer;
- markdown/text;
- structured JSON;
- generation method:
  - deterministic;
  - agent_submitted;
  - provider_llm;
  - manual;
- provider/model/template if applicable;
- whether source material left the machine;
- generated_at;
- freshness;
- source revision keys.

### `work_summary_claims`

Claim-level citations for summaries/context packs.

Fields:

- `claim_id`;
- `summary_id`;
- `work_id`;
- claim text;
- claim kind;
- source kind;
- source ID;
- record hash if available;
- freshness;
- redaction class.

### `work_search_docs`

Use SQLite FTS5 with normalized filter columns outside the FTS text.

Required columns:

- `workspace_id`;
- `work_id`;
- `doc_type`;
- `source_id`;
- `source_kind`;
- `event_time`;
- `repo_root`;
- `path`;
- `branch`;
- `commit_sha`;
- `pr_owner`;
- `pr_repo`;
- `pr_number`;
- `agent_provider`;
- `freshness`;
- `redaction_class`;
- `title`;
- `search_text_redacted`;

Requirements:

- Do not index raw transcript/tool text by default.
- Index redacted searchable text.
- Keep raw payload local and separately permissioned.
- Support chunked/lazy reindexing, not large startup migrations.
- Add an FTS5 availability test.
- Add a rebuild command or store API for search docs.

## Privacy Pipeline

Default rule:

> Search, context packs, reports, MCP resources, and exports use redacted
> searchable/renderable payloads unless the caller explicitly requests raw local
> detail and is authorized to see it.

Pipeline:

1. Ingest raw local event/payload.
2. Classify source and sensitivity.
3. Store raw payload only in local-private storage.
4. Produce redacted searchable text.
5. Index only redacted searchable text by default.
6. Produce report/context payloads from redacted records by default.
7. Require explicit local-only raw expansion for raw transcript/tool output.
8. Record access mode when raw payload is used.

Do not send raw transcripts, tool outputs, command logs, screenshots, file
contents, or proprietary snippets to a provider-backed summarizer without
explicit opt-in and recorded data classes.

Provider-backed summary requirements if implemented:

- redacted bounded prompt;
- explicit provider/model/template recorded;
- `source_material_left_machine: true`;
- source record IDs recorded;
- stale/partial state supported;
- tests proving redaction before provider call.

## Artifact Safety

Artifacts must be content-addressed local objects or existing ctx artifact refs.

Requirements:

- canonicalize paths;
- reject `..` traversal and symlink escapes;
- do not serve arbitrary absolute local files;
- enforce max size;
- MIME sniff or classify unknown/binary;
- sensitivity labels per artifact;
- export allowlist;
- stale/deleted artifact refs handled gracefully;
- screenshots/logs default to local/private unless explicitly exported.

Tests must cover:

- symlink;
- `../`;
- absolute path outside workspace;
- large file;
- binary file;
- sensitive filename/log text;
- deleted/stale artifact ref.

## Trust And Fidelity Threat Model

Visible trust states:

- `verified`: evidence is fresh for the current fingerprint and came from a
  trusted/known source.
- `stale`: evidence or summary source revisions no longer match current Work.
- `missing_evidence`: no relevant evidence exists.
- `partial`: some inputs are missing, imported, or redacted.
- `untrusted_local_capture`: record came from user-space shims/import/manual
  claims and is useful context only.
- `failed`: relevant evidence failed.

Truth rules:

- Git fingerprints correlate evidence with file state. They do not prove
  authorship or complete execution history.
- `git`/`gh` shim capture is observed, bypassable, user-space telemetry.
- User/agent notes are claims, not verified facts.
- Command output is untrusted unless independently verified.
- Summaries are derived and must cite sources.
- Artifacts can be misleading or stale; show sensitivity and freshness.

## Freshness Algorithms

Use revision keys.

Work revision keys:

- latest event sequence;
- latest transcript/message sequence;
- artifact set revision;
- current Git diff fingerprint;
- evidence hash set;
- review hash set;
- summary source hash set.

Evidence freshness:

- `fresh`: current Git fingerprint equals evidence fingerprint and linked
  artifact refs exist.
- `stale`: Git fingerprint changed or linked artifact refs changed/deleted.
- `partial`: imported/observed data lacks a fingerprint or untracked-file state
  is unknown.
- `unknown`: evidence source did not provide enough data.

Summary freshness:

- `fresh`: all recorded source revision keys match current Work state.
- `stale`: any important source revision changed.
- `partial`: generated from incomplete/redacted data.
- `missing`: no summary exists.
- `locked`: immutable summary bound to a merged/archived state.

## Compatibility And Export

Do not break current `ChangeSet`/`Contribution` import/export.

Preferred compatibility shape:

- keep current v1 export behavior;
- add optional Work v2 sections:
  - `work_records`;
  - `work_record_links`;
  - `work_events`;
  - `work_evidence`;
  - `work_summaries`;
  - `work_summary_claims`;
- old v1 imports continue to work;
- v2 import validates workspace ownership and source/fidelity labels;
- downgrade/export-v1 omits Work v2 records while preserving change
  sets/contributions.

If schema v2 is too large, keep Work v2 local-only and document that export is a
P1 follow-up. Do not silently corrupt v1 exports.

## Durable Git/PR Entrypoints

P0 must support local durable lookup:

- `ctx work link-pr <url>`;
- `ctx work link-commit <sha>`;
- `ctx work search --pr <url>`;
- `ctx work search --commit <sha>`;
- Work Report shows PR and commit links.

P1 optional:

- Git notes under a ctx-owned ref;
- commit trailer helper;
- PR comment/annotation helper;
- git-remote/ledger publication.

Do not mutate user commits by default.

## Agent Contract

P0 agent-native surface is CLI JSON plus daemon JSON API.

If MCP implementation is straightforward, add MCP. If not, record MCP as P1 and
ship clear agent instructions showing how to call `ctx work ... --json`.

### `ctx work search --json`

Example response:

```json
{
  "query": "onboarding sandbox failure",
  "results": [
    {
      "work_id": "wrk_123",
      "title": "Fix Linux sandbox launch failure",
      "score": 0.82,
      "matched_fields": ["summary", "evidence", "path"],
      "workspace_id": "w_...",
      "repo_root": "/repo",
      "state": "ready_for_review",
      "trust_verdict": "stale",
      "summary_freshness": "fresh",
      "linked_prs": ["https://github.com/ctxrs/ctx/pull/123"],
      "citations": [
        {
          "source_kind": "summary",
          "source_id": "sum_...",
          "freshness": "fresh"
        }
      ]
    }
  ]
}
```

Ranking rules:

- exact PR/commit/path matches beat text matches;
- fresh summaries/evidence rank above stale;
- same workspace/repo ranks above other workspaces;
- recent work ranks above old work when scores tie;
- trust state affects ranking but does not hide results.

No results behavior:

- return empty `results`;
- include suggested next commands;
- never fabricate context.

### `ctx work context <work-id> --json`

Example response:

```json
{
  "work_id": "wrk_123",
  "budget_tokens": 12000,
  "title": "Fix Linux sandbox launch failure",
  "state": "ready_for_review",
  "trust_verdict": "stale",
  "context": {
    "objective": "Fix sandbox startup failures on Linux",
    "current_result": "Patch implemented; evidence stale after diff changed",
    "relevant_files": ["core/crates/..."],
    "key_decisions": [
      {
        "text": "Use host fallback when bubblewrap is unavailable.",
        "citations": [
          {
            "source_kind": "event",
            "source_id": "evt_...",
            "freshness": "fresh"
          }
        ]
      }
    ],
    "evidence": [
      {
        "evidence_id": "ev_...",
        "claim": "Observed cargo test exited 0",
        "freshness": "stale",
        "status": "observed_pass"
      }
    ],
    "open_risks": ["Evidence is stale for current diff fingerprint"]
  },
  "raw_transcript_available": true,
  "raw_transcript_included": false
}
```

Token budget behavior:

- include summaries before raw events;
- include citations before raw excerpts;
- include evidence/risk state even under small budgets;
- exclude raw transcript by default.

## Daemon API

P0 endpoints:

```text
GET /api/workspaces/:workspace_id/work
GET /api/workspaces/:workspace_id/work/:work_id
GET /api/workspaces/:workspace_id/work/:work_id/context
GET /api/workspaces/:workspace_id/work/:work_id/report
GET /api/workspaces/:workspace_id/work/:work_id/timeline
GET /api/workspaces/:workspace_id/work/:work_id/evidence
POST /api/workspaces/:workspace_id/work/:work_id/evidence
POST /api/workspaces/:workspace_id/work/:work_id/summaries
```

Requirements:

- daemon auth required;
- localhost binding by default;
- workspace authorization checks;
- no cross-workspace search leakage;
- cursor pagination;
- explicit filters by repo/path/commit/PR/session/task/state/source/freshness;
- route-contract response types;
- daemon and HTTP tests.

## CLI

P0 commands:

```bash
ctx work search "query" --json
ctx work search --path core/foo.rs --json
ctx work search --pr https://github.com/org/repo/pull/123 --json
ctx work search --commit abc123 --json

ctx work context <work-id> --budget 12000 --json
ctx work report <work-id> --json
ctx work report <work-id> --markdown
ctx work timeline <work-id> --json

ctx work evidence <work-id> list --json
ctx work evidence <work-id> add --kind screenshot --file out.png
ctx work evidence <work-id> run --kind test -- cargo test -p foo
ctx work evidence <work-id> freshness --json

ctx work summarize <work-id> --kind context --json
ctx work link-commit <sha>
```

Default human output should stay compact. `--json` is the stable agent contract.

## Work Report UX

Build a dedicated Work Report page.

Suggested route:

```text
/workspaces/:workspaceId/work/:workId
```

First viewport must answer:

- what is this Work?
- what code/PR/commit does it relate to?
- what is the trust verdict?
- what evidence exists?
- is evidence fresh?
- what should the reviewer do next?

First viewport layout:

1. Header
   - title/objective;
   - repo/branch;
   - PR/commit links;
   - lifecycle state;
   - last activity.

2. Trust verdict strip
   - `verified`, `stale`, `missing evidence`, `partial`, `untrusted local
     capture`, or `failed`;
   - one-line reason;
   - recommended next action.

3. Evidence summary
   - passing/failing/stale/missing counts;
   - most important evidence rows;
   - fingerprint match/mismatch.

4. Change summary
   - files changed;
   - risky/generated/dependency/config files;
   - link to diff review.

Secondary tabs/sections:

- context summary;
- timeline;
- decisions and rationale;
- agent trace;
- artifacts;
- provenance/export.

Raw transcript/tool output must be collapsed and local-only by default.

## Capture Integration

P0 producers:

- session creation:
  - create/update Work Record;
  - link task/session/worktree.
- user/assistant messages:
  - create redacted searchable events;
  - update revision keys.
- tool calls/results:
  - create timeline events;
  - classify read/edit/bash/search/web where possible.
- artifact creation:
  - link content-addressed artifact ref;
  - create timeline event.
- command capture:
  - create Work Event and Evidence if command is test/build/lint/format;
  - preserve current Contribution behavior for compatibility.
- PR link:
  - attach PR to Work Record and ChangeSet;
  - create event.
- evidence command:
  - create `work_evidence` with fingerprint/freshness.

## Search

Use SQLite FTS5 over redacted `work_search_docs`.

P0 search must support:

- text query;
- path filter;
- PR filter;
- commit filter;
- workspace/repo filter;
- freshness/trust filter;
- limit/cursor.

Add lazy/chunked indexing:

- new writes index immediately;
- historical backfill runs in chunks or on explicit `ctx work index rebuild`;
- startup must not block on full-history reindex.

## Summary Generation

P0:

- deterministic or agent-submitted summaries only;
- summary records with source citations and freshness;
- `ctx work summarize` can generate simple deterministic summaries and accept
  agent-provided summaries if implemented safely.

P1:

- provider-backed LLM summaries with explicit opt-in and redacted bounded input.

No P0 feature may require sending local private transcript data to an external
provider.

## Staged Implementation

### Phase 1: Store and model

- migrations for `work_records`, `work_record_links`, `work_events`,
  `work_evidence`, `work_summaries`, `work_summary_claims`, `work_search_docs`;
- store APIs;
- compatibility with existing ChangeSet/Contribution;
- FTS5 availability and rebuild tests.

### Phase 2: Capture and indexing

- create Work Records from session creation and CLI capture;
- materialize messages/tool calls/artifacts/commands/PR links into Work Events;
- index redacted search docs;
- add evidence capture for command/test/build/lint/format.

### Phase 3: Agent contract

- `ctx work search --json`;
- `ctx work context --json`;
- `ctx work report --json/--markdown`;
- `ctx work timeline --json`;
- `ctx work evidence ...`;
- `ctx work link-commit`;
- daemon API equivalents;
- agent usage docs.

### Phase 4: Work Report UI

- web state/store;
- route/page;
- first-viewport trust/evidence/change summary;
- secondary tabs;
- component tests and visual checks.

### Phase 5: Reviews and hardening

- security/privacy review;
- architecture/data-model review;
- product/UX review;
- agent-contract review;
- test-coverage review;
- SDLC/resource review;
- final done-ness review.

## Testing Requirements

Required coverage:

- migrations and CRUD;
- WorkRecord identity creation/reuse;
- duplicate PR/commit handling;
- link validation and cross-workspace rejection;
- FTS indexing/search filters;
- redaction before FTS/context/report/export;
- summary freshness invalidation;
- evidence freshness against Git fingerprint and dirty/untracked state;
- artifact safety;
- CLI JSON stability;
- daemon route contracts and HTTP tests;
- Work Report component tests;
- seeded end-to-end scenario:
  - create workspace;
  - create task/session/run;
  - emit messages/tool calls/artifact;
  - link PR and commit;
  - add evidence;
  - generate summary;
  - search prior Work;
  - produce context pack;
  - open Work Report.

Security tests:

- secrets in prompts;
- secrets in command args/env;
- secrets in tool output/logs;
- artifact path traversal;
- symlink escape;
- outside-workspace path;
- large/binary artifact;
- deleted/stale artifact ref;
- cross-workspace access attempt;
- raw transcript not included by default.

## Resource-Safe Validation

Use staged validation tiers.

Tier 1, always:

- `git diff --check`;
- formatting for touched packages;
- focused unit tests for touched Rust crates/packages;
- focused web tests for touched components/state.

Tier 2, before done-ness review:

- `scripts/dev/cargo-safe.sh test` for touched Rust packages, serial where
  needed;
- `pnpm -C core/apps/web typecheck`;
- `pnpm -C core/apps/web lint`;
- focused or full web test suite depending on touched surface;
- route-contract/HTTP focused tests.

Tier 3, only if host resources allow or explicitly approved:

- broad Rust workspace tests;
- broad Bazel/Buildkite sweeps;
- desktop package build.

Record exact commands/results under this plan directory.

## Subagent Program

The primary agent should run this as a manager.

Implementation teams:

1. Data model/store worker.
2. Capture/indexing worker.
3. CLI/agent-contract worker.
4. Daemon/API worker.
5. ADE Work Report worker.
6. Docs/product worker.

Review teams:

- architecture/data-model reviewer;
- product/UX reviewer;
- agent-access reviewer;
- security/privacy/redaction reviewer;
- test-coverage reviewer;
- SDLC/resource-safety reviewer;
- final done-ness reviewer.

## Final Done Criteria

The primary agent must not send a final "done" message until a dedicated
done-ness subagent says PASS.

Required before PASS:

- durable Work Record identity exists and is tested;
- CLI and daemon API let agents search and fetch bounded context packs;
- Work Report page exists with trust verdict, evidence, change summary, and
  timeline/detail expansion;
- evidence freshness/staleness exists and is visible;
- summaries exist as derived records with citations and freshness;
- redaction/privacy pipeline protects default search/context/report/export;
- artifact safety is covered;
- docs explain agent and human flows honestly;
- all material review findings fixed;
- validation commands/results recorded under this plan directory;
- branch status clean.

Accepted deferrals must be explicit in final status.

