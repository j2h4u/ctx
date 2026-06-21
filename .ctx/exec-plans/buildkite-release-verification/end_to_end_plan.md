# Buildkite Release Verification Plan

## Context

Task: `feb64c1c-e58c-40f8-b1e9-1094dca0646e`

Canonical branch:

`ctx/agent-work-semantics-primary`

Canonical worktree:

`/home/daddy/code/ctx-multi-repo-workspace/worktrees/ctx/agent-work-semantics-primary`

Starting head:

`0ad8460 Correct final Work hardening status`

The local Work record / observability / Work Report branch has passed broad
local validation. The user now wants Buildkite/CI to be the release gate and
wants all shippable products covered across required architectures, including
Windows for the CLI.

## Goal

Drive this branch to a CI-backed release-candidate state:

- pushed branch and CI-visible PR/commit as needed;
- Buildkite pipelines configured and green for the shippable products;
- architecture coverage explicit, including Windows for the CLI;
- SDLC audit evidence recorded;
- failures fixed in code/config or documented as concrete infrastructure
  blockers with exact owner/action.

Do not stop with a final completion message until CI is actually green or a
specific external blocker prevents progress.

## Authority And Guardrails

Allowed:

- push `ctx/agent-work-semantics-primary` or a descendant branch to the remote;
- open or update a draft PR if required for Buildkite;
- inspect and modify repository CI/Buildkite config;
- add or update Buildkite pipeline definitions, scripts, and resource-safe
  check wrappers;
- create Buildkite pipelines, queues, pools, or workers only when credentials
  and infra docs make the operation safe and reversible;
- use subagents for CI mapping, platform-specific fixes, SDLC review, and
  final done-ness review.

Not allowed:

- merge to main;
- publish releases or public announcements;
- leak tokens, Buildkite secrets, GitHub credentials, environment dumps, or
  private repo details into logs/docs;
- weaken tests to get green;
- silently skip an architecture or product.

If Buildkite or cloud/worker credentials are missing, record the exact blocker
and the command/API that failed. Do not invent a pass.

## Shippable Product Coverage

Identify the actual shipped surfaces in this repo and make the CI matrix cover
them explicitly.

Minimum expected lanes:

- Rust workspace test/build on Linux x86_64.
- Web app typecheck, lint, test, and production build on Linux x86_64.
- CLI build/test on Linux x86_64.
- CLI build/test on Windows x86_64.
- Desktop/Tauri package build on supported desktop architectures if this repo
  currently ships desktop artifacts.
- Documentation/spec validation if a docs build/check exists.
- Security/privacy/static checks already present in the repo or easy to add
  without noisy false positives.

For each lane, record:

- product;
- platform/architecture;
- command;
- expected artifact, if any;
- Buildkite agent queue/pool;
- pass/fail/blocker;
- log/build URL.

## Fast And Correct Shift-Left

Before spending expensive CI minutes, make sure the branch has a clear local
preflight path:

- resource-safe Cargo wrapper for heavy Rust lanes;
- web package checks before packaging;
- platform-specific CLI build commands separated from desktop packaging;
- docs for how to reproduce CI failures locally;
- no fragile dependency on the running desktop AppImage environment;
- no hidden generated assets required without a generator or documented staging
  step.

If missing, add small scripts/docs/config so future contributors can run the
same checks locally.

## CI/Buildkite Work Program

1. Inspect current CI state:
   - Buildkite pipeline files and dynamic pipeline upload scripts;
   - GitHub branch/PR integration;
   - existing agent queues/pools;
   - platform coverage and known flaky lanes.
2. Push/open CI-visible branch/PR if needed.
3. Run Buildkite on the current branch.
4. For each failure:
   - read logs;
   - classify as code, test, packaging, infra, missing dependency, or flaky;
   - fix the root cause;
   - add regression coverage where appropriate;
   - rerun the smallest failing lane, then the full required matrix.
5. Add missing platform/product lanes if they are truly absent.
6. If a lane requires new Buildkite infrastructure:
   - prefer existing documented queues/pools;
   - if absent and credentials allow, create a minimal named queue/pool/worker;
   - document setup, cleanup, and costs/limits;
   - otherwise record a concrete external blocker.
7. Run final SDLC audit:
   - branch status and commits;
   - CI URLs and verdicts;
   - artifact coverage;
   - security/privacy/log hygiene;
   - no accidental releases;
   - final reviewer PASS.

## Required Artifacts

Create/update:

- `.ctx/exec-plans/buildkite-release-verification/implementation_status.md`
- links to Buildkite builds, GitHub PR, and logs;
- product/platform matrix;
- list of code/config/docs changes;
- remaining accepted deferrals, if any;
- cleanup notes for any created infra.

## Done Criteria

All must be true:

- branch pushed and visible to CI, if needed;
- Buildkite has run required lanes for every shippable product;
- Linux web/Rust/product lanes pass;
- Windows CLI lane passes, or a concrete missing-worker blocker is recorded
  with the exact remediation path;
- desktop/package lanes pass where this repo ships them, or blocker is concrete;
- SDLC/security/log-hygiene review passes;
- final done-ness reviewer passes;
- local worktree is clean;
- no merge, public release, or announcement happened.
