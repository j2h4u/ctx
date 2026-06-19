#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CORE_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

"${SCRIPT_DIR}/cargo-safe.sh" test --manifest-path Cargo.toml --workspace --locked

pnpm -C "${CORE_DIR}/apps/web" typecheck
pnpm -C "${CORE_DIR}/apps/web" lint
pnpm -C "${CORE_DIR}/apps/web" test
pnpm -C "${CORE_DIR}/apps/web" build
