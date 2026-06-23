# Work Recorder Finished Product Validation Log

Validation commands and Buildkite URLs for this phase will be recorded here with head SHAs and timestamps.

## 2026-06-23 First-Wave Integration

Head: `f53b5a8`

Commands:

- `cargo check -p work-record-core --locked`: passed.
- `TMPDIR=$PWD/target/tmp cargo-lowio test -p work-record-core -p work-record-store -p work-record-publish -p work-record-vcs --lib --locked`: passed.
- `./scripts/check-docs.sh`: passed while resolving release/security docs merge.
- `git diff --check`: passed while resolving release/security docs merge.

Notes:

- Worker full crate tests that invoked doctest/rustdoc sometimes hit local `Disk quota exceeded`; integrated validation used targeted `--lib` tests through `cargo-lowio` and `TMPDIR=$PWD/target/tmp`.
