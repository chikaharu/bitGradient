# bitRAG コンポーネントカタログ

bitRAG Core の独立コンポーネント一覧・F(入力)→出力 形式・依存グラフ。

各エントリの形式:
```
F: 演算カテゴリ (POPCOUNT / SHIFT / XNOR / ADD-ROL / 表引き / cyclic-XOR)
入力型 → 出力型
例外: 空入力・境界条件の扱い
```

---

## コンポーネント一覧

### C01 — ngram_extractor
**ファイル**: `bitrag-core/src/ngram.rs`
**API**: `ngrams(text: &str, n: usize) -> Vec<String>`, `gram_set(text: &str) -> BTreeSet<String>`
**役割**: テキストから n-gram 集合 (n=2,3,4) を生成する基盤コンポーネント。
**F**: 文字列スライス  
**入力型**: `&str, n: usize`  
**出力型**: `Vec<String>` / `BTreeSet<String>`  
**例外**: n > text.len() → 空コレクション
**依存**: なし

---

### C02 — vocab_builder
**ファイル**: `bitrag-core/src/idf.rs` (`Vocab` 構造体)
**API**: `Vocab::build(gram_lists, n_docs, n_bins) -> Vocab`
**役割**: コーパス全体の df・IDF²・bin 量子化を一括構築する語彙管理コンポーネント。
**F**: カウント集計 + log 変換 + 量子化  
**入力型**: `&[BTreeSet<String>], n_docs: usize, n_bins: usize`  
**出力型**: `Vocab`  
**例外**: n_docs=0 → idf = ln(1+ε) で下界クランプ
**依存**: C01

---

### C03 — docbits
**ファイル**: `bitrag-core/src/bitset.rs` (`DocBits` 構造体)
**API**: `DocBits::new(nw)`, `set(i)`, `jaccard_binary`, `jaccard_idf`, `sim_shift`, `xcorr`, `xcorr_idf`
**役割**: gram インデックスを bit 位置とする固定幅 bitset。SHIFT + POPCOUNT による
畳み込み類似度・相互相関関数を提供する中核コンポーネント。
**F**: SHIFT + POPCOUNT (jaccard_binary / jaccard_idf) | cyclic SHIFT + POPCOUNT (xcorr)  
**入力型**: `DocBits × DocBits` / `DocBits × DocBits × &[f32]`  
**出力型**: `f32` (スコア) / `Vec<(i32, u32)>` (相互相関)  
**例外**: nw=0 → popcount=0; ゼロ除算は分母クランプで回避
**依存**: C01, C02

---

### C04 — idf_planes
**ファイル**: `bitrag-core/src/idf.rs` (`IdfPlanes` 構造体)
**API**: `IdfPlanes::build(idf_sq, nw, b_bits) -> IdfPlanes`, `sum_bits`, `sum_inter`, `pair_jaccard`
**役割**: IDF 値を B-bit に量子化したビット平面表現。bit 積で Σ idf(g) を近似計算。
**F**: 量子化 (floor × scale) + bit 平面 AND + POPCOUNT  
**入力型**: `&[f32], nw: usize, b_bits: usize`  
**出力型**: `IdfPlanes` (build) / `f32` (スコア)  
**例外**: b_bits=0 → 精度なし (0 を返す)
**依存**: C02

---

### C05 — corpus_loader
**ファイル**: `bitrag-core/src/corpus.rs`
**API**: `load_corpus(path: &str) -> Vec<String>`
**役割**: テキストコーパスファイルを行単位で読み込む。
**F**: ファイル I/O  
**入力型**: `&str` (ファイルパス)  
**出力型**: `Vec<String>`  
**例外**: ファイル不在 → panic (fail-loud)
**依存**: なし

---

### C06 — similarity_matrix
**ファイル**: `bitrag-core/src/matrix.rs`
**API**: `build_mdoc_binary`, `build_mdoc_idf`, `build_mdoc_idf_masked`, `build_mdoc_shift`, `build_mdoc_shift_idf`
**役割**: 文書間類似度行列を各種手法で構築する行列ビルダー群。
**F**: POPCOUNT (binary) / POPCOUNT + IDF 加重 / SHIFT + POPCOUNT (shift)  
**入力型**: `&[DocBits], n: usize` (+Vocab)  
**出力型**: `Vec<f32>` (n×n 対称行列、row-major)  
**例外**: n=0 → 空ベクタ
**依存**: C02, C03

---

### C07 — ppr_propagator
**ファイル**: `bitrag-core/src/matrix.rs`
**API**: `row_normalize(m, n)`, `ppr(m, v0, lambda, steps) -> Vec<f32>`, `hop(m, v, n) -> Vec<f32>`
**役割**: Personalized PageRank (PPR) と多段ホップ伝播。OOV gram の 2 ホップ救済に使用。
**F**: 行正規化 (divide) + ベクトル行列積 (MUL+ADD)  
**入力型**: `&[f32]` (行列) × `&[f32]` (ベクトル) × スカラー定数  
**出力型**: `Vec<f32>`  
**例外**: 行和=0 の行はそのまま保持 (ゼロ除算ガード)
**依存**: C06

---

### C08 — evaluator
**ファイル**: `bitrag-core/src/eval.rs`
**API**: `gini`, `top_k`, `top_k_idx`, `shorten`, `reach_count`, `row_sum_stats`, `long_query_score`, `greedy_cover`, `greedy_cover_idf`
**役割**: スコア評価・ランキング・Greedy Set Cover による文書選択。
**F**: ソート + インデックス操作  
**入力型**: `&[f32]` (スコア配列)  
**出力型**: `Vec<(f32, usize)>` / `usize` / `f32`  
**例外**: 空スコア配列 → 空/0 を返す
**依存**: C03

---

### C09 — idf_bins
**ファイル**: `bitrag-core/src/idf.rs` (`Vocab::to_bins`)
**API**: `Vocab::to_bins(grams) -> Vec<Vec<u64>>`
**役割**: 文書 gram_set から IDF 量子化 bin 分解 (K=8) を生成。
`DocBits::jaccard_idf_bins` と組み合わせて高速近似類似度を計算。
**F**: 量子化 + bit 分割  
**入力型**: `&BTreeSet<String>`  
**出力型**: `Vec<Vec<u64>>` (K 個の bit 平面)  
**例外**: 空 gram_set → 各平面が全 0
**依存**: C02, C03

---

### C10 — xcorr_scorer
**ファイル**: `bitrag-core/src/bitset.rs`
**API**: `DocBits::xcorr`, `DocBits::xcorr_idf`, `DocBits::xcorr_to_score`
**役割**: bit 積の相互相関関数 (binary / IDF 加重)。
語彙空間シフトによる OOV 近傍補完を提供する。
**F**: cyclic SHIFT + POPCOUNT + decay 加重 (xcorr_to_score)  
**入力型**: `&DocBits × &DocBits` / + `&[f32]` (idf_sq) / + `f32 decay`  
**出力型**: `Vec<(i32, u32)>` / `Vec<(i32, f32)>` / `f32`  
**例外**: max_k=0 → k=0 のみ返す; decay 適用後 0 はスキップ
**依存**: C03

---

### C11 — nibble_hash_matrix
**ファイル**: `bitrag-core/src/nibble_hash.rs`
**API**: `nibble_hash_matrix(data: &[u8]) -> (hash_hi: u8, hash_lo: u8)`
**役割**: ADD+ROL+XOR の 2×4 行列ハッシュ。任意バイト列を (hash_hi, hash_lo) の
2 バイトに圧縮する。16×16 グリッド座標への変換 (`hash_to_grid`) と
P17 nibble_id パッキング (`pack_nibble_id`) を含む。
**F**: ADD → ROL → XOR (2 rows × 4 cols) + 2 段クロスミキシング  
**入力型**: `&[u8]`  
**出力型**: `(u8, u8)`  
**例外**: 空入力 → 長さシード初期値のみで演算 (panic なし)
**均一性**: 65536 2-byte 入力で hi/lo 全 256 値カバー, Pearson corr ≈ -0.006
**依存**: なし

---

### C12 — n4_gram_circ  *(NEW: E84)*
**ファイル**: `bitrag-core/src/object_capsule.rs`
**API**: `n4_gram_circ(bytes: &[u8]) -> Vec<u8>`
**役割**: UTF-8 バイト列を循環ビットストリームとして扱い、1-bit スライド窓で
4-bit gram を生成する。clz4/ctz4/inv4 (E83 sub1 spec.md §2 と同一実装) を内包。
**F**: cyclic-modular + SHIFT + POPCOUNT (clz4/ctz4)  
**入力型**: `&[u8]` (任意バイト列)  
**出力型**: `Vec<u8>` (各要素 ∈ 0..15, 長さ = 8 × bytes.len())  
**例外**: 空入力 → 空ベクタ (panic なし)
**内包 4-bit 演算**:

| 関数 | F | 入力 | 出力 |
|------|---|------|------|
| `inv4(x)` | XNOR(x,0) = (x XOR 0xF) & 0xF | `u8` (下位4bit) | `u8` (下位4bit) |
| `clz4(x)` | smear_right(x); 4−POPCOUNT | `u8` (下位4bit) | `u8` ∈ 0..4 (MBS→LBS 連続0数) |
| `ctz4(x)` | smear_left(x)&0xF; 4−POPCOUNT | `u8` (下位4bit) | `u8` ∈ 0..4 (LBS→MBS 連続0数) |

**依存**: なし

---

### C13 — ObjectCapsule  *(NEW: E84)*
**ファイル**: `bitrag-core/src/object_capsule.rs`
**API**: `ObjectCapsule::encode`, `xnor_l1`, `xnor_popcount`, `popcount`, `to_hex`, `decode_to_stub`
**役割**: 1024-bit ([u64; 16]) のオブジェクトカプセル。
Rust ソース文字列を XNOR/POPCOUNT/cyclic-XOR のみで操作可能な固定長ビット列に圧縮し、
カプセルから決定論的に Rust stub コードを生成する。
**F**: cyclic-XOR + POPCOUNT  

| メソッド | F | 入力型 | 出力型 | 例外 |
|---------|---|--------|--------|------|
| `encode(&[u8])` | n4_gram_circ + cyclic-XOR ビット写像 | `&[u8]` | `ObjectCapsule` | 空 → 全0カプセル |
| `xnor_l1(&Self)` | XOR + POPCOUNT (Hamming 距離) | `&ObjectCapsule` × 2 | `u32` ∈ 0..1024 | 同一 → 0 |
| `xnor_popcount(&Self)` | XNOR + POPCOUNT (一致ビット数) | `&ObjectCapsule` × 2 | `u32` ∈ 0..1024 | 同一 → 1024 |
| `popcount()` | POPCOUNT | `&ObjectCapsule` | `u32` ∈ 0..1024 | なし |
| `decode_to_stub()` | 表引き (nibble → Rust 型/識別子) | `&ObjectCapsule` | `String` (Rust stub) | 常に妥当な stub |

**依存**: C12

---

## パッキング仕様

### P17 — nibble_id_spec
**ファイル**: `bitrag-core/src/nibble_hash.rs` (`pack_nibble_id`)
**API**: `pack_nibble_id(hash_hi: u8, hash_lo: u8, c_class: u8, p_class: u8) -> u16`
**役割**: u16 = 4×u4 パッキング規則。C11 の出力を 16×16 グリッド座標として u16 単一フィールドにエンコードする。

```
u16 nibble_id の構造:
bits 15-12 (nibble3): P系 (位置・頻度分類)
bits 11- 8 (nibble2): hash_hi の上位4bit (16×16 グリッド行座標)
bits  7- 4 (nibble1): hash_lo の上位4bit (16×16 グリッド列座標)
bits  3- 0 (nibble0): C系 (カテゴリ・語彙分類)
```

**依存**: C11

---

## 依存グラフ

```
C05 ────────────────────────────────────────────┐
C01 ───────────────────────────────────┐        │
     C02 ──────────────────────┐       │        │
          C03 ────────┬──┬─────┼───────┤        │
               C04    │  │     │       │        │
               C09    │  │     │       │        │
               C10    │  C08   C06─────C07      │
                                ↑         ↑      ↓
                           [evaluation] [ppr] [corpus]

C11 (独立) ──→ P17
C12 (独立) ──→ C13
C12 ──(E84)──→ ObjectCapsule カプセル化 + Rust stub 生成
```

---

## 実験別コンポーネント使用状況

| 実験 | 使用コンポーネント |
|------|----------------|
| 21-23 | C01, C02, C03, C10 |
| 16-17 | C01, C02, C03, C06, C07 |
| 18 | C01, C02, C03, C04, C06, C08 |
| 34 | C01, C02, C03, C08 |
| 56 | C01, C11, P17 |
| 83 sub1 | C12 (clz4/ctz4/inv4, n4_gram_circ のみ) |
| 84 | C12, C13 |
| 91 | gold_cycle::gold_seq, C12 (n4_gram_circ), XOR-butterfly tree, MBS-Gg-LBS framer |
