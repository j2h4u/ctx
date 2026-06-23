# Release Install

Status: this is the `0.1.0` candidate installer contract for a future public
release.
The URLs below are placeholders until release artifacts are actually published;
do not present them as live installer URLs or as a working `ctx.rs/install`
endpoint.

Today, the truthful public install path for this branch is still a source
build:

```bash
cargo build -p ctx
cargo install --path crates/ctx-cli
```

The finished-product installer flow is metadata driven. Users download an
installer script as a local file, inspect it if needed, and run it against
release metadata that pins every artifact name and SHA-256 checksum.

Do not document or publish a `curl ... | sh` command. The supported shell
pattern is a local launch:

```sh
curl -fsSLO https://github.com/ctxrs/ctx/releases/download/vX.Y.Z/install.sh
curl -fsSLO https://github.com/ctxrs/ctx/releases/download/vX.Y.Z/ctx-release-metadata.env
bash install.sh --metadata ./ctx-release-metadata.env
```

PowerShell follows the same local launch model:

```powershell
Invoke-WebRequest -Uri https://github.com/ctxrs/ctx/releases/download/vX.Y.Z/install.ps1 -OutFile install.ps1
Invoke-WebRequest -Uri https://github.com/ctxrs/ctx/releases/download/vX.Y.Z/ctx-release-metadata.env -OutFile ctx-release-metadata.env
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -Metadata .\ctx-release-metadata.env
```

The version string in public copy is candidate wording only. Keep release
commands as examples until real release assets, checksums, and publication
proof exist.

The installers reject insecure metadata URLs, non-HTTPS artifact URLs,
placeholder checksums, unsupported platforms, and artifact names that contain
path traversal. They download the selected artifact to a temporary directory,
verify SHA-256 before installation, and then copy only the verified `ctx`
binary into the chosen bin directory.

## Metadata

Release metadata uses `release/install/ctx-release-metadata.env.template` as
the schema reference. Release dry-runs generate host-specific metadata files
beside the manifest and checksum artifact for the native release lanes. The
release-candidate metadata lane assembles those platform files into one
installer metadata file for the R2 staging layout in `docs/release-r2-layout.md`.
Real releases must replace every placeholder checksum with the SHA-256 digest
of the final published artifact.

The installer dry-run smoke lane validates this metadata shape with local
fixture metadata and `scripts/install.sh --dry-run`. It does not download,
install, upload, sign, or publish artifacts.

Required keys:

- `CTX_RELEASE_SCHEMA_VERSION=1`
- `CTX_RELEASE_VERSION`
- `CTX_RELEASE_BASE_URL`
- `CTX_RELEASE_ARTIFACT_<platform>`
- `CTX_RELEASE_SHA256_<platform>`

Release-candidate metadata also records the inert audit keys
`CTX_RELEASE_R2_BUCKET`, `CTX_RELEASE_R2_PREFIX`, and
`CTX_RELEASE_R2_OBJECT_<platform>`, plus installer script object keys
`CTX_RELEASE_INSTALLER_SH_R2_OBJECT` and
`CTX_RELEASE_INSTALLER_PS1_R2_OBJECT`. The installers ignore these keys; they
are for release review and upload cleanup.

Supported platform keys are `linux_x64`, `macos_arm64`, `macos_x64`, and
`windows_x64`. `freebsd_x64` stays omitted from candidate install metadata
until native release evidence exists; the candidate metadata records
`CTX_RELEASE_BLOCKER_FREEBSD_X64` instead.

## Public wording rules

- Do not claim `ctx.rs/install` is live until the release URL exists.
- Do not swap the source-build instructions out of public docs before the
  release artifacts are published and verified.
- Do not remove the SHA-256 or HTTPS requirements from installer docs.
- Do not turn the local-launch examples into pipe-to-shell examples.
