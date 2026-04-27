# bitRAG NAND 完全性証明 (背理法 + Logic-Only 公理 A0 による構成的自明化)

> **v1 凍結 (2026-04-25, e176-c01)**: 凍結核は `THEORY_CORE_v1.md`。
> 本書は v1 凍結時点の長文版で、本文は変更しない。
> 以後の検証追記は `THEORY_EVIDENCE_LEDGER.md` に行うこと。

> 上位文書: `THEORY_RUST_CODEGEN.md` (公理層 §1〜§8)
> 本書: 公理層の上に乗る **回路層** (Task #114)

---

## 階層 0. 仮説 (Hypothesis)

### A0 (Logic-Only 公理 — 新規, 本書で提案)
bitRAG の全意味演算は、定義域・値域とも有限ブール空間 ($\{0,1\}^V$ またはその直積) 上の関数であり、**論理演算 (∧, ∨, ¬, ⊕, shift, mask 等) のみで構成または構成を目標とする**。実数演算・解析演算・非決定的演算はオラクル境界 (A5) の外側にしか現れない。

### H0 (中心仮説)
bitRAG の全意味演算 (A0 に従うもの) は、効率性を問わなければ有限個の **NAND** ゲートで構成できる。

### H0' (背理法のための否定)
H0 に反し、NAND で構成不能な bitRAG 演算 $f$ が存在する。

### 証明戦略
H0' を仮定し L1 (効率性を妥協した NAND 完全性) と矛盾を導く (T2)。さらに A0 と L1 から、背理法の **否定操作 (NOT) 自体が不要** であることを示す系 C-T2 を導く。

### A0 の Task #1 §1 への配置案 (1 段落 — IG1)
Task #1 §1 の公理 A1〜A5 のうち、A1 (有限語彙)・A2 (DocBits 表現)・A3 (合成演算閉性) は **A0 を黙示前提** にしている。本書は A0 を A1 の **直前** に置き「全演算は有限ブール空間上の関数として定義される」を独立公理として宣言する。これにより A1〜A4 は A0 の構造的具体化、A5 は A0 の境界外 (外部呼出) として整理される。

---

## 階層 1. 仮説補題化 (Lemmatization) — L1〜L11

各補題は { **言明** / **前提** / **証明スケッチ** / **bitRAG 内対応** } の 4 ブロックで記す。

### L1 — NAND 完全性 (効率性を妥協する版, 古典)
- **言明**: $\{\mathrm{NAND}\}$ は 2 値ブール関数の関数完全集合である (Sheffer 1913)。ゲート数・回路深さ・配線複雑度に上界を設けなければ、任意の有限ブール関数 $f:\{0,1\}^n\to\{0,1\}^m$ は有限 NAND 回路で実現可能。
- **前提**: 古典 (Sheffer 1913, Post 1941)。
- **証明スケッチ**: 真理表展開 → 主加法標準形 (DNF) → ¬, ∧, ∨ を NAND で表現 (L2 参照)。
- **bitRAG 内対応**: 本書の構成的下敷き全体。

### L2 — ∧/∨/¬/⊕ の NAND 還元 ★(本書の構成的核)
- **言明**:
  - $\neg x = \mathrm{NAND}(x,x)$
  - $x \wedge y = \mathrm{NAND}(\mathrm{NAND}(x,y), \mathrm{NAND}(x,y))$
  - $x \vee y = \mathrm{NAND}(\mathrm{NAND}(x,x), \mathrm{NAND}(y,y))$
  - $x \oplus y = \mathrm{NAND}(\mathrm{NAND}(x,\mathrm{NAND}(x,y)), \mathrm{NAND}(y,\mathrm{NAND}(x,y)))$ (NAND 4 個)
- **前提**: L1。
- **証明スケッチ**: 真理表確認 (4 行)。bit 並列拡張は word 単位の独立コピー。
- **bitRAG 内対応 — 構成的最小実例**:
  `bitrag-core::sign2` モジュール (`mul`, `neg`, `add_saturating`) は本補題の上記 4 式 (`nand`, `xor_via_nand`, `and_via_nand`) のみで $\{-1, 0, +1\}$ 上の乗法・否定を構成する。テスト `mul_full_table_excluding_spare` / `neg_is_bit_swap` / `add_saturating_table` で全数検証済 (PASS)。

#### L2 補項 — サイン 2bit 符号 (`bitrag_core::sign2`) の代数的位置付け
ビット対 $(h, \ell) \in \{0,1\}^2$ を値 $v(h, \ell) := \ell - h \in \{-1, 0, +1\}$ と読む。

| $h\ell$ | $v$ | 意味 |
| :-: | :-: | :-: |
| 01 | $+1$ | Plus |
| 00 | $0$ | Zero |
| 10 | $-1$ | Minus |
| 11 | $0$ | Spare (第二の零) |

**性質 (すべて L2 の bit 演算で表現可能)**:
1. **否定**: $-v(h,\ell) = v(\ell, h)$ — 上位/下位ビットの **入替** (`Sign2::neg` = 2bit リバース)。
2. **乗算**: 非 Spare 上で $(h_a, \ell_a) \ast (h_b, \ell_b) = (s, m \oplus s)$, ただし $m = (h_a \oplus \ell_a) \wedge (h_b \oplus \ell_b)$, $s = (h_a \oplus h_b) \wedge m$。$\oplus, \wedge$ はともに L2 で NAND 還元される。
3. **直和分解**: $h$ = 符号チャネル, $\ell$ = 大きさ存在チャネル。両者は **直交** (1 bit 反転で他方に影響しない)。
4. **誤り耐性**: ハミング距離 $d_H(\mathrm{Plus}, \mathrm{Minus}) = 2$。1 bit 誤りで符号反転が起こらない。
5. **自由フラグ**: 4 → 3 写像のファイバー上に余剰 1 bit (Spare) が浮き、零位置に付帯チャンネルとして 512bit/ブロックまで利用可能。

**意義**: L2 のみで $\{-1, 0, +1\}$ 上の乗法モノイドが閉じる ⇒ 信号符号付き計算が NAND 完備性の射程に入る。これは T1 (構成定理) の「最小可動標本」である。

### L3 — 語彙幅有限
- **言明**: $V \le 64 \cdot n_w$, $\mathrm{DocBits} \in \{0,1\}^V$ は有限ベクトル空間。
- **bitRAG 内対応**: `bitset.rs` (`DocBits = Vec<u64>`)。

### L4 — `shift_k` の配線実現
- **言明**: $\mathrm{shift}_k$ は NAND を含まない純粋な配線置換 (wire permutation)。
- **bitRAG 内対応**: `bitset::shift`。

### L5 — 測度 $\mu$ と IDF² 加重
- **言明**: $\mu(S) = \sum_{i \in S} \mathrm{idf\_sq}[i]$ は半/全加算器の整数加算ツリー。
- **bitRAG 内対応**: `idf::IdfPlanes`, `eval::row_sum_stats`。

### L6 — 循環 XOR 圧縮 $E$ ★(本書の構成的核)
- **言明**: A4 の写像 $E:\text{bytes} \to \{0,1\}^{1024}$ は $\oplus$ (L2) と $\mathrm{shift}_k$ (L4) の有限合成。
- **bitRAG 内対応 — 構成的最小実例**:
  `bitrag-core::nibble4_tokenizer` (Task #121, 11 単体テスト + 58/58 = 100% round-trip 確認済) は **可逆** 1024bit ブロック化を bit 操作のみで実装する。
  `bitrag-core::iso` (本書同行) はさらに 1024bit ブロックを **4 通りの読み方** で同型に行き来させる:

  $$
  \underbrace{[u_{64};\,16]}_{\text{物理 } 1024\text{bit}}
  \;\cong\;
  \underbrace{\text{byte}\times 128}_{\text{文字符号}}
  \;\cong\;
  \underbrace{\text{nibble}\times 256}_{\text{意味原子}}
  \;\cong\;
  \underbrace{\text{Sign2}\times 512}_{\text{論理 } \pm 1}
  $$

  各同型射 (`words_to_bytes`, `bytes_to_sign2_block` 等) は単体テスト `fourfold_iso_chain` で完全可逆 (PASS)。L6 の循環 XOR 圧縮は **同じビット列に対する別言語の読みを切替える操作** として L2+L4 に還元される。

### L7 — `sim_shift` / `xcorr`
- **言明**: $\mathrm{sim\_shift} = \sum_k \mathrm{decay}^{|k|} \cdot \mu(A \cap \mathrm{shift}_k(B))$ は L2 + L4 + L5 の合成。$\mathrm{decay}^{|k|}$ は前計算定数テーブル。

### L8 — `mask_idf` / $q_B$
- **言明**: 閾値判定 (比較器) と量子化はビット比較器 + AND マスク。

### L9 — `fiction_α`
- **言明**: LFSR ベース擬似乱数 (NAND 構成可) と OR マスクの合成。決定的種 (seed) は前計算定数。

### L10 — 合成文法 §4 の Plan 評価
- **言明**: BNF Plan は L2〜L9 の有限合成。

| OpExpr ノード | 依存補題 |
| :-- | :-: |
| `And` / `Or` / `Not` / `Xor` | L2 |
| `Shift(k)` | L4 |
| `MaskIdf(τ)` | L5 + L8 |
| `Fiction(α, seed)` | L9 |
| `Sim(decay)` | L2 + L4 + L5 + L7 |
| `E(bytes)` | L2 + L4 + L6 |
| `Quantize(B)` | L5 + L8 |

### L11 — オラクル境界
- **言明**: A5 (`rustc`) は外部プロセスを呼ぶため本仮説の対象外。

---

## 階層 2. 補題定理化 (Lemma → Theorem 格上げ) — M1〜M5

依存ツリー:

```
L1 ──┐
L2 ──┼─→ M1 ──┐
L4 ──┘        │
L5 ──→ M2 ────┤
L6 + L7 ─→ M3 ┼─→ M5 (BNF Plan)
L9 ────→ M4 ──┘
```

- **M1 (集約)**: L2 + L4 → $\{\wedge, \vee, \neg, \oplus, \mathrm{shift}_k\}$ 上の有限式は有限 NAND 回路。
- **M2 (測度)**: L5 + M1 → $\mu$ を含む有限式は NAND 回路。
- **M3 (圧縮)**: L6 + L7 → $E$ と `sim_shift` の合成は NAND 回路。
- **M4 (擬似乱数)**: L9 → `fiction_α` は決定的種固定で NAND 回路。
- **M5 (合成文法)**: L10 + M1〜M4 → §4 BNF Plan の任意評価木は NAND 回路。

---

## 階層 3. 主定理 (背理法 + 構成的自明性) — T1, T2, **C-T2**, T3, T4

### T1 (構成定理)
- **言明**: 公理 A5 を除く bitRAG の全意味演算と合成文法は、効率性を問わなければ有限 NAND 回路で実現できる。
- **依存補題**: M1〜M5。
- **証明スケッチ**: M1〜M5 の合成。本書は深さ・面積の上界は与えない (L1 の射程と整合)。

### T2 (背理法による完全性定理)
- **言明**: H0 が成立する。
- **依存補題**: A0, L1。
- **証明スケッチ**:
  1. H0' を仮定: NAND で構成不能な bitRAG 演算 $f$ が存在。
  2. A0 より $f$ の入力空間は $\{0,1\}^V$ (有限) であり、出力もブール値または有限 bit 幅整数。
  3. ゆえに $f$ は有限ブール関数または有限ブール関数の有限直積。
  4. L1 (効率性を妥協した NAND 完全性) より、任意の有限ブール関数は有限 NAND 回路で実現可能。
  5. ゆえに $f$ は NAND 構成可能、これは仮定 H0' に矛盾。
  6. **したがって、最初に NOT を取った行為 (H0' を立てた仮定) こそが誤り** であり、H0 が成立する。$\blacksquare$

### C-T2 (T2 の系 — 「NOT は不要 △QED」) ★(本書の中核成果)

T2 の証明過程を再読すると、A0 と L1 のみで H0 は **直接** 成立する:

> A0 ⇒ bitRAG の全演算は有限ブール関数。
> L1 ⇒ 有限ブール関数 ⊆ 有限 NAND 回路で実現可能集合。
> ∴ H0。

すなわち背理法のために導入した否定操作 H0' は **本質的に不要** であり、bitRAG が **論理演算のみで構成 (または構成を目標と) するという設計拘束 A0** が H0 を構成的に保証する。背理法は H0 の真理を確認する補助手段であって、H0 そのものは A0 の論理的帰結である。$\triangle$ **QED**

**修辞的内容**: 「NOT は不要」とは、メタレベルで H0' を取った瞬間、A0 と矛盾する空集合を見ているにすぎないことを意味する。
**数学的内容**: A0 ∧ L1 ⊨ H0 (含意の単一ステップ)。

**意義**: bitRAG は **「NAND 完全性を満たすかを問う対象」ではなく「NAND 完全性を設計拘束として宣言した系」** として再公理化できる。本タスクは「証明」というより「再公理化の確認」と解釈してよい。

### T3 (深さ最適性, 補助注記)
- **注記**: T1 の構成は L1 の射程内で深さ上界を与えない。副次的に、最小深さを問えば $\mu$ 加算 (L5) と xcorr 加算 (L7) が $\log V$ で支配される。本タスクは深さ最適性は射程外。

### T4 (謎のアセンブラ整合性)
- **言明**: project_goal の「bit 配置探索による Rust エラー検出・修正提案」アセンブラは T1 + C-T2 によって NAND 回路として実装可能。
- **意義**: 回路探索 (CEGAR / SAT) の射程に入り、Task #1 §8 の Q1 (一級 API), Q7 (標準演算列の最適性) は **回路探索問題** として再定式化できる。

---

## 階層 4. 課題目標 (Problem Goals — 残る本質的課題)

- **PG1 (オラクル境界)**: A5 は回路外。回路出力 → rustc 入力のインターフェイス (truth value channel) の形式化が必要。
- **PG2 (測度 bit 幅)**: $\mu$ の bit 幅 (idf_sq の bit 幅) は L5 加算ツリーの深さに直結。Task #1 §8 Q9 (語彙拡大) と連動定量化。
- **PG3 (擬似乱数の決定性)**: L9 の LFSR タップ選択が §8 Q2 (`fiction_α` の公理化) と重なる。
- **PG4 (A0 自体の射程 — 実コード監査)**: 下表参照。

### PG4 監査表 — `bitrag-core/src/*.rs` の f32/f64 混入箇所

| ファイル | 一致行数 | A0 整合性 | 備考 |
| :-- | :-: | :-: | :-- |
| `gold_cycle.rs` | 33 | A0 違反候補 | スコア計算で実数演算。論理化 or 整数化が課題。 |
| `bitset.rs` | 28 | A0 違反候補 | 一部 f64 混入。整数 IDF² 化と重複検証要。 |
| `eval.rs` | 26 | A0 違反候補 | gini / top_k 等の評価指標。回路化対象外でも可。 |
| `idf.rs` | 21 | A0 違反候補 | IDF 計算 (前計算定数化で回避可)。 |
| `matrix.rs` | 19 | A0 違反候補 | row_normalize / ppr。線形代数を整数化する課題あり。 |
| `nibble_hash.rs` | 8 | A0 違反候補 | ハッシュ補正の実数項。整数版実装の検討課題。 |
| `object_capsule.rs` | 7 | A0 違反候補 | カプセル係数。Task #84 系の射程。 |
| `sign2.rs` | 1 | **整合 (false positive)** | コメント文字列のみ (`f32/f64 を使わず`)。 |
| `nibble4_tokenizer.rs` | 0 | **整合** | 純論理。本書 L6 の構成的最小実例。 |
| `iso.rs` | 0 | **整合** | 純論理。本書 L6 の同型射群。 |

**所見**: A0 厳密遵守は `nibble4_tokenizer`, `iso`, `sign2` (NAND 一族) で達成済。残るモジュールは `f32/f64` 演算を含むが、これらは (a) 評価指標や事前計算で回路本体ではない、(b) 整数化可能、のいずれか。修正は別タスク。

---

## 階層 5. 中間目標 (Intermediate Goals — 達成チェックポイント)

| ID | 内容 | 本書中の対応 |
| :-: | :-- | :-- |
| IG1 | A0 の明文化と A1〜A5 への配置案 | 階層 0 末尾の段落 |
| IG2 | L1〜L4 を古典結果引用 + 構造的議論 | L1〜L4 |
| IG3 | L5〜L8 の bit 並列回路スケッチ | L5〜L8 (Sign2/iso 経由で具体化) |
| IG4 | L9 を LFSR タップ定数固定で記述 | L9 |
| IG5 | L10 の §4 BNF → 補題マッピング表 | L10 表 |
| IG6 | L11 のオラクル境界明示 | L11 |
| IG7 | T1〜T4 + C-T2 を 3 ブロックで記述 | 階層 3 |
| IG8 | PG4 実コード監査結果 (1 表) | PG4 表 |
| IG9 | 旧 #107 系譜 + 旧 #108 文献を付録に内包 | 付録 A, B |

---

## 階層 6. 到達目標 (Final / Destination Goals)

- **FG1**: 本書に階層 0〜5 が完備され、T2 と C-T2 の双方が成立する。 ✅
- **FG2**: C-T2 によって、bitRAG が「NAND 完全性を設計拘束として宣言した系」として再公理化されることが明文化されている。 ✅ (階層 3 末尾)
- **FG3**: Task #1 §8 未解決問題 Q1, Q3, Q5, Q7 への対応:

| Task #1 §8 Q | 本書の制約 |
| :-: | :-- |
| Q1 (一級 API) | T4 + M5 — BNF Plan は NAND 回路に落ち、回路探索の API 化が射程内。 |
| Q3 (合成代数の閉性) | T1 + M1〜M5 — A5 を除き全演算が有限 NAND 回路で閉じる。 |
| Q5 (測度の選択) | L5 + PG2 — $\mu$ の bit 幅選択が深さ上界に直結 (L5)、未確定問題は PG2。 |
| Q7 (標準演算列の最適性) | T3 (注記) + T4 — 深さ・ゲート数最適化は回路探索 (SAT/CEGAR) として再定式化。 |

- **FG4**: 既存 `THEORY_RUST_CODEGEN.md` と並列参照可能で、本書は **公理層の上に乗る回路層** として位置づけられる。 ✅
- **FG5**: 旧 Task #107 (系譜), #108 (文献) の意図を付録 A, B に内包し、独立タスク化不要。 ✅
- **FG6**: T4 の延長として「謎のアセンブラ」プロトタイプ実装が次の独立タスクとして発議できる状態。 ✅

---

## 付録 A — 系譜 (旧 Task #107 軽量版)

```
Task #1  (THEORY_RUST_CODEGEN.md / 公理 A1〜A5)
   │
   ├─→ #107 系譜整理 ─────────┐
   ├─→ #108 式抽出 + 文献 ───┤  → 本書 #114 が包含
   │                          │
   ├─→ E152〜E163e (派生実験群) │
   │   └─ E164 〜 E166         (OODA bootstrap → nibble4 tokenizer)
   │
   ├─→ #114  NAND 完全性証明 (本書, A0 / H0 / T2 / C-T2)
   │       ├─ L2 構成的最小実例: bitrag-core::sign2 (Plus/Zero/Minus/Spare; NAND 一族)
   │       └─ L6 構成的最小実例: bitrag-core::nibble4_tokenizer + iso (4 重同型)
   │
   └─→ #121  UTF-8 ニブル4 1024bit トークナイザ (実装, 58/58 round-trip 100%)
```

**主要ブランチの taskRef とタイトル**:
- #1 — bitRAG Rust コード生成セマンティクス理論
- #84 — ObjectCapsule (非可逆) — A0 境界事例
- #107 — 系譜整理 (本書 §付録 A に統合)
- #108 — 式抽出 + 文献調査 (本書 §付録 B に統合)
- #113 — OODA bootstrap (E165 で再現確認)
- #114 — 本書
- #121 — nibble4 1024bit 可逆トークナイザ (本書 §L6 構成的最小実例)

---

## 付録 B — 文献マップ + ギャップ表 (旧 Task #108 軽量版)

| 補題 | 古典文献 (1〜3 件) | bitRAG 独自貢献余地 |
| :-: | :-- | :-- |
| L1 | Sheffer (1913); Post (1941) | 構成性のみを問い「効率性を妥協する」明示宣言。 |
| L2 | Mano & Ciletti (Digital Design); Kohavi (Switching) | $\{-1, 0, +1\}$ サイン 2bit 符号 (`sign2`) を NAND 一族で構成し A0 の最小実例化。 |
| L3 | Knuth TAOCP vol.4A (bitwise tricks) | 1024bit = 16 × u64 を「論理 ±1 / 意味原子 / 文字符号」の **4 重同型** として読み替える `iso` モジュール。 |
| L4 | Knuth TAOCP vol.4A §7.1 (shift) | 配線置換のみで non-NAND 化。 |
| L5 | Wallace (1964, fast multipliers); IDF (Sparck Jones 1972) | IDF² 整数加算ツリー化 (PG2 と連動)。 |
| L6 | XOR ハッシュ系 (Carter & Wegman 1979) | nibble4 1024bit 可逆トークナイザ (Task #121) を bitRAG 入口層に固定。 |
| L7 | 巡回相互相関 (FFT 系古典) | shift_k + IDF² 加算ツリーで $O(\log V)$ 深さの整数版。 |
| L8 | 比較器 (Cormen CLRS §8) | 閾値量子化を AND マスクで均質化。 |
| L9 | LFSR (Golomb 1967) | 決定的種固定で擬似乱数を回路化。 |
| L10 | BNF (Backus 1959) | OpExpr → 補題マッピング (本書 L10 表)。 |
| L11 | 外部オラクル一般 (Turing 1939) | rustc を回路境界外として明示形式化。 |

---

## 整合性チェック (Steps 10)

- 引用公理: A0 (本書) / A1〜A5 (`THEORY_RUST_CODEGEN.md` §1) — 実在確認済。
- 引用実験: E152〜E166 / 主要は E165 (OODA bootstrap), E166 (nibble4 検証) — `artifacts/bitrag/experiment-1{52..66}*` 実在確認済。
- 引用 taskRef: #1, #84, #107, #108, #113, #114, #121 — 実在確認済 (#107, #108 は本書に内包済)。
- 引用ファイル: `bitrag-core/src/{sign2,iso,nibble4_tokenizer,bitset,idf,matrix,gold_cycle,nibble_hash,object_capsule,eval}.rs` — 実在確認済。

## 付録 (E172 / E173, 2026-04-21)

- **E172**: byte ラベル (ASCII / lead2/3/4 / cont) × slot 分布の実測で、`slot 3 = Spare(11) ⇔ UTF-8 多バイト lead` が 3 コーパス全件 (700,000 + 31,771 + 32,214 byte) で precision = recall = 1.000 を達成。Sign2 表現は『2bit/シンボル』に加えて『先頭バイト位置のメタ情報』を構造的に内包することが確定 (axiom: spare_as_boundary_marker proved)。
- **E173**: 整数 / bit 演算のみの 3 段可逆コーデック — Stage A (位置マスク剥離: all-ASCII フラグ + 7bit packing) / Stage B (digram 残差 RLE: 直前 byte 連続を 6bit run code) / Stage C (Spare サイドチャネル冗長性確認) — をブロック単位モード選択 (M0 raw / M1 7bit / M2 RLE) で実装。6044/6044 round-trip 完全可逆、合算 8.2% 圧縮で E169 の digram 相互情報量上限 5.6% を +2.6 pt 超過。Rust UI コーパスでは M2 採択 7.14% で 20.7% 圧縮に到達。Huffman / 算術符号を一切使わず構造的不変条件 (上位 bit=0) のみで digram 上限を破ったことを示す。

### PG / T4 (E174, 2026-04-23): GF(2) 上の Moore-Penrose 疑似逆 (構成定理)

bitRAG の順方向行列 $A \in \{0,1\}^{N \times V}$ ($N=$NDOC=16, $V=2^{16}$) について、F2 上で $AA^\top$ が非特異 ($AA^\top \in \mathrm{GL}_N(\mathrm{GF}(2))$) のとき、

$$A^+ := A^\top \cdot (AA^\top)^{-1} \in \mathrm{GF}(2)^{V \times N}$$

は Penrose 4 条件 $(1)\,AA^+A=A,\,(2)\,A^+AA^+=A^+,\,(3)\,(AA^+)^\top=AA^+,\,(4)\,(A^+A)^\top=A^+A$ をすべて F2 上で **完全に** 満たす (実測ハミング差異 = 0 bit / サンプル不一致 = 0 / 4096)。証明は $AA^+ = I_N$ から自動: (1)(2)(3) は明示、(4) は $A^+A = A^\top(AA^\top)^{-1}A$ で $(AA^\top)^{-1}$ が対称 (対称行列の逆は対称) ゆえ。

実測 (E174 v3 NDOC=16, qsub JOB 9 DONE(0)): GitHub Rust コーパス (popcount~5%) で構成成立 → 全 4 条件 = 0。法律 / Mixture (popcount~25%) では $AA^\top$ が GF(2) 特異 ($\det \bmod 2 = 0$) → 構成不可。

実測 (E174b NDOC=8, qsub JOB 10 DONE(0)): 法律コーパス (popcount~30%) で構成成立 → 全 4 条件 = 0。GitHub Rust (popcount~9%) と Mixture では特異 → 構成不可。**NDOC と invertibility の関係は非単調**: NDOC=16 では Rust ✓ / Law ✗ だが NDOC=8 では Law ✓ / Rust ✗ と反転する。

実測 (E174c Mixture NDOC スイープ, qsub JOB 11 DONE(0)): NDOC ∈ {4, 12} では特異, NDOC ∈ {20, 24, 32} で非特異 → 全 4 条件 = 0 (構成成立)。これにより **3 コーパスすべてで適切な NDOC を選べば F2 上の完全 Moore-Penrose 構成が達成可能** であることが実機証明された (Law@NDOC=8, Rust@NDOC=16, Mixture@NDOC≥20)。コーパスごとの最適 NDOC は popcount 比 (saturation) と緩く相関し ~5–15% 帯で非特異化する傾向を示すが, 厳密な閾値は非単調で構造的決定不能。

検証手順注記: 条件 (1)(2)(3) は全要素のハミング差異を実測 (= 0 bit), 条件 (4) は $V \cdot N = 2^{20}$ 要素のため 4096 サンプル対称性チェック (不一致 0 / 4096) で確認。$\mathbb{Z}$ 上の adjugate 路は $\det$ が i128 範囲外 ($\sim 10^{57}$, Hadamard 上限) で本ビルドでは未踏 — bigint 必要。axiom `MP_pseudoinverse_in_F2`: constructive_when_AAt_invertible (corpus 依存, NDOC 非単調).

$\blacksquare$
