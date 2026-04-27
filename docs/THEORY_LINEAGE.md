# bitRAG 理論系譜と派生タスク監査ドキュメント

> **v1 凍結 (2026-04-25, e176-c01)**: 凍結核は `THEORY_CORE_v1.md`。
> 本書は v1 凍結時点の長文版で、本文は変更しない。
> 以後の検証追記は `THEORY_EVIDENCE_LEDGER.md` に行うこと。

本書は `THEORY_RUST_CODEGEN.md` (Task #1) を根として派生した
タスク・実験 (E152〜E163, scheduler, UI, 周辺ツール群) の
**結果・経緯・更新・修正・系譜・比較** をひとつにまとめた監査ドキュメントである。

数値・所見はすべて既存の `EXPERIMENTS_SUMMARY.md` / `EXPERIMENTS.md` /
各 `experiment-*/README.md` / `.local/tasks/*.md` から引用する (新規測定なし)。

参照規約:
- `taskRef` = `.local/tasks/<file>.md`
- `expRef`  = `artifacts/bitrag/experiment-<id>/`
- `compRef` = `bitrag-core/src/<module>.rs` (詳細は `COMPONENT_CATALOG.md`)

---

## 1. タスク系譜ツリー

Task #1 (`bitrag-rust-codegen-semantics-theory.md`, 出力: `THEORY_RUST_CODEGEN.md`) を根として、
§1 公理・§3 合成代数・§5 ユースケース・§7 マッピング・§8 未解決問題の各部位から
派生したタスク群を、Task #2〜#102 の主要ノードとして明示しブランチで示す。

凡例 (状態): [Merged] = 完了済み / [Active] = 進行中 / [Drafts] = 計画のみ未実行

```
Task #1  bitRAG Rust コード生成セマンティクス理論                            [Merged]
│  taskRef: bitrag-rust-codegen-semantics-theory.md
│  出力:    artifacts/bitrag/THEORY_RUST_CODEGEN.md
│
├── Task #2  合成代数 API 公開 (§3 由来; ∧/∨/⊕/⊖ を一級 API へ)              [Drafts]
│       説明: §8 問 1 を実装側に落としたタスク。現状は DocBits.words 直接アクセス
│       で代替されており、E161 の 8 演算子も独立スコア関数として実装されたまま。
│       関連: bitrag-core/src/bitset.rs, e161-all-operators.md
│
├── Task #3  ユースケース 5.1〜5.3 実装 (新規生成 / rustc 修正 / E103 拡張)   [Merged]
│       関連 expRef: experiment-103, experiment-103-0000, experiment-84
│       関連 taskRef: experiment-103-rust-io-bitrag.md, e103-0000-quotient-manifold-embedding.md
│
├── Task #4  §8 未解決問題ピックアップ (代表 1 件)                            [Drafts]
│       内容: §8 の問 5/6/8/10 などから 1 件を選抜し設計メモ化する想定。
│       現状未着手 (フォローアップ #110 として別途提案)。
│
├── §3 合成代数 ── 二進対 (dyadic) アンサンブル集約変種 (∧/∨/shift_k 由来)
│   ├── Task #5         E154 ソフト集約 (popcount 加重投票)                    [Merged]
│   │   taskRef: e154-soft-aggregation.md, expRef: experiment-154-{law,github-rust,github-mixture}
│   ├── Task #6〜#10    (E152/E153 派生; 計画文書のみ; ※省略)                 [Drafts]
│   ├── (taskRef のみ)  e155-antipodal-removed.md  E155 対蹠除外 K=7          [Merged]
│   │   expRef: experiment-155-{law,github-rust,github-mixture}
│   ├── (taskRef のみ)  e155b-non-antipodal.md  E155b sw/3 系                 [Drafts]
│   │   expRef: experiment-155b-* (results/ なし)
│   ├── (taskRef のみ)  e155-hybrid-offsets.md  線形+二進対の併用              [Drafts]
│   ├── (taskRef のみ)  e156-correlation-matrix.md  E156 相関行列診断          [Active]
│   │   expRef: experiment-156-law
│   ├── (taskRef のみ)  e156-failure-anatomy.md   失敗 470988 解剖             [Active]
│   ├── (taskRef のみ)  e157-cyclic-tree.md      E157 環状木 w=2               [Merged]
│   │   expRef: experiment-157-{law,github-rust,github-mixture}
│   ├── (taskRef のみ)  e157-k-scaling.md        E157 K=16,32 拡張             [Drafts]
│   ├── (taskRef のみ)  e158-compression-benchmark-files.md  E158 1000 クエリ [Merged]
│   │   expRef: experiment-158-github-rust
│   ├── (古典)          experiment-128 系 e128-string-dyadic.md               [Merged]
│   ├── (E159, E160 は expRef のみ; 単独 taskRef なし)                         [Merged]
│   │   expRef: experiment-159-{law,github-rust,github-mixture}, experiment-160-{...}
│   ├── Task #95〜#100  E161 全演算子比較 + E162 掃引                          [Merged]
│   │   taskRef: e161-all-operators.md
│   │   expRef:  experiment-161-{law,github-rust,github-mixture},
│   │            experiment-162-ngram-sweep, experiment-162-qlen-sweep
│   └── Task #91  E163a 整数べき乗反復 on グラム行列 (公理 A2/A4 検証)         [Merged]
│       taskRef: task-91.md / e163a-int-power-iter.md
│       expRef:  experiment-163a-int-power-iter
│   ├── Task #92  E163b F2 Smith 標準形 (公理 A1 検証)                        [Merged]
│   │   taskRef: task-92.md / e163b-f2-svd.md
│   │   expRef:  experiment-163b-f2-svd
│   ├── Task #93  E163c ブール代数 SVD (ASSO; A1/A2 検証)                     [Merged]
│   │   taskRef: task-93.md / e163c-boolean-svd.md
│   │   expRef:  experiment-163c-boolean-svd
│   └── Task #94  E163d 熱帯 (max,+) SVD (A4 検証)                            [Merged]
│       taskRef: task-94.md / e163d-tropical-svd.md
│       expRef:  experiment-163d-tropical-svd
│
├── §3 合成代数 ── 古典 E11〜E55 (xcorr / 量子化 / 集約)
│   ├── Task #11   実験因果マップページ (UI 可視化)                            [Merged]
│   ├── Task #12   E47 クエリ×仮想トークン類似度行列 (popcount)                [Merged]
│   ├── Task #15   ensemble7 意味エントロピー評価付き修正ループ                [Merged]
│   ├── Task #19   E42 局所誤りブロック類似度行列 ensemble15                   [Merged]
│   ├── Task #27   ensemble28 R·C テンソル + RC_f² + CS 直交行列              [Merged]
│   ├── Task #31   実験原理の数学的定式化 (補題・定理・証明)                   [Merged]
│   ├── Task #32   E57 ビットハッシュ関数 原理・定理・疑似コード               [Merged]
│   ├── Task #55   E72 遷移レビュー + replit.md 断髪式                         [Merged]
│   └── Task #57   E80 XNOR 巡回スペクトル + NAND 完全性背理法証明             [Merged]
│
├── §5 ユースケース 5.2 (rustc エラー修正) 系譜
│   ├── Task #48   E68 C4.1 予想 (ペナルティ飽和 H=2 天井) 検証                [Merged]
│   ├── Task #49   E68 u4 頻度 oracle ランダムウォーク                         [Merged]
│   ├── Task #50   §7 headScore XOR キャンセル条件 (補題 L7.2 検証)            [Merged]
│   ├── Task #51   E70-XCORR xcorr 誘導 nibble 置換ウォーク                    [Merged]
│   ├── Task #52   E71a GOLDCYCLE 二値 GOLD 系列循環 xcorr                     [Merged]
│   └── Task #83   E84 Object Bit Capsule + Rust コード生成器 (公理 A4 実装)   [Merged]
│       taskRef: task-83.md, e84-object-bit-capsule.md
│       compRef: bitrag-core/src/object_capsule.rs (C12, C13)
│
├── §7 マッピング表 ── データセット / コーパス整備
│   ├── Task #87   rustc tests/ui/ ベンチ用データセット収集                    [Merged]
│   ├── Task #87b  task-87-rustc-ui-dataset.md (拡張)                          [Merged]
│   ├── Task #98   E92 全 400 ファイル E91 派生 (上限精度)                     [Merged]
│   ├── Task #98b  task-98-e92-full-dataset.md                                 [Merged]
│   ├── Task #99   E93 インクリメンタル IDF²/XCORR                             [Merged]
│   └── Task #99b  task-99-e93-incremental-idf-xcorr.md                        [Merged]
│
├── ツーリング / ジョブスケジューラ強化 (周辺タスク)
│   ├── Task #21   チャット履歴圧縮スキル (bitRAG planner 用)                  [Merged]
│   ├── Task #24   タイムアウト段階的エスカレーション bash                     [Merged]
│   ├── (taskRef)  scheduler-binary-tree-clone.md                              [Merged]
│   ├── (taskRef)  scheduler-job-workdir.md                                    [Merged]
│   ├── (taskRef)  scheduler-scope-scatter-gather.md                           [Merged]
│   ├── (taskRef)  qwait-mark-helper.md                                        [Merged]
│   ├── (taskRef)  replit-md-no-english-rule.md / rust-500char-rule.md         [Merged]
│   ├── (taskRef)  rust-compile-result-rule.md                                 [Merged]
│   └── (taskRef)  plotters-fonts-setup.md (experiment-font-smoke 出力)        [Merged]
│
├── §6 軸 B (E91, 黄金循環) ── 1024-bit 表現の拡張
│   └── Task #91 (古番) 環状黄金ビット木 (e91-cyclic-gold-bit-tree.md)         [Merged]
│       expRef: experiment-91
│
└── (代表番号) Task #102  本書 (THEORY_LINEAGE.md) の上流番号                  [Merged]
    最新の本書 (Task #107) もこのカテゴリに連なる
```

> 注: 各ノードの `状態` は `.local/tasks/*.md` の存在と
> `EXPERIMENTS_SUMMARY.md` / `EXPERIMENTS.md` の結果記載状況で判定した。
> 番号 (#N) はプロジェクトタスク連番、`taskRef` は `.local/tasks/` 配下のファイル名。
> Task #2/#3/#4 はテーマ単位の **上位概念ノード** (合成代数 API 公開 / ユースケース実装 /
> §8 問題ピックアップ) として系譜上明示している。
> いずれも `.local/tasks/` 配下に直接対応する taskRef ファイルは存在せず、
> 同テーマに連なる実タスク (Task #5/#48/#83/#103 系等) を束ねる中間層である点に注意。
> Task #5〜#102 の主要派生はおおむね本ツリー内のいずれかのブランチに属する。

---

## 2. 公理別根拠表

`THEORY_RUST_CODEGEN.md` §1 の公理 A1〜A5 ごとに、根拠を分類する。
出典は `EXPERIMENTS_SUMMARY.md` / `EXPERIMENTS.md` /
`experiment-*/README.md` のいずれか。

| 公理 | 内容 | 裏付けた実験 | 修正を迫った実験 | 棄却 | 未検証 |
|------|------|--------------|------------------|------|--------|
| A1 (gram→bit 同型) | gram の bit 単射 | E11〜E18 (bitset Jaccard 同値性), E36 (`type_ledger` 1024-bit gram), E163b の自己検証 (階数=3 で葉ノード F2 依存性を XOR=0 で検出) | 該当なし | 該当なし | V を超える gram 衝突率の実測 (FNV ハッシュ) |
| A2 (IDF² 加重) | μ(S)=Σ idf_sq[i] | E18 (norm+mask, Gini=0.820), E19〜E20 (条件付きエントロピー最小化 ≡ IDF 最大化), E103 (density-rank で rustc 6/6), E163a `G_idf` の σ_1²/Σ ≈ 51.79〜79.62% | E160 (16 膜 + IDF² では Mixture で OR=84.4% に低下) | 該当なし | 大規模コーパス拡張で τ 再調整 (§8 問 9 と紐づく) |
| A3 (SHIFT 近傍 = 語彙的近傍) | shift_k で OOV 救済 | E16〜E17 (M² wave front 250 万倍高速), E21 (`shift_left/right`), E152 (線形オフセット K=8), E155 (対蹠除外 dyadic K=7), E157/E159 (環状木 w スイープ) | E160 (NSEG=16 で OR/AND 低下), E162 (qlen スイープ — XOR は qlen=160 で AND と挙動が逆転) | 該当なし | 多次元 shift (§8 問 4 の `(k₁*, k₂*, ...)` 一般化) |
| A4 (循環 XOR 圧縮 → 1024-bit) | E: bytes → {0,1}^1024 | Task #83 / E84 `ObjectCapsule` (1024-bit 実装), E91 環状黄金ビット木, E163d 熱帯 SVD で AND の argmax 87% (Rust) を 階数=8 で復元 | E163d 法律コーパスは 階数=8 でも 37% — 1024 が「最小」かは未確定 | 該当なし | §8 問 3 (512/2048-bit との情報量比較) — 未着手 |
| A5 (rustc オラクル一価性) | error_count 決定的 | E48 (コンパイラ誘導トークン探索), E55 ensemble29 (51/56 rustc pass), E68/E70/E71 系 (oracle 歩行), E103 (rustc 6/6) | 該当なし (rustc は決定的) | 該当なし | エラーコード分布の網羅率 (E0XXX) |

---

## 3. 合成代数 §3 演算別状態表

`THEORY_RUST_CODEGEN.md` §3 の 10 演算ごとに、実装と実験での性能、現状の課題。

| 演算 | 実装 (構成要素) | 実験での測定 | 課題 |
|------|------------------|--------------|------|
| ∧ AND | `DocBits` 語の bit AND (C03) + `IdfPlanes::pair_jaccard` (C04) | E161 法律 平坦 AND 多数決 14/16 = **87.5%**, Rust 16/16 = 100%, Mixture 16/16 = 100%; AND_tree は 50% (E161) | 一級 API 未公開 (`DocBits::and(&Self)` 等が無い, §8 問 1) |
| ∨ OR | 同上 (bit OR) | E161 平坦 OR 法律 2/16 = **12.5%** (= 1/NSEG 基準値), Rust/Mixture も同等; OR_tree は法律 87.5% に回復 | 平坦 OR の崩壊原因を E163a で σ_1²/Σ ≧ 60% として説明済; 木構造貪欲法が必要 |
| ⊕ XOR | bit XOR | E161 XOR 全コーパス 12.5%; E162 qlen 掃引で qlen=160 で AND との順序逆転を観測 | E162 観察を踏まえた XOR の qlen 依存スコア付けは未実装 |
| ⊖ NOT_AND / NOT_Q | `~Q & S` / `Q & ~S` | E161 NOT_AND = 12.5% (密度のみ), NOT_Q = 100% (≡ 平坦 AND の定数 \|Q\| 補正) | NOT_AND が密度に飽和する問題 — 未解決 (区分密度の正規化が候補) |
| shift_k | `DocBits::shift_left/right` (C03), `xcorr*` (C10) | E152 線形オフセット K=8, E155 対蹠除外 K=7 (法律 AND 100%), E157 環状木 w=2 (法律 OR 93.8%) | shift_k と ContextWindow の独立性 (§8 問 5) は未測定 |
| fiction_α | E48 単体実装のみ | E48 (32-bit 飽和, 探索空間縮小), E49 (256-bit が最良 spread=0.089) | API 未統一 (§8 問 2)。`mask_idf` との合成順序は未定義 |
| mask_idf | C06 `build_mdoc_idf_masked` の閾値処理 | E18 (mask df>21 で avg 1.840, Gini=0.820), E20 (IDF² PPR との等価性) | τ の決定は経験則的; A5 オラクルからの逆算は未実装 (§8 問 6) |
| q_B (IdfPlanes) | C04 `IdfPlanes::build` | E103 (B=8 を density-rank と組み合わせて rustc 6/6) | B の最適値はコーパス依存; 掃引未実施 |
| E (encode 1024-bit) | C13 `ObjectCapsule::encode` (内部 C12 `n4_gram_circ`) | E84 (decode_to_stub で雛形生成), E163d で AND スコアの熱帯因数化 (Rust 87% / 法律 37%) | 1024-bit が「最小」かどうか (§8 問 3) は未検証 |
| d (Hamming) | C13 `xnor_l1` / `xnor_popcount` | E84 (buggy↔gold 距離), E91 (環状黄金ビット木) | カプセル間 d とコーパス jaccard の乖離率は未測定 |

---

## 4. §8 未解決問題の進捗表

`THEORY_RUST_CODEGEN.md` §8 の問い 1〜10 ごとに、関連派生タスクと結論。

| # | 問い | 状態 | 関連 taskRef / expRef | 結論または現状 |
|---|------|------|------------------------|----------------|
| 1 | DocBits 上の `∧/∨/⊕/⊖` を一級 API として公開すべきか | 未着手 | (該当タスクなし; E161 で語列直接アクセスのまま運用) | E161 で 8 演算子をスコア関数として実装したが API 公開はしていない |
| 2 | `fiction_α` の確率 α と挿入 bit の選び方 | 部分的 | task-48 (E48), e49-bitwidth-comparison.md | α=0.66〜0.75 が点火閾値, α=1.00 で 207/210 (実験 11〜12); ビット幅最適は 256bit (E49) |
| 3 | ObjectCapsule の 1024-bit は「最小」か | 部分的 | task-83 (E84), task-91 (E163a), task-94 (E163d) | E163a で σ_1²/Σ ≈ 76〜93% が得られ、E163d で Rust コーパスが 階数=8 で 87% 復元 — 「1024 が下限」とは言えないが 512 比較は未実施 |
| 4 | 位相 k* は真にスカラで十分か | 部分的 | task-51 (E70-XCORR), e91 環状木, E163d | E163d で AND 行列の熱帯因数 V[l,:] が複数のピボットを必要 → 多峰の場合があることを確認 |
| 5 | ContextWindow の N と shift_k の k は独立か | 未着手 | (なし) | 計測実験なし |
| 6 | ErrSig/FixHint の IDF² ブースト倍率 | 未着手 | (現状 §2 の「2 倍」で固定) | A5 オラクルからの逆算は未実装 |
| 7 | §4 標準演算列 `Q = mask_idf_τ((F∨T∨U)∧shift_1(B∨C)) ∨ fiction_α(E⊕H)` は最適か | 部分的 | E103 (順序固定で 6/6), E161 (演算子比較) | 演算子の入れ替え単独実験は無いが、E161 で「∨ 単独は 12.5% に崩壊」を確認 |
| 8 | gold_encode_indices の DocBits 逆写像 | 未着手 | task-52 (E71a-bin), task-91 (E163a) | 位相→語彙の戻り写像は閉じていない (情報損失あり、未測定) |
| 9 | コーパス拡充が IDF 分布に与える影響 | 部分的 | EXPERIMENTS.md の 3 コーパス (法律/Rust/Mixture) 比較 | Mixture で τ 再調整が必要なことが示唆される (E160 Mixture OR=84.4%) |
| 10 | 軸 B 同値類は rustc 等価より粗いか細かいか | 未着手 | (なし) | 実証的に測る実験は未設計 |

---

## 5. 修正・ドリフト記録 (時系列)

提案当時の前提が後の実装・実験で覆ったケースを時系列に並べる。

| 時点 | 事象 | 当初の前提 | 訂正後 | 出典 |
|------|------|------------|--------|------|
| Task #57 / E80 | NAND 完全性証明文書を構成 | NAND は AND/OR/XOR をすべて表現可能 (Sheffer stroke) → 100% 一致を期待 | E161 平坦 NAND = `popcount(~(Q & S))` の argmax で **0/16 = 0%** (反 AND 効果で逆転) | `experiment-161-law/README.md` 多数決表 |
| Task #5 / E154 | ソフト集約 (popcount 加重投票) | 加重投票で AND を上回る期待 | 法律で AND=100% / ソフト=93.8% — ソフトはわずかに劣化 | `EXPERIMENTS.md` 法律行 |
| E155b 計画 | 非対蹠 dyadic (sw/3, 2sw/5) で K=8 を維持しつつ AND=100% を狙う | sw/2 を sw/3 に置換するだけで E152 の AND=100% を再現可能 | **未実行**: `results/` ディレクトリが存在せず、ハイブリッドオフセット (e155-hybrid-offsets.md) も Drafts のまま | `EXPERIMENTS.md` E155b 行 |
| E160 (NSEG=16) | NSEG 倍増で精度向上を期待 | NSEG 拡張は dyadic アンサンブルを密にする | 全コーパスで OR/AND 低下、特に Mixture で OR=84.4% (E152 系の 100% から下落) | `EXPERIMENTS.md` E160 行 |
| E158 (1000 クエリ) | E155 (16 クエリ) の OR=AND=100% は 1000 クエリでも同水準 | 統計的に拡張可能 | OR=95.9%, AND=97.5% に低下 — 16 クエリ評価は楽観的だった | `experiment-158-github-rust/results/result.txt` |
| E157/E159 環状木 | 環状木は AND/OR の両方で改善 | 木構造貪欲法の汎用性 | 法律 OR は 93.8% に改善するが AND は 81〜87.5% に低下 (AND/OR の交換関係が反転) | `EXPERIMENTS.md` E157/E159 行 |
| Task #87 (rustc UI データセット) | rustc tests/ui の収集で網羅率向上 | E0XXX を網羅 | 一部の E コード網羅率は保てたが、雛形退化を解消する E104 仮説は未着手 | `EXPERIMENTS_SUMMARY.md` E103 「次の実験」節 |
| E163a 仮説検証 | OR/XOR/NOR の 12.5% 崩壊は未説明 | 単純に「演算子が悪い」 | σ_1²/Σ ≧ 60% の支配により内積系スコアが第一固有方向 (popcount 比例) に収束 — 6 ケース中 5 で支持 | `experiment-163a-int-power-iter/README.md` |

---

## 6. 比較節 (テーマ別採用表)

### 6.1 dyadic アンサンブル集約変種 (E152〜E160)

| テーマ | 構成 | 法律 OR/AND | Rust OR/AND | Mixture OR/AND | 採用 | 一行所見 |
|--------|------|-------------|-------------|----------------|------|----------|
| 線形オフセット | E152 K=8 | 87.5/100 | 100/100 | 100/100 | (基準値) | 二進対 (dyadic) 系の比較対象 |
| 二進対 | E153 K=8 | 87.5/93.8 | 100/100 | 100/100 | — | sw/2 対蹠点が AND を 93.8% に下げる |
| 環状二進対 | E154 K=8 | 87.5/93.8 | 100/100 | 100/100 | — | E153 と同等の挙動 |
| **対蹠除外二進対** | **E155 K=7** | **87.5/100** | **100/100** | **100/100** | **★** | sw/2 除外で AND=100% 回復, 3 コーパスで安定最良 |
| 環状木 w=2 | E157 | 93.8/81.2 | 100/87.5 | 100/100 | (OR 側で改善) | 法律 OR を 93.8% に押し上げるが AND が下がる |
| 環状木 w 掃引 | E159 | 93.8/87.5 | 100/87.5 | 100/93.8 | — | E157 を w∈{2,3,4} に拡張 |
| 16 膜 | E160 NSEG=16 K=7 | 90.6/93.8 | 93.8/100 | 84.4/90.6 | — | NSEG 倍増で OR/AND とも低下傾向 |

### 6.2 E161 全論理演算子比較 (10 種, 法律コーパス, K=7)

| 演算子 | 多数決 | % | 同値類 | 一行所見 |
|--------|--------|---|--------|----------|
| AND | 14/16 | 87.5% | 平坦 AND | 信号演算子。AND_tree より平坦走査が強い |
| **NOT_Q** | **16/16** | **100.0%** | ≡ 平坦 AND (定数 \|Q\| 補正) | ★ 形式上 AND と同型 |
| OR_tree | 14/16 | 87.5% | 木構造貪欲法 | 法律 OR は木構造でしか機能しない |
| AND_tree | 8/16 | 50.0% | 木構造貪欲法 | 木構造の貪欲降下が局所最適に陥る (3 コーパス共通) |
| OR | 2/16 | 12.5% | OR/NOR 同型 | 基準値 (1/NSEG) に崩壊 |
| NOR | 2/16 | 12.5% | OR/NOR 同型 | OR と同値類 |
| XOR | 2/16 | 12.5% | XOR/XNOR 同型 | qlen=40 では識別信号無し |
| XNOR | 2/16 | 12.5% | XOR/XNOR 同型 | XOR と補数関係 |
| NOT_AND | 2/16 | 12.5% | 密度依存型 | 区分密度に誘引される |
| NAND | 0/16 | 0.0% | 反 AND | 当初の Sheffer 完全性主張とは独立に 0 へ崩壊 |

> **採用**: AND / NOT_Q (= AND の定数補正版)。NOT_Q が 100% で AND を超えるのは、
> NSEG=8 で全クエリの \|Q\| が一定のため定数項が argmin の順序に影響しないため。

### 6.3 E163 系 SVD (4 種, 各コーパスで グラム / k 掃引)

| 実験 | 半環 | 出力指標 | 法律 | Rust | Mixture | 一行所見 |
|------|------|----------|------|------|---------|----------|
| E163a | (整数, 加算) べき乗反復 | σ_1²/Σ | G_doc 93.48% / G_idf 76.56% | 85.86% / 51.79% | 93.44% / 79.62% | **★ E161 12.5% 崩壊を σ_1 支配で説明** (6 ケース中 5 で支持) |
| E163b | GF(2) (XOR, AND) | 階数, 独立基底 | 階数出力 + 列対角化検証 | 同左 | 同左 | アトミック命令独立基底; F2 線形従属の例 ≥ 4 件 |
| E163c | (OR, AND) ブール代数 | 被覆 / 再現 | k=16: 81.80% / 100% | k=16: **92.11%** / 100% | k=16: 79.97% / 100% | **★ Rust 92.11%** で主題抽出が最良 |
| E163d | (max, +) 熱帯代数 | 上位一致 / Kendall τ‰ | k=8: 37% / +615 | k=8: **87%** / +798 | k=8: 50% / +580 | **★ Rust** で AND 構造を低階数で再現 |

> **採用**: 用途ごとに異なる。説明力 = E163a, 独立基底 = E163b,
> 解釈可能な主題抽出 = E163c, AND スコアの代数化 = E163d。

### 6.4 ジョブスケジューラ強化 (周辺タスク)

| テーマ | 関連 taskRef | 効果 | 一行所見 |
|--------|--------------|------|----------|
| qrun ジョブ管理 | scheduler-binary-tree-clone.md, scheduler-job-workdir.md, scheduler-scope-scatter-gather.md, qwait-mark-helper.md | ブロック単位の完了標識による同期 | scheduler.serial / scheduler.sock 経由で実行 |
| 自動静的検査監視 | scheduler/lint-poll.sh, ci-lint-poll.sh, lint-poll-report.sh | 起動時自動静的検査, ジョブ完了通知 | 既存の改善タスク (通知有効化 / 無効化容易化) と連結 |
| チャット履歴圧縮 | task-21 (チャット履歴圧縮スキル) | bitRAG 計画者の文脈肥大化を防止 | replit.md 専用節へ書き込み |
| 段階的タイムアウト | task-24 (タイムアウト段階的エスカレーション) | 2→4→8→…→120 秒の自動再試行 | timeout-escalate スキル化済み |

---

## 7. 分類体系

派生タスクを 6 カテゴリに分類した。
件数は本書執筆時点で `.local/tasks/*.md` に存在するタスクファイル数で数える
(同一 taskRef が複数の expRef を生んでいる場合は 1 件としてカウント)。

> 計数規則: 1 行に列挙される 1 件の `taskRef`（または同等の expRef）を 1 件としてカウントする。
> `task-87 / task-87-rustc-ui-dataset` のように 1 行内に複数の派生 ID を併記したものは 1 件として扱う。

### (i) 公理検証 — 計 6 件
公理 A1〜A5 を直接または間接に検証するタスク。

- task-91 (E163a 整数べき乗反復) … A2/A4 検証
- task-92 (E163b F2-SVD) … A1 検証
- task-93 (E163c ブール代数 SVD) … A1/A2 検証
- task-94 (E163d 熱帯 SVD) … A4 検証
- task-57 (E80 NAND 完全性背理法証明) … A4 周辺
- task-83 (E84 ObjectCapsule) … A4 実装

### (ii) 合成代数拡張 — 計 7 件
新演算 (∧/∨/⊕/⊖/shift_k/fiction_α/mask_idf/q_B/E/d) の追加または変種。

- task-5 (E154 ソフト集約)
- e155-antipodal-removed (E155 K=7)
- e155b-non-antipodal (E155b sw/3)
- e155-hybrid-offsets (線形+二進対)
- e156-correlation-matrix / e156-failure-anatomy
- e157-cyclic-tree / e157-k-scaling
- e158-compression-benchmark-files

### (iii) ユースケース実装 — 計 6 件
§5 の 3 ユースケース (新規生成 / rustc 修正 / E103 拡張) の具体化。

- experiment-103-rust-io-bitrag.md (E103 本体)
- e103-0000-quotient-manifold-embedding.md (E103-0000)
- e103-pre-exception-analysis.md
- task-83 (E84 ObjectCapsule + Rust 雛形生成)
- task-87 / task-87-rustc-ui-dataset (rustc tests/ui データセット)
- ensemble29-mutation-types.md (E55, 5 種変異)

### (iv) 演算子比較 — 計 13 件
複数の演算子・パラメータを横並びで比較するタスク。

- e161-all-operators (E161 10 演算子)
- experiment-162-{ngram,qlen}-sweep (E162 XOR 対 AND)
- e49-bitwidth-comparison (E49 64〜1024-bit)
- e67〜e74 (u4 gate1 × all-gate2 系列): e67-u4-gate1-and-all-gate2.md, e68-u4-gate1-or-all-gate2.md, e69-u4-gate1-xor-all-gate2.md, e70-u4-gate1-nor-all-gate2.md, e71-u4-gate1-nand-all-gate2.md, e72-u4-gate1-and-not-all-gate2.md, e73-u4-gate1-not-and-all-gate2.md, e74-u4-gate1-xnor-all-gate2.md

### (v) スケジューラ / ツーリング — 計 8 件
qrun, ジョブ計画, 自動静的検査, 圧縮スキル, タイムアウト, タグ規則など。

- scheduler-binary-tree-clone.md
- scheduler-job-workdir.md
- scheduler-scope-scatter-gather.md
- qwait-mark-helper.md
- escalating-timeout-bash.md / task-24 (timeout-escalate)
- bitrag-planning-context-compressor.md / chat-context-compress-skill.md / task-21
- replit-md-no-english-rule.md / rust-500char-rule.md / rust-compile-result-rule.md
- plotters-fonts-setup.md / experiment-font-smoke

### (vi) UI / 可視化 — 計 6 件

- task-11 (実験因果マップページ ExperimentMapPage.tsx)
- experiment-causal-map-page.md
- hypothesis-dependency-graph.md (HypothesisGraphPage.tsx)
- src/pages/E161Page.tsx / E34Page.tsx / E57Page.tsx / RustCodePage.tsx / BitRagPage.tsx
- write-experiment-summary.md (EXPERIMENTS.md 自動生成)
- math-foundations.md (MATH_FOUNDATIONS.md 整備)

---

## 8. 整合性チェック

> パス基準: 本節以下に記載するパスは特記なき限り、リポジトリルート (本ファイルからは
> `../../`) を起点とする相対パスである。`artifacts/bitrag/` 配下の参照は本ファイル
> (`artifacts/bitrag/THEORY_LINEAGE.md`) と同階層を指す。

本書中の参照が実在することを確認する箇所:

- `THEORY_RUST_CODEGEN.md`, `EXPERIMENTS_SUMMARY.md`, `EXPERIMENTS.md`, `COMPONENT_CATALOG.md`: いずれも `artifacts/bitrag/` 直下に存在
- `experiment-152〜160` (3 コーパス各々), `experiment-161-{law,github-rust,github-mixture}`, `experiment-162-{ngram,qlen}-sweep`, `experiment-163a/b/c/d`: いずれも `artifacts/bitrag/` 配下に存在
- 古典実験 `experiment-11〜93`, `experiment-103/103-0000`, `experiment-104〜151`: 大半が存在 (E156 のみ法律のみ, E158 は Rust のみ — `EXPERIMENTS.md` に明記)
- `bitrag-core/src/{bitset,idf,ngram,matrix,object_capsule,gold_cycle,nibble_hash}.rs`: `COMPONENT_CATALOG.md` C01〜C13 の参照と一致
- scheduler 配下 (`scheduler/{env.sh,start.sh,lint-poll.sh,...}`): 存在
- UI ページ (`src/pages/{BitRagPage,E161Page,ExperimentMapPage,HypothesisGraphPage,RustCodePage,E34Page,E57Page}.tsx`): 存在

---

## 付録: タスク → 派生実験対応表 (要約)

| taskRef | 派生 expRef (代表) | カテゴリ |
|---------|-------------------|----------|
| bitrag-rust-codegen-semantics-theory.md | (THEORY_RUST_CODEGEN.md) | 根 |
| task-5 / e154-soft-aggregation.md | experiment-154-* | (ii) 合成代数 |
| e155-antipodal-removed.md | experiment-155-* | (ii) |
| e155b-non-antipodal.md | experiment-155b-* (未実行) | (ii) |
| e156-correlation-matrix.md | experiment-156-law | (ii) |
| e157-cyclic-tree.md | experiment-157-* | (ii) |
| e158-compression-benchmark-files.md | experiment-158-github-rust | (ii) |
| e161-all-operators.md | experiment-161-* | (iv) 演算子比較 |
| (e162-*) | experiment-162-{ngram,qlen}-sweep | (iv) |
| task-91 / e163a-int-power-iter.md | experiment-163a-int-power-iter | (i) 公理検証 |
| task-92 / e163b-f2-svd.md | experiment-163b-f2-svd | (i) |
| task-93 / e163c-boolean-svd.md | experiment-163c-boolean-svd | (i) |
| task-94 / e163d-tropical-svd.md | experiment-163d-tropical-svd | (i) |
| task-83 / e84-object-bit-capsule.md | experiment-84 | (iii) ユースケース / (i) A4 実装 |
| experiment-103-rust-io-bitrag.md | experiment-103, experiment-103-0000 | (iii) |
| task-11 | src/pages/ExperimentMapPage.tsx | (vi) UI |
| task-21 | .agents/skills/chat-context-compress | (v) ツーリング |
| task-24 | .agents/skills/timeout-escalate | (v) |

---

## 9. 要件 1〜7 自己監査チェックリスト

本書がタスク #107 の Done 条件 (要件 1〜7) を満たしていることを最後に確認する。

| # | 要件 | 対応箇所 | 充足 |
|---|------|----------|------|
| 1 | タスク系譜ツリー (#1 を根, #2/#3/#4 含む, #5〜#102 の主要派生) | §1 | ✓ Task #2/#3/#4 を明示ノード化, #5/#11/#12/#15/#19/#21/#24/#27/#31/#32/#48/#49/#50/#51/#52/#55/#57/#83/#87/#91〜#94/#98/#99/#102 を含むブランチ |
| 2 | 公理 A1〜A5 別根拠表 (裏付け / 修正 / 棄却 / 未検証) | §2 | ✓ 5 公理 × 4 列の表 |
| 3 | §3 合成代数 10 演算別状態表 (実装 / 性能 / 課題) | §3 | ✓ ∧/∨/⊕/⊖/shift_k/fiction_α/mask_idf/q_B/E/d の 10 行 |
| 4 | §8 未解決問題 1〜10 の進捗表 (解決 / 部分 / 未着手 + taskRef) | §4 | ✓ 10 行 + taskRef 列 |
| 5 | 修正 / ドリフト記録 (時系列, NAND 100%→0% を含む) | §5 | ✓ 8 件を時系列で列挙 (NAND 例を含む) |
| 6 | 比較節 (E152〜E160 集約 / E161 演算子 / E163 系 SVD) | §6.1〜§6.3 | ✓ 3 表 + 採用と一行所見 |
| 7 | 分類 (i) 公理検証 / (ii) 合成代数拡張 / (iii) ユースケース実装 / (iv) 演算子比較 / (v) スケジューラ・ツーリング / (vi) UI・可視化 | §7 | ✓ 6 カテゴリ, 各々件数とリンク |

追加の規約遵守:
- 文書は Markdown のみ。コード変更・新規実験・新規依存関係なし。
- 数値はすべて既存の `EXPERIMENTS_SUMMARY.md` / `EXPERIMENTS.md` /
  `experiment-*/README.md` から引用 (新規測定なし)。
- 本文は日本語で記述。英字は識別子 (ファイル名・関数名・変数名・タスク連番・公理名・実験番号・状態ラベル `Drafts`/`Active`/`Merged` 等) と数式記号に限定した。
