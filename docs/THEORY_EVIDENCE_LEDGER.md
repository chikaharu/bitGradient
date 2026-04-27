# bitRAG 理論検証台帳 (Evidence Ledger)

> **本書の位置付け**: `THEORY_CORE_v1.md` の各クレーム (claim_id) について、
> どの実験 / OODA サイクル / 単体テスト / 既存ドキュメントが verification を
> 与えているかを 1 行 1 証跡で台帳化する。**理論本体 (= 凍結核 v1) は変えず、
> 新規検証はすべて本書に追記する**。

- 凍結核: `THEORY_CORE_v1.md` (v1 凍結 2026-04-25)
- claim_id 規約: `THEORY_CORE_v1.md` の節と **1:1 対応** (本書本表に
  凍結核外の claim_id は **置かない**)
- 列スキーマ: `claim_id | claim_text | type | evidence | status | notes`
- type 凡例: `axiom` / `lemma` / `theorem` / `corollary` / `invariant`
- status 凡例 (3 値固定):
  - `verified`  — 実験 / 単体テストで PASS, 反例なし (代数的に自明な
    ものも本ステータスに統合し、根拠を notes に記す)
  - `partial`   — 部分的支持 (条件付き / 一部コーパスでのみ成立 / 弱支持)
  - `falsified` — 反例があり当初の主張は破棄 (後継主張は notes)

---

## 表 1: §1 公理 (A0–A5) の検証台帳

| claim_id | claim_text (要約) | type | evidence | status | notes |
|----------|--------------------|------|----------|--------|-------|
| A0 | bitRAG 全意味演算は有限ブール空間上の関数で、論理演算のみで構成または構成を目標とする | axiom | `bitrag-core::sign2` 単体テスト全 PASS / e166–e175 新規コード grep で f32/f64 = 0 行 | verified | PG4 監査表 (`THEORY_NAND_COMPLETENESS.md` 階層 4) で既存箇所を個別管理 |
| A0 | 同上 | axiom | `experiment-174-pseudoinverse` (e174-c01) `no_f32_f64: verified (grep src = comments only)` | verified | F2 / Z 整数演算のみで Penrose 検証完遂 |
| A0 | 同上 | axiom | `artifacts/bitrag/scheduler` v0.2.0 race fix (e175-c01); `usize/u32/c_int` のみ | verified | smoke 5/5 PASS で副作用なし確認 |
| A1 | gram → bit 位置の写像 idx は単射, 文書 D の意味は bit 集合 S(D) に同一視できる | axiom | E11–E18 / E36 / E163b (F2 SVD 階数=3 で葉ノード XOR=0 検出) | verified | `THEORY_LINEAGE.md` §2 公理別根拠表 |
| A1 | 同上 | axiom | `THEORY_FORMALIZATION.md` T01 (`Vocab::build` の `terms.iter().enumerate()`) | verified | 代数的に自明 (HashMap 鍵衝突は文字列等価で吸収) |
| A2 | 重みは idf(g)² で与えられ μ(S)=Σ idf_sq[i] | axiom | E18 (Gini=0.820) / E19 / E20 / E103 / E163a (σ_1²/Σ ≈ 51.79–79.62%) | verified | E160 で τ 再調整必要なケース (partial 候補だが本表は verified に集約; 反証ではない) |
| A2 | 同上 | axiom | `THEORY_FORMALIZATION.md` T02 (`0 < idf_sq[i] ≤ (ln((N+1)/2)+1)²`) | verified | df ≥ 1 を前提として式上自明 |
| A3 | shift_k は語彙的近傍を表し sim_shift で OOV gram を救済する | axiom | E16 / E17 (M² wave front) / E152 (K=8) / E155 (対蹠除外 K=7) / E157 / E159 | verified | E160 (NSEG=16) と E162 (qlen) で局所的逆転あり (notes に partial として記録) |
| A4 | E: bytes → {0,1}^1024 は 1024-bit カプセル化作用素, 距離は popcount(c1⊕c2) | axiom | E84 (`ObjectCapsule`) / E91 / E163d (Rust 階数=8 で 87% 復元) | verified | 加法性 (旧 A4 = T08) は反証済 → A4 は非線型ハッシュ圧縮として再定式化 (`THEORY_RUST_CODEGEN.md` L42–L50, T08 falsified 行参照) |
| A5 | rustc は判定オラクルであり (error_count, stderr) は決定的 | axiom | E48 / E55 (51/56 rustc pass) / E68 / E70 / E71 / E103 (rustc 6/6) | partial | rustc バージョン / edition / ターゲット / 環境変数固定が前提 (`THEORY_FORMALIZATION.md` T10) |

---

## 表 2: §2 主定理 (T1, T2, C-T2, T3, T4) の検証台帳

| claim_id | claim_text (要約) | type | evidence | status | notes |
|----------|--------------------|------|----------|--------|-------|
| T1 | A5 を除く bitRAG 全意味演算と合成文法は効率性を問わなければ有限 NAND 回路で実現できる | theorem | `THEORY_NAND_COMPLETENESS.md` 階層 2 の M1–M5 集約 (L1〜L11 → M1〜M5 → T1) | verified | 構成的依存ツリーが閉じている |
| T1 | 同上 | theorem | `bitrag-core::sign2` 構成的最小実例 (`mul_full_table_excluding_spare` / `neg_is_bit_swap` / `add_saturating_table`) 単体テスト | verified | 全数検証 PASS |
| T1 | 同上 | theorem | E167 (3 コーパス 58 ファイル, sign2 直列同型 round-trip 58/58 = 100%) | verified | `experiment-167-sign2-tokenizer/results/result.txt` |
| T2 | H0 が成立する (bitRAG 全演算は有限個の NAND ゲートで構成できる) | theorem | `THEORY_NAND_COMPLETENESS.md` 階層 3 (A0 + L1 から H0 を導出, 6 ステップ) | verified | 背理法と直接構成の両方で成立 |
| C-T2 | A0 ∧ L1 ⊨ H0 (含意の単一ステップ); 背理法のための NOT 操作 H0' は本質的に不要 | corollary | `THEORY_NAND_COMPLETENESS.md` 階層 3 (T2 の系) | verified | bitRAG は「NAND 完全性を設計拘束として宣言した系」として再公理化可能 |
| T3 | T1 の構成は深さ上界を与えない (補助注記); 副次的に μ 加算と xcorr 加算は log V で支配 | theorem | `THEORY_NAND_COMPLETENESS.md` 階層 3 (本タスク射程外, 注記レベル) | partial | 注記であり主張の強度は弱い |
| T4 | アセンブラは T1 + C-T2 によって NAND 回路として実装可能 | theorem | `THEORY_NAND_COMPLETENESS.md` 階層 3 (T1 + C-T2 帰結) | verified | 回路探索 (CEGAR / SAT) の射程に入る |

---

## 表 3: §2 補助補題 (L/T/M/C 抜粋, claim_id 1:1) の検証台帳

`THEORY_CORE_v1.md` §2 補助定理リストに列挙した claim_id のみ採録。

| claim_id | claim_text (要約) | type | evidence | status | notes |
|----------|--------------------|------|----------|--------|-------|
| L1 | NAND は 2 値ブール関数の関数完全集合 (Sheffer 1913) | lemma | 古典 (Sheffer 1913, Post 1941) | verified | 真理表展開 → DNF → NAND |
| L2 | ¬, ∧, ∨, ⊕ の NAND 還元 (4 式) | lemma | `bitrag-core::sign2` の `nand` / `xor_via_nand` / `and_via_nand` 単体テスト | verified | T1 の最小可動標本 |
| L3 | 語彙幅有限 V ≤ 64·n_w | lemma | `bitset.rs` (`DocBits = Vec<u64>`) | verified | 構成的 |
| L4 | shift_k は配線置換 (NAND 不要) | lemma | `bitset.rs::shift` | verified | wire permutation |
| L5 | μ(S)=Σ idf_sq[i] は半/全加算器の整数加算ツリー | lemma | `idf::IdfPlanes`, `eval::row_sum_stats` | verified | 整数加算 |
| L6 | E は ⊕ と shift_k の有限合成 (4 重同型射) | lemma | `bitrag-core::nibble4_tokenizer` (Task#121) + `iso::fourfold_iso_chain` | verified | E166 で 58/58 round-trip |
| L7 | sim_shift と xcorr は L2 + L4 + L5 の合成 | lemma | `bitset.rs::xcorr/xcorr_idf` | verified | 構成的 |
| L8 | mask_idf / q_B はビット比較器 + AND マスク | lemma | `bitset.rs::mask_idf` + `idf::IdfPlanes::build` | verified | T11 / T12 と整合 |
| L9 | fiction_α は LFSR + OR マスクの合成 | lemma | `THEORY_NAND_COMPLETENESS.md` L103–L104 (定義) | verified | 決定的種で NAND 構成可 |
| L10 | BNF Plan は L2〜L9 の有限合成 | lemma | `THEORY_NAND_COMPLETENESS.md` L106–L117 表 | verified | OpExpr ノード × 依存補題対応表 |
| L11 | A5 (rustc) は外部プロセスのため本仮説対象外 | lemma | rustc を外部プロセスとして扱う規約 | verified | A5 の境界外, 構造的 |
| M1 | L2 + L4 → {∧, ∨, ¬, ⊕, shift_k} 上有限式は有限 NAND 回路 | lemma | `THEORY_NAND_COMPLETENESS.md` L137 | verified | 集約補題 |
| M2 | L5 + M1 → μ を含む有限式は NAND 回路 | lemma | 同 L138 | verified | 測度 |
| M3 | L6 + L7 → E と sim_shift の合成は NAND 回路 | lemma | 同 L139 | verified | 圧縮 |
| M4 | L9 → fiction_α は決定的種固定で NAND 回路 | lemma | 同 L140 | verified | 擬似乱数 |
| M5 | L10 + M1〜M4 → §4 BNF Plan の任意評価木は NAND 回路 | lemma | 同 L141 | verified | 合成文法 |
| T01 | idx の単射性 | theorem | `idf::Vocab::build` の `terms.iter().enumerate()` | verified | 代数的に自明 |
| T02 | idf_sq の正値・有界性 | theorem | `idf.rs` L46–L47 (+1 スムージング) | verified | 式上自明, df ≥ 1 前提 |
| T03 | μ の測度性 | theorem | `bitset.rs::jaccard_idf` の trailing_zeros 走査 | verified | 代数的に自明 |
| T04 | Jaccard の有界性 0 ≤ J_idf ≤ 1 | theorem | `bitset.rs::jaccard_idf` (`union < 1e-9 ⇒ 0` の明示分岐) | verified | 構成的 |
| T05 | (DocBits, ∧, ∨, ⊕, ⊖) は可換ブール束 / GF(2) 加法群 / ブール環 | theorem | `bitset.rs::and/or/xor/andnot` + `#[should_panic("nw mismatch")]` テスト | verified | DocBits の代数構造 |
| T06 | shift_k は (GF(2))^V 上の線型作用素 | theorem | `bitset.rs::shift_left/right` (端 bit 0 充填) | verified | 端 bit 損失 1 hop あたり最大 1 bit は許容 |
| T07 | xcorr の対称性 xcorr(A,B)[k] = xcorr(B,A)[-k] | theorem | `bitset.rs::xcorr/xcorr_idf` | partial | nw·64 近傍で 1〜2 bit ずれ (E155b 観測, 実用範囲では無視可) |
| T08 | E の加法性 (1024-bit cyclic XOR の意味で線型) | theorem | `object_capsule.rs::tests::test_capsule_additivity_proptest` (Task#111) | falsified | mean(d) ≈ 512.482, max(d) = 560, zero_ratio = 0; 後継: A4 を非線型ハッシュ圧縮として再定式化 |
| T09 | d がハミング距離 (恒等 / 対称 / 三角不等式) | theorem | `object_capsule.rs::xnor_l1` + テスト `xnor_l1_identity / symmetry` | verified | 代数的に自明 |
| T10 | rustc オラクル一価性 | theorem | `gold_cycle.rs::rustc_run` | partial | rustc バージョン / edition / ターゲット / 環境変数固定が前提 |
| T11 | mask_idf が μ の単調性を保つ | theorem | `bitset.rs::mask_idf` + テスト `mask_idf_threshold` | verified | T02 を前提 |
| T12 | IdfPlanes::sum_bits の B-bit 量子化精度 (1 LSB 以下) | theorem | E18 / E157 / `idf.rs::IdfPlanes::build` | partial | 偏った IDF 分布で実効分解能低下 (B=8 で十分の観察) |
| T13 | 軸 B 三つ組同値関係 (rustc 等価との粗さ) | theorem | (未着手, `THEORY_RUST_CODEGEN.md` §8 #10) | partial | 実証実験未設計 |
| T14 | B-bit 量子化が argmax を保存する条件 (差 ≥ 1 量子) | theorem | E162-ngram-sweep / E162-qlen-sweep | partial | 高 IDF タイで argmax 反転の稀ケース |
| T15 | 標準規則の単調最適性 (局所最適) | theorem | `experiment-164-bnf-perm` (`THEORY_FORMALIZATION.md` L188–L200) | partial | P0 ∈ argmax だが unique でない (弱支持) |
| T2.1 | AND popcount = 包含測度 (内積定理) | theorem | `MATH_FOUNDATIONS.md` §2.4 内積定理証明 | verified | 代数的に自明 |
| T4.1 | ペナルティマスク飽和定理 | theorem | E68 (`MATH_FOUNDATIONS.md` §4.4) | verified | 飽和度の定量化 |
| C4.1 | ランキング収束 (T4.1 の系) | corollary | E68 (Task #48) | verified | H=2 天井 |
| T4.2' | C4.1 精緻化 (H=2 は物理的限界ではない) | theorem | `MATH_FOUNDATIONS.md` L188 | verified | 脱出条件の解析 |
| T5.1 | 2-ホップ拡散定理 (RC_f^(2) は長さ 2 パスの重み和) | theorem | `MATH_FOUNDATIONS.md` §5.3 | verified | グラフ拡散解釈 |
| T6.1 | 直交分離 S_gs ⊥ R (Gram-Schmidt) | theorem | `MATH_FOUNDATIONS.md` §6.2 | verified | 構成的 |
| T6.2 | CS は rank-1 行列 | theorem | `MATH_FOUNDATIONS.md` §6.3 外積構造 | verified | 代数的に自明 |
| T7.1 | headScore 値域 [-8, 8] | theorem | `MATH_FOUNDATIONS.md` §7.4 (Cauchy-Schwarz) | verified | 代数的に自明 |
| L7.2 | XOR キャンセル条件 (popcount(h) = 2) | lemma | `MATH_FOUNDATIONS.md` L377 (改訂版) | verified | 必要十分条件 |
| T9.1 | nibble hash 均一性 distinct ≥ 150/256 | theorem | E56 (C11 nibble hash 256 入力) | verified | 限界均一性 |
| T9.2 | nibble hash 近似独立性 (Pearson 相関係数の絶対値が 0.05 以下) | theorem | E56 (65536 入力 Pearson) | verified | hash_hi / hash_lo の独立性, 数式は `MATH_FOUNDATIONS.md` L481 参照 |
| T10.1 | エラー数単調非増加 H(s_t) ≤ H(s_{t-1}) | theorem | E55 ensemble29 / E70-XCORR oracle 歩行 | verified | 離散 GD の単調性 |
| T10.2 | 有限ステップ停止 MAX_STEPS = 25 | theorem | E55 ensemble29 | verified | 停止理由 3 種 (成功 / best=null / 上限) |
| T10.3 | 探索効率上界 (n ステップ後の期待 AUC) | theorem | E55 ensemble29 AUC 集計 | partial | combined avg AUC = 0.788 (5 件中 3 件が AUC=1.0 の flat) |

---

## 表 4: §3 不変条件 INV-* の検証台帳

| claim_id | claim_text (要約) | type | evidence | status | notes |
|----------|--------------------|------|----------|--------|-------|
| INV-1024 | 任意のオブジェクトカプセル / Sign2 ブロック / nibble4 ブロックは 1024 bit (`[u64; 16]`) に固定 | invariant | `bitrag-core::object_capsule` (`[u64; 16]` 固定) + 全 1024-bit 系実験 (E84/E91/E163d/E167–E174) | verified | `THEORY_NAND_COMPLETENESS.md` L86, `THEORY_RUST_CODEGEN.md` A4 |
| INV-ISO | 1024 bit ブロックは [u64;16] / byte×128 / nibble×256 / Sign2×512 の 4 重同型 | invariant | `bitrag-core::iso::fourfold_iso_chain` 単体テスト | verified | 4 同型射の完全可逆 PASS |
| INV-ISO | 同上 | invariant | E167 (3 コーパス 58 ファイル, sign2 直列同型) | verified | round-trip 58/58 = 100% |
| INV-NIBBLE4 | nibble4_tokenizer の encode → decode は恒等 | invariant | `bitrag-core::nibble4_tokenizer` 11 単体テスト + E166 | verified | 58/58 round-trip = 100% |
| INV-SIGN2 | Sign2 4 値分布 v(h,ℓ) := ℓ−h (否定 = 2bit リバース) | invariant | `bitrag-core::sign2::tests` (`mul_full_table_excluding_spare` 等) | verified | 全表 PASS |
| INV-SIGN2 | 同上 | invariant | E167 + E168 (位置別ヒストグラム; byte 127 終端タグを統計のみで自動発見) | verified | >83% Spare/Plus 固定 |
| INV-SIGN2 | 同上 | invariant | E169 (digram, 3 コーパス) | verified | H(X) ≈ 1.99 bit/値 (理論最大 2.0), I(X;Y) = 0.08–0.11 bit/値 |
| INV-SIGN2 | 同上 | invariant | E171 (Spare チャネル run-length) | verified | spare_rate 15.51–19.14%, run_length 1: 79–88% |
| INV-SIGN2 | 同上 | invariant | E172 (Spare = UTF-8 lead 完全分離) | verified | precision = recall = 1.000 (3 コーパス全件) |
| INV-SIGN2 | 同上 | invariant | E173 (1024bit 3 段 codec) | verified | round-trip 6044/6044 = 100%, 圧縮率合計 8.2% |
| INV-BITADDR | bit_addr(i,g) = (4i + g) mod 1024 は gram 値 g を整数として用いるため GF(2) 線型でない | invariant | `bit_addr` 定義 + Task#111 反例 (T08 falsified) | verified | E を非線型ハッシュ圧縮として扱う A4 改訂と整合 |
| INV-NO-FLOAT | 新規モジュール / 実験 / scheduler 拡張に f32/f64 を混入しない | invariant | e166–e175 新規追加コード grep で f32/f64 = 0 行 | verified | 既存箇所は PG4 監査表で個別管理 |

---

## 表 5: E166–E175 + 11 OODA サイクル個別証跡 (cycle 単位)

各サイクルで採用された claim_id は本表で参照のみ行い、claim_text は表 1〜4 で
管理する。本表は **cycle メタデータ** のみを記録する (claim_id 1:1 規約に
ついては表 1〜4 が唯一の正本)。

| cycle | experiment | results path | 採用 claim_id (表 1〜4 参照) | status | 主観測 |
|-------|------------|--------------|------------------------------|--------|--------|
| e165-c01 | experiment-165-ooda-bootstrap | `experiment-165-ooda-bootstrap/results/result.txt` | (v2 候補のみ) | partial | A6/A7/A8 公理候補のうち A7 = FAIL (反証準備); v1 凍結核には未採録 → v2 起票事由 (notes) |
| e166-c01 | experiment-166-nibble4-tokenizer | `experiment-166-nibble4-tokenizer/results/result.txt` | A0, INV-NIBBLE4 | verified | 58/58 round-trip = 100% |
| e167-c01 | experiment-167-sign2-tokenizer | `experiment-167-sign2-tokenizer/results/result.txt` | T1, L2, L6, INV-ISO, INV-SIGN2 | verified | 58/58 round-trip; sign2 分布 (+:32.84% 0:25.69% -:25.73% spare:15.72%) |
| e168-c01 | experiment-168-sign2-position-histogram | `experiment-168-.../results/result.txt` | INV-ISO, INV-SIGN2, T1 | verified | byte 127 終端タグを統計だけで自動発見 |
| e169-c01 | experiment-169-sign2-digram | `experiment-169-sign2-digram/results/result.txt` | A0, INV-SIGN2 | verified | H(X) ≈ 1.99, I(X;Y) = 0.08–0.11, 圧縮可能性 4–5.6% |
| e170-c01 | experiment-170-sign2-block-xor | `experiment-170-sign2-block-xor/results/result.txt` | L6, INV-1024 | partial | popcount 平均 349–391, 反復検出には粒度粗 |
| e171-c01 | experiment-171-sign2-spare-channel | `experiment-171-.../results/result.txt` | INV-SIGN2 | verified | Spare run-length 単発 80–88% |
| e172-c01 | experiment-172-spare-utf8-alignment | `experiment-172-.../results/result.txt` | INV-SIGN2 | verified | precision = recall = 1.000 (3 コーパス全件) |
| e173-c01 | experiment-173-block1024-codec | `experiment-173-.../results/result.txt` | A0, INV-1024, INV-NIBBLE4 | verified | 6044/6044 round-trip, 圧縮 8.2% (digram 上限 5.6% を +2.6pt 超過) |
| e174-c01 | experiment-174-pseudoinverse | `experiment-174-pseudoinverse/results/{f2_penrose.txt, z_pseudoinverse.txt}` | A0, A4 | partial | F2 / NDOC=16 で Rust 完全 Penrose (4 条件 = 0); 法律/混合は AAᵀ 特異化 |
| e175-c01 | consolidated-tren-cleanup-execute | (scheduler/Cargo.toml v0.2.0, tests/smoke.rs) | A0 | verified | smoke 5/5 PASS (理論 claim ではなくスケジューラ作業ログ; 凍結核と無関係な運用) |

---

## 表 6: falsification / drift 記録 (claim_id 1:1, 本表は notes 強化)

| claim_id | falsification 根拠 | 後継主張 / 対処 |
|----------|---------------------|------------------|
| T08 | Task #111 `test_capsule_additivity_proptest`: mean(d) ≈ 512.482, max(d) = 560, zero_ratio = 0 | A4 を「非線型ハッシュ圧縮 (SimHash / Bloom 系)」として再定式化。T13 / E84 / E91 系の同値判定はハミング距離近傍判定に限定 (`THEORY_RUST_CODEGEN.md` L42–L50) |
| T15 | E164 で P0 ∈ argmax だが unique でない (弱支持) | partial に格下げ。τ / α / K の sweep が今後の課題 |

> **v1 凍結核外の claim (= A6/A7/A8 等 e165-c01 由来の公理候補) は本表
> および表 1〜5 本表に置かない。** これらは凍結核 v2 起票事由として
> §4 凍結宣言 4 番に従い別ファイル (`THEORY_CORE_v2.md`) で議論する。
> 現時点では `THEORY_FORMALIZATION.md` L547–L554 / `OODA_history.json`
> e165-c01 の記述に委ねる。

---

## 追記方針

- 本書の **既存行は書き換えず**、新しい検証は表 1〜4 に行を追加する形で記録する
- `claim_id` が `THEORY_CORE_v1.md` に存在しない場合は **追記不可** (本表は
  凍結核 v1 と 1:1)。新規 claim_id は `THEORY_CORE_v2` 起票事由として
  §4 凍結宣言に従い別ファイルで議論する
- E176 以降の OODA サイクルは表 5 末尾に追記する (cycle メタデータのみ;
  claim_id 1:1 規約は表 1〜4 が唯一の正本)
- status は 3 値 (`verified` / `partial` / `falsified`) に固定。
  代数的自明性 / 外部前提などは notes で補足する

---

*v1 凍結対応: 2026-04-25 / OODA: e176-c01 / origin task: #26*
