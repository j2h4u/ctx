# Architecture Reviews

Record architecture review checkpoints and sign-offs.

## Pending Checkpoints

- After Work CLI/import/export design.
- After plugin SDK/contribution schema implementation.
- After hot reload implementation.
- Before final done-ness review.

## Plan Review Baseline

- Reviewer Ohm inspected the branch direction and first draft. Findings around
  Work source-of-truth, CLI/import/export, plugin security, hot reload, ACP
  provider contract, UX artifacts, review gates, and subagent workflow were
  incorporated into `exec_plan.md`.
- Reviewer Locke inspected the updated plan. Findings around SDK scope,
  storage semantics, ACP conformance, bundle safety, declarative data/action
  contracts, network-adjacent boundaries, and worker base-commit rules were
  incorporated into `exec_plan.md`.

Reviewer agents:

- Ohm: `019ee0b6-defd-7c51-be16-514e06259ca5`
- Locke: `019ee0bb-d702-7541-811a-585a218a38d1`

## Blocking Contract Base

- `3d1b60a` documents the Work namespace, Work source-of-truth/storage,
  ACP provider plugin, and plugin contribution contracts. Broad implementation
  workers must base on this commit or a later manager-owned contract commit.

## Contract Gap Review

- Reviewer Boyle (`019ee0c6-f450-7f50-bf4f-e48fa2bad5ee`) found six contract
  gaps after `3d1b60a`: diagnostics durability, importer write boundaries, ID
  collision policy, ACP target drift, old control-plane import semantics, and
  concrete worker write ownership.
- The manager resolved the first five by adding durable diagnostics, approved
  import/capture actions, ID-class collision rules, a local ACP v1 conformance
  target, and old control-plane historical import boundaries.
- Worker write ownership remains manager-enforced per spawned worker; no broad
  overlapping plugin/provider/runtime workers should start until each write set
  is assigned explicitly.
- Current contract base after this review is `8123c74`.

## Harness Starter Hooks Review

- Reviewer Linnaeus (`019ee13c-cf32-7380-99d1-da5531f859ae`) reviewed commit
  `725edbf` and found no blockers.
- Sign-off: OK to keep integrated.
- Findings: the docs preserve ACP as the public provider protocol, keep CRP as
  internal/native compatibility, describe the harness starter as optional future
  scaffolding rather than a competing protocol, exclude hosted/team/enterprise
  scope, and avoid trademark/positioning risk.
- Non-blocking docs organization suggestion: move the harness boundary link out
  of the top docs "Start here" list. The manager applied this by placing the
  link under an Examples heading in `docs/README.md`.

## Declarative Plugin Contribution Contract Review

- Reviewer Poincare (`019ee1b3-9618-7ba2-b6d6-47bf1d4f5340`) reviewed worker
  commit `276b773` after earlier blocker fixes and found no remaining blockers.
- The slice keeps the public plugin contract manifest-first: ACP/provider,
  command, collector, observer, and `ui_surfaces` remain the executable/local
  runtime buckets; new Workbench `templates`, `toolbar_actions`,
  `artifact_renderers`, `card_renderers`, `detail_sections`, and
  `review_sections` are declarative host-owned buckets only.
- The contract explicitly does not introduce arbitrary React/webview execution
  or redaction/export processor execution in the manifest. Processor markers
  remain SDK sidecars until provenance, permissions, and lifecycle semantics are
  specified.
- Toolbar `command` targets now mean declared commands in the same plugin,
  while `action` targets remain an approved ctx action enum.
- Residual architecture note: JSON Schema can validate target shape but cannot
  enforce same-manifest command cross-reference. That parity is enforced in the
  SDK and Rust model tests.
