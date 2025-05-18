#!/usr/bin/env bash
set -euo pipefail

# ────────────────────────────────────────────────────────────────
#  Lurhook ASCII Fishing RL – project bootstrap script
# ────────────────────────────────────────────────────────────────
# 前提:
#   - リポジトリを git clone 済みで、このスクリプトはルートに置かれている
#   - Rust (rustup / cargo) がインストール済み
#   - bash が使える (Linux, macOS, WSL2)
#
# 処理概要:
#   1. Rust toolchain 更新
#   2. wasm32 ターゲット追加
#   3. dev ツール (cargo-watch, wasm-pack) が無ければ install
#   4. rustfmt / clippy を追加
#   5. `cargo check` & `cargo test --no-run`
# ────────────────────────────────────────────────────────────────

msg() { printf "\033[1;32m==>\033[0m %s\n" "$*"; }
err() { printf "\033[1;31m[ERROR]\033[0m %s\n" "$*" >&2; exit 1; }

# 0. 前提チェック
for cmd in rustc cargo rustup; do
  command -v "$cmd" >/dev/null || err "$cmd が見当たりません。Rust をインストールしてください。"
done

# 1. Rust toolchain 更新
msg "🔄 Updating Rust toolchain…"
rustup update

# 2. wasm32 ターゲット追加
msg "🌐 Adding wasm32-unknown-unknown target…"
rustup target add wasm32-unknown-unknown

# 3. 開発ツールインストール（無ければ）
for CRATE in cargo-watch wasm-pack; do
  if ! command -v "$CRATE" >/dev/null; then
    msg "🔧 Installing $CRATE ..."
    cargo install "$CRATE"
  fi
done

# 4. rustfmt / clippy コンポーネント追加
for COMPONENT in rustfmt clippy; do
  if ! rustup component list --installed | grep -q "^$COMPONENT"; then
    msg "🔧 Adding rustup component $COMPONENT ..."
    rustup component add "$COMPONENT"
  fi
done

# 5. ビルド確認
msg "🦀 cargo check (debug)…"
cargo check

msg "🧪 cargo test (unit tests)…"
cargo test --all --no-run

msg "✅ Setup complete!  これで開発を始められるワン！"
cat <<EOF
──────────────────────────────────────────────
  ▸ 開発中ホットリロード  : cargo watch -x run
  ▸ フォーマッタ          : cargo fmt
  ▸ Lint (警告をエラー化) : cargo clippy -- -D warnings
  ▸ WASM ビルド           : wasm-pack build --release --target web
──────────────────────────────────────────────
EOF
