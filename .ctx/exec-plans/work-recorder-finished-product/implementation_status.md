# Work Recorder Finished Product Status

## Current Phase

- Program started: 2026-06-23
- Public repo: `ctxrs/ctx`
- Public worktree: `/home/daddy/code/ctx-multi-repo-workspace/worktrees/ctx/work-record-product`
- Public branch: `work-record`
- Starting head: `83cf0639d659aa35d557a530fe2ca49476af950e`
- Plan checkpoint head: `cb274a2d17bc000016b7e86b4cfe6f748d594a58`
- First-wave integration head: `f53b5a8`
- Private hosted worktree, when needed: `/home/daddy/code/ctx-multi-repo-workspace/worktrees/ctx-private/work-recorder-hosted-team`

## Launch Scope Decision

The public Work Recorder release track is **local-first and local-only** until a complete hosted staging path is proven. Public CLI/docs must not imply that hosted/team sync is part of this launch. Hosted/team work remains private and future-facing unless explicitly promoted by a later launch decision.

## Active Workstreams

- Storage and schema foundation: first implementation merged. Typed Store APIs now cover capture sources, sessions, session edges, runs, events, artifacts, VCS workspaces/changes, pull requests, summaries, files touched, work record links, and sync cursors. Search/dashboard/provider consumers still need to adopt those APIs.
- Provider import and passive capture: discovery complete. Codex has local JSONL sessions available for gated E2E; Pi binary exists but no local sessions; Claude is fixture-only on this host. No provider adapters or provider import commands exist yet.
- Shims, hooks, Git, gh, and jj: VCS metadata foundation merged. Git status/upstream/recent commits and jj parser structs/tests exist. CLI/shim integration, Windows shims, shell hooks, streaming behavior, gh/PR capture, root uninstall integration, and real jj E2E remain.
- Search, context, and agent access: discovery complete. Search/context packets are usable but mostly cover records plus evidence; rich sessions/events/files/VCS/PRs/summaries are not searched yet.
- Dashboard and reports: discovery complete. Current dashboard/report are MVP summaries; finished work needs a rich review DTO, deterministic fixture, screenshots, timeline/transcript/tool/VCS/artifact sections, and PR evidence Markdown/JSON.
- PR publishing: library foundation merged. Deterministic marker-bounded PR Markdown, redacted-by-default raw transcript opt-in API, mockable GitHub upsert planning, and GitLab defer behavior exist. CLI wiring and real HTTP client remain.
- Installer, release, and Buildkite: installer/certificate scaffold merged. Shell and PowerShell installer scripts, release metadata template, release supply-chain docs, and completion certificate script exist. Real public artifact publication, SBOM/provenance/signing/notarization, and installer smoke remain.
- Security, privacy, docs, and site: docs foundation merged. Public `SECURITY.md`, threat model, redaction corpus docs/fixture, dependency/license audit docs, and local-only hosted positioning exist. Privacy doctor and code-backed security tests remain.
- Hosted/team contract audit: discovery complete. Public launch uses Option A local-only. Private hosted skeleton exists but is not production-ready and must not be claimed in public docs.

## Validation Log

- Initial branch status was clean at `83cf0639d659aa35d557a530fe2ca49476af950e`.
- `cargo check -p work-record-core --locked` passed after the plan checkpoint.
- `TMPDIR=$PWD/target/tmp cargo-lowio test -p work-record-core -p work-record-store -p work-record-publish -p work-record-vcs --lib --locked` passed at first-wave integration head `f53b5a8`.
- `./scripts/check-docs.sh` and `git diff --check` passed while resolving the release/security docs merge conflict.

## Review Status

- Initial scout reviews completed for storage/schema, provider capture, VCS/shims/jj, search/agent access, dashboard/report, PR publish, release/CI, and security/docs/hosted.
- Final adversarial done-review is required before this program can be called complete.

## Blockers

- None recorded yet.

## Immediate Implementation Order

1. Provider fixture importers and import commands using typed Store APIs.
2. Search/context over rich sessions/events/files/VCS/PRs/summaries using typed Store APIs.
3. Dashboard/report v2 with rich fixture data and screenshot harness.
4. CLI wiring for PR publish, VCS metadata output, and provider import/status.
5. Privacy doctor and code-backed security fixtures.
6. Release/installer/Buildkite completion lanes after product surface stabilizes.
