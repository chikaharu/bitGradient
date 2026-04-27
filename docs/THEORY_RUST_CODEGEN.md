# bitRAG Rust コード生成セマンティクス理論

> **v1 凍結 (2026-04-25, e176-c01)**: 凍結核は `THEORY_CORE_v1.md`。
> 本書は v1 凍結時点の長文版で、本文は変更しない。
> 以後の検証追記は `THEORY_EVIDENCE_LEDGER.md` に行うこと。

## 0. 本書の射程 (定義)

本書で「セマンティクス理論」と呼ぶものは次の二軸の同時定義を指す。

- **軸 A — 入力の構造化規則**: bitRAG をコード生成器とみなしたとき、自然言語クエリ・不完全コード・`rustc` エラーといった非定型入力を、bitRAG 中核 (`DocBits` / `IdfPlanes` / `ObjectCapsule`) が直接処理できる「クエリ bitset + 演算列」へ機械的に翻訳する規則。
- **軸 B — アルゴリズムの bit 列最小表現**: 生成対象となる Rust 上のアルゴリズム (関数・型・修正パッチ) を、`ObjectCapsule` の 1024-bit と DocBits 上の n-gram 集合の二層で「これ以上削れない最小単位」に圧縮する規則。

本書はこの二軸を貫く公理・パーツ・合成代数・合成文法を提示する。コード変更・実験実行・数値記録は含まない (replit.md 第一節 禁止事項を遵守)。

---

## 1. セマンティクス公理

bitRAG の bit 空間 (語彙幅 V, ワード幅 nw, V ≤ 64·nw) における「意味」を以下で定義する。

### 公理 A1 (gram-bit 同型)
gram → bit 位置の写像 `idx: G → {0..V-1}` は単射であり、文書 D の意味は集合 S(D) = { idx(g) | g ∈ gram_set(D) } と同一視できる。
すなわち D の意味は bitset 一つに尽きる (C01, C02, C03)。

### 公理 A2 (IDF² 加重)
gram g の意味的重みは `idf(g)² = (ln((N+1)/(df(g)+1)) + 1)²` で与えられる。
集合演算は常に IDF² 加重された測度 μ を持つ:
```
μ(S) = Σ_{i ∈ S} idf_sq[i]
```
バイナリ Jaccard はこの重みを 1 に潰した特殊系である。

### 公理 A3 (SHIFT 近傍 = 語彙的近傍)
gram インデックスは辞書順で割り当てられているため、`B >> k` / `B << k` は「語彙的に k だけずれた gram 集合」を表す (C03 sim_shift / C10 xcorr 参照)。
よって畳み込み
```
sim_shift(A, B) = Σ_{k=-K}^{K} decay^|k| · μ(A ∩ shift_k(B))
```
は OOV gram を語彙近傍で救済する作用素である。

### 公理 A4 (循環 XOR 圧縮) — ☆ 加法性は反証済 (Task#111)
任意のバイト列はカプセル化作用素 `E: bytes → {0,1}^1024` (C13 `ObjectCapsule::encode`) で固定長 1024-bit に圧縮できる。
距離はハミング距離 (`xnor_l1`) で与えられる。

**改訂 (Task#111):** 当初 A4 は「`E` は加法的 (循環 XOR): `E(x⊕y) = E(x)⊕E(y)`」と
公理化していたが、`bitrag-core::object_capsule::tests::test_capsule_additivity_proptest`
にて 1024B × 1000 ペア / 512B-1024B 長さ違い × 200 ペアで実測した結果、
`E(x⊕y)` と `E(x)⊕E(y)` のハミング距離は平均 ≈ 512 / 1024 (= ランダム期待値)、
zero_ratio = 0 となり、加法性は完全に成立しないことが示された。
理由は `bit_addr(i, g) = (4i + g) mod 1024` が gram 値 `g` を整数として用いるため、
GF(2) 上の線型性が壊れる点にある。`E` は今後**非線型ハッシュ圧縮** (SimHash / Bloom 系)
として扱い、加法性に依存する派生 (軸 B 三つ組 T13、E84/E91 系の同値判定など) は
ハミング距離の近傍判定に限定する。詳細は `THEORY_FORMALIZATION.md` §2 T08。

### 公理 A5 (オラクル一価性)
`rustc` は判定オラクルである。任意の候補ソース `s` に対し `(error_count, stderr) = rustc_run(s)` は決定的であり、`error_count = 0` を真理値「コンパイル可能」とする。
意味の正しさはこの真理値に対してのみ最終評価される (gold_cycle::rustc_run)。

---

## 2. セマンティクスパーツ一覧

「コード生成器 bitRAG への入力」を構成する最小単位 (semantic part) を以下に列挙する。各パーツは「DocBits 上の bit 集合」+ 「随伴する IDF² マスク」+ 「随伴する `ObjectCapsule` (任意)」の三つ組として定義される。

| パーツ | 記号 | 内容 | DocBits 構築規則 | カプセル |
|--------|------|------|------------------|----------|
| `UseSet` | U | use 宣言・外部 path 集合 | `gram_set(use 行のみ)` を `vocab.to_docbits` | 不要 |
| `TypeSet` | T | 型名 (識別子・ジェネリクス) 集合 | n=3,4 gram のみ (n=2 は破棄) を投影 | 任意 |
| `FnSig` | F | 関数シグネチャ (名・引数型・戻り型) | n=4 gram + `nibble_to_type` キーワードを強制 set | 必須 (C13 decode_to_stub の逆問題) |
| `Body` | B | 関数本体 (式・制御構造) | n=2,3,4 全て、IDF² の上位を mask_idf で抽出 | 任意 |
| `ErrSig` | E | rustc エラーメッセージ要約 | エラーコード `E\d{4}` と span 周辺 1 行を gram 化 | 不要 |
| `FixHint` | H | 修正候補ヒント (`help:` / `note:`) | 同上、IDF² 重みを 2 倍ブースト | 不要 |
| `ContextWindow` | C | カーソル前後 ±N 行 | 行単位で `gram_set` し union | 任意 |
| `IOExample` | X | 入出力例 (E103 入力) | 入力テキスト・期待出力テキスト両方を gram 化し or 結合 | 不要 |
| `GoldRef` | R | 既知の正例ソース (E59 gold) | `to_nib_seq` + `gold_encode_indices` で位相空間へ | 不要 |

各パーツは独立に DocBits を持ち、後段の合成代数で結合される。空入力は空 bitset として明示的に保持する (暗黙的な代替値による黙殺は禁止)。

---

## 3. 合成代数

パーツ間の合成は次の演算で閉じる (DocBits の語彙幅は共通 V)。

| 演算 | 記法 | 定義 | 意味論 |
|------|------|------|--------|
| 連言 | `P ∧ Q` | ビット単位 AND (DocBits.words) | 両方に現れる意味のみ残す (合流) |
| 選言 | `P ∨ Q` | ビット単位 OR | どちらかに現れる意味を採用 (拡張) |
| 排他 | `P ⊕ Q` | ビット単位 XOR | 差分のみ抽出 (差し込み点候補) |
| 差 | `P ⊖ Q` | `P AND NOT Q` | Q を取り除く (マスク) |
| 近傍 | `shift_k(P)` | `DocBits::shift_left/right(k)` | 語彙距離 k 内の意味を許容 (公理 A3) |
| 虚構 | `fiction_α(P)` | E48 由来。確率 α で擬似 bit を挿入 | コンパイル誘導下で OOV を仮設する |
| マスク | `mask_idf(P, τ)` | `idf(g) ≥ τ` の gram のみ残す | 高 IDF gram に集中させる (C06 `build_mdoc_idf_masked` と同義) |
| 量子化 | `q_B(P)` | `IdfPlanes::build(., B)` | B-bit 平面に分解 (C04) |
| カプセル化 | `E(text)` | `ObjectCapsule::encode` | 1024-bit に潰す (公理 A4) |
| 距離 | `d(c1, c2)` | `xnor_l1` | カプセル間ハミング距離 |

これらは可換半環 (∧, ∨) と Z₂ ベクトル空間 (⊕) の構造を併せ持つ。`shift_k` と `mask_idf` は線型作用素であり、∧/∨/⊕ と分配的に交換する (実装: 全て bit 並列)。

---

## 4. 合成文法 (BNF 風)

「自然言語クエリ・不完全コード・rustc 出力」を入力とし、最終 DocBits Q を生成する文法を以下に与える。

```
Query        ::= NLPart? CodePart? ErrPart?
NLPart       ::= "@nl" Text
CodePart     ::= "@code" RustText                 ; 不完全コード可
ErrPart      ::= "@err" RustcStderr

PartSeq      ::= Part ("," Part)*
Part         ::= UseSet | TypeSet | FnSig | Body | ErrSig
               | FixHint | ContextWindow | IOExample | GoldRef

OpExpr       ::= Atom
               | OpExpr "∧" OpExpr
               | OpExpr "∨" OpExpr
               | OpExpr "⊕" OpExpr
               | OpExpr "⊖" OpExpr
               | "shift_" Int "(" OpExpr ")"
               | "fiction_" Float "(" OpExpr ")"
               | "mask_idf_" Float "(" OpExpr ")"
               | "q_" Int "(" OpExpr ")"
Atom         ::= Part | "(" OpExpr ")"

Plan         ::= PartSeq "⇒" OpExpr "⇒" "Q"
```

### 標準コンパイル規則 (クエリ → プラン)

1. `NLPart` → `IOExample` ∨ `Body` (n=3,4 の高 IDF gram のみ)
2. `CodePart` → 構文的に `UseSet`, `TypeSet`, `FnSig`, `Body`, `ContextWindow` に分解
3. `ErrPart` → `ErrSig` + `FixHint` (`help:` 行から)
4. 既定の演算列は次の通り:
   ```
   Q = mask_idf_τ(
         (FnSig ∨ TypeSet ∨ UseSet)
         ∧ shift_1( Body ∨ ContextWindow )
       ) ∨ fiction_α( ErrSig ⊕ FixHint )
   ```
   ここで τ, α はタスクごとに固定 (本書では値は与えない、A5 のオラクル評価で決める)。

---

## 5. ユースケース展開

### 5.1 新規関数生成

入力: 自然言語「u32 の Vec を昇順ソートして中央値を返す関数」。

```
Plan:
  P1 = IOExample("[1,3,2] -> 2")
  P2 = TypeSet({"Vec<u32>", "u32"})
  P3 = FnSig({name="median", args=["Vec<u32>"], ret="u32"})
  Q  = mask_idf_τ(P3 ∨ P2) ∧ shift_1(P1)
期待挙動:
  - DocBits Q をコーパスに対し jaccard_idf で検索 → 中央値・ソート系スニペット上位
  - greedy_cover_idf で gram 被覆を最大化する k 件を選択
  - 各候補を ObjectCapsule にカプセル化、xnor_l1 で重複排除
  - 最終的に decode_to_stub が C13 経由で stub を返す
```

### 5.2 rustc エラー修正提案

入力: `cannot borrow ... as mutable` エラー + 当該関数本体。

```
Plan:
  P1 = ErrSig (E0502 と span 1 行)
  P2 = FixHint (`help: consider ...`)
  P3 = Body (当該関数)
  P4 = ContextWindow (±5 行)
  Q  = (P1 ∨ P2)              ; エラー核
       ∧ shift_2(P3 ∨ P4)     ; 文脈の語彙近傍
       ⊖ mask_idf_τ_low(P3)   ; ありふれた gram を引く
期待挙動:
  - Q を gold (E59) 集合に対し sim_shift_idf で検索 → 修正パターン上位
  - GOLDCYCLE walk (BinField/NibField) を P3 に適用、Phase1 で xcorr-peak シフト k* を確定
  - apply_nib_pos で nibble 単位のパッチ候補列を生成、A5 オラクルで採否判定
```

### 5.3 既存スニペット合成 (E103 拡張)

入力: I/O 例 (X) のみ。

```
Plan:
  P1 = IOExample (入力・期待出力)
  P2 = R (E59 gold の中で popcount が中央値帯にあるもの上位)
  Q  = q_8(P1) ∨ shift_1(P2)
期待挙動:
  - C04 IdfPlanes (B=8) で文書スコアを Σ idf(g) 近似算出 (高速)
  - 上位文書を ObjectCapsule にカプセル化し、E(P1) との xnor_l1 で再ランク
  - 最終候補 1 件を decode_to_stub もしくはそのまま提示
```

---

## 6. 軸 B — アルゴリズムの bit 列最小表現

ここまでが「入力の構造化」(軸 A) であった。軸 B は「生成対象アルゴリズム自体の最小表現」を定める。

### B-1. 二層表現
任意の Rust アルゴリズム a は次の二層に圧縮される:

- **下層 (疎)**: `gram_set(a)` を `Vocab` に投影した DocBits — 語彙レベルの意味、長さ V bit、密度低
- **上層 (密)**: `ObjectCapsule::encode(a.as_bytes())` の 1024-bit — 構造レベルの指紋、密度高

### B-2. 最小表現の不変量
a の最小表現 ⟦a⟧ は次の三つ組として定義する。
```
⟦a⟧ = ( DocBits(a) , ObjectCapsule(a) , k* )
       ─疎─           ─密─              ─位相─
```
ここで `k*` は gold 集合 R に対する `xcorr` のピークシフト位置であり、a を gold 位相空間へ整列するための単一スカラ。

### B-3. 同値類
二つのアルゴリズム a, a' が ⟦·⟧ 同値であるとは、
```
DocBits(a) = DocBits(a')           (語彙的同値)
xnor_l1(E(a), E(a')) = 0           (構造的同値)
k*(a) = k*(a')                     (位相同値)
```
が同時に成立すること。これは `rustc` 等価ではないが、bitRAG の検索/生成器内では同一視される。

### B-4. 最小性
- DocBits は「これ以上 gram を捨てると jaccard_idf でランクが落ちる」点で局所最小。
- ObjectCapsule は固定 1024-bit で大域最小 (これ以下では公理 A4 の同型が崩れる)。
- 三つ組は `(V/8 + 128 + 4)` バイトに収まり、1 関数あたり高々 ~1 KB のオーダー。

---

## 7. C01–C13 マッピング表

軸 A のパーツ・演算と既存コンポーネントの実装対応。

| 理論側 | 実装側 (component / API) |
|--------|---------------------------|
| 公理 A1 (gram→bit) | C01 `ngram::gram_set` + C02 `Vocab::build` |
| 公理 A2 (IDF²) | C02 `Vocab::idf_sq` |
| 公理 A3 (SHIFT 近傍) | C03 `DocBits::sim_shift` / C10 `xcorr*` |
| 公理 A4 (循環 XOR 圧縮) | C13 `ObjectCapsule::encode` (内部 C12 `n4_gram_circ`) |
| 公理 A5 (オラクル) | `gold_cycle::rustc_run` |
| `UseSet` / `TypeSet` / `Body` | C01 + C02 + C03 (`to_docbits`) |
| `FnSig` | C13 (`decode_to_stub` の逆) + C01 |
| `ErrSig` / `FixHint` | C01 + C02 のみ (カプセル不要) |
| `ContextWindow` | C01 を行単位で適用後 C03 で union |
| `IOExample` | C01 + C04 (`IdfPlanes`) |
| `GoldRef` | `gold_cycle::{gold_seq, build_phi, gold_encode_indices}` |
| `∧ / ∨ / ⊕ / ⊖` | C03 `DocBits::words` のビット単位演算 (実装小, 未公開 API として要追加) |
| `shift_k` | C03 `shift_left/right` |
| `fiction_α` | E48 系 (現状単体実装、API 未統一) |
| `mask_idf` | C06 `build_mdoc_idf_masked` の thresholding 部 |
| `q_B` | C04 `IdfPlanes::build(., B)` |
| `E (encode)` | C13 |
| `d (ハミング)` | C13 `xnor_l1` |
| ランキング/被覆 | C08 `top_k_idx` / `greedy_cover_idf` |
| 多段伝播 | C07 `ppr` / `hop` |
| u16 グリッド ID | C11 + P17 `pack_nibble_id` |

---

## 8. 未解決問題 (次の実験案: 問いの形)

実験番号は割り当てない。理論側のギャップを問いとして列挙する。

1. **DocBits 上の `∧ / ∨ / ⊕ / ⊖` を一級 API として公開すべきか?**
   現状は `words` への直接アクセスで間に合っているが、合成代数を文法層から呼ぶには不便。
2. **`fiction_α` の確率 α と挿入 bit の選び方に関する公理化は可能か?**
   現状は実験 (E48) ごとに個別実装。`mask_idf` との合成順序も未定義。
3. **`ObjectCapsule` の 1024-bit は「最小」と言えるか?**
   公理 A4 では 1024 を所与 (仮定) としているが、512 / 2048 との情報量比較が無い。ハミング距離の分散と語彙数 V の関係は?
4. **位相 `k*` (軸 B-2) は真にスカラで十分か?**
   xcorr のピークが多峰の場合、`(k₁*, k₂*, ...)` への一般化が必要かどうか。
5. **`ContextWindow` の窓幅 N と `shift_k` の k は独立に選べるか?**
   両者はともに「近傍」を表すが、片方を増やせばもう片方を減らせる可能性がある。
6. **`ErrSig` / `FixHint` を IDF² ブーストするときの倍率はオラクルからどう推定するか?**
   現状はヒューリスティック (本書 §2 の「2 倍」)。A5 を逆算に使う仕組みが要る。
7. **合成文法 §4 の標準演算列 `Q = mask_idf_τ((F∨T∨U) ∧ shift_1(B∨C)) ∨ fiction_α(E⊕H)` は最適か?**
   各パーツの登場順序を入れ替えた場合の `rustc` pass 率は未測定。
8. **`GoldRef` を `gold_encode_indices` で位相空間に写したあと、DocBits 空間に戻す逆写像は閉じているか?**
   位相空間 (L=1023) と語彙空間 (V) の往復が情報を失わない条件を明記する必要がある。
9. **コーパス拡充 (例: TRPL / Rust by Example) が公理 A2 の IDF 分布をどう変えるか?**
   df 分布が偏ると `mask_idf` の τ が再調整必要。
10. **軸 B の同値類 (§B-3) は `rustc` 等価より粗いか細かいか?**
    粗ければ偽陽性、細かければ無駄な分割。実証的に測る設計が要る。

---

## 付録: 一覧 (要約)

- 公理 5 本: A1 (gram→bit) / A2 (IDF²) / A3 (SHIFT 近傍) / A4 (循環 XOR 圧縮) / A5 (オラクル)
- パーツ 9 種: UseSet / TypeSet / FnSig / Body / ErrSig / FixHint / ContextWindow / IOExample / GoldRef
- 演算 10 種: ∧ / ∨ / ⊕ / ⊖ / shift_k / fiction_α / mask_idf / q_B / E / d
- ユースケース 3 種: 新規関数生成 / rustc エラー修正 / E103 拡張スニペット合成
- 軸 B 三つ組: ⟦a⟧ = (DocBits, ObjectCapsule, k*)
