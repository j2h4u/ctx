# Troubleshooting

## No Sources Found

Run:

```bash
ctx sources --json
```

Confirm the provider keeps history on this machine and pass an explicit path if
needed:

```bash
ctx import --path ~/.codex/sessions
```

## Search Misses Recent Work

Re-run import:

```bash
ctx import --resume
ctx search "the missing phrase"
```

If the raw provider file moved, indexed text may still be searchable, but source
citations should report that the raw path is unavailable.

## JSON Consumer Fails

Run the same command without `--json` to inspect warnings, then run:

```bash
ctx doctor --json
ctx validate
```

Check whether the command marked any nested schema as provisional.

## Store Problems

Find the active root:

```bash
ctx status
```

The default is `~/.ctx`. Check permissions and available disk space. Treat the
database and logs as private local history when collecting diagnostics.
