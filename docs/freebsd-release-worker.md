# FreeBSD Release Worker Notes

Status: feasibility notes only. No FreeBSD worker was provisioned by this
branch.

The public Buildkite matrix records a `freebsd-x64` blocker because no native
FreeBSD queue is documented in this repo. The required target triple is
`x86_64-unknown-freebsd`.

## Discovery

Run these non-mutating checks from a manager environment that has credentials:

```bash
buildkite-agent --version
hcloud image list | grep -i freebsd
hcloud server-type list
```

If a FreeBSD image is available, choose the smallest instance that can build the
workspace without swapping heavily. Start with a 4 vCPU / 8 GiB class or larger,
then downsize only after timing data exists.

## Candidate Worker

Provisioning must be explicitly approved. Use a unique name and delete it after
the release feasibility run.

```bash
HCLOUD_TOKEN=... hcloud server create \
  --name ctx-freebsd-x64-bk-010-$(date +%Y%m%d%H%M%S) \
  --type cpx31 \
  --image <freebsd-image-from-hcloud-image-list> \
  --location <approved-location> \
  --ssh-key <approved-buildkite-worker-key>
```

Install prerequisites on the host: Bash, Git, Rust stable, Cargo, and either
`sha256sum` or `shasum`. Register a Buildkite agent tagged exactly:

```text
queue=freebsd-x64
os=freebsd
arch=x86_64
ctx-runner-class=release-freebsd-x64-stage
```

The native dry-run command for the worker is:

```bash
CTX_ARTIFACT_DIR=artifacts/buildkite/release-dry-run/freebsd-x64 \
CTX_RELEASE_PLATFORM=freebsd-x64 \
CTX_RELEASE_TARGET_TRIPLE=x86_64-unknown-freebsd \
CTX_EXPECT_HOST_TRIPLE=x86_64-unknown-freebsd \
./scripts/release-dry-run.sh
```

## Cleanup

Delete the temporary server after the feasibility run:

```bash
hcloud server delete <ctx-freebsd-x64-bk-010-name>
```

Record the Buildkite URL, server type, elapsed build time, and monthly cost
estimate in the release manager handoff before enabling a required FreeBSD lane.
