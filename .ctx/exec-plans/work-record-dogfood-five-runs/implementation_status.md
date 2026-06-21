# Work Record Dogfood: Five Scratch Runs

Updated: 2026-06-21T15:50:54Z

Branch: `ctx/agent-work-semantics-primary`

Head used for tooling: `4b7e72c27b8ac58ed1a9924820d582b3abcad68d`

ctx binary used:
`/home/daddy/code/ctx-multi-repo-workspace/worktrees/ctx/agent-work-semantics-primary/bazel-bin/core/crates/ctx-http/ctx`

Scratch root:
`/home/daddy/code/ctx-multi-repo-workspace/scratch/work-record-dogfood-five-runs-20260621-1540`

Chrome open log:
`/home/daddy/code/ctx-multi-repo-workspace/scratch/work-record-dogfood-five-runs-20260621-1540/chrome-open-work-reports.log`

Chrome status: all five Work Report HTML file URLs were passed to
`/usr/bin/google-chrome --new-window`; Chrome exited `0` after opening them in
the existing browser session.

No scratch repo was pushed. No ctx product code was changed. No merge, release,
or announcement was performed.

## Runs

| # | Task | Work ID | Commit | Report opened |
| --- | --- | --- | --- | --- |
| 1 | Canvas game | `wrk_94013bd5d7644c378a90872f9529d9b7` | `fce183bc0d12646c220470e059434a855ae690a1` | yes |
| 2 | Static productivity app | `wrk_0ac64205948e490486e91237d078c3ce` | `676d156123bea7a7b4bbd3a96131fe68047079e0` | yes |
| 3 | CLI utility | `wrk_0039952db3db4e4e97a3f0b4928cb3a2` | `da594081db15355b37f749671f07d1def520261d` | yes |
| 4 | Docs/content site | `wrk_b2180fd3c05d4ec391ca0ad96b9c4eba` | `7b7a4cad3bf9aee7f259b50e22365cf9b55b571c` | yes |
| 5 | Local API/data visualization | `wrk_106b376ecff4403a989f9b42f19a2a87` | `54c39f4da24b867571c3588ce616d1e02286502e` | yes |

## Task Details

### 1. Canvas Game

Objective: build `Orbit Dodge`, a dependency-free browser canvas game with
logic tests and a screenshot.

Project path:
`/home/daddy/code/ctx-multi-repo-workspace/scratch/work-record-dogfood-five-runs-20260621-1540/01-canvas-game/orbit-dodge`

ctx data path:
`/home/daddy/code/ctx-multi-repo-workspace/scratch/work-record-dogfood-five-runs-20260621-1540/01-canvas-game/ctx-data`

Report:
`file:///home/daddy/code/ctx-multi-repo-workspace/scratch/work-record-dogfood-five-runs-20260621-1540/01-canvas-game/orbit-dodge/artifacts/generated/work-report.html`

Evidence:

- `node test.mjs` passed.
- `node --check game.js` passed.
- `artifacts/generated/app-screenshot.png` is a valid `900 x 900` PNG.
- `work-context.json`, `work-report.json`, `work-timeline.txt`, and
  `work-evidence.txt` were generated.

Notes: the worker fixed an initially too-strict frame-delta test before final
validation.

### 2. Static Productivity App

Objective: build `Focus Stack`, a dependency-free static focus planner with
tests and a screenshot.

Project path:
`/home/daddy/code/ctx-multi-repo-workspace/scratch/work-record-dogfood-five-runs-20260621-1540/02-productivity-app/project`

ctx data path:
`/home/daddy/code/ctx-multi-repo-workspace/scratch/work-record-dogfood-five-runs-20260621-1540/02-productivity-app/ctx-data`

Report:
`file:///home/daddy/code/ctx-multi-repo-workspace/scratch/work-record-dogfood-five-runs-20260621-1540/02-productivity-app/project/artifacts/generated/work-report.html`

Evidence:

- `node test.mjs` passed.
- `node --check app.js` passed.
- `artifacts/generated/app-screenshot.png` is a valid `1365 x 900` PNG.
- Objective, freshness, context, report, timeline, and evidence artifacts were
  generated.

Notes: the app is intentionally small and static; timer state does not survive
reloads.

### 3. CLI Utility

Objective: build a dependency-free JSONL stats CLI with tests and sample smoke
output.

Project path:
`/home/daddy/code/ctx-multi-repo-workspace/scratch/work-record-dogfood-five-runs-20260621-1540/03-cli-utility/jsonl-stats`

ctx data path:
`/home/daddy/code/ctx-multi-repo-workspace/scratch/work-record-dogfood-five-runs-20260621-1540/03-cli-utility/ctx-data`

Report:
`file:///home/daddy/code/ctx-multi-repo-workspace/scratch/work-record-dogfood-five-runs-20260621-1540/03-cli-utility/jsonl-stats/artifacts/generated/work-report.html`

Evidence:

- `env -u PYTHONHOME -u PYTHONPATH /usr/bin/python3 -m unittest discover -s tests`
  passed three tests.
- `env -u PYTHONHOME -u PYTHONPATH /usr/bin/python3 -m py_compile jsonl_stats.py tests/test_jsonl_stats.py`
  passed.
- `artifacts/generated/cli-smoke.txt` was captured as a log artifact.
- Context, report, timeline, evidence, and freshness artifacts were generated.

Product/environment gap: inherited `PYTHONHOME`/`PYTHONPATH` pointed at a ctx
AppImage temp mount and broke `/usr/bin/python3`; the worker had to unset those
variables for Python validation.

### 4. Docs/Content Site

Objective: build a dependency-free JSON/Markdown-generated docs site with build
and content tests.

Project path:
`/home/daddy/code/ctx-multi-repo-workspace/scratch/work-record-dogfood-five-runs-20260621-1540/04-docs-site/project`

ctx data path:
`/home/daddy/code/ctx-multi-repo-workspace/scratch/work-record-dogfood-five-runs-20260621-1540/04-docs-site/ctx-data`

Report:
`file:///home/daddy/code/ctx-multi-repo-workspace/scratch/work-record-dogfood-five-runs-20260621-1540/04-docs-site/artifacts/generated/work-report.html`

Evidence:

- `node build.mjs` passed.
- `node test.mjs` passed.
- `node --check build.mjs` passed during direct validation.
- `artifacts/generated/site-screenshot.png` is a valid `1440 x 1100` PNG.
- Context, report, timeline, evidence, and freshness artifacts were generated.

Product gap: `ctx work show` did not resolve the `wrk_...` Work ID even though
`summarize`, `evidence`, `freshness`, and report generation worked. The worker
generated a custom report wrapper around the working ctx outputs. The resulting
report is understandable but marginal compared with the other four.

### 5. Local API/Data Visualization

Objective: build `Energy Pulse`, a dependency-free Node local API and browser
data visualization.

Project path:
`/home/daddy/code/ctx-multi-repo-workspace/scratch/work-record-dogfood-five-runs-20260621-1540/05-local-api-viz/energy-pulse`

ctx data path:
`/home/daddy/code/ctx-multi-repo-workspace/scratch/work-record-dogfood-five-runs-20260621-1540/05-local-api-viz/ctx-data`

Report:
`file:///home/daddy/code/ctx-multi-repo-workspace/scratch/work-record-dogfood-five-runs-20260621-1540/05-local-api-viz/energy-pulse/artifacts/generated/work-report.html`

Evidence:

- `node --test test.mjs` passed.
- `node --check server.mjs` passed.
- `GET /api/summary` API smoke output was saved as `api-summary.json`.
- `artifacts/generated/app-screenshot.png` is a valid `1440 x 1000` PNG.
- Context, report, timeline, evidence, and freshness artifacts were generated.

Notes: the worker corrected an initially wrong renewable-share assertion and
reran validation successfully. No local server process was left running.

## Reviewer Result

Fresh read-only reviewer verdicts:

- Canvas game: PASS.
- Productivity app: PASS.
- CLI utility: PASS.
- Docs site: PASS, marginal.
- Local API/data visualization: PASS.

Overall reviewer conclusion: FAIL for Work report product readiness, because a
fresh agent can reconstruct each run only by using the report plus adjacent
evidence/log artifacts and the scratch git repos. The reports are not yet
self-contained enough for the intended product experience.

## Work Report Product Gaps

- Reports do not list changed files or commit stats even though clean commits
  are available.
- Most reports omit the commit SHA in the Markdown/HTML body.
- Artifact references are text, not links, and screenshots are not rendered as
  thumbnails.
- `work-context.json` is too thin for agent handoff: repo, branch, commit,
  changed files, commands, and evidence counts are mostly absent.
- `work-report.json` shape is inconsistent; the docs-site run had a minimal
  summary/claims shape while the other runs exposed richer evidence.
- Trust remains `partial` with a generic next action and does not explain which
  evidence is weak or how to upgrade the record.
- Evidence is uneven: some runs contain duplicate manual and fresh command
  evidence, while others rely more heavily on partial/manual artifacts.
- `ctx work show` and first-class `wrk_...` observability commands are not fully
  aligned, as shown by the docs-site run.

## Final State

- Five disposable scratch projects were created outside the ctx repo.
- Five Work records were captured with real commits and validation evidence.
- Five Work Report HTML pages were generated and opened in Chrome.
- One read-only reviewer inspected the reports for fresh-agent legibility.
- The ctx product code was not changed.
- This status note is the only intended ctx worktree change from this dogfood
  pass.
