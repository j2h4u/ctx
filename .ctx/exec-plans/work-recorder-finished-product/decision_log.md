# Work Recorder Finished Product Decision Log

## 2026-06-23: Public Launch Scope

Decision: launch scope for the public `work-record` branch is local-first/local-only.

Rationale:

- The product definition and earlier strategy emphasize local capture, local search/context, local dashboard/reporting, and explicit redacted sharing.
- Hosted/team sync is valuable but should not be half-promised in public CLI/docs.
- The private hosted worktree can continue to hold staging and contract work without blocking the local product from becoming coherent and releasable.

Implications:

- Public CLI must not expose dead hosted commands.
- Public docs should describe hosted/team sync as future direction only.
- Completion criteria for hosted/team use Option A from the plan unless a later explicit decision moves to Option B.
