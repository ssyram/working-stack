# 工具链引导与 Rust 安装器入口。
$ErrorActionPreference = "Stop"
$RepoDir = Split-Path -Parent $MyInvocation.MyCommand.Path

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    $tmp = "$env:TEMP\rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $tmp
    & $tmp -y --default-toolchain stable --profile minimal
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    $env:Path += ";$env:USERPROFILE\.cargo\bin"
}

cargo run --locked --manifest-path "$RepoDir\Cargo.toml" --example distribute -- install @args
exit $LASTEXITCODE
