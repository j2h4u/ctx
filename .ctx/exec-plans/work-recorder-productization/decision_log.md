# Work Recorder Productization Decision Log

Updated: 2026-06-22T17:19:00-05:00

## Decisions

- Use `ctxrs/ctx` branch `work-record` for public Work Recorder productization.
- Preserve the reviewed manager plan in this worktree before implementation.
- Treat `ctxrs/ade` as frozen unless a maintenance-only need is discovered.
- Avoid the dirty canonical `ctx-private` checkout; hosted/private work will use
  a separate manual `ctx-private` worktree after reading private repo
  instructions.

## Pending Decisions

- Exact public crate/module split after current-code mapper output.
- Whether any existing ADE surfaces are quarantined, hidden, or removed in this
  branch.
- Hosted staging environment choice and whether credentials allow deployment
  from this machine.
- Buildkite runner/platform availability and any required queue/pool changes.
