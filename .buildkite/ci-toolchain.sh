#!/usr/bin/env bash

ctx_ci_require_command() {
  local command_name="$1"
  local hint="${2:-}"
  if ! command -v "${command_name}" >/dev/null 2>&1; then
    if [[ -n "${hint}" ]]; then
      echo "error: required command '${command_name}' is missing from PATH. ${hint}" >&2
    else
      echo "error: required command '${command_name}' is missing from PATH" >&2
    fi
    exit 127
  fi
}

ctx_ci_sha256_file() {
  local path="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${path}" | awk '{print $1}'
  else
    shasum -a 256 "${path}" | awk '{print $1}'
  fi
}

ctx_ci_expected_node_sha256() {
  case "$1" in
    node-v20.19.5-darwin-arm64.tar.gz)
      printf '%s\n' 'cfed7503d8d99fbcf2f52e408ec52f616058eb0867b34dbc3437259993ef5cba'
      ;;
    node-v20.19.5-darwin-x64.tar.gz)
      printf '%s\n' 'f9cff058f2766d4d0631dc69b5f7f27664b3a42ff186e25ac7e1ac269af7e696'
      ;;
    node-v20.19.5-linux-arm64.tar.xz)
      printf '%s\n' 'd462267863ae8ee556039ebdf559055a8ec562c633889ef1403f3adb449ba1dd'
      ;;
    node-v20.19.5-linux-x64.tar.xz)
      printf '%s\n' '315046739a513a70e03a4a55a8afda8cf979f30852e576075c340084e3f8ac0f'
      ;;
    *)
      return 1
      ;;
  esac
}

ctx_ci_bootstrap_node() {
  local node_version="${CTX_BUILDKITE_NODE_VERSION:-v20.19.5}"
  local platform=""
  case "$(uname -s)" in
    Linux)
      platform="linux"
      ;;
    Darwin)
      platform="darwin"
      ;;
    *)
      echo "error: unsupported OS for Node bootstrap: $(uname -s)" >&2
      exit 1
      ;;
  esac

  local machine=""
  machine="$(uname -m)"
  local node_arch=""
  case "${machine}" in
    x86_64|amd64)
      node_arch="x64"
      ;;
    arm64|aarch64)
      node_arch="arm64"
      ;;
    *)
      echo "error: unsupported machine architecture for Node bootstrap: ${machine}" >&2
      exit 1
      ;;
  esac

  local install_root="${HOME}/.local/node"
  local target_dir="${install_root}/${node_version}-${platform}-${node_arch}"
  local tarball=""
  local tar_extract_args=()
  case "${platform}" in
    linux)
      tarball="node-${node_version}-linux-${node_arch}.tar.xz"
      tar_extract_args=(-xJf)
      ;;
    darwin)
      tarball="node-${node_version}-darwin-${node_arch}.tar.gz"
      tar_extract_args=(-xzf)
      ;;
  esac

  if [[ ! -x "${target_dir}/bin/node" || ! -x "${target_dir}/bin/corepack" ]]; then
    ctx_ci_require_command curl "Node bootstrap requires curl."
    ctx_ci_require_command tar "Node bootstrap requires tar."
    local expected_sha256=""
    if ! expected_sha256="$(ctx_ci_expected_node_sha256 "${tarball}")"; then
      echo "error: no pinned SHA256 for Node bootstrap tarball ${tarball}; add the checksum before changing CTX_BUILDKITE_NODE_VERSION" >&2
      exit 2
    fi
    mkdir -p "${install_root}"
    local tmpdir=""
    tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/ctx-buildkite-node.XXXXXX")"
    local download_url="https://nodejs.org/dist/${node_version}/${tarball}"
    curl -fsSL "${download_url}" -o "${tmpdir}/${tarball}"
    local actual_sha256=""
    actual_sha256="$(ctx_ci_sha256_file "${tmpdir}/${tarball}")"
    if [[ "${actual_sha256}" != "${expected_sha256}" ]]; then
      echo "error: Node bootstrap checksum mismatch for ${tarball}" >&2
      echo "error: expected ${expected_sha256}, got ${actual_sha256}" >&2
      exit 1
    fi
    mkdir -p "${target_dir}"
    tar "${tar_extract_args[@]}" "${tmpdir}/${tarball}" -C "${tmpdir}"
    cp -R "${tmpdir}/node-${node_version}-${platform}-${node_arch}/." "${target_dir}/"
    rm -rf "${tmpdir}"
  fi

  export PATH="${target_dir}/bin:${PATH}"
}

CTX_CI_PNPM_CMD=()

ctx_ci_resolve_pnpm() {
  if (( ${#CTX_CI_PNPM_CMD[@]} > 0 )); then
    return 0
  fi

  if command -v pnpm >/dev/null 2>&1; then
    CTX_CI_PNPM_CMD=(pnpm)
    return 0
  fi

  if ! command -v node >/dev/null 2>&1 || ! command -v corepack >/dev/null 2>&1; then
    ctx_ci_bootstrap_node
  fi
  ctx_ci_require_command node "Node bootstrap failed; verify network access to nodejs.org."
  ctx_ci_require_command corepack "Use a Node.js distribution that includes Corepack."

  export COREPACK_DEFAULT_TO_LATEST="${COREPACK_DEFAULT_TO_LATEST:-0}"
  export COREPACK_ENABLE_DOWNLOAD_PROMPT="${COREPACK_ENABLE_DOWNLOAD_PROMPT:-0}"
  export COREPACK_HOME="${COREPACK_HOME:-${HOME}/.cache/corepack}"
  mkdir -p "${COREPACK_HOME}"

  corepack prepare "${CTX_CI_PNPM_PACKAGE_MANAGER:-pnpm@9.15.1}" --activate >/dev/null
  CTX_CI_PNPM_CMD=(corepack pnpm)
}

ctx_ci_pnpm() {
  ctx_ci_resolve_pnpm
  "${CTX_CI_PNPM_CMD[@]}" "$@"
}
