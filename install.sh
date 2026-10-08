#!/bin/sh
# 工具链引导与 Rust 安装器入口。
set -eu
REPO_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"

if ! command -v cargo >/dev/null 2>&1; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
    . "$HOME/.cargo/env"
fi

cargo run --locked --manifest-path "$REPO_DIR/Cargo.toml" --example distribute -- install "$@"
