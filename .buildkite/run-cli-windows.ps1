$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptDir
$coreRoot = Join-Path $repoRoot "core"

Set-Location $coreRoot

cargo test --manifest-path Cargo.toml -p ctx-http --bin ctx agent_work_cli::tests --locked
cargo build --manifest-path Cargo.toml -p ctx-http --bin ctx --locked

$ctxExe = Join-Path $coreRoot "target/debug/ctx.exe"
if (-not (Test-Path $ctxExe)) {
  throw "expected Windows ctx CLI artifact at $ctxExe"
}
