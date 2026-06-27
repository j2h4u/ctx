# Agent Skill Install

The ctx agent skill is named `ctx-agent-history-search`.

Install the CLI first, then install the local skill file:

```bash
curl -fsSL https://ctx.rs/install | sh
ctx skill install
```

`ctx skill install` writes the universal skill file under the configured ctx
data root and prints the next steps for supported agents. It does not silently
write Claude Code, Codex, Cursor, or other agent configuration files.

## Claude Code

After `ctx skill install`, add the ctx plugin marketplace in Claude Code and
install the skill:

```text
/plugin marketplace add ctxrs/ctx
/plugin install ctx-agent-history-search@ctx
```

For local testing from a checkout, replace `ctxrs/ctx` with the checkout path.

## Codex

After `ctx skill install`, add the ctx plugin marketplace:

```bash
codex plugin marketplace add ctxrs/ctx
```

Then open `/plugins` in Codex and install `ctx-agent-history-search`.

## Cursor

This repository includes a Cursor plugin manifest at
`plugins/ctx-agent-history-search/.cursor-plugin/plugin.json` and a root
`.cursor-plugin/marketplace.json` catalog for submission.

If plugin install is available, add `ctx-agent-history-search` from Cursor:

```text
/add-plugin ctx-agent-history-search
```

If that is not available, use the skill file installed by `ctx skill install`
as project or user instructions.

## Any Shell-Capable Agent

Ask the agent to read the `SKILL.md` path printed by `ctx skill install`, or
give it this prompt:

```text
Use ctx to search prior local coding-agent sessions before you answer or edit.

Run:
ctx search "<focused query>" --json

Inspect the best result:
ctx show event <ctx-event-id> --window 3
or:
ctx show session <ctx-session-id> --mode lite

If retrieved history affects your answer, cite the ctx_event_id or
ctx_session_id and include the ctx command you ran.
```
