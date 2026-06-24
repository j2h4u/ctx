#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repo_root}"

cargo_bin="${CARGO:-cargo}"
cargo_locked_args=()
if [[ "${CTX_CARGO_LOCKED:-1}" != "0" && -f Cargo.lock ]]; then
  cargo_locked_args+=(--locked)
fi

"${cargo_bin}" fmt --all -- --check
"${cargo_bin}" check --workspace --all-targets "${cargo_locked_args[@]}"
"${cargo_bin}" test --workspace --all-targets "${cargo_locked_args[@]}"
bash scripts/check-docs.sh
CTX_AUDIT_SKIP_RELEASE_BUILD="${CTX_AUDIT_SKIP_RELEASE_BUILD:-0}" bash scripts/audit-search-mvp-package.sh
git diff --check

"${cargo_bin}" build -p ctx --bin ctx "${cargo_locked_args[@]}"
ctx_bin="target/debug/ctx"
case "$(uname -s 2>/dev/null || true)" in
  MINGW*|MSYS*|CYGWIN*) ctx_bin="target/debug/ctx.exe" ;;
esac

data_root="$(mktemp -d "${TMPDIR:-/tmp}/ctx-search-mvp-flow.XXXXXX")"
fixture="tests/fixtures/provider-history/codex-sessions"

CTX_DATA_ROOT="${data_root}" "${ctx_bin}" setup
CTX_DATA_ROOT="${data_root}" "${ctx_bin}" sources --json
CTX_DATA_ROOT="${data_root}" "${ctx_bin}" import --provider codex --path "${fixture}" --json
list_json="$(CTX_DATA_ROOT="${data_root}" "${ctx_bin}" list --json)"
record_id="$(printf '%s\n' "${list_json}" | sed -n 's/.*"id": "\([^"]*\)".*/\1/p' | head -n1)"
test -n "${record_id}"
CTX_DATA_ROOT="${data_root}" "${ctx_bin}" search onboarding --json
CTX_DATA_ROOT="${data_root}" "${ctx_bin}" show "${record_id}" --json
CTX_DATA_ROOT="${data_root}" "${ctx_bin}" context onboarding --json
CTX_DATA_ROOT="${data_root}" "${ctx_bin}" status --json
CTX_DATA_ROOT="${data_root}" "${ctx_bin}" doctor --json
CTX_DATA_ROOT="${data_root}" "${ctx_bin}" validate --json

printf 'search MVP checks ok\n'
