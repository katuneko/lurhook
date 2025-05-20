# Lurhook 外部設計書（叩き台）

> **版数:** v0.1（2025‑05‑18）
> **作成者:** レオ（アシスタント）
> **参照:** Lurhook 要件定義書 v0.1

---

## 1. システム全体構成

```
┌────────────┐   Cargo workspace   ┌────────────┐
│  Executable │───────────────────▶│ game-core  │  ─┐ メインループ
└────────────┘                     └────────────┘   │
        ▲                                           │uses
        │path                                       ▼
┌────────────┐  uses  ┌──────────┐  uses  ┌──────────┐  uses  ┌──────────┐
│    ui      │◀───────│  fishing │◀───────│ ecology  │◀───────│ mapgen   │
└────────────┘        └──────────┘        └──────────┘        └──────────┘
```

* **game‑core**: 入力／状態遷移／スケジューラ。
* **mapgen**: BSP + ノイズ生成、タイル深度計算。
* **ecology**: 魚スポーン & 行動 AI。
* **fishing**: キャスト／テンションバー／捕獲判定。
* **ui**: 描画ラッパ & ウィジェット。
* **assets/**: JSON データ + RON セーブファイル。
* **common**: 共有の型とエラー定義。
* **game-core 内部モジュール**: `ai.rs` と `ui.rs` にロジックと描画処理を分割し拡張に備える。

## 2. 実行環境 / ビルドターゲット

| ターゲット             | 出力                      | バックエンド    | 入力    | ビルド方法                                            |
| ----------------- | ----------------------- | --------- | ----- | ------------------------------------------------ |
| Desktop (default) | `lurhook.exe` / bin     | OpenGL    | キーボード | `cargo run --release`                            |
| Terminal          | `lurhook_tui` (feature) | Crossterm | キーボード | `cargo run --no-default-features --features tui` |
| WebAssembly       | `lurhook.wasm`          | WebGPU    | キーボード | `wasm-pack build --target web`                   |

## 3. UI 仕様 (ターミナル 80×25)

```
┌──────────────────────────────────────────────┐
│ [32mLURHOOK v0.1[0m                                         □     │  <釣具アイコン>
│──────────────────────────────────────────────│
│                                                │
│  @  )))  ~~~                                   │  ← マップ領域 (60×18)
│  .. ~~~                                        │
│                                                │
│────────────────────────┬──────────────────────│
│Log                      │ステータス            │
│> 開始地点だ…            │HP: ♥♥♥  Food:[#####-----]│
│> 餌を付けた。            │Line: ▓▓▓▓           │
│> ...                    │Depth: 12m          │
│                         │Time: Dawn          │
└────────────────────────┴──────────────────────┘
```

* **Map Window**: `mapgen` が生成するタイルを描画。
* **Log Panel**: 最大 8 行。スクロールは PgUp/PgDn。
* **Status Panel**: HP / Food / Line / Depth / 時刻。
* **テンションバー**: 釣り中のみ Map Bottom に表示。
* **照準モード**: キャスト開始時に `*` でターゲットタイルをハイライトし、方向キーで移動後 `c` で確定。
* **キャストアニメ**: 確定後、`*` が投擲軌跡を描き最後に `o` を1ターン表示。
* **カラーブラインドモード**: `lurhook.toml` に `colorblind = true` を記述すると
  高コントラストの配色で表示される。

## 4. 入力コマンド一覧

| 操作      | デフォルトキー         | 説明               |
| ------- | --------------- | ---------------- |
| 移動      | h/j/k/l or ↑↓←→ | 8方向に1タイル歩く       |
| キャスト    | c               | 照準モードに入り方向＋距離を選択 |
| 引き上げ    | r               | テンション調整 (釣り中)    |
| インベントリ  | i               | 所持アイテム確認 / 竿・餌変更 |
| 生食      | x               | 選択中の魚を食べて満腹回復 |
| 調理      | f               | 陸上で魚を焼いて満腹+40/HP+2 |
| 携行食使用 | g               | 缶詰を食べて満腹+60 |
| ログスクロール | PgUp/PgDn       | 過去ログ閲覧           |
| ヘルプ      | F1              | 操作ガイドを表示/閉じる     |
| セーブ     | S               | 即時保存             |
| ロード     | L               | タイトルでロード画面へ      |
| ラン終了    | Enter           | スコアを集計して終了画面へ |
| 終了      | Q               | 保存確認後に終了         |

キーリマップは `lurhook.toml` に保存。

## 5. データファイル仕様

### 5.1 魚種データ `assets/fish.json`

```jsonc
[
  {
    "id": "LUR1",
    "name": "Lurker Bass",
    "rarity": 0.8,      // 0–1 低いほどレア
    "strength": 12,     // テンション増加係数
    "min_depth": 10,
    "max_depth": 30,
    "fight_style": "Aggressive", // ファイトパターン
    "legendary": false
  }
]
```

### 5.2 セーブデータ `save_*.ron`

```ron
(
  player: (
    pos: (x:12, y:7),
    hp: 3,
    inventory: [ (item:"BasicRod", dmg:0) ]
  ),
  map_seed: 123456,
  time_of_day: Dawn,
)
```

### 5.3 アイテムデータ `assets/items.json`

```jsonc
[
  {
    "id": "BASIC_ROD",
    "name": "Basic Rod",
    "kind": "Rod",
    "tension_bonus": 0,
    "reel_factor": 1.0,
    "bite_bonus": 0.0
  }
]
```
* `kind` は `Rod` / `Reel` / `Lure` / `Food` のいずれか。
* `tension_bonus` は最大テンションへの加算値。
* `reel_factor` はリールの効果係数。1.0 で等倍、2.0 で2倍引き寄せ。
* `bite_bonus` はバイト確率への加算値。

## 6. 主要ロジックシーケンス

### 6.1 ターン処理フロー

```
Player Input → Update Systems → AI Move / Spawn → Resolve Collisions → Render → Wait
```
* ターン終了時に満腹度を1減少。0の場合はHPが1減る。
* プレイヤーの入力があった場合のみ1ターン進み、無入力では時間経過しない。
* インベントリ画面で`X`を押すと魚を生食して満腹度+20。
* 陸上で`F`を押すと魚を調理して満腹度+40、HP+2。
* `G`を押すと携行食を消費して満腹度+60。


### 6.2 釣りシーケンス (成功)

```
キャスト → 待機 (タイマー) → バイト発生 → テンションゲーム → 成功 → 魚を Inventory へ
```

* テンションが0になった場合はフックアウトし失敗扱い。
* 魚の `fight_style` によりテンション増加挙動が変化する。

### 6.3 視界制限

* プレイヤーが `DeepWater` タイル上にいる場合、視界半径は 5 マスに限定する。
* 範囲外のタイルは暗灰色で描画し、位置のみ判別可能とする。
* 深海でランダムに嵐イベントが発生すると、5 ターンの間視界半径を 3 まで低下させる。
* 陸上では探索中に低確率で休息 (HP+1) または缶詰入手イベントが発生。
* 水域では低確率でクラゲが出現し、接触するとHPを1失いライン強度も10減少する。

## 7. モジュール I/F 詳細

| Producer | Consumer  | 関数 / Channel                   | 内容            |
| -------- | --------- | ------------------------------ | ------------- |
| mapgen   | game-core | `pub fn generate(seed) -> Map` | 新マップ生成        |
| ecology  | game-core | `pub fn spawn_fish(map)`       | 魚 Entity 配置 (水タイルからランダム選択) |
| fishing  | ui        | `pub struct TensionMeter`      | Draw + 更新メソッド |
| ui       | game-core | `pub struct UIContext`         | ログ追加, リフレッシュ  |

## 8. エラーハンドリング

* `Result<T, GameError>` 共通型を crates 間で共有。
* 重大エラー時はログ出力後にタイトルへフォールバック。

## 9. ロギング

* `env_logger` で trace/debug/info レベル切替。
* ゲーム内ログとコンソールログを分離 (feature flag)。

## 10. テスト方針

| 階層       | 方法                           | カバレッジ目標   |
| -------- | ---------------------------- | --------- |
| ビジネスロジック | `cargo test` ユニット            | 80%+      |
| マップ生成    | Golden Master スナップ比較         | 変更時レビュー必須 |
| WASM     | Headless `wasm-bindgen-test` | 起動～タイトル表示 |

## 11. CI パイプライン (GitHub Actions)

1. **Lint**: `cargo clippy -- -D warnings`
2. **Test**: `cargo test --all`
3. **Build Matrix**: ubuntu-latest, windows-latest, macos-latest
4. **WASM Build**: `wasm-pack build` + `npm run test`
5. **Release Upload**: tag pushで 3OS バイナリ + wasm.zip を Release

## 12. 未決定事項 / TODO

* バイト確率の数式調整 (要ゲームバランス検証)
* 色弱フレンドリーパレット: `colorblind` 設定で切り替え済み

---

> ### 次のステップ
>
> 1. 本外部設計の粒度や UI レイアウトを確認。
> 2. 未決定事項の優先度を整理し、Issue 化。
> 3. I/F 詳細を実装着手前にマイクロ設計へ展開。

🐾 **レオより:** 突っ込みや追加要望があればぜひワン！
