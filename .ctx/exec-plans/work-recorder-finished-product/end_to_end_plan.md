# Work Recorder Finished Product Implementation Plan

## Purpose

The current `ctxrs/ctx` `work-record` branch is a green local MVP. It is not yet
the finished product. The goal of this program is to turn it into a measurable
finished Work Recorder product:

- passive local capture of agent work, not only explicit `ctx record`;
- provider import/hook support for Codex, Claude, and Pi;
- first-class Git and jj evidence;
- PR-ready reports and optional idempotent PR comment publishing;
- local static dashboard/report that makes agent work legible to humans;
- agent-readable search/context/export APIs;
- public installers and released binaries;
- security/threat review and privacy controls;
- hosted/team integration contract where needed;
- Buildkite certification across supported platforms;
- final completion certification by an adversarial done-review agent.

This plan should be handed to the primary implementation agent. The manager
should not implement directly. The primary agent should create and coordinate
implementation/review subagents, merge the work, and continue until the final
completion certificate is green.

## Current Baseline

Public repo:

- Canonical worktree:
  `/home/daddy/code/ctx-multi-repo-workspace/worktrees/ctx/work-record-product`
- Branch: `work-record`
- Latest verified head when this plan was written:
  `83cf0639d659aa35d557a530fe2ca49476af950e`
- Buildkite:
  `https://buildkite.com/luca-king/ctx-public-release-verification/builds/64`
- Buildkite #64 passed:
  pipeline contract, fmt, docs, check, clippy, tests, examples, Bazel,
  Linux smoke/release dry-run, macOS arm64/x64 smoke/release dry-run,
  Windows smoke/release dry-run, and FreeBSD blocker artifact.

Private repo:

- Canonical worktree:
  `/home/daddy/code/ctx-multi-repo-workspace/worktrees/ctx-private/work-recorder-hosted-team`
- Branch: `ctx/work-recorder-hosted-team`
- Contains hosted worker/staging work and private docs/services.

Known MVP capabilities:

- explicit Work Records;
- `ctx evidence run`;
- local Git/jj/gh shims;
- capture spool import/repair;
- search/context/report/dashboard export;
- VCS inspection and PR URL parse/link;
- export/import;
- validate/doctor/uninstall;
- cross-platform release dry-run CI.

Known MVP gaps:

- no passive provider hooks/imports for existing Codex/Claude/Pi history;
- no real Pi/Codex/Claude end-to-end certification;
- no full jj end-to-end workflow;
- no PR comment publisher;
- no public installer URLs or published binaries;
- no hosted sync from public CLI;
- no formal threat model/security audit;
- dashboard is useful but not a full evidence/review page;
- no CI completion certificate.

## Product Definition

Finished means a developer can install ctx, enable local recording, run their
normal agent workflow, and later use ctx or an agent to answer:

- what task happened;
- which agent/provider/session produced it;
- what prompts/messages/tool calls/commands occurred;
- what files/repos/commits/jj changes/PRs were involved;
- what evidence/tests/artifacts were produced;
- what is safe to attach to a PR;
- what can be searched or reused by future agents;
- what can optionally be shared with a team.

The product must preserve the core policy:

- local-first;
- no hosted account required;
- no raw transcript upload by default;
- capture failure must not break the wrapped tool;
- sharing/publishing is explicit and redacted by default;
- uninstall/disable must be reversible and trustworthy.

## Non-Negotiable Completion Criteria

The final done agent must verify every item below. Any skipped item must have a
concrete written launch decision and must not be claimed as supported.

### 1. Passive Capture

Done when:

- `ctx setup` or an explicit enable command can configure local passive capture.
- A user can run a normal agent session without manually creating a record and
  ctx still captures useful work history.
- Capture supports at least:
  - Codex;
  - Claude;
  - Pi;
  - shell command evidence;
  - Git;
  - jj;
  - gh.
- Capture failures are best-effort and never change the wrapped tool's stdout,
  stderr, or exit code.
- `ctx status`, `ctx doctor`, and `ctx repair` expose pending/failed/stuck
  capture state.
- Disable/uninstall removes ctx-owned hooks/shims and stops future capture.
- Tests prove failure isolation and uninstall safety.

### 2. Provider Import And Session Model

Done when:

- Provider adapters import or capture Codex, Claude, and Pi into one normalized
  local model.
- Storage has typed APIs, not raw SQL from adapters, for:
  - records;
  - sessions;
  - runs;
  - turns/events/messages;
  - tool calls;
  - command evidence;
  - artifacts/blobs;
  - files touched;
  - Git commits/branches/worktrees;
  - jj changes/bookmarks;
  - PR links;
  - summaries;
  - source/provenance/redaction metadata.
- Provider imports are idempotent using stable provider session IDs plus event
  index/hash/cursor data.
- Import reports include imported/skipped/failed/redacted counts.
- Fixture replay covers sanitized real provider data.
- Real provider e2e runs exist where credentials/tools are available.

### 3. Search And Agent Access

Done when:

- Agents can use `ctx search`, `ctx context --json`, and/or dedicated commands
  to retrieve prior work with citations.
- Search covers records, transcripts/messages, commands, tool calls, evidence,
  files touched, PRs, summaries, and tags.
- Results include why-matched explanations and stable IDs/links.
- Large histories have performance tests and budgeted truncation behavior.
- Redaction policy applies before output to agents.

### 4. Dashboard And Review Report

Done when:

- `ctx dashboard export` produces a static, polished local page that includes:
  - overview metrics;
  - records/tasks;
  - provider sessions/runs;
  - timeline;
  - transcript/message/tool-call views;
  - command evidence and output previews;
  - files touched;
  - Git/jj state;
  - PR links;
  - artifacts;
  - redaction/privacy summary;
  - share/publish preview.
- `ctx report` can generate a deterministic Markdown/JSON PR evidence report.
- Dashboard/report never include raw secrets from the redaction test corpus.
- Screenshot review is required on desktop and mobile widths.
- Visual review agents must reject sparse/empty pages for rich fixture data.

### 5. PR Publishing

Done when:

- `ctx publish pr-comment <record-id> --dry-run` renders exact Markdown without
  network mutation.
- Live publishing can upsert one marker-bounded ctx comment on a GitHub PR.
- Re-running updates the same comment rather than duplicating comments.
- Auth failures and permission errors are clear.
- Raw transcript inclusion is explicit opt-in only.
- Tests include a mock GitHub API and a gated real-private-repo smoke.
- GitLab can be implemented or explicitly deferred; do not claim support unless
  tested.

### 6. Git And jj

Done when:

- Git repo detection, commits, branches, remotes, dirty state, and PR inference
  are tested end-to-end.
- jj support is first-class:
  - `jj root/status/log` parsing;
  - change IDs;
  - bookmarks;
  - working-copy commit;
  - parents;
  - colocated Git compatibility;
  - evidence linked to the observed jj state.
- CI has a real jj e2e lane or an explicit blocker artifact. If jj is claimed
  as supported, the e2e must pass.

### 7. Installer And Release

Done when:

- Public installer URLs work:
  - Linux/macOS shell install;
  - Windows PowerShell install.
- Installers use pinned release metadata/checksums.
- Binaries are published, not only dry-run built, for:
  - Linux x64;
  - macOS arm64;
  - macOS x64;
  - Windows x64.
- FreeBSD is either shipped and tested or explicitly removed from launch scope.
- Clean-machine install smoke proves:
  - install;
  - `ctx setup`;
  - record/search/context/dashboard;
  - provider fixture import;
  - uninstall.
- Release promotion, rollback, checksum, SBOM/provenance, and signing/notarizing
  decisions are documented.

### 8. Hosted/Team Contract

Done when either:

Option A, local-only launch:

- README/site clearly say hosted sync is not part of launch.
- Public CLI has no dead hosted commands.
- Hosted is documented as future direction only.

Option B, hosted/team launch:

- Public CLI can login/connect to staging.
- Sync uploads redacted metadata by default.
- Raw transcript/blob sync is explicit opt-in.
- Team/project identity, retention, visibility, and deletion are defined.
- Hosted worker has staging CI, migrations, R2/Neon readiness, auth tests, and
  API contract tests.
- End-to-end sync from local record to hosted dashboard is proven.

Pick one deliberately. Do not leave hosted half-promised.

### 9. Security And Privacy

Done when:

- There is a written threat model covering:
  - local data root;
  - shims/hooks;
  - provider transcript import;
  - capture spool;
  - archive import/export;
  - dashboard export;
  - PR publishing;
  - installer/update supply chain;
  - hosted sync if in scope.
- File permissions are checked for data root, blobs, inbox, and shims.
- Malicious archive fixtures cannot write outside the data root.
- Symlink/path traversal cases are tested.
- Blob hash verification is tested.
- Redaction corpus covers common tokens/secrets and is used in dashboard,
  search/context, report, PR publish, and hosted sync tests.
- `ctx doctor --privacy` or equivalent privacy health check exists.
- Dependency/license audit runs in CI or is documented with a script.

### 10. CI And Completion Certification

Done when Buildkite latest head passes:

- pipeline contract;
- fmt;
- docs;
- cargo check;
- clippy;
- tests;
- examples;
- Bazel;
- provider fixture import;
- provider real/gated e2e status;
- shim/hook behavior on Linux/macOS/Windows;
- jj e2e;
- malicious archive/security fixtures;
- dashboard/report screenshot review artifacts;
- PR publish mock and gated real smoke;
- release publishing or release dry-run plus explicit launch blocker decision;
- installer smoke on Linux/macOS/Windows;
- hosted staging readiness if hosted is in scope.

Also done when a final completion certificate artifact exists with:

- commit SHA;
- platform matrix;
- provider support matrix;
- jj support status;
- PR publish status;
- installer/release status;
- security review status;
- docs/site links;
- known limitations;
- explicit launch/no-launch recommendation.

## Parallel Workstream Plan

Use manual worktrees under:

- `/home/daddy/code/ctx-multi-repo-workspace/worktrees/ctx/`
- `/home/daddy/code/ctx-multi-repo-workspace/worktrees/ctx-private/`

Create one branch per workstream from latest `origin/work-record` or the
current integration branch. Keep write sets disjoint.

### Workstream A: Storage And Schema Foundation

Branch: `ctx/work-record-storage-rich`

Owns:

- `crates/work-record-core`
- `crates/work-record-store`
- archive schema/versioning
- migration compatibility tests
- validation/doctor extensions

Deliver:

- typed APIs for sessions/runs/events/artifacts/files/vcs/PRs/summaries;
- schema versioning and migrations;
- v1 archive compatibility and v2 rich archives;
- integrity and blob validation.

Blocks provider imports, dashboard detail, and search expansion.

### Workstream B: Provider Import And Passive Capture

Branch: `ctx/work-record-provider-imports`

Owns:

- provider adapters/importers;
- capture envelopes;
- fixture corpora;
- `ctx import provider ...` commands;
- setup/enable/disable capture UX for providers.

Deliver:

- Codex importer/e2e;
- Claude importer/e2e;
- Pi importer/e2e;
- idempotent cursors;
- source/fidelity metadata;
- subagent/session tree handling where exposed by provider logs.

Do not write direct SQL; consume Workstream A APIs.

### Workstream C: Shims, Hooks, And jj

Branch: `ctx/work-record-shims-jj-hooks`

Owns:

- `crates/work-record-capture`
- `crates/work-record-vcs`
- CLI shim/hook commands
- jj parsing/e2e

Deliver:

- Git/jj/gh wrapper robustness;
- shell hook install/status/uninstall if in scope;
- Windows PowerShell/cmd shim behavior;
- real jj repo tests;
- failure-isolation tests.

### Workstream D: Search, Context, And Agent Access

Branch: `ctx/work-record-agent-access`

Owns:

- `crates/work-record-search`
- `ctx search`
- `ctx context`
- agent skill/docs

Deliver:

- search over rich records/events/evidence/files/PRs;
- JSON packets suitable for agents;
- ranking/citations/truncation;
- large-history performance tests.

### Workstream E: Dashboard And Reports

Branch: `ctx/work-record-dashboard-report`

Owns:

- `crates/work-record-report`
- dashboard static assets/rendering
- report generation
- visual review artifacts

Deliver:

- rich dashboard with timeline/transcript/tool/evidence/files/PR panes;
- deterministic Markdown/JSON evidence report;
- screenshot tests/review artifacts;
- redaction tests.

### Workstream F: PR Publishing

Branch: `ctx/work-record-pr-publish`

Owns:

- `ctx publish pr-comment`
- GitHub/GitLab publish clients or `gh` integration
- idempotency markers
- auth/error UX

Deliver:

- dry-run;
- mock API tests;
- gated private-repo smoke;
- publish audit records;
- redaction enforcement.

### Workstream G: Installer, Release, And Buildkite

Branch: `ctx/work-record-release-install`

Owns:

- installers;
- release pipeline;
- checksums/SBOM/provenance;
- platform workers;
- Buildkite completion certificate wiring.

Deliver:

- shell installer;
- PowerShell installer;
- published binary artifacts or a blocked release decision;
- clean-machine install smoke;
- FreeBSD decision.

### Workstream H: Security, Privacy, Docs, And Site

Branch: `ctx/work-record-security-docs`

Owns:

- threat model;
- privacy docs;
- public README/docs/site copy;
- security tests/corpus;
- completion certificate template.

Deliver:

- `SECURITY.md`;
- threat model;
- privacy mode docs;
- redaction corpus;
- docs matching actual behavior;
- site rewrite for Work Recorder.

### Workstream I: Hosted/Team Integration

Branch public: `ctx/work-record-hosted-client` if hosted is in scope.

Branch private:
`ctx-private/work-recorder-hosted-finish`

Owns:

- hosted sync API contract;
- staging readiness;
- Neon/R2 migrations;
- auth/team/project/retention semantics;
- public CLI client commands if chosen.

Deliver either:

- local-only launch with hosted explicitly deferred;

or:

- staging-hosted sync e2e with redacted-default upload and raw opt-in.

## Integration Strategy

1. Freeze launch definition first:
   - local-only finished product, or
   - local plus hosted staging sync.
2. Merge Workstream A early; all richer work depends on typed storage APIs.
3. Merge C shims/jj and B provider imports after A APIs are stable.
4. Merge D search and E dashboard once rich data exists.
5. Merge F PR publishing after E reports and H redaction policy.
6. Merge G release/installers continuously, but final release proof comes last.
7. H security/docs reviews every workstream before final merge.
8. I hosted integration only if chosen as launch scope.
9. Run full Buildkite after every integration merge.
10. Final done-certification agent must review from a clean checkout.

## Required Subagents

The primary agent must create implementation agents for each active workstream,
with disjoint write scopes. It must also create review agents:

- architecture review;
- security/privacy review;
- docs/product truth review;
- CI/release review;
- dashboard visual review using screenshots;
- provider fixture/e2e review;
- final completion certification review.

The final certification agent must be explicitly adversarial and must not be an
implementation agent. Its only job is to verify completion criteria against the
repo, Buildkite, release artifacts, docs, screenshots, and any hosted staging
URLs. It should return PASS only when every criterion is either implemented and
tested or explicitly removed from launch scope.

## Buildkite Token And Monitoring

Buildkite API token:

```bash
token=$(cd /home/daddy/code/ctx-multi-repo-workspace/ctx-private/core && infisical secrets get BUILDKITE_API_ACCESS_TOKEN --plain --env prod --path /)
```

Public pipeline:

```text
https://buildkite.com/luca-king/ctx-public-release-verification
```

API:

```bash
curl -H "Authorization: Bearer ${token}" \
  "https://api.buildkite.com/v2/organizations/luca-king/pipelines/ctx-public-release-verification/builds?per_page=8"
```

Manager monitoring cadence after handoff:

- poll the primary agent every 15 minutes;
- if it is working, wait another 15 minutes;
- if it reports completion, independently verify Buildkite/latest head and the
  completion certificate;
- if it reports blocked, pass concrete blocker context back to it;
- do not spawn parallel implementation agents from the manager unless the user
  explicitly redirects.

## Final Definition Of Done

The work is done only when:

- latest public branch is clean and pushed;
- latest private hosted branch is clean/pushed if hosted is in launch scope;
- Buildkite latest head is green for the expanded completion matrix;
- public installers and binaries are either published and smoke-tested or
  explicitly deferred from launch scope;
- supported providers have fixture and real/gated e2e status recorded;
- jj e2e is green or jj support is explicitly reduced;
- PR report/publish flow is tested;
- dashboard screenshots are reviewed and attached;
- security threat model and test corpus are complete;
- README/docs/site match actual behavior;
- final completion certification agent returns PASS with evidence links.

