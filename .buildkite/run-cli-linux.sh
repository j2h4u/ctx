#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(dirname "${SCRIPT_DIR}")"
CORE_ROOT="${REPO_ROOT}/core"

cd "${CORE_ROOT}"

scripts/dev/cargo-safe.sh test --manifest-path Cargo.toml -p ctx-http --bin ctx agent_work_cli::tests --locked
scripts/dev/cargo-safe.sh build --manifest-path Cargo.toml -p ctx-http --bin ctx --locked

test -x "${CORE_ROOT}/target/debug/ctx"
