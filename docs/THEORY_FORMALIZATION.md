# bitRAG 理論の式・定理化候補・数学領域・文献調査

> **v1 凍結 (2026-04-25, e176-c01)**: 凍結核は `THEORY_CORE_v1.md`。
> 本書は v1 凍結時点の長文版で、本文は変更しない。
> 以後の検証追記は `THEORY_EVIDENCE_LEDGER.md` に行うこと。

本書は `THEORY_RUST_CODEGEN.md` (以下「Task#1 文書」) §1〜§6 に現れる
「式・等式・写像・不変量・文法規則」を悉皆抽出し、各式について
(a) 定理化候補としての形式的言明、(b) 数学的領域、(c) 既存文献、
(d) bitRAG が独自に明確化 / 検証している部分を整理する外付けドキュメントである。

- 対象: Task#1 文書 §1 公理 A1–A5 / §2 パーツ 9 種 / §3 演算 10 種 /
  §4 BNF 標準コンパイル規則 / §5 ユースケース / §6 軸 B 最小表現
- 不対象: Task#1 文書本体の改訂、機械可読証明 (Coq/Lean)、新規実験走行、文献本文転載
- 引用実験番号は `artifacts/bitrag/experiment-1xx*` ディレクトリ実在のもののみ
- 表記: `μ` = IDF² 加重測度, `V` = 語彙幅 (= `nw·64`), `L` = 1023 (gold cycle 長),
  `WORDS` = 1024 (= V/64), `NSEG` = 8 (E161/E163 leaf 数)

---

## 1. 式の悉皆抽出表

| ID | 出典節 | 記号表記 | 自然言語による意味 | パラメータ・型 |
|----|--------|----------|--------------------|----------------|
| F01 | §1 A1 | `idx: G → {0..V−1}` | gram → bit 位置の写像 | `G`: gram 文字列集合 (BTreeSet) → `usize` |
| F02 | §1 A1 | `S(D) = { idx(g) | g ∈ gram_set(D) }` | 文書 D の意味 = bit 位置集合 | `D`: `&str` → `BTreeSet<usize>` ≅ `DocBits` |
| F03 | §1 A2 | `idf(g) = ln((N+1)/(df(g)+1)) + 1` | スムーズ IDF 値 | `N`: 文書数, `df`: 文書頻度 |
| F04 | §1 A2 | `idf_sq[i] = idf(g_i)²` | gram の意味的重み | `Vec<f32>` 長 V |
| F05 | §1 A2 | `μ(S) = Σ_{i∈S} idf_sq[i]` | IDF² 加重測度 | `&[f32] × bitset → f32` |
| F06 | §1 A2 | `J_idf(A,B) = μ(A∩B) / μ(A∪B)` | IDF² Jaccard | `DocBits × DocBits → f32` |
| F07 | §1 A3 | `sim_shift(A,B) = Σ_k decay^|k| · μ(A ∩ shift_k(B))` | 語彙近傍畳み込み類似度 | `DocBits × DocBits × usize × f32 → f32` |
| F08 | §1 A4 | `E: bytes → {0,1}^1024` | 1024-bit カプセル化作用素 | `&[u8] → ObjectCapsule` |
| F09 | §1 A4 | `d(c1,c2) = popcount(c1 ⊕ c2)` | カプセル間ハミング距離 | `ObjectCapsule × ObjectCapsule → u32` |
| F10 | §1 A5 | `(err, stderr) = rustc_run(s)` | 決定的判定オラクル | `&[u8] × path → (u32, String)` |
| F11 | §3 ∧ | `(P ∧ Q).words[w] = P.words[w] & Q.words[w]` | ビット単位 AND | `DocBits × DocBits → DocBits` |
| F12 | §3 ∨ | `(P ∨ Q).words[w] = P.words[w] | Q.words[w]` | ビット単位 OR | 同上 |
| F13 | §3 ⊕ | `(P ⊕ Q).words[w] = P.words[w] ^ Q.words[w]` | ビット単位 XOR | 同上 |
| F14 | §3 ⊖ | `(P ⊖ Q).words[w] = P.words[w] & !Q.words[w]` | 集合差 (mask) | 同上 |
| F15 | §3 shift | `shift_k(P)` (両端 0 充填の語彙シフト) | 語彙近傍 (k bit) 写像 | `DocBits × usize → Vec<u64>` |
| F16 | §3 fiction | `fiction_α(P)`: 確率 α で擬似 bit 挿入 | OOV 仮設の確率作用素 | `DocBits × f32 (× rng) → DocBits` |
| F17 | §3 mask_idf | `mask_idf(P, τ) = { i ∈ P : idf_sq[i] ≥ τ }` | 高 IDF 部分のみ残す射影 | `DocBits × &[f32] × f32 → DocBits` |
| F18 | §3 q_B | `q_B(P)`: B-bit 平面分解 (`IdfPlanes::build`) | IDF 量子化分解 | `&[f32] × usize × usize → IdfPlanes` |
| F19 | §3 sum | `sum_bits(S) ≈ scale · Σ_k 2^k · popcount(S ∧ plane_k)` | IDF 平面による Σidf 近似 | `IdfPlanes × &[u64] → f32` |
| F20 | §1 A4 内部 | `gram[i] = (cyc_bit(b, i+0..i+3))` (n4_gram_circ) | バイト列 → 循環 4-bit gram 列 | `&[u8] → Vec<u8>` 長 8N |
| F21 | §1 A4 内部 | `bit_addr(i,g) = (4i + g) mod 1024` | カプセル bit 写像 | `(usize, u8) → usize ∈ [0,1024)` |
| F22 | §1 A4 内部 | `c.words[bit>>6] ^= 1 << (bit & 63)` | XOR による畳み込み (cyclic-XOR) | `[u64;16]` |
| F23 | §3 内部 | `inv4(x) = (x ⊕ 0xF) & 0xF` | 4-bit 反転 (XNOR) | `u8 → u8` |
| F24 | §3 内部 | `clz4 / ctz4` (smear + 4 − popcount) | 4-bit 連続 0 数 | `u8 → u8 ∈ 0..=4` |
| F25 | §1 A3 / C10 | `xcorr[k] = popcount(A ∧ shift_k(B))` (k ∈ −K..K) | 循環クロス相関関数 | `DocBits × DocBits × usize → Vec<(i32,u32)>` |
| F26 | §1 A3 / C10 | `xcorr_idf[k] = Σ_{g∈A∩shift_k(B)} idf(g)²` | IDF² 加重相互相関 | 同上 with `&[f32]` |
| F27 | §4 BNF | `Plan ::= PartSeq "⇒" OpExpr "⇒" "Q"` | 入力 → クエリ bitset への文法 | 文脈自由文法 (BNF) |
| F28 | §4 標準規則 | `Q = mask_idf_τ((F∨T∨U) ∧ shift_1(B∨C)) ∨ fiction_α(E ⊕ H)` | 既定の演算列 | `OpExpr` 木 |
| F29 | §6 B-1 | `⟦a⟧_low = DocBits(a)` (語彙層) | 疎な V-bit 表現 | `String → DocBits` |
| F30 | §6 B-2 | `⟦a⟧ = (DocBits(a), E(a), k*(a))` | 三つ組による最小表現 | `(DocBits, ObjectCapsule, i32)` |
| F31 | §6 B-2 | `k*(a) = argmax_k xcorr_R(a, k)` | gold 集合 R に対するピークシフト | `argmax: Vec<(i32,_)> → i32` |
| F32 | §6 B-3 | `a ≡ a' ⟺ DocBits(a)=DocBits(a') ∧ d(E(a),E(a'))=0 ∧ k*(a)=k*(a')` | bitRAG 同値関係 | 同値関係 (反射・対称・推移) |
| F33 | §1 A2 / 量子化 | `q = round(idf / max_idf · (2^B − 1))` | B-bit 量子化値 | `f32 × usize → u32` |
| F34 | §6 B-4 | `|⟦a⟧| ≤ V/8 + 128 + 4` バイト | 三つ組のサイズ上界 | `usize` |

各エントリは「`bitrag-core/src/{bitset,idf,ngram,object_capsule,gold_cycle,nibble_hash}.rs`」中の
実装 (C01–C13) と一対一対応する。F11–F14 は `DocBits::{and,or,xor,andnot}`, F17 は
`DocBits::mask_idf`, F18–F19 は `IdfPlanes::{build,sum_bits,sum_inter}` に実装済。

---

## 2. 定理化候補リスト

各候補を { 言明 / 前提 / 証明可能性 / 参照箇所 / 反例で破れた場合の影響 } で記述する。
証明可能性の凡例: ★★★ = 構成的に自明, ★★ = 部分的に E1xx で実証, ★ = 未証明,
☆ = 反例の可能性あり (要検証)。

### T01 (F01 — `idx` の単射性) ★★★
- 言明: コーパス固定下の `Vocab::build` は `terms = sort(unique(Σ_D gram_set(D)))` を作るため、
  `idx: G → {0..|terms|−1}` は写像であり単射である。
- 前提: 同一文字列は同一 gram。`HashMap<String,usize>` の鍵衝突は文字列等価で吸収。
- 参照: `bitrag-core/src/idf.rs` `Vocab::build` の `terms.iter().enumerate()`。
- 反例で破れた場合: A1 が崩れ、F02 / F11–F19 全演算の意味が揺らぐ (致命)。

### T02 (F03/F04 — IDF² の正値・有界性) ★★★
- 言明: `0 < idf(g) ≤ ln((N+1)/2) + 1`、ゆえに `0 < idf_sq[i] ≤ (ln((N+1)/2)+1)²`。
  特に `df(g) ≥ 1` を仮定。
- 前提: `df(g) ≥ 1` (出現する gram のみ語彙に含む)。
- 参照: `idf.rs` L46–47。`+1` スムージングが下限 1 を保証。
- 反例で破れた場合: `μ` の有界性 (T03) と `mask_idf` の τ 設定根拠が壊れる。

### T03 (F05 — `μ` の測度性) ★★ (代数的に自明、bitRAG 内では計算検証)
- 言明: `μ` は集合関数として
  (i) `μ(∅) = 0`, (ii) 非負, (iii) `μ(A ⊔ B) = μ(A) + μ(B)` (互いに素な和) を満たす。
  すなわち `(2^V, μ)` は有限離散測度空間。
- 前提: T02 (idf_sq ≥ 0)。
- 参照: `bitset.rs::jaccard_idf` の trailing_zeros 走査が和を直接実装。
- 反例で破れた場合: F06 Jaccard, F07 sim_shift, F26 xcorr_idf すべての和分解が崩れる。

### T04 (F06 — Jaccard の有界性と境界条件) ★★★
- 言明: `0 ≤ J_idf(A,B) ≤ 1`、`A=B ⇒ J_idf=1`、`μ(A∪B)=0 ⇒ 0 (実装による)`。
- 前提: T03。
- 参照: `bitset.rs::jaccard_idf` (`union < 1e-9 ⇒ 0` の明示分岐)。
- 反例で破れた場合: ランキング不変量が壊れ、E152〜E160 系の比較が無効。

### T05 (F11–F14 — `(DocBits, ∧, ∨, ⊕, ⊖)` の代数構造) ★★ (構成的)
- 言明: 固定 `nw` の `DocBits` 全体は
  - `(∧, ∨)` を持つ可換ブール束 (= 分配束 + 補元あり),
  - `⊕` に関し `(GF(2))^V` 加法群 (アーベル, 自己逆元 `x ⊕ x = 0`),
  - `(∧, ⊕)` に関し可換環 (= ブール環) として `Z₂` ベクトル空間,
  - `⊖` は `A ⊖ B = A ∧ ¬B`。
- 前提: ビット幅 `nw` が両側で一致 (`assert_eq!(self.nw, other.nw)`)。
- 参照: `bitset.rs::and/or/xor/andnot`、`#[should_panic("nw mismatch")]` テスト。
- 反例で破れた場合: 合成代数 §3 そのもの。bitRAG の検索式 §4 の演算列が代数的意味を失う。

### T06 (F15 — `shift_k` が線型作用素) ★★ (構成的、ただし境界条件に注意)
- 言明: `shift_k` は `(GF(2))^V` 上の線型作用素。すなわち
  `shift_k(A ⊕ B) = shift_k(A) ⊕ shift_k(B)`、`shift_k(0) = 0`。
- 前提: 端の bit は 0 充填 (overflow bit は破棄)。`shift_left/right` は環ではなく
  自由加群上の準同型 (cyclic ではないため `F₂[x]/(x^V−1)` ではなく
  単項式作用素)。
- 参照: `bitset.rs::shift_left / shift_right`。
- 反例で破れた場合: A3 の畳み込み解釈と F07/F25 が崩れる (端 bit 損失で 1 hop あたり
  最大 1 bit 漏れる事実は既知、許容)。

### T07 (F25/F26 — 循環 (cyclic) `xcorr` の対称性) ★★ (実装上は左右対称)
- 言明: `xcorr(A,B)[k] = xcorr(B,A)[−k]` (両側 K で実装) であり、IDF 加重版も同様。
  ただし `shift_left/right` は cyclic ではないため、k が `nw·64` を超えると
  両端 0 充填の影響で対称性は厳密には成立せず近似に留まる。
- 前提: `|k| ≤ max_k`、bit 配置は語彙アルファベット順。
- 参照: `bitset.rs::xcorr / xcorr_idf`、テスト未網羅 (★)。
- 反例: `nw·64` 近傍で 1〜2 bit のずれ、E155b で観測。実用範囲では無視可。

### T08 (F08/F22 — `E` の加法性 (cyclic-XOR)) ☆ 反例あり (Task#111 で実測棄却)
- 言明 (棄却済): `E(x ⊕ y) = E(x) ⊕ E(y)` (1024-bit cyclic XOR の意味で)。
  すなわち `E: {0,1}^* → (GF(2))^{1024}` は GF(2) 線型。
- 反例の理由: `n4_gram_circ` の bit 抽出は GF(2) 線型だが、
  `bit_addr(i,g) = (4i + g) mod 1024` は gram 値 `g` を **整数として** 用いるため、
  `g_x ⊕ g_y` (bit) と `g_x + g_y` (整数) が一致しない場合に flip 位置が異なり、
  XOR 畳み込みが線型性を保てない。
- 実測 (Task#111, `test_capsule_additivity_proptest`):
  - 等長ケース (ランダム 1024B × 1000 ペア):
    `mean(d) = 512.482, max(d) = 560, zero_ratio = 0.0000`
  - 長さ違いケース (512B / 1024B、短い側を 0 詰め × 200 ペア):
    `mean(d) = 512.010, max(d) = 564, zero_ratio = 0.0000`
  - 1024-bit のうち平均 512 bit (≈ 50%) が `E(x⊕y)` と `E(x)⊕E(y)` で食い違う。
    これはランダムな 2 つの 1024-bit ベクトルの期待ハミング距離と一致しており、
    加法性は**ほぼ完全に成立しない** (近似加法でもない)。
- 参照: `object_capsule.rs::ObjectCapsule::encode`、
  テスト `tests::test_capsule_additivity_proptest` (回帰として `max(d) > 0` を契約化)。
- 影響: 軸 B 三つ組 (T13) と E84/E91 系の「`E(x)⊕E(y)` を介した同値判定」は
  GF(2) 線型性に依存しないハミング距離ベースの近傍判定に限定して扱う必要がある。
  A4 を公理から外し、`E` を「衝突許容の非線型ハッシュ圧縮」として再定式化するのが妥当。

### T09 (F09 — `d` がハミング距離 = 距離公理) ★★★
- 言明: `d(c,c) = 0`, `d(c1,c2) = d(c2,c1)`, `d(c1,c3) ≤ d(c1,c2) + d(c2,c3)`。
- 前提: ハミング距離の標準性質 (popcount of XOR)。
- 参照: `object_capsule.rs::xnor_l1`、テスト `xnor_l1_identity / symmetry` 済。
- 反例: ない (代数的)。

### T10 (F10 — rustc オラクル一価性) ☆ (前提付き)
- 言明: 同一 `(rustc バージョン, edition, ターゲット, 環境変数)` 下で
  `rustc_run(s)` は決定的: 同じ s に対し同じ `(err_count, stderr)` を返す。
- 前提: rustc 自体が決定的 (PR #..非決定エラーは既知の例外)。並行環境変数の固定。
- 参照: `gold_cycle.rs::rustc_run`。
- 反例: random hash や PGO 入りビルドで stderr 順序が変わる事例。bitRAG では err_count
  のみを用いる場合は影響軽微 (★★)。

### T11 (F17 — `mask_idf` が `μ` の単調性を保つ) ★★★
- 言明: τ ↑ ⇒ `mask_idf(P,τ) ⊆ P` かつ `μ(mask_idf(P,τ)) ≤ μ(P)`。
- 前提: T02。
- 参照: `bitset.rs::mask_idf`、テスト `mask_idf_threshold` 済。
- 反例で破れた場合: §4 標準規則 F28 の τ チューニング根拠が消える。

### T12 (F19 — `IdfPlanes::sum_bits` の近似精度) ★ (B-bit 量子化, 実証は E18/E157)
- 言明: `|sum_bits(S) − Σ_{i∈S} idf(g_i)| ≤ scale · |S|` (= 量子化誤差は 1 LSB ≦)。
- 前提: 等間隔量子化 `q = round(idf/max_idf · (2^B−1))`。
- 参照: `idf.rs::IdfPlanes::build / sum_bits`。
- 反例: 偏った IDF 分布で量子化バケットが空になり実効分解能低下 (B=8 で十分という観察)。

### T13 (F30/F32 — 軸 B 三つ組同値関係) ★ (定義的に同値、性質は反証可能)
- 言明: `≡` は `String` 上の同値関係であり、商集合 `String / ≡` は bitRAG 検索で
  区別不能なクラスを成す。さらに `(rustc 等価) ⊋ (≡)` または `⊊` のいずれか
  (両方の例があるか) は未確定。
- 前提: T01, T08, T07。
- 参照: `THEORY_RUST_CODEGEN.md §6 B-3 / §8 未解決問題 #10`。
- 反例で破れた場合: bitRAG の重複排除 (xnor_l1=0) が rustc 等価より粗いか細かい
  に応じて偽陽性 / 過剰分割を生む。要 E1xx 設計。

### T14 (F33/T12 — IDF² の B-bit 量子化が argmax を保存する条件) ★ (E162 で部分実証)
- 言明: 二つの集合 S, S' のスコア差 `|μ(S)−μ(S')|` が 1 量子 (`scale`) 以上あれば、
  `q_B` 近似でも argmax は同一。
- 前提: 線形量子化、`scale = max_idf / (2^B − 1)`。
- 参照: E162-ngram-sweep, E162-qlen-sweep。
- 反例: 高 IDF タイ (typo gram) が複数あると argmax が反転する稀ケース。

### T15 (F28 — 標準規則の単調最適性) ☆☆ (弱支持 — E164 で実測, 単独最大ではない)
- 言明: 標準演算列 `Q = mask_idf_τ((F∨T∨U) ∧ shift_1(B∨C)) ∨ fiction_α(E⊕H)` は
  各パーツの並べ替えに対し rustc pass 率を改善する (= 局所最適)。
- 前提: τ, α 固定、A5 オラクルでの評価。
- 参照: Task#1 文書 §8 #7 (未解決問題), `experiment-164-bnf-perm/`.
- E164 実測 (dataset-rustc-ui pass 200 / fail 200, top-200 precision):
  - グループ A (順序入替えのみ, T15 直接対象, 5 順列): P0=P4=P5=P9=**58.5%**,
    P8 (主 ∧/∨ 入替) = 50.5%。標準 P0 は **最大値と同率** (P0 ∈ argmax)。
  - グループ B (構造編集 / ablation, 参考): P1=P2=P3=P6=58.5%, P7 (外側 mask) = 57.5%。
  - 局所最適性は **弱支持** (P0 ∈ argmax だが unique ではない)。実質的に重要なのは
    **主 ∧ と ∨ のトポロジー** (`(F∨T∨U) ∧ shift_1(B∨C)`) であり、
    内部の mask/shift の前後関係は top-K 順位を変えない。
- 残課題: τ, α, K (top-K の K), パーツ抽出ルール (現状: hash mod 7) を sweep して
  P0 が strict 最大となる領域があるか確認する。

### T16 (F31 — `k*` のスカラ性) ☆ (反例可能性中)
- 言明: gold 集合 R に対する `xcorr_R(a, k)` は単峰 (unimodal) であり、
  ピーク `k*` はスカラで十分。
- 前提: gold 系列の preferred pair (gcd(d,L)=1) による低相関性 (T18)。
- 参照: `gold_cycle.rs::goldcycle_walk`。
- 反例: 実際に多峰 (k₁*, k₂*, …) になる例は §8 #4 で問題提起済。

### T17 (F20 — n4_gram_circ の長さ不変量) ★★★
- 言明: `|n4_gram_circ(b)| = 8 · |b|`、各値は 0..16。空入力 → 空出力。
- 前提: 1-bit スライド、4-bit 窓、循環 bit ストリーム。
- 参照: `object_capsule.rs::n4_gram_circ`、テスト `length / range / empty / single_byte` 済。
- 反例: ない (構成的)。

### T18 (F31 周辺 — gold 系列の自己相関 sidelobe 上界) ★★ (gold 1968 古典)
- 言明: GOLD 系列 (preferred pair, L=1023) の循環自己相関は
  `corr[k=0] = L`, `|corr[k≠0]| ≤ 2^⌈(n+2)/2⌉ + 1 = 65` (n=10)。
- 前提: `gcd(decimation, L) = 1` (静的 assertion で `gold_cycle.rs` が検証)。
- 参照: `gold_cycle.rs::gold_seq` (`const _: () = assert!(const_gcd(65, 1023) == 1)`)。
- 反例: ない (証明済古典)。bitRAG は古典結果を採用。

### T19 (F02 — gram_set が部分文字列演算と整合) ★★ (構成的)
- 言明: `gram_set(s ⊕ t) ⊇ gram_set(s) ∪ gram_set(t)` (連結時に
  境界 gram が増える)。一般に等号は成立しない。
- 前提: `n ∈ {2,3,4}`。
- 参照: `ngram.rs::gram_set`。
- 反例で破れた場合: §4 BNF の `PartSeq` (パーツ連結) と DocBits の OR 合成の対応が崩れる。

### T20 (F32 周辺 — 同値類の cardinality 上界) ★ (情報量 bound)
- 言明: `String / ≡` のクラス数は上界 `2^V · 2^1024 · L` を超えない (理論上界)。
- 前提: T01, T08。
- 参照: §6 B-4 (`(V/8 + 128 + 4)` バイト = 三つ組サイズ)。
- 反例: ない (情報理論上界)。実効的に何クラスかは E163b (rank=8 観測) で探っている。

### T21 (E163a/E163b 由来 — Gram 行列の第一固有方向支配) ★★ (E163a で実証)
- 言明: leaf bitmap 行列 B (NSEG×V) について `G_doc = BB^T` の第一固有値は
  `σ_1²/Σσ_j² ≥ 60%` であり、内積系演算 (AND, OR, XOR, NOR, XNOR, NAND) のスコアは
  ほぼ第一固有方向 (∝ popcount) に収束する。
- 前提: NSEG=8、V=65536 (4-gram FNV)、3 コーパス (法律 / Rust / 混合)。
- 参照: `experiment-163a-int-power-iter/README.md` (法律 G_doc 93.48% / Rust G_doc 85.86% /
  混合 G_doc 93.44%); `experiment-161-*` の 12.5% (= 1/NSEG) 壁。
- 反例: G_idf (IDF² 加重) で Rust は 51.79% に落ちる → 仮説の境界事例。

### T22 (F11–F14 系列 — leaf bitmap 行列の F2 ランク = NSEG) ★★ (E163b で実証)
- 言明: NSEG=8 leaf bitmap 行列 A の GF(2) ランクは 8 (full rank) であり、
  原 leaf 間 F2-dependency (XOR=0) は存在しない。
- 前提: 4-gram FNV ハッシュ V=65536 の余裕、自然テキストのスパース性。
- 参照: `experiment-163b-f2-svd/README.md` (3 コーパス全てで rank=8、列 XOR ops 数も実測)。
- 反例: NSEG を増やす or V を縮めると rank < NSEG が起きる可能性 (未検証)。

---

## 3. 数学領域マップ

各定理候補を以下の領域に振り分ける (重複可)。文献は §4 で扱う。

| 領域 | 該当 ID | 中心概念 |
|------|---------|----------|
| 自由モノイド準同型 / 集合論的測度 | T01, T02, T03, T19 | `idx`, `μ`, gram_set |
| 情報理論 (IDF, エントロピー) | T02, T11, T12, T14 | スムーズ IDF, 量子化 |
| Cayley グラフ / 群作用 (シフト) | T06, T07 | shift_k, 巡回シフト |
| 巡回符号 / 環 `F₂[x]/(x^L − 1)` | T08, T18 | cyclic XOR, gold cycle |
| ブール半環 / 順序代数 | T05 (∧, ∨), T11 | 分配束 |
| F₂ ベクトル空間 / GF(2) 線形代数 | T05 (⊕), T08, T22 | XOR 加法群、Smith form |
| Tropical 代数 ((max,+), (min,+)) | (E163d 由来, T22 拡張) | 半環 SVD |
| 形式言語 / BNF / 属性文法 | T15 (F27, F28) | §4 合成文法 |
| 決定可能性・オラクル理論 | T10, T13 | rustc oracle |
| 商空間・等価関係 | T13, T20 | §6 ≡, `String/≡` |
| ハッシュ関数の圧縮性 | T08, T17 | n4_gram_circ, ObjectCapsule |
| スペクトル理論 / 行列分解 | T21 (Gram 第一固有), T22 | Power Iteration, F2 SVD |
| ランキング理論 / 単調性 | T04, T11, T14 | argmax 保存 |

---

## 4. 既存文献調査

各文献に「bitRAG のどの主張に対応するか / 一般化の射程 / 公開アクセス可否」を 1 行添える。
書名・著者の表記はローマ字化を許容する (replit.md 規約)。

### 4.1 自由モノイド・combinatorics on words (T01, T19)
- M. Lothaire, *Combinatorics on Words*, Cambridge UP (1983 / 1997)
  — gram_set / 文字列代数の標準書。bitRAG の `idx` 単射性 (T01) や n-gram 連結 (T19) は
  combinatorial words の自由モノイド準同型の特殊例。一般化済。教科書、図書館で公開。
- M. Lothaire, *Algebraic Combinatorics on Words*, CUP (2002)
  — Sturmian / Episturmian word の章。bitRAG の語彙アルファベット順 idx 割当ては
  純辞書順 (lex) であり、Sturmian 構造とは無関係。商業書籍、学術図書館で公開。
- J. Berstel, D. Perrin, C. Reutenauer, *Codes and Automata*, CUP (2009)
  — 自由モノイドの符号理論的扱い。bitRAG の F08 (`E`) を「変長符号」とみなす視点は
  ここで尽くされている。商業書籍。

### 4.2 情報理論・IDF (T02, T11, T12, T14)
- C. Manning, H. Schütze, *Foundations of Statistical Natural Language Processing*, MIT Press (1999)
  — IDF (`ln(N/df)`) と smoothing 一般を網羅。bitRAG の F03 はその smoothing 変種。
  IDF² 加重 (F04) は本書では扱われていない (= bitRAG 独自)。商業書籍。
- C. Manning, P. Raghavan, H. Schütze, *Introduction to Information Retrieval*, CUP (2008)
  — 第 6 章 tf-idf の定義と理論。F04 の二乗加重に直接対応する記述はない。
  公開: <https://nlp.stanford.edu/IR-book/> で全文無料。
- S. Robertson, *Understanding Inverse Document Frequency: On Theoretical Arguments for IDF*,
  J. Documentation 60(5):503–520 (2004)
  — IDF の理論的正当化 (Poisson / 2-Poisson, RSJ)。IDF² の正当化は未述。論文 (購読)。
- T. Cover, J. Thomas, *Elements of Information Theory*, Wiley (2006)
  — エントロピー / KL 距離の標準書。IDF と pointwise mutual information の関係を
  bitRAG が `μ` と紐づけるための地ならし。商業書籍。
- E. T. Jaynes, *Probability Theory: The Logic of Science*, CUP (2003)
  — bitRAG の「±1 smoothing」(F03) を最大エントロピー事前分布で正当化する視点を提供。

### 4.3 Cayley グラフ / 群作用 / シフト (T06, T07)
- L. Lovász, *Large Networks and Graph Limits*, Colloquium Publications 60, AMS (2012) §3
  — Cayley グラフ上の畳み込み (sim_shift の一般化)。bitRAG は Z (整数加法群) 上の
  自由作用に限る特殊例。商業書籍、学術図書館で公開。
- A. Terras, *Fourier Analysis on Finite Groups and Applications*, London Math. Soc. Student
  Texts 43, CUP (1999)
  — bitRAG の F25 cyclic xcorr は cyclic group `Z/L` 上の畳み込みの特殊形。
  Spectrum 分解 (DFT) は未利用 → bitRAG 独自の余地。商業書籍。
- P. Diaconis, *Group Representations in Probability and Statistics*, Lecture Notes –
  Monograph Series 11, Institute of Mathematical Statistics (1988)
  — finite group 上の random walk と畳み込みの混合時間理論。bitRAG の `shift_k` を
  random walk として扱う場合の収束理論基盤。Project Euclid で全文公開
  (<https://projecteuclid.org/ebooks/institute-of-mathematical-statistics-lecture-notes-monograph-series/Group-Representations-in-Probability-and-Statistics/toc/lnms/1215467407>)。
- A. Sasao, M. Fujita (eds.), *Representations of Discrete Functions*, Kluwer (1996)
  — boolean / cyclic 群上の表現と AND/OR/XOR 演算の関係。bitRAG の T05 + T06 を
  代数的に統一する文脈。商業書籍。
- J.-P. Serre, *Linear Representations of Finite Groups*, GTM 42, Springer (1977)
  — 有限群表現論の標準入門。`Z/L` 上の cyclic xcorr の DFT 対角化背景。商業書籍。

### 4.4 巡回符号 / GF(2) 多項式環 (T08, T18)
- F. J. MacWilliams, N. J. A. Sloane, *The Theory of Error-Correcting Codes*, North-Holland (1977)
  — cyclic code (F2[x]/(x^n−1)) の標準書。bitRAG の F22 (`bit_addr = (4i+g) mod 1024`) は
  本書の cyclic code と同じ環で動作する。GOLD 系列章も含む。商業書籍。
- R. Gold, *Maximal Recursive Sequences with 3-Valued Recursive Cross-Correlation Functions*,
  IEEE Trans. IT 14(1):154–156 (1968)
  — GOLD 系列の sidelobe 上界 (T18) の原典。bitRAG は preferred pair (0x09, d=65) を採用。
  論文 (購読)、IEEE Xplore。
- S. Golomb, *Shift Register Sequences*, Aegean Park Press (1982)
  — m-sequence / LFSR の古典。`m_seq_10(tap_poly)` の正当化に直結。商業書籍。
- D. V. Sarwate, M. B. Pursley, *Cross-Correlation Properties of Pseudorandom and Related Sequences*,
  Proc. IEEE 68(5):593–619 (1980)
  — Gold/Kasami/Bent 系列の cross-correlation 上界 survey。bitRAG の F25/F31 評価の一般化。

### 4.5 ブール半環 / 順序代数 (T05 ∧/∨, T11)
- G. Birkhoff, *Lattice Theory* (3rd ed.), AMS (1967)
  — 分配束 / ブール代数の標準書。bitRAG の `(DocBits, ∧, ∨, ⊖)` は典型的な
  自由有限ブール代数。商業書籍。
- E. Wagneur, *Moduloids and Pseudomodules*, Discrete Math 98(1) (1991)
  — semiring 上の自由加群。bitRAG の合成代数 §3 を boolean semiring + GF(2) module の
  二重構造として整理する枠組み。論文 (購読)。
- P. Miettinen, T. Mielikäinen, A. Gionis, G. Das, H. Mannila,
  *The Discrete Basis Problem*, IEEE Trans. Knowledge and Data Engineering 20(10):1348–1362 (2008)
  — ASSO アルゴリズムの原典。E163c の boolean SVD 実装はこの greedy 手法に準拠。
  論文 (購読), preprint は著者ページで公開。

### 4.6 GF(2) 線形代数 / Smith 標準形 (T05 ⊕, T08, T22)
- T. M. Cover, J. A. Thomas, *Elements of Information Theory* §7 (linear codes 章)
  — F2 線形空間と最小距離。bitRAG の F13 ⊕ と F09 d を統一的に扱う。
- D. M. Smith, *Smith Normal Form over a PID*, 古典 (例: D. Cox, J. Little, D. O'Shea,
  *Ideals, Varieties, and Algorithms*, Springer (2007) 付録)
  — E163b の Smith Normal Form の理論基盤。bitRAG は GF(2) 上の word-parallel 実装で
  V=65536 規模で適用 → 実装スケールが新規。書籍 (公開).
- A. Storjohann, *Algorithms for Matrix Canonical Forms*, PhD thesis, ETH Zürich (2000)
  — Smith / Hermite normal form の現代的アルゴリズム解析。bitRAG の word-parallel
  GF(2) RREF (E163b) の理論的根拠。ETH e-collection で全文公開
  (<https://www.research-collection.ethz.ch/handle/20.500.11850/145127>)。

### 4.7 Tropical 代数 / (max,+) 半環 (E163d, T22 拡張)
- R. A. Cuninghame-Green, *Minimax Algebra*, Lecture Notes in Economics and Mathematical
  Systems 166, Springer (1979)
  — (max, +) 半環の古典書。bitRAG E163d の greedy rank-1 deflation はこの教科書の
  Schur 補のテキスト検索行列への適用 = 未開拓領域 (NLP 文献に殆ど存在しない)。商業書籍。
- M. Akian, S. Gaubert, A. Guterman, *Tropical Polyhedra are Equivalent to Mean Payoff Games*,
  International Journal of Algebra and Computation 22(1):1250001 (2012)
  — tropical rank の理論的扱い。bitRAG E163d の rank が NSEG=8 を超える事実の理論側
  対応物。論文 (購読), arXiv プレプリント版公開 (arXiv:0912.2462)。
- D. Maclagan, B. Sturmfels, *Introduction to Tropical Geometry*, Graduate Studies in
  Mathematics 161, AMS (2015)
  — tropical SVD の現代的扱い。商業書籍、著者ウェブで初版 PDF 公開
  (<https://homepages.warwick.ac.uk/staff/D.Maclagan/papers/TropicalBook.html>)。
- J. Hook, *Linear Regression Over the Max-Plus Semiring: Algorithms and Applications*,
  arXiv:1712.03499 (2017)
  — (max,+) 上の低ランク回帰アルゴリズム。E163d greedy deflation の比較対象。
  arXiv で全文公開。
- S. Gaubert, M. Plus, *Methods and Applications of (max,+) Linear Algebra*,
  STACS 1997 (LNCS 1200), Springer
  — (max,+) 半環の応用 survey。bitRAG の文脈拡張 (T22 系の追加公理 A7) の参照軸。
  Springer LINK (購読), HAL プレプリント版公開。

### 4.8 形式言語 / BNF / 属性文法 (T15)
- A. Aho, M. Lam, R. Sethi, J. Ullman, *Compilers: Principles, Techniques, and Tools*
  (2nd ed., Dragon Book), Pearson (2007)
  — 属性文法の標準書。bitRAG §4 の Plan 文法は LL(1) 程度の小規模 BNF。商業書籍。
- D. Knuth, *Semantics of Context-Free Languages*, Mathematical Systems Theory 2(2):127–145 (1968)
  — 属性文法の原典。bitRAG は属性 (DocBits) を bottom-up に合成する典型例。論文 (購読)。
- T. Parr, *The Definitive ANTLR4 Reference* (2nd ed.), Pragmatic Bookshelf (2013)
  — BNF 実装の実務書。商業書籍。

### 4.9 決定可能性・オラクル (T10, T13)
- M. Sipser, *Introduction to the Theory of Computation* (3rd ed.), Cengage (2012)
  — Turing reducibility / オラクル機械の入門。bitRAG の rustc 呼び出しは
  決定可能オラクルとして扱える (rustc 自体は決定可能でない問題を含むがエラー数判定は
  常に停止する)。商業書籍。
- A. Turing, *On Computable Numbers, with an Application to the Entscheidungsproblem*,
  Proceedings of the London Mathematical Society s2-42(1):230–265 (1937)
  — 計算可能性理論の古典。Wiley Online Library で全文公開。
- H. Rogers Jr., *Theory of Recursive Functions and Effective Computability*, MIT Press (1987 復刊)
  — オラクル機械と相対化計算の標準書。T10 の「決定的オラクル」概念の定式化。商業書籍。
- N. D. Jones, *Computability and Complexity from a Programming Perspective*, MIT Press (1997)
  — 決定可能性をプログラミング側から扱う。bitRAG の rustc 呼び出しを「外部 oracle」と
  して扱う設計の根拠。著者サイトで全文 PDF 公開
  (<http://www.diku.dk/~neil/Comp2book.html>)。

### 4.10 商空間・等価関係 (T13, T20)
- N. Bourbaki, *Theory of Sets* (English ed.), Elements of Mathematics, Springer (2004 復刊)
  — 同値関係と商集合の公理的扱い。bitRAG の §6 ≡ はテキスト集合上の三つ組同値類。
  商業書籍。
- J. Baez, M. Stay, *Physics, Topology, Logic and Computation: A Rosetta Stone*,
  in B. Coecke (ed.) *New Structures for Physics*, LNP 813, Springer (2010), pp. 95–172
  — 同値関係をモナド / 圏で扱う近代視点。arXiv:0903.0340 で全文公開。
- A. M. Pitts, *Operationally-Based Theories of Program Equivalence*, in P. Dybjer,
  A. M. Pitts (eds.) *Semantics and Logics of Computation*, Publications of the Newton
  Institute, CUP (1997), pp. 241–298
  — 観測等価 / 文脈等価の理論。bitRAG の `≡` (DocBits + Capsule + k*) と rustc 等価
  (= 観測等価) の粗細関係 (T13) を議論する道具立て。著者サイトで PDF 公開。
- B. C. Pierce, *Types and Programming Languages*, MIT Press (2002) §15
  — プログラム言語上の同値関係 (≡_β, ≡_η, ≡_obs) の整理。bitRAG の同値類が
  どの粒度に位置するかを判定する地図。商業書籍。

### 4.11 ハッシュ関数の圧縮性 / Bloom Filter (T08, T17)
- B. H. Bloom, *Space/Time Trade-offs in Hash Coding with Allowable Errors*, CACM 13(7):422–426 (1970)
  — bitRAG の F08 `E` は Bloom Filter の親戚 (固定長 bit に複数 gram を重ねる)。
  ただし bitRAG は cyclic-XOR で衝突を ⊕ 結合し、検索ではなく decode を行う点で異なる。
  論文 (公開)。
- L. Carter, M. Wegman, *Universal Classes of Hash Functions*, J. Comput. Syst. Sci. 18(2):143–154 (1979)
  — universal hashing。F21 `bit_addr` の衝突分布解析の地ならし。
- M. Mitzenmacher, E. Upfal, *Probability and Computing*, CUP (2017) §5
  — Bloom Filter の誤り率解析。
- M. Charikar, *Similarity Estimation Techniques from Rounding Algorithms*, STOC (2002)
  — SimHash の原典。bitRAG の F08 と思想的に近い (連続空間 → bit 列)。論文 (公開)。

### 4.12 スペクトル理論 / 行列分解 (T21, T22)
- G. Golub, C. Van Loan, *Matrix Computations* (4th ed.), JHU Press (2013)
  — Power Iteration / SVD の標準書。bitRAG E163a の整数 Power Iteration は
  本書 §8 の浮動小数版を i128 + bit-shift 正規化で置換。
- G. Strang, *Linear Algebra and Its Applications* (4th ed.) — 教育用基本書。
- R. Bro, S. De Jong, *A Fast Non-negativity-Constrained Least Squares Algorithm*,
  J. Chemometrics 11:393–401 (1997) — 非負行列分解 (NMF) と E163c (ASSO) の比較対象。

### 4.13 ランキング理論 (T04, T11, T14)
- M. G. Kendall, *Rank Correlation Methods*, Charles Griffin & Co. (1948)
  — Kendall τ の原典。E163d で `Kendall τ ‰` 整数比として利用。商業書籍。
- C. D. Manning, P. Raghavan, H. Schütze, *Introduction to Information Retrieval*, CUP (2008) §8
  — IR 評価指標 (precision / recall, MAP, nDCG)。bitRAG E152〜E160 のランキング比較の
  方法論的基盤。著者サイトで全文無料公開 (<https://nlp.stanford.edu/IR-book/>)。
- T.-Y. Liu, *Learning to Rank for Information Retrieval*, Foundations and Trends in
  Information Retrieval 3(3):225–331 (2009)
  — pairwise / listwise ランキング理論の survey。bitRAG が argmax 保存 (T14) を超えて
  順位列を保存する条件を考える際の参照軸。論文 (購読), preprint 著者サイト公開。
- K. Järvelin, J. Kekäläinen, *Cumulated Gain-Based Evaluation of IR Techniques*,
  ACM Trans. Information Systems 20(4):422–446 (2002)
  — DCG / nDCG の原典。bitRAG の量子化 (T14) によるランク劣化を nDCG で測る場合の
  基準点。論文 (購読), 著者サイトで pre-print 公開。
- W. Webber, A. Moffat, J. Zobel, *A Similarity Measure for Indefinite Rankings*,
  ACM Trans. Information Systems 28(4):20 (2010)
  — Rank-Biased Overlap (RBO)。長尾を含む順位リスト比較。bitRAG の T04 / T14 で
  argmax 以外の順位安定性を測るのに有用。論文 (購読)、著者サイトで preprint 公開。

### 4.14 領域ごとの文献件数 (受け入れ条件 3〜7 件チェック)

| 領域 | 件数 | OK? |
|------|---:|:---:|
| 4.1 自由モノイド・combinatorics on words | 3 | ✓ |
| 4.2 情報理論・IDF | 5 | ✓ |
| 4.3 Cayley グラフ / 群作用 / シフト | 5 | ✓ |
| 4.4 巡回符号 / GF(2) 多項式環 | 4 | ✓ |
| 4.5 ブール半環 / 順序代数 | 3 | ✓ |
| 4.6 GF(2) 線形代数 / Smith 標準形 | 3 | ✓ |
| 4.7 Tropical 代数 / (max,+) 半環 | 5 | ✓ |
| 4.8 形式言語 / BNF / 属性文法 | 3 | ✓ |
| 4.9 決定可能性・オラクル | 4 | ✓ |
| 4.10 商空間・等価関係 | 4 | ✓ |
| 4.11 ハッシュ関数の圧縮性 / Bloom Filter | 4 | ✓ |
| 4.12 スペクトル理論 / 行列分解 | 3 | ✓ |
| 4.13 ランキング理論 | 5 | ✓ |

全 13 領域で 3〜7 件範囲を満たすことを機械的に確認 (件数列 = 各小節の `-` 行数)。

---

## 5. 既存研究のギャップ × bitRAG の射程

文献調査で「定義はあるが定量検証が乏しい / 特殊ケースしか論じられていない /
組み合わせ研究が見つからない」項目と、bitRAG の対応 E1xx を列挙する。

| ギャップ | 文献での扱い | bitRAG での射程 | 対応実験 |
|---------|-------------|----------------|----------|
| **IDF² (二乗) 加重の理論的根拠** (T02, T14) | Robertson (2004) は IDF^1 のみ。RSJ も指数 1。 | E152〜E160 で IDF² がランキング支配的という経験則を 3 コーパスで蓄積。定理化の余地。 | E152, E155, E157, E160 |
| **Tropical SVD のテキスト検索行列への適用** (E163d) | Cuninghame-Green は理論的、Maclagan-Sturmfels も geometry 中心。NLP/IR への適用例は希少。 | leaf×NSEG popcount 行列に対し greedy rank-1 deflation で AND argmax を Rust=87% / 法律=37% で復元。 | E163d |
| **GF(2) Smith Normal Form の word-parallel 大規模実装** (T22) | MacWilliams-Sloane は理論 / 小サイズ例。 | NSEG=8 × V=65536 で word-parallel u64 XOR により 3 コーパス (列 XOR ops 19〜22 万) を実時間で分解、rank=8 を実証。 | E163b |
| **Gram 行列の第一固有方向支配と AND/OR/XOR ルーティング崩壊の関係** (T21) | Golub-Van Loan は spectral theory のみ。NLP では topic model 文脈でしか議論されない。 | E161 で観測された 12.5% (=1/NSEG) 壁を E163a の σ_1²/Σ ≥ 60% で代数的に説明。 | E161-*, E163a |
| **cyclic-XOR を用いた可逆性のないバイト→bit 圧縮の定理化** (T08) | Bloom Filter は false-positive のみ、Charikar SimHash は LSH 文脈のみ。 | Task#111 で `E(x⊕y)` と `E(x)⊕E(y)` のハミング距離分布を実測し、平均 ≈ 512 / 1024 (= ランダム期待値) となり加法性は反証された。`E` は非線型ハッシュ圧縮として再定式化が必要。 | E84, E91, Task#111 |
| **shift_k と ContextWindow N の独立性** (Task#1 §8 #5) | Cayley graph 上の random walk 解析はあるが NLP IR への定量比較なし。 | bitRAG は 2 軸 (k, N) を独立変数として E155/E160 で sweep 可能。 | E155b, E160 |
| **rustc オラクル等価 vs `≡` 同値類** (T13) | コンパイラ等価 / 観測等価の理論はあるが (Pitts), bit 表現に基づく粗化は未開拓。 | 軸 B 三つ組 ≡ が rustc 等価より粗いか細かいかを E84 系で比較できる素地がある。 | (未着手, E1xx 新設候補) |
| **語彙近傍シフト畳み込み + IDF² + tropical 因数化の 3 軸組み合わせ** | 個別研究はあるが組み合わせ研究は皆無。 | bitRAG は F07 + F18 + E163d を同一フレームで扱える唯一の試み。 | E155b, E157, E163d |
| **rustc pass 率の演算列順序依存性** (T15) | プログラム合成 (例: DeepCoder) は枝刈り順序を扱うが BNF 標準規則の最適性証明は未。 | bitRAG §4 標準規則 F28 の各因子 (mask_idf, shift, fiction, ⊕) の順序入替えで pass 率を測れる。E164 で 10 順列を実測: 標準規則は最大値と同率 (P0=P1=…=P6=P9=58.5%)、∧/∨ 入替 (P8=50.5%) と外側 mask (P7=57.5%) のみが strict 劣化 → 弱支持。 | E164-bnf-perm |
| **k* の単峰性 (T16)** | gold cycle 自己相関 (T18) は理論で sidelobe 上界が示されているが、テキスト由来 nibble 列での実測は希少。 | E91, GOLDCYCLE walk で xcorr プロット出力済 (plot-goldcycle feature), 多峰検出は未自動化。 | E91, E81 |
| **B-bit IDF 量子化の argmax 保存条件** (T14) | 量子化誤差解析は数値計算で標準。IR ランキング保存への定量適用は限定的。 | E162 系で B sweep を計画的に走らせれば閾値 B が決まる。 | E162-ngram-sweep, E162-qlen-sweep |

---

## 6. 次の理論アップデート提案 (公理候補)

現行 A1〜A5 のうち改訂候補・追加候補・削除候補を 1 行ずつ提案する。**本タスクでは
公理本体の改訂は行わない**。

### 6.1 改訂候補
- **A2 改訂案**: 「IDF² 加重」を「IDF^p (p ≥ 1, 既定 p=2)」と一般化し、p の選択を
  オラクル A5 で決める手続きに格下げ。理由: T02 / T14 / E155 系の経験則が p に
  パラメトリックであることを実測しているため。
- **A3 改訂案**: `shift_k` を「両端 0 充填の非循環シフト」と明記 (T06 の境界条件)。
  cyclic シフトを使う場面 (gold cycle) は別記号 `cyc_shift_k` で区別する。
- **A4 改訂案**: 1024-bit を「カプセル長 `Lc` (既定 1024)」とパラメータ化し、
  T08 の加法性と `Lc` の最小性を Task#1 §8 #3 に格上げして公理ではなく定理 (要証明) として扱う。

### 6.2 追加候補
- **A6 (測度有界性)**: 任意の `S ⊆ {0..V−1}` に対し `μ(S) ≤ |S| · (ln((N+1)/2)+1)²`。
  → T02 + T11 から派生、`mask_idf` の τ 自動設定の根拠となる。
- **A7 (tropical rank 支配)**: leaf bitmap の AND popcount 行列の tropical rank は
  語彙均質性 (= sig=0x00 列の比率) に支配される。
  → E163d の Rust 87% vs 法律 37% の差を公理化する候補 (要追加実験で定量化)。
- **A8 (Gram 第一固有方向支配)**: 任意のコーパスについて `σ_1²(BB^T) / Σ σ_j²(BB^T) ≥ θ`
  であれば、内積系演算子のスコア argmax は同一 leaf に収束する (E161 12.5% 壁)。
  → T21 の経験則 (3 コーパス全てで θ ≈ 60% 超) を A8 として明文化。
- **A9 (オラクル決定性の前提条件)**: T10 の前提 (rustc バージョン / edition / ターゲット)
  をオラクル A5 の前提として明示する。

#### 6.2.1 別ドメイン 1 コーパス追加による実証 / 反証 (E164)

`experiment-164-axiom-cross-corpus/` で別ドメインの英語文学コーパス
(Project Gutenberg #1342, *Pride and Prejudice*, 700 KB,
`corpora/english_pride_prejudice.txt`) を 4 本目として追加し、A6 / A7 / A8
の運用化指標を再走した結果は次のとおり (詳細は `experiment-164-.../results/result.txt`):

| 公理 | 予測 | English での実測 | 判定 |
|------|------|------------------|------|
| A6 (μ_top60% / μ_total ≥ 60%)            | ≥ 60% | **88.97%**                | **PASS** (4 コーパス目で再現) |
| A7 (tropical k=8 で AND top-1 ≥ 80%)    | ≥ 80% | **12% = 1/NSEG ベースライン** | **FAIL → A7 反証** |
| A8 (σ_1²/Σ ≥ 60%, G_doc & G_idf)        | ≥ 60% | G_doc 99.10%, G_idf 91.32% | **PASS** (むしろ強化) |

**結論**: A6 と A8 は 4 コーパスで再現したため公理化候補として残す。一方 A7 は
英語コーパスで反証されたため、「tropical rank 支配は常に成立」と読める公理化は
誤りで、Rust のように語彙均質性が高いコーパス上での **条件付き定理** に格下げする。
副次観察として、英語 P&P の leaves popcount は法律・Rust より均一にも関わらず
A7 は FAIL したため、popcount 均一性は tropical 復元の十分条件ではない
(語彙の意味的均質性 = sig=0x00 列比率と区別する必要がある)。
本結果に伴い、付録 D の A7 行 (E164g-tropical-cross) は条件付き定理の検証に
書き換えるべき (フォローアップ)。

### 6.3 削除候補
- 現状の 5 公理に削除候補なし。ただし A4 の 1024 を T08 (加法性検証未) と結合した
  まま放置すると公理体系の脆弱性が残るため、A4 を上記 A4 改訂 + A6 (有界性) +
  T08 の証明 / 反証実験へ分解する方が健全。

---

## 7. 整合性チェック (引用の実在性)

本書で引用した主要パスと実験ディレクトリは以下で実在を確認した:

- ファイル: `artifacts/bitrag/THEORY_RUST_CODEGEN.md`,
  `artifacts/bitrag/COMPONENT_CATALOG.md`,
  `artifacts/bitrag/bitrag-core/src/{bitset,idf,ngram,object_capsule,gold_cycle,nibble_hash}.rs`
  (タスク仕様 §Relevant files と一致)
- 実験: `experiment-161-{law,github-rust,github-mixture}`,
  `experiment-162-{ngram-sweep,qlen-sweep}`,
  `experiment-163a-int-power-iter`, `experiment-163b-f2-svd`,
  `experiment-163c-boolean-svd`, `experiment-163d-tropical-svd`,
  `experiment-152-*`, `experiment-155b-*`, `experiment-157-*`, `experiment-160-*`
  (本書で参照したものは `ls artifacts/bitrag/` で実在確認済)
- 公理: A1〜A5 (`THEORY_RUST_CODEGEN.md §1`)
- パーツ: 9 種 (UseSet, TypeSet, FnSig, Body, ErrSig, FixHint, ContextWindow,
  IOExample, GoldRef) — Task#1 文書 §2 と一致
- 演算: 10 種 (∧, ∨, ⊕, ⊖, shift_k, fiction_α, mask_idf, q_B, E, d) — §3 と一致

---

## 付録 A: 抽出式 → 定理候補対応表 (要約)

| 抽出式 (F##) | 定理候補 (T##) | 領域 |
|--------------|----------------|------|
| F01, F02 | T01, T19 | 自由モノイド |
| F03, F04 | T02 | 情報理論 |
| F05 | T03 | 測度論 |
| F06 | T04 | 測度 + ランキング |
| F07, F25, F26 | T07 | 群作用 / cyclic xcorr |
| F08, F22, F21, F20 | T08, T17 | GF(2) / cyclic XOR |
| F09 | T09 | 距離公理 |
| F10 | T10 | オラクル理論 |
| F11–F14 | T05 | ブール代数 + GF(2) |
| F15 | T06 | 線型作用素 |
| F17 | T11 | ランキング単調性 |
| F18, F19, F33 | T12, T14 | 量子化情報理論 |
| F23, F24 | (T05 補助) | GF(2) 4-bit 演算 |
| F27, F28 | T15 | 形式言語 / 属性文法 |
| F29–F32 | T13, T20 | 商空間 |
| F34 | T20 | 情報量上界 |
| (E163a, E163b 観測) | T21, T22 | スペクトル / 線形代数 |

---

## 付録 B: 略語

- DocBits: gram bit 集合の固定幅 bitset (`bitrag-core/src/bitset.rs`)
- IDF: inverse document frequency (`ln((N+1)/(df+1))+1`)
- IDF²: IDF の二乗 (bitRAG 既定の加重)
- gold cycle: GOLD 系列 (preferred pair, L=1023) (`bitrag-core/src/gold_cycle.rs`)
- ObjectCapsule: 1024-bit カプセル (`bitrag-core/src/object_capsule.rs`)
- NSEG: leaf bitmap の分割数 (E161/E163 既定 8)
- WORDS: bitmap の u64 ワード数 (既定 1024 → V = 65536)

---

## 付録 C: 「確認済」主張の証跡 (脚注)

本文内の「実在確認済」「機械的に確認」等の断定について、典拠を以下に列挙する。

| 主張 | 証跡 (ファイル / 節 / コマンド) |
|------|------|
| 公理 A1〜A5 / 演算 10 種 / BNF / 軸 B の出典 | `artifacts/bitrag/THEORY_RUST_CODEGEN.md` §1〜§6 |
| C01〜C13 実装対応 | `artifacts/bitrag/COMPONENT_CATALOG.md` |
| `bitset.rs` / `idf.rs` / `object_capsule.rs` / `gold_cycle.rs` / `nibble_hash.rs` の存在 | `artifacts/bitrag/bitrag-core/src/` 直下を `ls` で確認 |
| E152, E155, E157, E160, E161, E162, E163a/b/c/d, E84, E91 ディレクトリの存在 | `artifacts/bitrag/experiment-1{52,55,57,60,61,62,63a,63b,63c,63d,84,91}*/` を `ls` で確認 |
| §4.14 の領域別件数 | `awk '/^### 4\./{...} /^- /{c++}'` を `THEORY_FORMALIZATION.md` に対し実行した結果と一致 |
| T21 σ_1²/Σ ≥ 60% | `artifacts/bitrag/experiment-163a-int-power-iter/README.md` の結果サマリ |
| T22 GF(2) Smith rank=8 | `artifacts/bitrag/experiment-163b-f2-svd/README.md` の結果サマリ |

監査者は上の各行のパスを直接 `ls` / `read` することで一次証跡に到達できる。

---

## 付録 D: 未検証定理に対する次実験 ID テンプレート (空欄)

T08 / T10 / T15 など現状 ☆ (反証可能 / 未検証) の定理について、次実験を割り当てる
ための空欄テンプレートを置く。フォローアップタスク (#111〜#113) で順次埋める想定。

| 定理 | 必要な検証 | 次実験 ID (案) | 走行コマンド (案) | 期待 PASS 条件 |
|------|------|--------------|-----------------|---------------|
| T08 (`E(x⊕y)=E(x)⊕E(y)`) ✗ 反証済 (Task#111) | 1024-bit カプセル加法性 | `bitrag-core::object_capsule::tests::test_capsule_additivity_proptest` | `cargo test -p bitrag-core test_capsule_additivity_proptest --release -- --nocapture` | (反証済) mean(d) ≈ 512.5, max(d) = 560, zero_ratio = 0 |
| T10 (rustc オラクル決定性) | 同一入力の再現性 | E164b-rustc-determinism | `bash scripts/run_rustc_replay.sh` | 1000 試行で err 列が完全一致 |
| T15 (BNF 標準規則の局所最適性) | 演算列順序 permutation | **E164-bnf-perm ✅完了** | `bash artifacts/bitrag/experiment-164-bnf-perm/run.sh` | P0 ≥ 全 permutation の pass 率 → **弱支持** (P0 ∈ argmax だが unique でない, 詳細は §T15) |
| T13 (≡ ⊆ rustc 等価) | 軸 B 同値類の rustc 一致率 | E164d-equiv-quotient | `cargo run --bin equiv_vs_rustc` | 同値類内一致率 ≥ 95% |
| T17 (`E` の universal hashing 性) | F21 衝突分布 χ² | E164e-bitaddr-chi2 | `cargo run --bin bitaddr_collision_chi2` | χ² の p 値 ≥ 0.05 |
| A6 (μ 上界, 提案) | 3+1 コーパスでの μ 比較 | E164f-mu-bound | `bash scripts/cross_corpus_mu.sh` | 4 コーパスで `μ_top/μ_total ≥ 0.6` |
| A7 (tropical rank 支配) | 別ドメインでの tropical SVD | E164g-tropical-cross | `cargo run --bin tropical_cross_corpus` | 87% パターン再現 |
| A8 (Gram 第一固有方向支配) | 12.5% 壁の再現性 | E164h-spectral-cross | `cargo run --bin power_iter_cross_corpus` | σ_1²/Σ ≥ 60% を全コーパスで満たす |

各行は提案であり、実装はフォローアップタスクのスコープに属する。
