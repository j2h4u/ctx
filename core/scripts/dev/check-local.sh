#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CORE_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REPO_DIR="$(cd "${CORE_DIR}/.." && pwd)"

usage() {
  cat <<EOF
Usage: $(basename "$0") [quick|full]

Modes:
  quick  Schema syntax plus TypeScript hygiene for @ctx/types and web.
  full   Full local-done check. Runs quick checks, Rust tests, and web lint/test/build.

Default: full
EOF
}

run() {
  printf '\n+'
  printf ' %q' "$@"
  printf '\n'
  "$@"
}

check_schema_syntax() {
  local -a schema_files
  mapfile -t schema_files < <(find "${REPO_DIR}/schemas" -type f -name '*.json' | sort)
  if ((${#schema_files[@]} == 0)); then
    echo "error: no schema files found under ${REPO_DIR}/schemas" >&2
    return 1
  fi
  run node "${REPO_DIR}/schemas/validate-json-schemas.mjs" "${schema_files[@]}"
}

check_type_hygiene() {
  run pnpm -C "${CORE_DIR}/packages/ctx-types" typecheck
  run pnpm -C "${CORE_DIR}/apps/web" typecheck
}

check_web_full() {
  run pnpm -C "${CORE_DIR}/apps/web" lint
  run pnpm -C "${CORE_DIR}/apps/web" test
  run pnpm -C "${CORE_DIR}/apps/web" build
}

check_full() {
  check_schema_syntax
  check_type_hygiene
  run "${SCRIPT_DIR}/cargo-safe.sh" test --manifest-path "${CORE_DIR}/Cargo.toml" --workspace --locked
  check_web_full
}

if (($# > 1)); then
  usage >&2
  exit 2
fi

mode="${1:-full}"
case "${mode}" in
  -h|--help|help)
    usage
    exit 0
    ;;
  --quick)
    mode="quick"
    ;;
  --full|local-done)
    mode="full"
    ;;
esac

case "${mode}" in
  quick)
    check_schema_syntax
    check_type_hygiene
    ;;
  full)
    check_full
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac
