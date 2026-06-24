# Threat Model

The current CLI protects a local search index for developer agent history.

## Assets

- provider transcripts in provider-owned homes;
- the ctx SQLite index;
- configuration and import cursors;
- logs and diagnostic output;
- JSON and Markdown command output.

## Boundaries

ctx reads provider history and writes only to the ctx data root during normal
setup, import, search, and context commands. Source repositories and provider
homes remain outside ctx ownership.

## Risks

- indexed prompts or output may contain secrets;
- local paths and repository names may reveal private work;
- copied JSON output may leave the machine;
- stale citations may point to moved or deleted raw files;
- unsupported provider formats may be parsed incorrectly if adapters are too
  permissive.

## Mitigations

- keep imports explicit and resumable;
- reject unknown provider formats;
- store bounded previews for large outputs;
- preserve citations and source availability warnings;
- keep setup local and side-effect-limited;
- document that searchable text is copied into SQLite.
