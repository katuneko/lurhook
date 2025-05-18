# Lurhook 開発タスクリスト (GitHub 用)

> **目的**: `docs/requirements.md` と `docs/designs.md` に基づき、ローカル開発 (オンプレ環境、`cargo run`) を段階的に進めながら品質を結合ポイントで固めていく。
> **管理方法**: 各項目は GitHub Issue として登録し、以下のチェックリストをコピーして進捗を管理してください。

---

## マイルストーン構成

| Milestone               | フェーズ       | 主な完成条件                 |
| ----------------------- | ---------- | ---------------------- |
| **M0 Foundation**       | Step 1–4   | ビルド通過 + プレイヤー移動 (空マップ) |
| **M1 World & Ecology**  | Step 5–7   | ランダムマップ + 魚配置＆移動       |
| **M2 Fishing Core**     | Step 8–10  | 釣りミニゲーム完成 + UI充実       |
| **M3 Persistence & QA** | Step 11–12 | セーブ/ロード + CI & テスト拡充   |

---

## 詳細タスク

### Milestone **M0 Foundation**

<details>
<summary>クリックして展開</summary>

#### Step 1 — 環境セットアップ & CI 基盤

* [x] Rust 1.78+ のインストール (`rustup`)
* [x] リポジトリをクローンし、`cargo run` が "Welcome to Lurhook!" を表示することを確認
* [x] `.github/workflows/ci.yml` を作成し、Ubuntu 最新版で以下を実行

  * `cargo clippy -- -D warnings`
  * `cargo test --all --offline`
* [x] CI パイプラインがグリーンになることを確認

#### Step 2 — インターフェース定義 & スタブ実装

* [x] 各クレートの公開 API を明文化 (mapgen / ecology / fishing / ui / data)
* [x] スタブ関数・構造体を実装し、ドキュメントコメントを付与
* [x] `game-core::run()` でスタブを順に呼び出し、ビルドが通ることを確認

#### Step 3 — 基本 UI ループ統合

* [x] `bracket-lib` 依存を追加
* [x] `LurhookGame` 構造体で `GameState` を実装
* [x] 画面にプレースホルダ文字 (タイトル or `@`) を描画
* [x] ウィンドウの作成・終了が正常なことを確認

#### Step 4 — プレイヤー移動 & 入力ハンドリング

* [x] h/j/k/l & 矢印キーで 8 方向移動を実装
* [x] 画面端で移動を抑制する境界チェック
* [x] 移動ロジックのユニットテスト

</details>

---

### Milestone **M1 World & Ecology**

<details>
<summary>クリックして展開</summary>

#### Step 5 — マップ生成 (Mapgen)

* [x] `Map` 構造体と `TileKind` 列挙型を設計
* [x] `mapgen::generate(seed)` を BSP + パーリンノイズ (プレースホルダ可) で実装
* [x] 生成マップを UI へ描画
* [x] 固定シードのスナップショットテストを追加
* [x] マップサイズを120×80に拡張し、プレイヤー中心にスクロール描画

#### Step 6 — 魚スポーン (Ecology)

* [x] `Fish` 構造体 & 種別列挙を定義
* [x] `ecology::spawn_fish(&mut Map)` で水タイルへ魚を配置
* [x] 魚シンボルを描画し、位置が妥当かテスト

#### Step 7 — 魚 AI & ターン処理

* [x] `ecology::update_fish` でランダム移動 AI を実装
* [x] ゲームループへ統合 (入力→AI→描画)
* [x] 境界・水域判定のユニットテスト

</details>

---

### Milestone **M2 Fishing Core**

<details>
<summary>クリックして展開</summary>

#### Step 8 — 基本釣りフロー

* [x] `c` キーでキャスト → 釣りモード遷移
* [x] 待機ターン後、固定確率でバイト判定
* [x] 成功時: 魚をインベントリへ、失敗時: ログに逃亡メッセージ
* [x] 魚が残っていない場合、`cast` を無効にする事で虚偽の捕獲を防止

#### Step 9 — テンションバー・ミニゲーム

* [x] `TensionMeter` 構造体とテンション計算ロジック
* [x] テンションバー UI を釣りモード時に描画
* [x] 成功/失敗判定とユニットテスト (テンション計算)

#### Step 10 — UI パネル & ログ強化

* [x] ログウィンドウ (最大 8 行, PgUp/PgDn でスクロール)
* [x] ステータスパネル (HP, Line, Depth, Time)
* [x] 標準レイアウト/釣りレイアウトの切替

</details>

---

### Milestone **M3 Persistence & QA**

<details>
<summary>クリックして展開</summary>

#### Step 11 — データロード & セーブ

* [x] `assets/fish.json` を Serde で読み込み、魚種リスト生成
* [x] 魚強度などゲームロジックをデータ駆動化
* [x] ゲーム状態を RON 形式で保存 (`save_<datetime>.ron`)
* [x] エラー時ハンドリングとロード機能 (任意)

#### Step 12 — テスト拡充 & CI 強化

* [ ] 各クレートでユニットテストを追加し、80%+ カバレッジ
* [ ] ゴールデンマスター & スナップショットテスト導入
* [x] スナップショットテストの改行差異を吸収しクロスプラットフォーム化
* [x] GitHub Actions を Linux/Windows/macOS + WASM マトリクスに拡張
* [x] `cargo clippy -- -D warnings` を CI に組み込み、パフォーマンス回帰テスト(任意)

</details>

---

## 進め方ガイド

1. **Issue 化**: 上記チェック項目を GitHub Issue として登録し、ラベル `foundation` / `world` / `fishing` / `persistence` を付与。
2. **マイルストーン設定**: M0～M3 を GitHub Milestone に登録し、関連 Issue を紐付ける。
3. **ブランチ戦略**: Milestone ごとに `feature/m0-*`, `feature/m1-*` などのプレフィックスを付け、`main` ブランチへ PR (Pull Request) ベースで統合。
4. **レビュー & CI**: PR 作成時に CI が走り、グリーンになればレビュー。レビュー完了後にマージ。
5. **進捗更新**: Issue のチェックボックスを更新し、完了したら `Closed`。

> **メモ**: 本 `task.md` は進捗管理のハブとして README とは分けてリポジトリ直下に配置し、随時更新してください。
