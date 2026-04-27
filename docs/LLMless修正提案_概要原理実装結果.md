# LLMless で Rust コード修正提案 — タスク/実験対応表 + 概要・原理・実装・結果

**集計日**: 2026-04-22
**プロジェクト目標 (T4)**: bit 配置の探索だけで Rust コードのエラー検出・修正提案を行う「謎のアセンブラ」の構築
**前提**: f32/f64 禁止 (公理 A0) / rustc オラクル単一真理 (公理 A5) / LLM・ニューラル・GPT 系 API は一切使わない

---

## 0. タスク/実験番号 早見表

LLMless で Rust コード修正提案を行う**直接的な**タスクと実験は以下の 7 系列です。

| 系列 | Task # | 実験 # | 役割 | 主要結果 |
|---|---|---|---|---|
| **A. ensemble29 系列 (起点)** | #27 / #46 周辺 | **E39** ensemble18→29 | 51/56 ファイル修正成功 (LLMless 単独達成) | **fix-pass = 51/56 = 91.1%** |
| **B. C4.1 反証 (理論固め)** | **#48** (`e68-c4-1-conjecture-verify`) | **E68 exhaustive** | 残 5 件で μ\* が候補空間に存在することを実験的に確定 | 1537 件中 H<H_buggy = 681 件、H_buggy−1=6 = 40 件 |
| **C. nibble × rustc greedy walk** | **#49** (`e68-nibble-freq-oracle`) | **E68-walker** | u4 nibble 確率変換で gold 方向への前進を実証 | 平均 **gc_H = +0.27, gc_u4 = +0.005** (E63〜E67 で唯一の正値) |
| **D. xcorr 誘導ウォーク** | (E70 系列) | **E70 / E71a** | nibble 置換に xcorr 位相を導入 | gc_H ±0、E71a で PSR=1.5 (失敗、tap_poly 誤り) |
| **E. 保存則ニブルウォーク** | **#55** (`task-55`) / **#57** (`task-55-e72-review`) | **E72** (MBS-アンカーI-GOLD-アンカーII-LBS) | 保存則を満たす 5 ヘアピン構造、5 ファイル詳細レビュー | gc_H **+0.2167**、gc_u4 **+0.1615** (E68 比 **+3195%**) |
| **F. 補題 L7.2 改訂** | **#50** | E57 周辺 | headScore=0 条件を popcount(h)=2 に必要十分化 | 旧版 (h∈{0,5,10,15}) を反証して改訂 |
| **G. インクリメンタル IDF²/XCORR** | **#99** (`task-99-e93-incremental-idf-xcorr`) | **E93** | バッチ → 1 ファイル単位ストリーミングへ書換、bit 完全一致を assert_eq! 検証 | IDF² / XCORR / OR-prototype 全件 bit 一致、N=400 で 320μs/file |

参考: 共通基盤タスク
- `goldcycle-common-extract.md` … `gold_cycle.rs` 共通化
- `rust-compile-result-rule.md` … rustc オラクルの呼び出し規約
- `ensemble29-mutation-types.md` / `ensemble28-rc-cs.md` … 変異型カタログ

---

## 1. 概要 (なぜ LLM なしで Rust が直せるのか)

### 1.1 目的
- 入力: コンパイルエラーを含む Rust ソース `s_buggy`
- 出力: 修正候補 `s_fix` (rustc が `error_count=0` を返す)、もしくは候補空間に修正が**存在しないこと**の証明

### 1.2 全体像
```
s_buggy ──▶ ① TypeSet T (n=3,4 gram)
         └▶ ② DocBits 投影 (1024-bit space, A4)
              │
              ▼
         ③ 候補生成 (ensemble29 / nibble 変換 / 保存則ウォーク)
              │
              ▼
         ④ rustc オラクル (A5) で error_count を測定
              │
              ▼
         ⑤ greedy hill climbing: H(t) < H(s) なら採用
              │
              ▼
         s_fix (or 不可能性の証明)
```

### 1.3 どこが LLMless なのか
- **生成側**: 候補は「コード片を表す nibble (4-bit) 列を確率的に置換」して作る (エンベディングを使った再構成ではなく**ビット幅の組合せ探索**)
- **評価側**: rustc を**唯一の意味判定器**として用いる (人間言語モデル不要)
- **絞り込み**: 1024-bit カプセル空間 + IDF² + 巡回相関 (XCORR) + Gold 系列 (LFSR x¹⁰+x⁷+1) で**疑似ランダム位相基底**を作り、修正に有効な変換を統計的に選ぶ

---

## 2. 原理

### 2.1 公理 (`THEORY_RUST_CODEGEN.md`)
- **A0**: 計算は **u8/u32/u64 整数演算と bitwise 演算**のみ。f32/f64 禁止
- **A4 (改訂版)**: `E: bytes → {0,1}^1024` は**非線型ハッシュ圧縮** (SimHash/Bloom 系)。加法性は持たない (Task #111 で反証済)
- **A5**: `rustc` は判定オラクルで決定的。`error_count = 0` を「コンパイル可能」の真理値とする
- **A6/A7/A8**: コーパス公理 (3 コーパスで PASS、英語文学を加えると A7 のみ FAIL → 3 コーパス公理に降格)

### 2.2 nibble (u4) gram と DocBits
- バイト列 `b ∈ {0..255}*` を nibble 列 `n = (n₀, n₁, …) ∈ {0..15}*` (1 byte → 上位 + 下位) に展開
- n=3, n=4 の gram (n=2 は破棄) を辞書化
- `bit_addr(i, g) = (4i + g) mod 1024` で 1024-bit DocBits に投影 (= 公理 A4 の `E`)
- 距離はハミング距離 `xnor_l1` (`bitset.rs::popcount`)

### 2.3 候補生成 (3 段階)
1. **ensemble29 (E39)**: 21 候補トークン × 30 語彙 × 全出現位置 × 5 種変異
   = 1537 件の SNV (Single Nucleotide Variant 風) 全列挙
2. **nibble 確率変換 (E68-walker)**: バイト 1 個の上位 or 下位 nibble を乱数 nibble 値に置換、`Lcg64` で再現性確保
3. **保存則ニブルウォーク (E72)**:
   ```
   [MBS: 高 nibble 列 (削除空間: 二進 10)]
     + [アンカー I: 16nibble, OOV 率=1.0 → H_cond=∞]
     + [GOLD: 1023nibble, Galois LFSR x¹⁰+x⁷+1, PSR=16.24]
     + [アンカー II: 同上]
     + [LBS: 低 nibble 列 (挿入空間: 二進 01)]
   ```
   - **保存的ステップ**: MBS[i] と LBS[j] を同時修正 (削除+挿入で ±0)
   - **保存則破綻**: 片側のみ → 自己回帰遷移点 (E72 では 0 件で対称進行)

### 2.4 スコア関数
- `H(s)` = rustc が返した `error_count` (A5)
- `gc_H = (H_start − H_min) / H_start` … エラー削減率 (1=全消去)
- `gc_u4 = 1 − L2(u4_final, u4_gold) / L2(u4_buggy, u4_gold)` … 頻度空間で gold 方向への進度
- `headScore` (補題 L7.2): popcount(h)=2 のとき**かつそのときに限り** XOR キャンセル (h ∈ {0011, 0101, 0110, 1001, 1010, 1100})

### 2.5 penMask と C4.1' (探索失敗の正体)
- 一度 rustc に拒否された変異の使用ビットは `penMask` に蓄積
- 飽和 (ρ\* ≈ 9.2%, |P\*|₁ ≈ 94 bit) すると、修正に必要な i32/String/Vec/Copy/IntoIter 等が `P(被ブロック) ≈ 0.62〜0.65` で全件フィルタアウト
- → 「H=2 から脱出不可能」は**物理的限界ではなくアルゴリズム的探索失敗** (定理 T4.2')
- 反証データ: `MATH_FOUNDATIONS.md` §4.4

---

## 3. 実装

### 3.1 中核モジュール

| ファイル | 行数 | 役割 |
|---|---|---|
| `bitrag-core/src/gold_cycle.rs` | 556 | LFSR / Gold 系列 / xcorr / nibble 操作 / **rustc_run** / **goldcycle_walk** の共通基盤 |
| `experiment-39/ensemble.mjs` 〜 `ensemble10.mjs` | (JS) | LLMless 修正提案の本丸。SNV 全列挙 + rustc 並列実行で 51/56 達成 |
| `experiment-68/src/main.rs` | 715 | exhaustive μ\* 探索 (1537 件) + greedy walk |
| `experiment-69/src/main.rs` | 220 | xcorr 非線形射影 + γ スコア圧縮 |
| `experiment-72/src/main.rs` | 558 | 保存則ニブルウォーク (MBS-アンカー-GOLD-LBS-アンカー) |
| `experiment-93/src/main.rs` | 974 | インクリメンタル IDF²/XCORR (バッチと bit 完全一致) |
| `experiment-59/src/main.rs` | 457 | gold_cycle 基盤の起点 (gold ファイル: 5 件の修正済み正例) |

### 3.2 `gold_cycle.rs` の主要 API (抜粋)
```rust
pub fn m_seq_10(tap_poly: u32) -> Vec<i32>;          // 10-bit LFSR m 系列
pub fn gold_seq() -> Vec<i32>;                       // 2 本 m 系列 XOR で Gold 系列
pub fn circ_xcorr_nib(a: &[u8], b: &[u8]) -> Vec<i64>;
pub fn build_phi(idf_sq: &[f32], v: usize, l: usize) -> Vec<usize>; // ← f32 残存 (PG4 課題)
pub fn nibble_mutate_rng(rng: &mut Lcg64, src: &[u8])
    -> (Vec<u8>, usize, bool, u8);                    // 単一 nibble 置換
pub fn rustc_run(src: &[u8], tmp_path: &str) -> (u32, String); // ← オラクル
pub fn goldcycle_walk<S: WalkStrategy>(...);         // greedy hill climbing 共通骨格
pub fn strip_annotations(src: &str) -> String;       // // ERROR 等を剥がして rustc に渡す
```

### 3.3 ensemble29 (E39) のアルゴリズム概要 (mjs ベース)
1. `s_buggy` を AST レベルで字句化 (rustc の `--error-format=json` で span 取得)
2. **21 候補トークン** × **30 語彙** × **全出現位置** で SNV 候補集合 𝒞 を生成
3. 𝒞 の各要素について並列に rustc を実行、`error_count` を収集
4. `H < H_buggy` を満たす最良候補を選択 (greedy)
5. `penMask` に既使用 nibble ビットを蓄積し、次サイクルから除外
6. 多段化 (ensemble2..29) で異なる変異型をラウンドロビンに切替
7. 51/56 達成、残 5 件は C4.1 で「penMask 飽和」と判明 → E68 で反証

### 3.4 E68-walker (greedy hill climbing) 擬似コード
```
seed = 0x00680000cafebabe
for trial in 0..100:
    (s', i, hi_lo, new_nib) = nibble_mutate_rng(rng, s)
    H' = rustc_run(s')
    if H' < H:                  // greedy
        s = s'; H = H'
        if H == 0: break
record (H_start, H_min, H_final, accept_count, gc_H, gc_u4)
```

### 3.5 E72 保存則ニブルウォーク
```
GOLD = gold_seq()                       // 1023 nibble, PSR=16.24
anchor = top_k_OOV_nibbles(corpus, k=16) // H_cond=∞ 配列を貪欲探索
state  = MBS || anchor || GOLD || anchor || LBS
loop:
    pick i in MBS, j in LBS              // 同時修正で保存則維持
    state' = mutate_pair(state, i, j)
    H' = rustc_run(decode(state'))
    if H' <= H: state = state'; H = H'
```

### 3.6 重要な実装規約
- **rustc 呼出**: `--edition=2021 --error-format=short`、ASCII パスのみ (日本語 dir で linker 失敗するため)
- **再現性**: 全実験で固定シード (Lcg64 `state.wrapping_add(1)` → wrapping_mul 64bit MUL)
- **BIT 完全一致テスト**: E93 で `assert_eq!(batch_idf2, incremental_idf2)` をトークン全件 (14132 個) で検査
- **f32/f64 禁止**: 現在 `gold_cycle.rs` 内に 33 箇所 f32/f64 残存 (PG4 課題、整数化が次タスク)

---

## 4. 結果

### 4.1 修正成功率 (修正 = rustc error_count=0)
| 実験 | 手法 | 入力 | 修正成功 | 備考 |
|---|---|---|---|---|
| **E39 (ensemble18→29)** | LLMless SNV 並列 + greedy | rustc UI test 56 件 | **51/56 = 91.1%** | LLM 不使用で達成。残 5 件は C4.1 ターゲット |
| E39 (LLM 比較群) | gpt-oss-120b / qwen-3-235b | 同上 | 別途記録 (`fixed_llm/`) | 比較対照 |

### 4.2 残 5 件 (難関セット) の進捗
| ファイル | E68 H_start→H_min | E72 H_start→H_min | gc_H (E72) | compile-pass | 原因 |
|---|---|---|---|:-:|---|
| type-ascription | 1→1 | 2→2 | +0.0000 | ✗ | nightly 機能 `type_ascription` |
| type-check-defaults | 7→1 | 8→2 | **+0.7500** | ✗ | 意味論的型制約 (残留 H=2) |
| type-unsatisfiable | 1→1 | 2→2 | +0.0000 | ✗ | revision directives |
| typeid-consistency | 2→1 | 3→2 | +0.3333 | ✗ | 外部クレート aux1/aux2 必須 |
| verbose | 1→1 | 2→2 | +0.0000 | ✗ | revision directives |

→ stable rustc 単体では構造的に修正不能 (E39 時点と整合)。ただし E68 で「**H<H_buggy を達成する単一変異 μ\* は候補空間に存在**」(681/1537 件) を確認、限界は penMask 経路妨害であると確定。

### 4.3 頻度空間での進度比較
| 実験 | 手法 | 平均 gc_u4 |
|---|---|---|
| E63 | rustc エラー削減ベース | -0.102 |
| E66 | u8 単段 XOR (最良ゲート) | -4.241 |
| E67 | u4 二段 AND→XOR (最良) | -3.560 |
| **E68-walker** | nibble 変換 greedy (100 試行) | **+0.005** ← 唯一の正値 |
| **E72** | 保存則ニブルウォーク | **+0.1615** ← E68 比 +3195% |

### 4.4 E93 同値検証 (バッチ ↔ インクリメンタル)
| 量 | 検証件数 | 結果 |
|---|---|---|
| IDF² | 14,132 トークン (vocab) | **bit 完全一致** |
| XCORR | 64 シフト | **bit 完全一致** |
| OR-prototype | 1 値 | **bit 完全一致** |

スループット (N=400): batch 121.8ms / incremental 128.0ms (per-file 320μs)

### 4.5 反証された主張 (= 実験で潰したもの)
1. **C4.1 強い命題** (51/56 天井は物理的限界) → E68 で反証 → C4.1' (アルゴリズム的探索失敗) に改訂
2. **A4 加法性** → Task #111 で反証 → A4 を非線型ハッシュに改訂
3. **L7.2 旧版** (headScore=0 ⇔ h∈{0,5,10,15}) → E57 で反証 → popcount(h)=2 ⇔ headScore=0 に改訂

詳細は `全反証履歴.md` 参照。

---

## 5. 同梱物 (zip 内訳)

```
LLMless-rust-fix-proposal.zip
├── README.md                            ← 本ファイル
├── theory/
│   ├── THEORY_RUST_CODEGEN.md          (公理 A0..A8、改訂履歴)
│   ├── MATH_FOUNDATIONS.md              (§4.4 C4.1 反証、§7 L7.2 改訂)
│   └── 全反証履歴.md
├── core/
│   └── bitrag-core/src/gold_cycle.rs   (556 行、LLMless 修正提案の共通基盤)
├── experiments/
│   ├── e39-ensemble29/                  (ensemble*.mjs, fixed/, fixed_llm/, results/)
│   ├── e59-gold-cycle/                  (基準正例 5 件、src/)
│   ├── e68-exhaustive-walker/           (1537 件 μ* 探索 + greedy walk)
│   ├── e69-gamma-projection/            (xcorr 非線形射影 + γ スコア)
│   ├── e72-conservation-walk/           (MBS-アンカー-GOLD-LBS、レビュー 6 件)
│   └── e93-incremental-idf/             (バッチ ↔ ストリーミング bit 一致)
└── tasks/
    ├── task-48.md                       (e68-c4-1-conjecture-verify)
    ├── task-49.md                       (e68-nibble-freq-oracle)
    ├── task-55.md                       (E72 review)
    ├── task-57.md                       (E72 review replit-haircut)
    ├── task-99.md                       (e93 incremental)
    ├── ensemble29-mutation-types.md
    ├── ensemble28-rc-cs.md
    ├── goldcycle-common-extract.md
    └── rust-compile-result-rule.md
```

※ 各実験ディレクトリの `target/` (Cargo build cache) と `e39_*/` (rustc 一時 cache 約 100MB) は zip から除外。
