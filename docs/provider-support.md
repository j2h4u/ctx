# Provider Support

This public `0.1.0` candidate preview uses a strict provider taxonomy. Do not
call a provider "supported" unless it has at least supported-import or
supported-wrapper proof with reviewable output and docs that match the actual
implementation.

This branch proves provider integration through explicit, local import commands
and a conservative local discovery command. It does not install passive
provider-native hooks for Codex, Claude, Pi, or long-tail providers. The first
shipped passive capture surface is the local Git/jj/gh wrapper shim path
installed by `ctx setup`.

Authoritative machine-readable metadata lives in
`docs/provider-support-matrix.json`. The shared provider adapter and normalized
envelope contract is described in `docs/provider-adapter-api.md` and backed by
typed `work-record-core` provider structs.

The public release taxonomy uses hyphenated labels. The current
machine-readable metadata serializes equivalent implementation status ids in
snake_case where needed, such as `supported_import` and `fixture_only`.

## Taxonomy

| Public status | Metadata id | Public meaning |
| --- | --- | --- |
| `supported-live` | `supported_live` | Native or wrapper capture, real live proof, review surfaces, and gated evidence are all green. |
| `supported-import` | `supported_import` | Stable existing-history import is proven, but passive live capture is unavailable or intentionally not implemented yet. |
| `supported-wrapper` | `supported_wrapper` | ctx can capture the surface through a wrapper or shim even when native provider hooks are unavailable. |
| `fixture-only` | `fixture_only` | Normalized fixture import works, but no real provider data is proven in the public candidate yet. |
| `detected-unsupported` | `detected_unsupported` | ctx can detect a local install or directory, but there is no safe import or capture path to claim publicly. |
| `blocked` | `blocked` | A concrete blocker exists and needs provider-specific proof before the public docs can upgrade the claim. |

## Current candidate matrix

| Provider | Current status | Implemented path | Source format | Fidelity | Captured | Not captured | Proof |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Codex | `fixture_only` | `ctx capture import-provider --provider codex --input <fixture.jsonl>` | normalized provider fixture JSONL | imported | sessions, events, exposed parent-child session edges, cursors, source metadata | native history discovery, assistant/tool fidelity beyond fixture content | `tests/fixtures/provider/codex.jsonl`; capture and CLI tests |
| Codex | `supported_import` | `ctx capture import-codex-history --input ~/.codex/history.jsonl` or `ctx capture import-local-providers` | Codex prompt history JSONL with `session_id`, `ts`, `text` | summary_only | user prompt events grouped by Codex session id | assistant replies, tool calls, command output, artifacts, child session relationships | `tests/fixtures/provider-history/codex-history.jsonl`; capture and CLI tests; local blocker notes below |
| Claude Code | `fixture_only` | `ctx capture import-provider --provider claude --input <fixture.jsonl>` | normalized provider fixture JSONL | imported | sessions, events, cursors, source metadata present in fixture | native Claude Code history discovery, hooks, live capture | `tests/fixtures/provider/claude.jsonl`; capture and CLI tests |
| Pi | `fixture_only` | `ctx capture import-provider --provider pi --input <fixture.jsonl>` | normalized provider fixture JSONL | imported | sessions, events, source metadata present in fixture, secret-key redaction in metadata | native Pi history discovery, hooks, live capture | `tests/fixtures/provider/pi.jsonl`; capture and CLI tests |

`ctx capture import-local-providers` checks known local locations:

- Codex: `~/.codex/history.jsonl`; imported idempotently as prompt history when present.
- Claude Code: `~/.claude/projects` or `~/.claude`; reported as
  `detected_unsupported` when present because no native Claude Code transcript
  parser or hook is implemented.
- Pi: `~/.pi/agent` or `~/.pi`; reported as `detected_unsupported` when
  present because no native Pi transcript/history parser or hook is implemented.
- P1/P2 provider inventory paths listed below: reported as
  `discovered_unsupported` when one of the documented config/history paths
  exists. These probes check path existence only and do not read provider
  settings, tokens, session stores, logs, or transcripts.

## P1/P2 Classification

The rows below are release inventory and blocker truth, not public support
claims. Their machine-readable rows include primary source links in
`metadata.evidence` and point at the path-existence detector proof in
`crates/ctx-cli/src/main.rs` and `crates/ctx-cli/tests/cli.rs`.

| Provider ID | Provider | Priority | Status | Detection paths | Evidence basis | Current blocker / next action |
| --- | --- | --- | --- | --- | --- | --- |
| `copilot_cli` | Copilot CLI | P1 | `detected_unsupported` | `~/.copilot`, `~/.config/gh/extensions/gh-copilot` | [GitHub config docs](https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-config-dir-reference), [hooks](https://docs.github.com/en/copilot/reference/hooks-reference), [session data](https://docs.github.com/en/copilot/how-tos/copilot-cli/use-copilot-cli/chronicle) | Build a read-only `~/.copilot` parser or ctx hook adapter before import/capture claims. |
| `factory_droid` | Factory / Droid | P1 | `detected_unsupported` | `~/.factory`, `./.factory` | [CLI reference](https://docs.factory.ai/reference/cli-reference), [settings](https://docs.factory.ai/cli/configuration/settings), [hooks](https://docs.factory.ai/reference/hooks-reference), [droid exec](https://docs.factory.ai/cli/droid-exec/overview) | Prefer hooks or JSON-RPC over scraping private session files. |
| `goose` | Goose | P1 | `detected_unsupported` | `~/.config/goose/config.yaml`, `~/.local/share/goose/sessions/sessions.db`, legacy sessions directory | [Goose logs](https://goose-docs.ai/docs/guides/logs/), [config files](https://goose-docs.ai/docs/guides/config-files/) | Add redacted SQLite/legacy JSONL fixtures and parser before import. |
| `openhands` | OpenHands | P1 | `detected_unsupported` | `~/.openhands/conversations`, settings files | [CLI install](https://docs.openhands.dev/openhands/usage/cli/installation), [CLI commands](https://docs.openhands.dev/openhands/usage/cli/command-reference), [conversation architecture](https://docs.openhands.dev/sdk/arch/conversation) | Normalize ConversationState/EventLog fixtures before import. |
| `amp` | Amp | P1 | `detected_unsupported` | `~/.config/amp/settings.json[c]`, `./.amp/settings.json[c]` | [Manual](https://ampcode.com/manual), [security](https://ampcode.com/security), [SDK](https://ampcode.com/manual/sdk) | Use SDK streaming or wrapper capture; no stable local thread importer is implemented. |
| `cagent` | Docker cagent | P1 | `detected_unsupported` | `~/.cagent`, `~/.cagent/cagent.debug.log` | [Docker Agent CLI reference](https://docs.docker.com/ai/docker-agent/reference/cli/) | Debug logs are not a transcript contract; prefer OpenTelemetry or structured output. |
| `qwen` | Qwen Code | P1 | `detected_unsupported` | `~/.qwen/settings.json`, `~/.qwen/tmp`, `~/.qwen`, `./.qwen` | [Qwen settings](https://qwenlm.github.io/qwen-code-docs/en/users/configuration/settings/), [repository](https://github.com/QwenLM/qwen-code) | Documented `shell_history` is not a full transcript; confirm stable session store. |
| `mistral` | Mistral Vibe | P1 | `detected_unsupported` | `~/.vibe/config.toml`, `~/.vibe`, `./.vibe` | [Vibe install/setup](https://docs.mistral.ai/vibe/code/cli/install-setup), [repository](https://github.com/mistralai/mistral-vibe) | Need stable transcript source or ACP adapter. |
| `kimi` | Kimi Code | P1 | `detected_unsupported` | `~/.kimi-code`, `~/.kimi-code/config.toml` | [Kimi CLI support](https://platform.kimi.ai/docs/guide/kimi-cli-support), [repository](https://github.com/MoonshotAI/kimi-code) | Build a redacted session-record fixture; no parser/hook/ACP adapter exists. |
| `aider` | Aider | P2 | `detected_unsupported` | `./.aider.chat.history.md`, `./.aider.input.history`, `./.aider.conf.yml`, `~/.aider.conf.yml` | [Options](https://aider.chat/docs/config/options.html), [FAQ](https://aider.chat/docs/faq.html) | Add explicit markdown chat-history fixtures and redaction before import. |
| `cline_roo` | Cline / Roo | P2 | `detected_unsupported` | common VS Code globalStorage IDs for Cline and Roo | [Cline task management](https://docs.cline.bot/core-workflows/task-management), [Roo chat interface](https://roocodeinc.github.io/Roo-Code/basic-usage/the-chat-interface/) | Collect redacted task-directory fixtures across extension versions. |
| `continue_cody` | Continue / Cody | P2 | `detected_unsupported` | `~/.continue/config.yaml`, `~/.continue/logs`, Continue/Cody VS Code globalStorage IDs | [Continue config](https://docs.continue.dev/customize/deep-dives/configuration), [troubleshooting logs](https://docs.continue.dev/troubleshooting), [Cody export note](https://sourcegraph.com/blog/cody-vscode-0-10-release) | Use explicit exports/fixtures rather than scraping extension internals. |
| `auggie` | Auggie | P2 | `detected_unsupported` | `~/.augment`, `~/.augment/settings.json`, `./.augment` | [Auggie CLI reference](https://docs.augmentcode.com/cli/reference) | Prototype structured print-mode, session-list export, or ACP capture. |
| `junie` | Junie | P2 | `detected_unsupported` | `~/.junie`, `~/.junie/allowlist.json` | [Junie CLI quickstart](https://junie.jetbrains.com/docs/junie-cli.html) | Need documented local session schema or export path. |
| `kilo` | Kilo Code | P2 | `detected_unsupported` | common VS Code globalStorage IDs for Kilo | [Kilo auto cleanup](https://kilo.ai/docs/getting-started/settings/auto-cleanup) | Collect redacted task-directory fixtures; cleanup can delete history. |
| `swe_agent` | SWE-agent | P2 | `detected_unsupported` | `./trajectories`, `~/.swe-agent` | [SWE-agent trajectories](https://swe-agent.com/latest/usage/trajectories/) | Add `.traj` fixture parser with redaction and version handling. |

The current matrix also keeps `copilot` and `droid_factory_ai` as separate
blocked historical-surface rows. The source branch classified their current CLI
surfaces as `copilot_cli` and `factory_droid`; release management should decide
whether the historical rows become aliases or remain separate contracts.

The command is intentionally conservative. It imports only the sources this
branch can prove safely and refuses to upgrade unsupported providers into
invented capture claims.

## Broader provider work not yet claimed here

The metadata file also carries blocked or classification-pending rows for
Cursor, OpenCode, Gemini CLI, Antigravity CLI, Copilot CLI, Factory/Droid,
Goose, Amp, OpenHands, cagent, Qwen, Mistral, Kimi, Aider, Cline/Roo,
Continue/Cody, Auggie, Junie, Kilo, SWE-agent, and legacy ADE surfaces. Do not
treat those providers as publicly supported based on this branch alone. Until
real import or capture proof lands with matching docs, keep those entries at
`blocked`, `fixture-only`, or `detected-unsupported`.

## Source and Fidelity Metadata

Imported provider rows record `source_format`, source trust, raw-retention
mode, redaction boundary, cursor checkpoints, and explicit idempotency fields
in sync metadata. Normalized fixture imports use
`normalized_provider_fixture_jsonl` and `fidelity=imported`. Codex
prompt-history imports use `codex_history_jsonl` and `fidelity=summary_only`.
Detected-unsupported inventory rows use `path_existence_only` detection and do
not claim prompt, message, tool, file, artifact, cost, token, or child-session
fidelity.

The Codex history path intentionally does not create parent/child edges. The
local history format observed for this branch exposes prompt log rows only, not
subagent relationships.

## Shared Provider Contract

Provider workers should normalize native inputs into
`work_record_core::ProviderCaptureEnvelope` values and then persist them through
`work_record_capture::import_normalized_provider_captures`. That common path
owns:

- session/event/source normalization;
- provider event idempotency via provider/session/index/hash dedupe keys;
- cursor checkpoint persistence in `sync_cursors`;
- redaction sanitization before provider payload or metadata is stored;
- shared session-edge creation for parent/child relationships.

`work_record_capture::ProviderFixtureJsonlAdapter` and
`work_record_capture::CodexHistoryJsonlAdapter` are the reference adapters in
this branch.

## Local E2E Evidence and Blockers

Local provider inventory on 2026-06-23:

- Codex: `/home/daddy/.codex/history.jsonl` exists and contains prompt-history
  JSONL rows with `session_id`, `ts`, and `text`. This validates the gated
  parser shape, but the real file is private local data and was not committed.
- Claude: `/home/daddy/.claude` was not present, so native Claude history E2E
  could not run.
- Pi: `/home/daddy/.pi/agent/auth.json` was present, but no local Pi transcript
  or history file was found under `/home/daddy/.pi` within four directory
  levels, so native Pi history E2E could not run.

Gated real-data check for Codex:

```bash
CTX_DATA_ROOT="$(mktemp -d)" \
  ctx capture import-codex-history --input "$HOME/.codex/history.jsonl" --json
```

Run this only on a machine where importing local prompt history into a temporary
ctx data root is acceptable. The output should report `source_format:
codex_history_jsonl`, `fidelity: summary_only`, and limitations explaining that
assistant replies, tools, command output, and child sessions are not present.

Gated real-data checks for Claude and Pi remain blocked until their native
history locations and schemas are available. Until then, only the normalized
fixture adapters are proven.
