# Agent Usage

Agents should query ctx before repeating investigation work.

## Recommended Flow

1. Run `ctx status --json` to confirm the local store is readable.
2. Run `ctx sources --json` when source freshness matters.
3. Search narrowly with provider, repository, file, or date filters.
4. Use `ctx context` for the best matching query before changing code.
5. Cite ctx material in notes or final answers when it influenced the work.

Example:

```bash
ctx search "sqlite migration failed" --repo ctx --json
ctx context "sqlite migration failed" --repo ctx --max-tokens 6000
```

## Deterministic Use

Treat ctx output as retrieved source material. Do not state that ctx inferred a
decision unless the cited text explicitly says so. If you synthesize a conclusion
from multiple retrieved snippets, say that the conclusion is your synthesis and
cite the snippets that support it.

## When To Re-Import

Run `ctx import` when:

- `ctx sources` shows provider history newer than the last cursor;
- a search misses something you know happened recently;
- the current task depends on a previous session from another provider;
- a raw provider path was configured after setup.

## JSON For Harnesses

Agent harnesses should prefer JSON for routing and ranking:

```bash
ctx status --json
ctx sources --json
ctx search "release blocker" --json
ctx context "release blocker" --json
```

Use human Markdown context when the next step is to paste retrieved material
into an agent prompt.
