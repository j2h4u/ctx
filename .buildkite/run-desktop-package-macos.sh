#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(dirname "${SCRIPT_DIR}")"
CORE_ROOT="${REPO_ROOT}/core"

corepack enable
pnpm -C "${CORE_ROOT}" install --frozen-lockfile --prefer-offline --store-dir "${PNPM_STORE_DIR:-${HOME}/.cache/pnpm-store}"
pnpm -C "${CORE_ROOT}" desktop:build
