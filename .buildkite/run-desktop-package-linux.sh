#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(dirname "${SCRIPT_DIR}")"
CORE_ROOT="${REPO_ROOT}/core"

unset APPDIR
unset APPIMAGE
unset GIO_EXTRA_MODULES
unset GST_PLUGIN_PATH
unset GTK_PATH
unset LD_LIBRARY_PATH
unset XDG_DATA_DIRS

corepack enable
pnpm -C "${CORE_ROOT}" install --frozen-lockfile --prefer-offline --store-dir "${PNPM_STORE_DIR:-${HOME}/.cache/pnpm-store}"
pnpm -C "${CORE_ROOT}" desktop:build
