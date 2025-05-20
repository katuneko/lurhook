# Lurhook

🐾 **ASCII 釣りローグライクゲーム** — *質実剛健、ターミナルで大物を狙え！*

> レオ（わんこ助手）がお届けする個人開発プロジェクトです。

---

## 🎣 概要

**Lurhook** はランダム生成される海域を探索し、希少魚を釣り上げて帰還することを目指すローグライクゲームです。80×25 のターミナルを主戦場とし、シンプルな ASCII 表現で奥深い釣りメカニクスと探索を両立します。
最新バージョンではマップが画面サイズを超えて生成され、プレイヤーの移動に合わせてスクロール表示されます。

* 👾 ランダム生成マップ
* 🐟 テンションバーで駆け引きする釣りシステム
* 🌊 潮流・時間帯・群れ行動で変化する生態系
* 🏝️ ランダム地形イベント (休息・嵐)
* 🪓 モジュール化された Rust + bracket-lib エンジン
* 💾 セーブ&ロード対応
* 🧳 インベントリ表示で捕獲魚を確認
* 🖥️ Windows / macOS / Linux / WASM 対応予定
* 🔱 難易度モード (Easy/Normal/Hard)
* ⭐ キャスト時に軌跡と水しぶきを ASCII 演出

## 🏗️ ビルド方法

### 前提

* **Rust** stable (1.78 以上)
* Git

```bash
# 1. クローン
$ git clone https://github.com/yourname/lurhook.git
$ cd lurhook

# 2. テスト
$ cargo test --all

# 3. ビルド
$ cargo run --release
```

実行すると bracket-lib がウィンドウを開き、画面中央に `@` が表示されます。

```
```

#### ターミナル専用バックエンド

```bash
$ cargo run --no-default-features --features tui
```

#### WebAssembly (WASM)

```bash
# wasm-pack が必要です
$ wasm-pack build --target web
# ビルド後は `index.html` をブラウザで開いてプレイ
```

> **Tip:** 開発中は `cargo watch -x run` で保存ごと即実行が便利！

## ⌨️ 操作方法（デフォルト）

| アクション   | キー                          |
| ------- | --------------------------- |
| 移動      | h / j / k / l (または ← ↓ ↑ →) |
| キャスト    | c                           |
| テンション調整 | r                           |
| インベントリ  | i                           |
| 生食      | x                           |
| 調理      | f                           |
| 携行食使用 | g                           |
| ログスクロール | PgUp/PgDn                   |
| ヘルプ      | F1                          |
| オプション  | O                           |
| ラン終了    | Enter                       |
| セーブ     | S                           |
| 終了      | Q                           |

キーリマップや音量は `lurhook.toml` を編集するか、ゲーム内 Options で変更できます。
例:
```toml
left = "A"
right = "D"
up = "W"
down = "S"
cast = "C"
reel = "R"
volume = 5
font_scale = 1
```
`colorblind = true` を追加すると、色弱向けの高コントラスト表示に切り替わります。
ゲーム内 Options メニューで切り替えた場合も自動でこの設定が保存されます。
`font_scale` を 2 以上にするとフォントを拡大表示できます。

## 📦 ディレクトリ構成

```
 lurhook/
 ├─ Cargo.toml          # workspace 定義
 ├─ src/               # 実行バイナリ (main)
 ├─ crates/
 │   ├─ game-core/     # ゲームループ
 │   ├─ mapgen/        # マップ生成
 │   ├─ ecology/       # 魚 AI
 │   ├─ fishing/       # 釣りメカニクス
 │   └─ ui/            # 描画＆ログ
 └─ assets/
     ├─ fish.json
     └─ items.json
```

## 🚧 ロードマップ

| フェーズ     | ステータス  | 内容             |
| -------- | ------ | -------------- |
| 0. 準備    | ✅      | リポジトリ + ビルド通過  |
| 1. 移動基盤  | ✅      | プレイヤー移動／空マップ   |
| 2. マップ生成 | ✅      | ランダム水域         |
| 3. 釣り実装  | ⏳      | テンションバー / 捕獲判定 |
| 4. 生態系拡張 | ⏳      | 群れ AI / 潮流システム (一部実装) |
| 5. 仕上げ   | ⏳      | 調整＋WASM デプロイ   |

## 📝 開発ルール

1. **main ブランチは常にビルド可能**。機能追加は `feature/*` → PR → マージ。
2. `cargo clippy` & `cargo test` を GitHub Actions で自動実行。
3. `devlog/` に mdBook 形式で開発ログを残す（モチベ維持！）。

## 🤝 コントリビューション

現在はプライベート開発ですが、フィードバック歓迎です。Issue または PR をお送りください。

## 📜 ライセンス

MIT License

---

> **ワン！** 釣果報告やご意見はお気軽に。大物ゲットのスクショ、お待ちしてます🐾
