# Work Productization End-To-End Plan

## Context

This plan continues task `feb64c1c-e58c-40f8-b1e9-1094dca0646e` from the
canonical ctx branch:

- Worktree:
  `/home/daddy/code/ctx-multi-repo-workspace/worktrees/ctx/agent-work-semantics-primary`
- Branch: `ctx/agent-work-semantics-primary`
- Latest known HEAD when this plan was written:
  `c481fb0 Record final polish commits`

The current branch is a strong local foundation, but it is not yet the full
Work-first product. The main remaining gap is not another ADE UI pass. The gap
is making `ctx work` useful without requiring the ctx ADE: setup, capture,
linking, honest docs, and proof.

The product direction is:

> ctx is a local Work record system for coding agents, with an optional ADE as
> the rich UI over the same records.

Use "Work" as the public noun. Avoid "control plane" in public user-facing
copy. Avoid exposing new `agent-work` phrasing to users except where needed for
compatibility in schemas, APIs, or existing internal paths.

## Objective

Make the current branch fulfill the immediate Work-first vision:

1. A CLI-only user can install/setup ctx for a workspace and have useful Work
   records created or enriched without adopting the ADE.
2. The ADE uses the same Work substrate and is presented as an optional rich
   UI over those records.
3. Git/GitHub linking is real enough to support the next public README claim.
4. The root README and docs are rewritten around Work-first positioning without
   overpromising hosted/team sync or arbitrary plugin UI.
5. Local validation, adversarial review, and documentation review all pass.

Do not push to remote. Do not create a PR unless explicitly asked.

## Product Decisions Already Made

- Public noun: `Work`.
- Top-level organizational construct: workspace.
- Workspace can contain one or more repos.
- Default storage policy: no repo-local `.ctx` by default. Use user-local ctx
  state under `~/.ctx`. Repo-local files are explicit opt-in only.
- CLI-only support matters. The ADE must not be required for Work recording.
- No required `ctx work begin` step for normal usage. Setup is allowed; daily
  work should be automatic where practical, with agent-mediated fallback.
- Capture/linking strategy:
  - Automatic capture/linking where robust and low-friction.
  - Agent-mediated enrichment where automatic capture cannot know intent.
  - Git/GitHub linking should start with practical local mechanisms, not vague
    "ctx can inspect the repo" claims.
- Start with `git` and `gh` only for shims/hooks.
- Prefer Git Trace2 for Git facts if it gives sufficient coverage. Use a shim
  if Trace2 is insufficient for command invocation metadata or task/workspace
  context propagation.
- `gh` has no equivalent universal Trace2-style event stream; use shim or
  agent-mediated commands for PR linking.
- Provider management should be low-friction:
  - Prefer installed harnesses on `PATH`.
  - Use existing harness auth automatically.
  - Managed downloads/sandbox/provider pinning should be optional advanced
    setup, not first-run friction.
- Scratch workspace should be an empty git repo in user-local ctx storage, with
  no sandbox by default.
- Hosted/team sync is not the next implementation priority. Keep schema/design
  compatible with future hosted Work sharing, but do not build hosted just to
  force shapes.
- ADE-level plugins/extensions are useful, but not the immediate headline.
  Current declarative Workbench/plugin substrate can stay, but Work capture and
  positioning are higher priority.

## Non-Goals For This Pass

- Hosted/team product implementation.
- Remote CI/Buildkite execution unless specifically available and safe.
- Production release publishing.
- Arbitrary executable webview/UI plugin runtime.
- Full Pi-like plugin marketplace.
- Rebuilding all ADE layout around composable custom UIs.
- Pushing or opening a PR.
- Migrating every remaining internal `agent-work` compatibility name if it
  would create churn. Public UX should say Work; compatibility layers can remain
  if documented.

## Workstreams

Use subagents aggressively. The main agent should coordinate, integrate, and
verify. Each implementation worker needs an explicit write scope. Each review
worker should be read-only unless reassigned.

### 1. CLI Setup And Workspace UX

Goal: a new user can run a small setup command and understand where Work is
stored and what will be captured.

Implement or complete:

- `ctx setup` or equivalent workspace setup entrypoint if missing.
- Workspace creation/listing/selection semantics for CLI-only users.
- User-local storage defaults under `~/.ctx`.
- Scratch workspace creation as an empty git repo under user-local state.
- Clear behavior for:
  - path is git root;
  - path is inside git repo;
  - path is not a git repo;
  - multi-repo workspace root;
  - existing workspace.
- A reversible setup path:
  - `ctx setup uninstall`, `ctx setup remove`, or equivalent command that can
    remove installed shims/hooks/env snippets owned by ctx.
  - Must be idempotent and safe if files are missing.

End condition:

- CLI help and docs explain setup without requiring the ADE.
- Tests cover setup, idempotency, uninstall, and storage-path behavior.

### 2. Automatic Git/GitHub Capture

Goal: Work records get created/enriched when existing agent workflows use Git
or GitHub CLI, without the human manually starting a Work item.

Investigate first, then implement the least fragile path:

- Git:
  - Decide whether `GIT_TRACE2_EVENT` is sufficient.
  - If sufficient, use Trace2 for command facts and keep shims minimal or
    unnecessary.
  - If insufficient, implement a `git` shim that forwards perfectly and records
    best-effort metadata.
- GitHub CLI:
  - There is no known `gh` Trace2 equivalent.
  - Implement a `gh` shim or agent-mediated fallback.
- Shims must:
  - apply only inside ctx-managed agent/session environments where possible;
  - never break the underlying command if ctx capture fails;
  - preserve argv, stdin/stdout/stderr, exit code, cwd, env behavior;
  - record capture failures separately as diagnostics;
  - be easy to uninstall.

Capture at minimum:

- cwd/workspace/repo identity;
- command name and argv classification, with sensitive args redacted;
- git branch/HEAD/base where available;
- commits created;
- PR creation/edit/view events from `gh`;
- PR URL/number/repo owner/name;
- link between PR and current workspace/repo/change set when deterministic.

End condition:

- Tests prove forwarded command behavior and exit code preservation.
- Tests prove capture failure does not break `git`/`gh`.
- Tests prove PR URL/link extraction from representative `gh` commands.

### 3. Agent-Mediated Work Enrichment

Goal: agents can enrich Work records when automatic capture cannot infer intent.

Add or complete narrow CLI commands that agents can call:

- Link current task/session/workspace to a PR URL.
- Add a summary/decision/artifact/check/evidence note to current Work.
- Record a review/evidence result with a change-set fingerprint.
- Query recent Work context for future agent use.

Important constraint:

- Do not require humans to run `ctx work begin`.
- Agent-mediated commands should be optional enrichment, not the primary
  capture path.

End condition:

- There is a clear agent instruction snippet/skill text for Claude, Codex, Pi,
  etc. telling agents when to call `ctx work ...`.
- Tests cover idempotency and duplicate link handling.

### 4. Work Record Data Model Integration

Goal: tighten the model around what users actually need.

Review current `ChangeSet` and `Contribution` semantics:

- ChangeSet = coherent Git diff/fingerprint/PR-bound unit.
- Contribution = relationship/event/link explaining who/what contributed to
  the change set or Work graph.

Ensure:

- PR links can attach to change sets.
- command captures can create contributions or diagnostics without pretending
  they are authoritative evidence.
- imported/observed/admitted sources retain source/fidelity/trust distinctions.
- local-only records do not overclaim tamper-proof audit behavior.

Decision point:

- Review the separate `control-plane` worktree and decide which ledger/hash
  verification ideas should be ported now.
- Recommendation: do not port all hosted/team control-plane behavior now.
  Harvest only source-record verification and export/import safety if missing.

End condition:

- Docs and tests make ChangeSet vs Contribution understandable.
- `ctx work show` output is legible for a real PR-linked example.

### 5. README And Docs Repositioning

Goal: root README and docs match the product we actually want users to try.

Rewrite root README around:

- Work-first value:
  - record what coding agents did;
  - link prompts/transcripts/commands/diffs/PRs/artifacts;
  - make future agents and humans able to inspect/reuse that history.
- ADE as optional rich workbench.
- CLI-only flow.
- Honest capability table:
  - works now;
  - experimental;
  - next.
- Storage policy: user-local by default, no repo-local `.ctx` unless opt-in.
- Provider policy: use installed harnesses on PATH; managed provider setup is
  optional.
- Clear quick-start commands.

Also update:

- `docs/index.mdx`
- Work docs that still imply ADE-first strategy
- plugin docs where root README currently says plugin primitives do not exist.

End condition:

- A new reader should not think ctx is merely an open-source Codex desktop app.
- A terminal-first user should understand why they can try `ctx work` without
  the ADE.
- No public page claims fully automatic capture if that slice is incomplete.

### 6. ADE First-Open UX Alignment

Goal: reduce friction while preserving serious workspace setup.

Implement or document:

- first-open path can enter workbench quickly;
- add "Quick start in scratch workspace" if absent;
- scratch workspace = empty git repo under user-local ctx state, no sandbox;
- existing workspace and serious setup remain available;
- provider selector shows PATH/auth readiness rather than forcing downloads.

End condition:

- UI tests or component tests cover the new first-open/scratch choice.
- README screenshots/copy do not contradict the UX.

### 7. Plugin/Extension Scope Cleanup

Goal: keep plugin work honest and useful without letting it distract from Work.

Clarify and test current plugin surface:

- ADE-level plugin/contribution primitives are declarative and host-owned.
- Provider/harness plugins are not the same as ADE plugins.
- Provider slash commands belong to providers and should pass through when the
  provider/protocol exposes them.
- ctx extension commands should not collide confusingly with provider `/`
  namespace unless deliberately designed.

Do not build arbitrary UI code execution in this pass.

End condition:

- Docs explain current extension capability and future boundaries.
- Existing plugin tests still pass.

### 8. Validation And Review Program

Before claiming done, run:

- `git status --short`
- `git diff --check`
- Rust fmt for touched Rust crates or full workspace if safe.
- Focused Rust tests for all changed crates.
- Web typecheck/lint/tests if web changed.
- Web build if README/docs route imports or web app surfaces changed.
- Buildkite/Bazel local validation where safe.
- Resource-safe wrappers for Rust/Cargo; do not overload the host.

Use review subagents:

- Product/docs reviewer: README honesty, Work-first clarity, no overpromising.
- CLI/capture reviewer: setup/shim/failure-mode correctness.
- Data-model reviewer: ChangeSet/Contribution/source/fidelity semantics.
- Security/privacy reviewer: redaction, local storage, shim safety, no accidental
  hosted/team leak.
- Test reviewer: make sure claims are covered by tests.

End condition:

- All material review findings fixed.
- Any accepted deferrals are explicit and written in the exec-plan status.
- Final branch status is clean except intentional generated/ignored artifacts.

## Suggested Sequencing

1. Spawn read-only explorers for current setup/shim/work CLI/docs/UX state.
2. Decide exact automatic capture mechanism for Git and gh.
3. Implement CLI setup/uninstall and storage/workspace basics.
4. Implement Git/gh capture slice.
5. Implement agent-mediated enrichment commands and instruction snippets.
6. Update README/docs.
7. Align ADE first-open/scratch workspace if feasible in this pass.
8. Run adversarial reviews.
9. Run verification gates.
10. Record final status, validation, and deferrals on disk.

## Do Not Stop Criteria

Do not send a final "done" message until:

- the Work-first README/docs are updated;
- CLI-only setup and capture story is implemented or explicitly downgraded with
  product copy adjusted to match;
- tests prove the new setup/capture/linking behavior;
- review subagents have passed the implementation or all findings are resolved;
- validation gates are recorded;
- branch status is clean.

