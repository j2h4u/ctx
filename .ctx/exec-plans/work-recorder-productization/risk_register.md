# Work Recorder Productization Risk Register

Updated: 2026-06-22T17:19:00-05:00

| Risk | Impact | Current Mitigation |
| --- | --- | --- |
| Scope is large enough to span public local product, private hosted staging, CI/release, and dogfood. | High schedule and integration risk. | Milestone gates and status files will track concrete blockers instead of vague deferrals. |
| Private repo canonical checkout is dirty with unrelated work. | Risk of overwriting unrelated user/agent changes. | Use a separate manual `ctx-private` worktree before edits. |
| Broad Rust/Bazel/build verification can overload this host. | Machine instability and false failures. | Use existing resource-safe wrappers and avoid overlapping heavy jobs. |
| Dashboard can pass tests but remain visually sparse. | Product-quality failure. | Require screenshot generation, manual inspection, and adversarial UI review. |
| Hosted staging credentials or runner access may be unavailable. | External blocker for completion criteria. | Record exact attempted command, missing credential/runner, and remediation; keep unblocked tracks moving. |

## Accepted Risks

None accepted yet.
