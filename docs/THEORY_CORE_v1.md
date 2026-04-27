# bitRAG 理論凍結核 v1 (Frozen Theory Core)

> **本書の位置付け**: 散逸する 5 本の長文 THEORY_*/MATH_FOUNDATIONS と
> E166–E175 までの検証結果を 1 枚に閉じる **凍結核**。新規主張は持たず、
> 既存条文の **採録のみ** で構成する。証明本体・反例・周辺議論は既存
> ドキュメント (§5 SHA1 を凍結) に委ねる。日々の検証は本書ではなく
> `THEORY_EVIDENCE_LEDGER.md` に追記する。

- **凍結日**: 2026-04-25 (e176-c01)
- **対応 OODA サイクル**: e176-c01 (origin task #26)
- **claim_id 規約**: 本書節番号と `THEORY_EVIDENCE_LEDGER.md` の `claim_id` 列が 1:1 対応

---

## §1 公理 (Axioms) — 採録のみ

bitRAG 理論の **動かさない出発点**。文言は出典の原文ママ。

### claim A0 — Logic-Only 公理 (`THEORY_NAND_COMPLETENESS.md` L10–L11)

> bitRAG の全意味演算は、定義域・値域とも有限ブール空間 ($\{0,1\}^V$ または
> その直積) 上の関数であり、**論理演算 (∧, ∨, ¬, ⊕, shift, mask 等) のみで
> 構成または構成を目標とする**。実数演算・解析演算・非決定的演算はオラクル
> 境界 (A5) の外側にしか現れない。

- 配置: A1 の **直前** (同書 L23 の配置案に従う)
- 運用上の含意: 新規コードの f32/f64 混入は A0 違反候補として PG4 監査表
  (`THEORY_NAND_COMPLETENESS.md` 階層 4) に従い処理する

### claim A1 — gram-bit 同型 (`THEORY_RUST_CODEGEN.md` §1)

> gram → bit 位置の写像 `idx: G → {0..V-1}` は単射であり、文書 D の意味は
> 集合 S(D) = { idx(g) | g ∈ gram_set(D) } と同一視できる。すなわち D の
> 意味は bitset 一つに尽きる (C01, C02, C03)。

### claim A2 — IDF² 加重 (`THEORY_RUST_CODEGEN.md` §1)

> gram g の意味的重みは `idf(g)² = (ln((N+1)/(df(g)+1)) + 1)²` で与えられる。
> 集合演算は常に IDF² 加重された測度 μ を持つ:
> `μ(S) = Σ_{i ∈ S} idf_sq[i]`
> バイナリ Jaccard はこの重みを 1 に潰した特殊系である。

### claim A3 — SHIFT 近傍 = 語彙的近傍 (`THEORY_RUST_CODEGEN.md` §1)

> gram インデックスは辞書順で割り当てられているため、`B >> k` / `B << k`
> は「語彙的に k だけずれた gram 集合」を表す (C03 sim_shift / C10 xcorr 参照)。
> よって畳み込み
> `sim_shift(A, B) = Σ_{k=-K}^{K} decay^|k| · μ(A ∩ shift_k(B))`
> は OOV gram を語彙近傍で救済する作用素である。

### claim A4 — 循環 XOR 圧縮 (`THEORY_RUST_CODEGEN.md` §1, 加法性は反証済 Task#111)

> 任意のバイト列はカプセル化作用素 `E: bytes → {0,1}^1024`
> (C13 `ObjectCapsule::encode`) で固定長 1024-bit に圧縮できる。距離は
> ハミング距離 (`xnor_l1`) で与えられる。
>
> **改訂 (Task#111):** 当初 A4 は「`E` は加法的 (循環 XOR):
> `E(x⊕y) = E(x)⊕E(y)`」と公理化していたが、…(中略)…
> 加法性は完全に成立しないことが示された。…`E` は今後
> **非線型ハッシュ圧縮** (SimHash / Bloom 系) として扱い、加法性に依存
> する派生はハミング距離の近傍判定に限定する。

### claim A5 — オラクル一価性 (`THEORY_RUST_CODEGEN.md` §1)

> `rustc` は判定オラクルである。任意の候補ソース `s` に対し
> `(error_count, stderr) = rustc_run(s)` は決定的であり、`error_count = 0`
> を真理値「コンパイル可能」とする。意味の正しさはこの真理値に対してのみ
> 最終評価される (gold_cycle::rustc_run)。

---

## §2 主定理 (Main Theorems) — ステートメントのみ採録

証明本体は出典に委ねる。証明スケッチを再掲しない。

### claim T1 — 構成定理 (`THEORY_NAND_COMPLETENESS.md` 階層 3)

> 公理 A5 を除く bitRAG の全意味演算と合成文法は、効率性を問わなければ
> 有限 NAND 回路で実現できる。
>
> 依存補題: M1〜M5 (= L1〜L11 の集約)。

### claim T2 — 背理法による完全性定理 (`THEORY_NAND_COMPLETENESS.md` 階層 3)

> H0 が成立する。すなわち bitRAG の全意味演算 (A0 に従うもの) は、
> 効率性を問わなければ有限個の NAND ゲートで構成できる。
>
> 依存補題: A0, L1。

### claim C-T2 — T2 の系「NOT は不要」(`THEORY_NAND_COMPLETENESS.md` 階層 3)

> A0 ∧ L1 ⊨ H0 (含意の単一ステップ)。背理法のために導入した否定操作 H0'
> は本質的に不要であり、bitRAG が論理演算のみで構成 (または構成を目標
> と) するという設計拘束 A0 が H0 を構成的に保証する。
>
> 意義: bitRAG は **「NAND 完全性を満たすかを問う対象」ではなく
> 「NAND 完全性を設計拘束として宣言した系」** として再公理化できる。

### claim T3 — 深さ最適性 (補助注記) (`THEORY_NAND_COMPLETENESS.md` 階層 3)

> T1 の構成は L1 の射程内で深さ上界を与えない。副次的に、最小深さを
> 問えば μ 加算 (L5) と xcorr 加算 (L7) が log V で支配される。本タスクは
> 深さ最適性は射程外。

### claim T4 — アセンブラ整合性 (`THEORY_NAND_COMPLETENESS.md` 階層 3)

> project_goal の「bit 配置探索による Rust エラー検出・修正提案」アセンブラ
> は T1 + C-T2 によって NAND 回路として実装可能。回路探索 (CEGAR / SAT) の
> 射程に入り、Task #1 §8 の Q1 (一級 API), Q7 (標準演算列の最適性) は
> **回路探索問題** として再定式化できる。

### §2 補助定理リスト (採録参照のみ — 本文は出典)

凍結時点で確定とみなす補助補題群。文言は再録せず、出典行番号で参照する。

| claim_id | 名称 | 出典 |
|----------|------|------|
| L1 | NAND 完全性 (Sheffer 1913) | `THEORY_NAND_COMPLETENESS.md` L31–L35 |
| L2 | ∧/∨/¬/⊕ の NAND 還元 (構成的核) | 同 L37–L66 |
| L3 | 語彙幅有限 | 同 L67–L69 |
| L4 | `shift_k` の配線実現 (NAND 不要) | 同 L71–L73 |
| L5 | 測度 μ と IDF² 加重の加算ツリー | 同 L75–L77 |
| L6 | 循環 XOR 圧縮 E (構成的核, iso 同型射) | 同 L79–L95 |
| L7 | sim_shift / xcorr | 同 L97–L98 |
| L8 | mask_idf / q_B | 同 L100–L101 |
| L9 | fiction_α (LFSR + OR マスク) | 同 L103–L104 |
| L10 | 合成文法 §4 の Plan 評価 | 同 L106–L117 |
| L11 | オラクル境界 (A5 は対象外) | 同 L119–L120 |
| M1 | L2+L4 の集約 | 同 L137 |
| M2 | L5 + M1 (測度) | 同 L138 |
| M3 | L6 + L7 (圧縮) | 同 L139 |
| M4 | L9 (擬似乱数, 決定的種) | 同 L140 |
| M5 | L10 + M1〜M4 (BNF Plan) | 同 L141 |
| T01 | `idx` の単射性 | `THEORY_FORMALIZATION.md` L68–L73 |
| T02 | IDF² の正値・有界性 | 同 L75–L80 |
| T03 | μ の測度性 | 同 L82–L88 |
| T04 | Jaccard の有界性 | 同 L90–L94 |
| T05 | (DocBits, ∧, ∨, ⊕, ⊖) の代数構造 | 同 L96–L104 |
| T06 | shift_k 線型作用素 | 同 L106–L114 |
| T07 | xcorr の対称性 | 同 L116–L122 |
| T08 | E の加法性 — **反証済** (Task #111) | 同 L124–L143 |
| T09 | d がハミング距離 | 同 L145–L149 |
| T10 | rustc オラクル一価性 | 同 L151–L157 |
| T11 | mask_idf が μ の単調性を保つ | 同 L159–L163 |
| T12 | IdfPlanes::sum_bits の近似精度 | 同 L165–L169 |
| T13 | 軸 B 三つ組同値関係 | 同 L171–L178 |
| T14 | B-bit 量子化が argmax を保存する条件 | 同 L180–L185 |
| T15 | 標準規則の単調最適性 — **弱支持** (E164) | 同 L187–L200 |
| T2.1 | AND popcount = 包含測度 | `MATH_FOUNDATIONS.md` L92 |
| T4.1 | ペナルティマスク飽和 | 同 L158 |
| C4.1 | ランキング収束 | 同 L166 |
| T4.2' | C4.1 精緻化 | 同 L188 |
| T5.1 | 2-ホップ拡散 (グラフ拡散) | 同 L235 |
| T6.1 | 直交分離 (S_gs ⊥ R) | 同 L277 |
| T6.2 | CS はランク 1 行列 | 同 L298 |
| T7.1 | headScore 値域 [-8, 8] | 同 L373 |
| L7.2 | XOR キャンセル条件 (改訂版) | 同 L377 |
| T9.1 | nibble hash 均一性 | 同 L476 |
| T9.2 | nibble hash 近似独立性 | 同 L481 |
| T10.1 | エラー数単調非増加 | 同 L543 |
| T10.2 | 有限ステップ停止 (MAX_STEPS=25) | 同 L563 |
| T10.3 | 探索効率上界 (AUC 解釈) | 同 L582 |

---

## §3 不変条件 (Invariants) — 凍結

bitRAG 実装側で **常に成立する** べき構造的拘束。違反は実装バグまたは
理論側の v2 起票事由とする。

### claim INV-1024 — ブロック幅は 1024bit

任意のオブジェクトカプセル / Sign2 ブロック / nibble4 ブロックは物理
1024 bit (= `[u64; 16]` = 128 byte) に固定する (`THEORY_NAND_COMPLETENESS.md`
L86, `THEORY_RUST_CODEGEN.md` 公理 A4)。

### claim INV-ISO — 4 重同型射 (`THEORY_NAND_COMPLETENESS.md` L83–L95, `bitrag-core::iso`)

同一の 1024 bit 列について次の 4 つの読みを単体テスト
`fourfold_iso_chain` で完全可逆に行き来できる:

```
[u64; 16] (物理)
  ≅ byte × 128 (文字符号)
  ≅ nibble × 256 (意味原子)
  ≅ Sign2 × 512 (論理 ±1)
```

各同型射 (`words_to_bytes`, `bytes_to_sign2_block` 等) は L6 に従い
L2 + L4 (= NAND + 配線) のみで構成される。

### claim INV-NIBBLE4 — nibble4 トークナイザの可逆性

`bitrag-core::nibble4_tokenizer::{encode,decode}` は対象 58 ファイル
(corpora 3 群) に対して `encode → decode == identity` を満たす
(11 単体テスト + ラウンドトリップテスト, OODA `e166-c01`)。

### claim INV-SIGN2 — Sign2 4 値分布の構造

ビット対 (h, ℓ) で値を v(h, ℓ) := ℓ − h ∈ {−1, 0, +1, 0(spare)} と
読む (`THEORY_NAND_COMPLETENESS.md` L48–L65, `bitrag-core::sign2`)。
否定は 2bit リバース、乗算・加算飽和は L2 に従い NAND 還元される。

### claim INV-BITADDR — bit_addr ツリー (`bit_addr(i,g) = (4i + g) mod 1024`)

`bit_addr` は gram 値 g を整数として用いるため GF(2) 線型性を持たず
(T08 反証の根拠)、`E` を非線型ハッシュ圧縮 (SimHash / Bloom 系) として
扱う。これは A4 改訂と整合する。

### claim INV-NO-FLOAT — A0 由来: 新規コードに f32/f64 を混入しない

新規追加するモジュール / 実験 / scheduler 拡張は整数および bit 演算のみで
構成する。既存の f32/f64 混入箇所は PG4 監査表
(`THEORY_NAND_COMPLETENESS.md` 階層 4) で個別管理する。

---

## §4 凍結宣言 (Freeze Declaration)

1. **本書 (THEORY_CORE_v1.md) の本文は v1 として凍結する。** 文言の差し替え
   は禁止。誤記訂正であっても本書では行わず、`THEORY_EVIDENCE_LEDGER.md`
   側に「v1 採録時の表記揺れ」として記録する。
2. **理論本体を差し替える必要が生じた場合は v2 を新設する** (THEORY_CORE_v2.md
   を別ファイルとして作成し、v1 を歴史層として保存)。v1 ファイルへの上書きは
   行わない。
3. **新規定理・新規公理・新規不変条件の追加** は本書では行わず、
   THEORY_CORE_v2 起票時にまとめて議論する。
4. **検証結果 (E176 以降) はすべて `THEORY_EVIDENCE_LEDGER.md` に追記する。**
   本書のクレーム文言を変えずに verified / partial / falsified の status だけ
   が動く。
5. **既存 5 本 (THEORY_FORMALIZATION / THEORY_LINEAGE / THEORY_NAND_COMPLETENESS
   / THEORY_RUST_CODEGEN / MATH_FOUNDATIONS) は本書の長文版** として温存し、
   各冒頭にリダイレクトブロックを置く。本文の書き換え・短縮・統合は禁止。
6. **凍結核 v1 が動かない限り bitRAG 理論は閉じている** とみなす。

---

## §5 v1 凍結時点の参照ドキュメント SHA1

下記 SHA1 と本書のクレーム採録は 1:1 対応する。これらのファイル本文が
変わった場合は本書の整合性を再確認し、v2 起票の要否を判定する。

| ファイル | SHA1 |
|----------|------|
| `artifacts/bitrag/THEORY_FORMALIZATION.md`     | `d495c497698f9e52d9cff05cd4f4222d54a007b9` |
| `artifacts/bitrag/THEORY_LINEAGE.md`           | `8d014ea5baaad0a851cf6ca04f7204466d0cb0de` |
| `artifacts/bitrag/THEORY_NAND_COMPLETENESS.md` | `9d7ef12915c1589aa60001f4c1136c7af12bb32e` |
| `artifacts/bitrag/THEORY_RUST_CODEGEN.md`      | `ada4a3125a5bcd2565cf050ba6f5e1b071fef7f9` |
| `artifacts/bitrag/MATH_FOUNDATIONS.md`         | `decb05b27947fc49347c117225eb930f31acaee1` |

> 注: 上記 SHA1 は v1 凍結時 (e176-c01) に冒頭リダイレクトブロック挿入後の値。
> 凍結前 (リダイレクトブロック挿入前) の値は履歴として `OODA_history.json`
> e176-c01 サイクルの `before_state.sha1_pre_redirect` に保存する。

> 再算出は `cd artifacts/bitrag && sha1sum THEORY_FORMALIZATION.md
> THEORY_LINEAGE.md THEORY_NAND_COMPLETENESS.md THEORY_RUST_CODEGEN.md
> MATH_FOUNDATIONS.md` で行う。SHA1 が一致しない場合は本書冒頭リダイレクト
> ブロック挿入以外の改変が混入している可能性があるので確認する。

---

*v1 凍結: 2026-04-25 / OODA: e176-c01 / origin task: #26*
