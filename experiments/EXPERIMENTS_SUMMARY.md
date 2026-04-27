# bitRAG 実験まとめ（実験11〜36, E48, E49〜E55, E103）

---

## E103: experiment-103 — I/O→bitRAG 検索のみで Rust コード合成

**結果: rustc 通過 6 / 6 (うち 1 件は use 6 + type 3 + fn 3 を保持した実合成)**

### 仮説
LLM を使わず、E93 IDF² + IncrementalTokenizer で得たコーパス検索だけで、
ヒットしたコード片を「use → type → fn → main」の順に連結 + 重複排除し、
rustc を通らない場合は最低密度のスニペットから順にドロップするだけで、
rustc 構文/型チェックを通る Rust ソースを合成できる。

### 設計
```
dataset-rustc-ui/pass/*.rs (200 件)
  → split_items(): brace+string-aware 字句スキャナで top-level item 分割
                   ⇒ 817 スニペット (Use=349, Type=128, Fn=340)
  → tokens_of():    3-byte FNV-1a n-gram → u64
  → IdfBuilder:     df → idf² (純整数, ilog2)  vocab=8,299

クエリ → tokens → score = Σ_{t∈q∩s} idf²[t]
       → density-rank: score × 1000 / |snippet.tokens|
       → topK per kind (Use=6, Type=3, Fn=3)

synthesize_greedy():
  loop (max=20):
    1. 連結 → rustc --emit=metadata
    2. ok? → return
    3. else: 最低密度の Type/非main Fn → Use → main の順にドロップ
```

### サンプルクエリ結果
| query                       | iters | kept (u/t/f) | rustc | 備考 |
|-----------------------------|------:|--------------|:-----:|------|
| `fn main pub fn`            |     6 | 0/0/3        |   ✓   | 短い `pub fn foo()` × 3 を結合 |
| `trait A T fn main`         |     4 | 0/0/3        |   ✓   | `trait A<T:A<T>> {}` 系 |
| `use std use crate fn main` |    13 | 0/0/0        |   ✓   | stub に退化 |
| `struct field pub fn main`  |     6 | 0/0/3        |   ✓   | 短い fn 3 件 |
| `enum variant fn main`      |    12 | 0/0/0        |   ✓   | stub に退化 |
| **`const static fn main`**  |   **1** | **6/3/3**  | **✓** | **★実合成** (use 6 + static 3 + async fn 1) |

### 鍵となる工夫
- **density-rank**: 生スコア (Σ idf²) ではなく `score × 1000 / |snippet.tokens|` で
  ランク。長い fn 本体が「単に多くのトークンを含むから」上位に来るのを抑え、
  短く密にマッチするスニペットを優先 → 短い `pub fn foo()` 系が浮上、未解決
  シンボル参照を含む巨大 fn 本体が排除される。
- **drop priority**: `Type / 非main Fn` → `Use` → `main Fn` の順にドロップ。
  最後に main すら落とせば自動 stub 化されるため、最悪ケースで rustc 通過保証。
- **`use` 終端の特別扱い**: `use foo::{a, b};` の `}` で誤って分割されないよう、
  Use 種別では `;` のみで終端する。

### 出力
- `experiment-103/results/synth_<query>.rs` × 6
- `experiment-103/results/synth_log.tsv`
- `experiment-103/results/result.md`

### 次の実験 (novelHypothesis.md)
- **E104**: fn 本体の自由識別子集合 → 必要 use の bit-AND 補完で stub 退化を解消
- **E105**: I/O ペア合致まで踏み込んだ動作合格率測定
- **E106**: rustc stderr E0XXX エラーコードを bit table-lookup でルール化、
  greedy iters の短縮

---

## E55: experiment-55 / ensemble29 — 5種変異 + 先頭トークン 4×4 u4 行列スコア

**スコア: 51/56** (退行なし、変異種拡張確認)

### 変更点 (ensemble28 → ensemble29)

**拡張1: 5種変異操作**

| 変異種 | 説明 | 観測結果 |
|--------|------|----------|
| **SNV** | 置換 rem→add (edit-dist=1 の場合 -1.0 ボーナス) | 引き続き主要変異; `[8:mut_type=SNV]SNV1:res→ret` |
| **INSERT** | error スパン行先頭にトークンを行挿入 | type-unsatisfiable.rs で動作確認 |
| **DEL_F** | extra[0] (スパン先頭エラートークン) の最初の出現を削除 | 前方デリーション |
| **DEL_B** | extra[-1] (スパン末尾エラートークン) の最後の出現を削除 | 後方デリーション |
| **CNV** | rem → rem rem (inline 複製) | 試行済み |

**拡張2: 先頭トークン 4×4 u4 行列 XOR 自己埋め込みスコア (δ=0.2)**

スパン先頭エラートークン `headTok = extra[0]` の nibHex 先頭バイト上位4bit (b0..b3) を取り出し、4×4 双極行列 H を構成する:
```
row[0] = [b0, b1, b2, b3]              (nibble 原値)
row[1] = [b1, b2, b3, b0]              (左ローテーション 1)
row[2] = [b2, b3, b0, b1]              (左ローテーション 2)
row[3] = [b3, b0, b1, b2]              (左ローテーション 3)
各要素: 0 → -1, 1 → +1 (双極化)
```

XOR ハッシュ行で自己埋め込み:
```
xorHash[j]    = row[0][j] XOR row[1][j] XOR row[2][j] XOR row[3][j]
H_embed[i][j] = H[i][j] * sign(xorHash[j])   (sign: 0→-1, 1→+1)
headScore     = Σ_i (||H_embed[i]|| * dot(H_embed[i], addVec) / 4)
              ∈ [-8, 8]
```

addVec: addBits[0] の下位4bitから双極化した4次元ベクトル。

**スコア統合:**
```
score(mut) = penScore - α·nibbleSim - β·rcf2Score - γ·csScore - δ·headScore
α=1.0, β=0.5, γ=0.3, δ=0.2 (新規パラメータ)
```

**ログ形式:**
```
[step:mut_type=SNV]rem→add(H=N,pen=P/1024=X%,headSc=Y.YYY,sim=S,rf2=R,cs=C)
[step:mut_type=SNV]SNV1:rem→add(H=N,pen=P/1024=X%,headSc=Y.YYY,sim=S,rf2=R,cs=C)
[step:mut_type=DEL_B]DEL_B:tok@LN(H=N,pen=P/1024=X%,headSc=Y.YYY)
[step:mut_type=DEL_F]DEL_F:tok@LN(H=N,pen=P/1024=X%,headSc=Y.YYY)
[step:mut_type=INSERT]INSERT:tok@LN(H=N,pen=P/1024=X%,headSc=Y.YYY)
[step:mut_type=CNV]CNV:tok×2(H=N,pen=P/1024=X%,headSc=Y.YYY)
```
変異タイプ採用割合が実行末尾に出力される:
```
変異タイプ採用割合: SNV:N(X%) | DEL_B:N(X%) | DEL_F:N(X%) | INSERT:N(X%) | CNV:N(X%)
```

**buildH4 / h4Score 実行証跡 (node smoke test):**
```
headTok=Abc  H_embed → headSc(crate)=4.00, headSc(fn)=2.00   (δ·headSc: +0.80 / +0.40)
headTok=self H_embed → headSc(crate)=-4.00, headSc(fn)=-2.00  (δ·headSc: -0.80 / -0.40)
headTok=Zxq  H_embed → headSc(all)=0.00                       (全ビット同一でキャンセル)
スコア式: score = pen - 1.0·sim - 0.5·rf2 - 0.3·cs - 0.2·headSc  (δ=0.2)
```

### 高速化
- `toksBits` / `uniqBits` を oracleStep 先頭で一括事前計算 (tokBits 呼び出し削減)
- `pairScore` 関数でビット演算をインライン展開: rcf2Score/csScore を統合
- `Float32Array.subarray()` でゼロコピー行取得
- **rcf2Score バグ修正**: `unique[j%m]` → `toksBits[j*32..]` (正しい位置→ビット対応)

### コードアーキテクチャ
```
候補 = { type, score, apply:(src)→newSrc|null, log, headSc }
→ score = pen - α·sim - β·rf2 - γ·cs - δ·headSc
→ スコア昇順ソート → コンパイラオラクルで上位 MAX_CANDS=40 を評価
```

### 残り5件の現状
- type-unsatisfiable.rs: H=2 →stop (ins/bwd は H 変化なし)
- typeid-consistency.rs: H=3→H=2 →max (25ステップ完走、突破できず)
- type-ascription.rs: H=2 →stop
- type-check-defaults.rs: H=2 →stop
- verbose.rs: H=2 →stop

---

## E49: ビット幅比較実験（64〜1024bit）

**ディレクトリ:** `e49/`

### 仮説

E48 で 32bit フィンガープリントの「飽和問題」が発覚した（コンテキストビット C が 31/32 ≈ 97% 充填され、ランキングがトークン長の順位に退化）。ビット幅を広げることで飽和率が下がり、XNOR 類似度が意味的識別力を持ち始める閾値があるはずだ。

### 設計

```
BitSet = Vec<u64>  (幅 64/128/256/512/1024bit ← 定数ジェネリクス不要の実装)
ハッシュ関数: djb2 / FNV / poly31 / poly37  ← 4本で分散向上
to_bits_n(s, width): n-gram (n=1,2,3) × 4本 → bit位置 = hash % width

XNOR スコア = (popcount(!(C XOR T)) − 末尾ゴミビット) / width
飽和率      = popcount(C) / width

CLI:
  --bits 64|128|256|512|1024  (デフォルト 128)
  --compare                   (全幅一括比較テーブル)
  --top-k N                   (上位 N 候補, デフォルト 10)
  --json                      (JSON 出力)
```

### 実験結果（テンプレート: `fn main() { let x: ___ = 42; }`）

#### 飽和率テーブル

| ビット幅 | 飽和ビット数 | 飽和率  | コンパイル成功 | 所見                     |
|---------|-------------|---------|--------------|--------------------------|
| 64bit   | 63 bits     | 98.4%   | 0/5          | 【飽和】ランキング退化    |
| 128bit  | 116 bits    | 90.6%   | 0/5          | 【飽和】ランキング退化    |
| 256bit  | 165 bits    | 64.5%   | 0/5          | 【中飽和】実用範囲        |
| 512bit  | 207 bits    | 40.4%   | 1/5          | 【低飽和】識別力良好      |
| 1024bit | 241 bits    | 23.5%   | 1/5          | 【低飽和】識別力良好      |

#### 各ビット幅での上位候補の変化

```
64bit:   ✗Result<i32, String>(0.922)  ✗Option<String>(0.859)  ✗Vec<String>(0.828) ...
         → 長いトークン（ビット多く設定される）が上位を独占 ← トークン長バイアス

128bit:  ✗Result<i32, String>(0.773)  ✗Option<String>(0.641)  ✗Vec<String>(0.586) ...
         → 依然トークン長バイアス継続

256bit:  ✗Result<i32, String>(0.562)  ✗Option<i32>(0.508)  ✗Option<String>(0.508) ...
         → スコア差が縮まり識別力向上。適度な分散

512bit:  ✗42(0.615)  ✓i32(0.613)  ✗val(0.613)  ✗impl(0.613)  ✗f32(0.607) ...
         → i32 が上位2位に浮上（コンパイル成功）。短いプリミティブが競争可能に

1024bit: ✗42(0.774)  ✗x(0.768)  ✗n(0.768)  ✗val(0.768)  ✓i64(0.766) ...
         → スコアが集中（識別力低下）。高スコア帯に全候補が密集
```

#### スコア分布

| ビット幅 | max   | min   | spread | 所見               |
|---------|-------|-------|--------|--------------------|
| 64bit   | 0.922 | 0.750 | 0.172  | ⚠ 飽和・退化        |
| 128bit  | 0.773 | 0.523 | 0.250  | ⚠ 飽和・退化        |
| 256bit  | 0.563 | 0.473 | 0.089  | ◎ 適度な識別力      |
| 512bit  | 0.615 | 0.607 | 0.008  | △ 識別力低（集中）  |
| 1024bit | 0.774 | 0.766 | 0.009  | △ 識別力低（集中）  |

### 所見・考察

1. **飽和閾値**: ビット幅 256bit あたりで飽和率が ~65% まで下がり、意味的識別力が現れ始める。64/128bit は E48 と同様にコンテキストがほぼ全ビットを埋め尽くすため、短いトークンと長いトークンの区別ができない。

2. **i32 浮上**: 512bit で `i32` が上位2位（コンパイル成功）に浮上した。64/128bit では長い型名が上位を独占していたが、ビット幅を広げることでトークン長バイアスが緩和される。

3. **最適幅トレードオフ**:
   - 256bit: spread が最大（0.089）で識別力が最も高い
   - 512bit: 飽和率が低いが全候補スコアが密集（spread=0.008）
   - 1024bit: さらに密集（spread=0.009）＋メモリコスト増大

4. **推奨ビット幅**: コンテキスト飽和率 ≤ 75% かつ spread 最大の 256bit が実用的最適。さらに候補語彙が多い場合は 512bit が有効。

### 実行方法

```sh
cd artifacts/bitrag/e49
cargo build --release

# 単一幅モード（デフォルト 128bit）
./target/release/bitrag_e49 'fn main() { let x: ___ = 42; }'

# ビット幅指定
./target/release/bitrag_e49 --bits 512 'fn add(x: i32) -> ___ { x + 1 }'

# 全幅一括比較テーブル
./target/release/bitrag_e49 --compare 'fn main() { let x: ___ = 42; }'

# JSON 出力
./target/release/bitrag_e49 --bits 256 --json 'let _: ___ = vec![1, 2, 3];'
```

### 主要ファイル

- `e49/src/main.rs` — スタンドアロン Rust バイナリ（外部依存なし）
- `e49/Cargo.toml`

---

## E48: コンパイル誘導型虚構トークン探索（XOR 反パターン埋め込み）

**ディレクトリ:** `e48/`

### 設計

- Rust コード中の `___` プレースホルダーに候補トークンを虚構挿入
- `rustc --edition 2021` でコンパイルチェック（tmpファイル経由）
- 失敗した候補のビットセットを `XOR 0xFFFFFFFF`（全ビット反転 = NOT）で反パターン化
- 反転後の XNOR 類似度 = 1 - 元スコア（方向が完全に反転）

### ビット演算原理

```
コンテキストビット C = 32bit hash fingerprint(コード - プレースホルダー)
候補ビット         T = 32bit hash fingerprint(トークン)
XNOR スコア        = popcount(XNOR(C, T)) / 32

失敗後: T' = NOT(T)
XNOR スコア' = popcount(XNOR(C, NOT(T))) / 32
             = (32 - popcount(XNOR(C, T))) / 32
             = 1 - 元スコア  ← 方向の反転
```

### 実行方法

```sh
cd artifacts/bitrag/e48
cargo build --release

# stdin から
echo 'fn main() { let x: ___ = 42; }' | ./target/release/bitrag_e48

# CLI 引数
./target/release/bitrag_e48 'fn add(x: i32) -> i32 { x ___ 1 }'
```

### 出力

- 候補テーブル: トークン / 元 XNOR スコア / コンパイル結果 / 反転後スコア / 32bit ビット列
- 探索空間縮小グラフ: イテレーションごとの有効候補数 (ASCII バーチャート)
- XNOR スコア分析サマリー

### 主要ファイル

- `e48/src/main.rs` — スタンドアロン Rust バイナリ（外部依存なし）
- `e48/Cargo.toml`

---

## 実験36: 型推定インフラ² + 虚構挿入 + Shift+popcount bitset Jaccard

**ディレクトリ:** `experiment-36/`

### 設計
- TYPE × VAL 複式簿記 ledger gram (nibble u4, TYPE=6byte / VAL=4byte)
- インフラ行列 65 gram を rust 口座に事前投入 (corpus なし対応)
- **虚構挿入** (knownSet 事前拡張):
  - sim(Ti, Tj) ≥ θ=0.25 の型ペアで VAL を相互転写 → knownSet に事前挿入
  - TYPE 構造的類似 (Option<X>→Option<Y>、Vec<X>→Vec<Y> など) を一般化
- **Shift+popcount bitset Jaccard**:
  - nibble gram → 1024-bit bitset (32×Uint32、セグメント 0-3 各 256 bits)
  - bit = (segIdx << 8) | (gramInt & 0xFF)
  - `AND/OR + popcount32` で O(n²) 虚構挿入 + nearestType/Val を高速化
  - gram 折り畳み効果で構造的類似型の スコアが適切に増加
- **rust 動的参照**: `RUST_ID = typeVocab.get(nibHex('rust',6))` を各スコープで動的取得
- 5段フォールバック: ①exact(+虚構) → ②rust口座 → ③nearest(TYPE) → ④nearest(VAL) → ⑤両次元

### leave-one-out 評価 (35ファイル)
```
avg alien: 11.61%   total: 491 entries   alien: 57  (-36 vs ベースライン 93)

① exact       238  (48.5%)
② rust_fb      38  (7.7%)
③ type_sim      9  (1.8%)  ← bitset で 8倍に
④ val_sim     134  (27.3%)
⑤ both_sim     15  (3.1%)  ← bitset で 3倍に
→ alien        57  (11.6%)

実行時間: 0.436s
```

### 主要ファイル
- `type_ledger.mjs` — ledger 構築・虚構挿入・bitset 推定
- `loo_eval.mjs` — leave-one-out 評価
- `github_corpus/` — bore/htmlq/git-absorb/xh/starship の 35 .rs ファイル

---

## 実験37: (次実験用)

**ディレクトリ:** `experiment-37/`  
実験36のコピーから開始。

---

# bitRAG 実験まとめ（実験11〜24）

コーパス: 芥川龍之介 210文  |  n-gram (n=2,3,4)  |  語彙: 18,499 gram  |  bitset: 290 u64/文書

---

## 第1章: 虚構挿入（実験11〜12）

### 定義
```
Gv(α) = { g ∈ Gi∪Gj | score(g,α) > threshold }
score(g,α) = α·idf_w(g) + (1-α)·idf_e(g)
```

### 結果

| 相      | α範囲       | 変化                             |
|---------|-------------|----------------------------------|
| 停滞期  | 0.00〜0.64  | 103文のまま                      |
| 点火期  | 0.66〜0.82  | +1〜+7/0.02刻み                  |
| 爆発期  | 0.84〜1.00  | α=0.98 で +22文（最大爆発）      |

```
α* ≈ 0.66〜0.75 が虚構挿入の閾値
α=1.00: 到達 207/210 (98.6%)  Gv 拡張倍率: 326×
```

---

## 第2章: 仮想トークン空間（実験13）

```
VT_candidates = Gi_only × Gj_only = 143 × 182 = 26,026 対
うち仮想対（実共起なし）: 25,446 (97.8%)
```

**二層構造**: 語彙拡張(連続型) × 文書到達(ジャンプ型)

---

## 第3章: VT × クエリ冪乗（実験14）

```
VT¹_score = W_score × E_score   (片側ゼロ→全体ゼロ)
VT²_score = VT¹_score²          (アンカー両側 gram 識別)
```
bitset vs HashSet: ∞高速化（計測限界以下）

---

## 第4章: アンカー自動構築（実験15）

```
build_anchor(query, k=2) = top-2 文書 → gram 合併
k=2: |Gi|=270  (k=3 で変化なし → k=2 が実用最適)
k=2 で停滞期消滅: α=0.1 から文書が動き始める
```

---

## 第5章: M² OOV 救済（実験16〜17）

### 定義
```
M¹[q][i]   = jaccard(G_q, G_i)
M_doc[i][j] = jaccard(G_i, G_j)  事前計算 9ms
M²[q][j]   = Σ_i M¹[q][i] × M_doc[i][j]  2ホップ伝播
```

### Wave Front（完全OOV）
```
M¹: 1文書 → M²: 190文書(+189, 90.5%) → M³: 210文書
bitset vs HashSet: 250万倍高速
```

### ハブ問題
```
k≥4: 「ながら」ハブが PageRank 的に収束 → k=2 が実用最適
M_doc 行和: avg=3.376, max=5.564
```

---

## 第6章: 正規化 + マスク（実験18）

### マスク効果
```
mask df>N/10 (語彙の 0.2% = 45 gram):
  avg: 3.376 → 1.840  (ハブ度 54%減)
  上位ハブ: 「ながら」文 → 「涙を一ぱいためながら」(内容 gram 付き)
```

### norm + mask(df>21) の各ホップ
```
M²: 寂滅[0.570] top  到達80文書  (精度重視)
M⁴: 「別れ」文書群が浮上  (OOV「失恋」→「別れ」補完)
```

**Gini=0.820** (最高値、基準点)

---

## 第7章: 条件付きエントロピー最小化（実験19〜20）

### 等価性の証明
```
H(D|g) = log(df(g))          条件付きエントロピー
IDF(g) = log(N/df(g)) = log(N) - H(D|g)
→ H(D|g) 最小化 ≡ IDF 最大化  (数学的等価)
```

### IDF² 加重 M_doc
```
ハブ抑制比率:
  ハードマスク df>21: avg 1.840  (0.543×)
  IDF² 加重:          avg 1.834  (0.543×)  ← 同等の抑制をソフトに達成
```

### 行正規化 PPR
```
v_k = (1-λ)·M_row·v_{k-1} + λ·v₀
行正規化後: 固有値 ≤ 1 → 収束保証 (実験19の273億発散を解決)
```

### Gini 比較
```
mask+L1 k=2:              Gini=0.82  到達 80文書  (最高精度)
IDF² PPR λ=0.50:          Gini=0.81  到達210文書  (競合)
IDF² PPR λ=0.20:          Gini=0.49  到達210文書
```

---

## 第8章: Shift+Popcount 高速化（実験21）

### モジュール化 (bitrag-core)
```
artifacts/bitrag/bitrag-core/src/
  ngram.rs    — ngrams(), gram_set()
  corpus.rs   — load_corpus()
  bitset.rs   — DocBits (shift+pop/binary/bins Jaccard, xcorr)
  idf.rs      — Vocab (df, idf_sq, bins量子化)
  matrix.rs   — build_mdoc_*, row_normalize, ppr, hop
  eval.rs     — gini, top_k, shorten, reach_count, row_sum_stats
```

### BTreeSet vs Bitset shift+popcount
```
コア実装:
  while bits != 0 {
      pos = bits.trailing_zeros()   // shift で LSB 特定
      score += idf_sq[base + pos]   // 重み積算
      bits &= bits - 1              // LSB クリア
  }

速度 (全ペア M_doc):
  BTreeSet IDF:       254ms  1.0x (baseline)
  Bitset shift+pop:    23ms  11x 高速
  スコア誤差: 1.01e-9 (浮動小数のみ)
```

---

## 第9章: 語彙空間シフト畳み込み（実験22〜23）

### 数式
```
x & y                → bit 積 (gram 集合の ∩)
popcount(x & y)      → |A ∩ B| の近似カウント
x << k, x >> k       → n-gram ずれ (語彙インデックス k シフト)

xcorr[k] = popcount(A & (B << k))
         → 離散相互相関 (bit 空間の畳み込み)

score = Σ_k decay^|k| · xcorr_idf[k]  /  union_idf
```

### 相互相関の形状
```
自己相関 (A==B):        peak集中度 0.363  (鋭い)
類似ペア (西遊記×西遊記): peak集中度 0.099  (肩広: 語彙近傍も共有)
非類似ペア:             k=0 のみスパース
```

### decay × Gini グリッド (IDF+shift, PPR λ=0.20)
```
K\decay   d=1.0  d=0.7  d=0.5  d=0.3
K=0       0.489  0.489  0.489  0.489  (= 実験20 IDF² PPR)
K=1       0.453  0.456  0.460  0.467
K=2       0.409  0.423  0.436  0.453
K=5       0.326  0.366  0.404  0.442
→ K=1, decay=0.3 がバランス最良
```

### OOV への効果
```
「失恋 西遊記」(部分OOV):
  K=0: top-1 score 0.0090
  K=3: top-1 score 0.0188  (2.1x)  — 語彙隣接 gram が追加ヒット
  
「後悔」→「後悔し」: shift=1 で語彙隣接 gram がマッチ (実証済み)
```

---

## 総合: 累積高速化

| 対象操作              | 方式                | 速度            |
|-----------------------|---------------------|-----------------|
| 文書スコアリング      | Python 素朴実装     | 1×（基準）      |
| 文書スコアリング      | HashSet (Rust)      | 30,000×         |
| 文書スコアリング      | Bitset popcount     | ∞（<1µs）       |
| M_doc 全ペア事前計算  | BTreeSet IDF        | 254ms           |
| M_doc 全ペア事前計算  | Bitset shift+pop    | **23ms (11×)**  |
| M_doc 全ペア事前計算  | Binary bitset       | 9ms (28×)       |

---

## 推奨設定（実用）

```
最高精度:   mask df>N/10 + L1 正規化 + k=2 ホップ  → Gini=0.82
高速+精度:  IDF² shift+pop + 行正規化 + PPR λ=0.50  → Gini=0.81
再現重視:   IDF² PPR λ=0.20                          → Gini=0.49
近似OOV:    xcorr K=1, decay=0.3                      → 語彙隣接補完

アンカー構築: build_anchor(query, k=2)
仮想空間:    α=0.75〜1.00  (α* ≈ 0.66〜0.75 が閾値)
```

---

## ライブラリ (bitrag-core)

```toml
# 各実験の Cargo.toml
[dependencies]
bitrag-core = { path = "../../bitrag-core" }
```

```rust
use bitrag_core::{
    load_corpus, gram_set, DocBits, Vocab,
    build_mdoc_binary, build_mdoc_idf, build_mdoc_idf_masked,
    build_mdoc_shift, build_mdoc_shift_idf,
    row_normalize, ppr, hop,
    gini, top_k, shorten, reach_count, row_sum_stats,
};
// DocBits メソッド: jaccard_binary, jaccard_idf, sim_shift, sim_shift_idf
//                  xcorr, xcorr_idf, xcorr_to_score
```

---

## 未解決・次の問い

1. **α* の解析式**: α* ≈ 0.66〜0.75 の普遍性と導出  
2. **M^k 収束先**: PageRank 的固有ベクトルがコーパスの「主題ベクトル」か  
3. **SIMD 256bit xcorr**: AVX2 で 4× 256bit 並列処理 → さらなる高速化  
4. **位置シフト vs 語彙シフト**: 文書内 n-gram 位置ベース bitset との比較  
5. **最適 decay の語彙統計導出**: decay の理論的最適値は語彙密度の関数か  

---

## 第10章: 長クエリ × bit文再構成（実験24）

### 問い
> 長クエリ × IDF² 加重スコアリングが安定と確認された (ユーザー Python 実験)。
> これを Rust 化し、さらに「検索 → 生成」を bit 演算のみで繋げられるか？

---

### A: 長クエリスコアリング (Python 実装の Rust 版)

```rust
// long_query_score: eval.rs
score(Q, D) = Σ_{g∈Q∩D} idf(g)² + 0.5·|Q∩D|/|Q| + 0.5·max idf(g)²
```

**実装: trailing_zeros ループで bit を走査し idf_sq を積算**

| クエリ | top1 score | top1 cov | 特徴 |
|--------|-----------|---------|------|
| 比喩評価 (93 gram) | 1961.4 | 0.742 | 同一文 (変体仮名差) が突出 |
| 西遊記・水滸伝 (92 gram) | 1084.6 | 0.457 | 1位+2位で意味分割 |
| 感情OOV語列 (57 gram) | 79.5 | 0.053 | 低 cov → OOV支援が必要 |

ユーザー観察 **「長クエリは完全に安定」** を Rust で再現・検証。

---

### B/C: bit Greedy Set Cover による文再構成（生成）

**核心アルゴリズム:**

```
remaining = G_q      ← クエリ bitset のコピー
repeat:
    j* = argmax_j popcount(remaining & G_j)   ← bit積
    remaining &= ~G_{j*}                       ← カバー済みを除去
    output.push(texts[j*])
until remaining == 0
```

- **B: binary greedy cover** — gram 数で貢献量を測る
- **C: IDF² 加重 greedy cover** — 高 IDF gram (rare/distinctive) を優先

**理論保証:** greedy set cover の最適近似比 = ln(|Q|)

#### 比喩評価クエリ (|Q|=93) の生成ステップ:

| Step | 新規IDF² | 累積cov | 選択文書 |
|------|---------|--------|---------|
| 1 | 1945.0 | 0.863 | 子供の時の愛読書は「西遊記」が第一であった… |
| 2 | 130.9 | 0.921 | じゃ忘れないでね、──私も昨日あたりまでは… |
| 3 | 76.4 | 0.955 | わたしは今でもこの事だけは、感心だと思つて… |
| 4 | 68.7 | 0.986 | してみれば女に遇っているのは、全然夢とばかり… |
| 5 | 32.0 | 1.000 | 実を云ふと彼は、かうなるまでに、師匠と… |

→ **5文で クエリを 100% カバー**（神経網なし）

---

### D: 再構成テキスト例 (感情OOVクエリ)

```
クエリ: 失恋 無常 寂滅 孤独 別れ 悲しみ 後悔

[Step1 cov=0.34] それだけに、一層戦友の言葉は、ちょうど傷痕にでも触れられたような、
                 腹立たしい悲しみを与えたのだった。
[Step2 cov=0.52] そこで金花は今更のやうに、彼女の軽率を後悔しながら、涼しい視線を
                 外へ転じて、仕方なく更にきつぱりと、もう一度頭を振つて見せた。
[Step3 cov=0.69] いわばこの桶の中の空のように、静かながら慕わしい、安らかな寂滅の
                 意識であった。
[Step4 cov=0.87] 森は木の芽を煙らせながら、孤独に苦しんでいる彼の耳へも、人懐しい
                 山鳩の声を送って来る事を忘れなかった。
```

**「失恋・後悔・孤独・寂滅」の感情空間を 4文で構成** — RAG生成の bitRAG 版

---

### E: 速度

| 手法 | 1クエリあたり |
|------|-------------|
| long_query_score (検索) | 59.1 µs |
| greedy_cover binary | 212.6 µs |
| greedy_cover IDF² | 265.9 µs |
| **検索+生成 合計** | **325 µs** |

---

### F: 検索→生成の完全接続

```
short query → xcorr K=1 (語彙近傍) + M² PPR λ=0.5 (OOV救済)
           → top-k 文書

long  query → long_query_score (IDF² coverage)
           → greedy_cover_idf (bit set cover)
           → 再構成テキスト

両方とも: 神経網なし、bit積 + popcount のみ
```

---

### 定理 (bitRAG 生成の近似保証)

```
OPT = 最小カバー集合のサイズ
greedy のステップ数 ≤ OPT × ln(|Q|)

|Q| = 93 gram のとき ln(93) ≈ 4.5
```

**つまり: 最適が 1文でカバーできるなら greedy は 5文以内に収まる**

---

### まとめ

| 観点 | 結果 |
|------|------|
| Python → Rust 変換 | 完全再現 (IDF² 加重スコア) |
| 生成アルゴリズム | greedy set cover = bit積のみ |
| 理論保証 | ln(|Q|) 近似比 (set cover 古典結果) |
| 速度 | 検索+生成 325µs/クエリ |
| OOV耐性 | 長クエリは安定; 短OOVは M²/xcorr で補完 |

**実験24 で検索・生成の両輪が完結**

---

## 第7章: LLM統合 & コンテキスト圧縮（実験28〜29）

### 実験28: bitRAG → LLM パイプライン

**パイプライン**:
```
query → 短/長判定 → nonce+score → greedy IDF² cover → コンテキスト整形 → gpt-5.2
```

**コンテキスト形式**: `[SYSTEM] + [CONTEXT] + [QUERY] + [INSTRUCTION]`

**精度改善 (2段階)**:

| 改善 | 施策 | 効果 |
|------|------|------|
| 検索フィルタ | adaptive Jaccard 閾値 (`best_j × 0.15`) | nonce偶発ヒット除外 (Retrieval 4.0→5.0) |
| プロンプト強化 | 4ルール明示 (引用のみ/解釈禁止/記載なし明記/簡潔) | 過剰解釈抑制 (Faithfulness 4.25→5.0) |

**LLM-as-Judge 改善前→後**:
```
Faithfulness:  4.25 → 5.00
Relevance:     4.75 → 5.00
Grounding:     3.75 → 5.00
Conciseness:   4.25 → 5.00
Retrieval:     4.00 → 5.00
```

**速度**: 検索+コンテキスト生成 113µs + LLM応答 1.3-7.3s

### 実験29: RAGコンテキスト圧縮 (SVD/PCA on bit matrices)

**手法**: IDF重み付き gram×文 行列に SVD (power iteration) → PCA importance で文選択

```
M ∈ R^{V_active × n_sents}   M[g,s] = idf(g) × bit(g,s)
SVD: M = UΣV^T
PCA importance(s) = Σ_k σ_k × |v_k[s]|
複合スコア = idf_query_coverage × (0.5 + 0.5 × pca_importance)
→ greedy 文選択 (IDF coverage ベース)
```

**圧縮結果**:

| クエリ | 元文字数 | 圧縮文字数 | 圧縮率 | 情報保持率 | トークン削減 |
|--------|----------|------------|--------|-----------|-------------|
| 比喩談〜傑作 | 50 | 28 | 56% | 77% | 9.4% |
| 失恋 孤独 後悔 別れ | 144 | 43 | **30%** | **100%** | **31%** |
| 子供の時の〜 | 50 | 50 | 100% | 94% | 1.1% |

**LLM-as-Judge (Full vs Compressed)**: B勝ち, TIE, A勝ち → **圧縮版は品質同等〜改善**

**特異値エネルギー分布**:
- クエリ1: σ₁=63.7%, σ₂=100% (2成分で完全)
- クエリ2: σ₁=36.1%, σ₂=72.2%, σ₃=100% (3成分 — 各キーワードが独立成分)
- クエリ3: σ₁=36.9%, σ₂=73.1%, σ₃=100% (3文均等 — 完全一致文書)

**発見**: 
- **短キーワードクエリで最大効果**: 3文書144字 → 3文43字 (70%削減) で情報保持100%
- SVD の特異値分布が「キーワード数 = 必要成分数」を自然に反映
- Cross-doc cos類似度 = 0.000 → 各文書が直交 (冗長なし)

### 実験30: 隣接行列 3モード比較 — 分岐 vs 連続 vs plain SVD

**動機**: 実験29 の SVD は文を独立点として扱い、「〜ながら、〜た」の節跨ぎ依存が失われる

**非対称 DAG (下三角 / 上三角 / 対角)**:
```
上三角 A[i,j] (i<j, 前向き辺 i→j):
  = α_U × seq_w + (1-α_U) × jac    α_U=0.40 (意味寄り)

下三角 A[i,j] (i>j, 後向き参照 i←j):
  = α_L × seq_w + (1-α_L) × jac    α_L=0.80 (系列寄り)

対角 A[i,i] = 1.0

seq_w = 1 - |i-j|/doc_len  (同文書内線形減衰, 異文書=0)

→ A[i,j] ≠ A[j,i]  (方向性を明示的に区別)
  例 Q4: s1→s2 = 0.32 (前向き/意味)
         s2→s1 = 0.64 (後向き/系列)

出次数 out[i] = Σ_{j>i} A[i,j]  (前向き影響力)
入次数 in[i]  = Σ_{j<i} A[j,i]  (過去からの流入)
→ 始端: out最大・in=0, 末端: out=0・in最大

SVD列スケール (Mode D): √(1 + out[i])
adjスコア:
  Mode C: 1 - max_{sj∈sel} dag[si,sj]
  Mode D: Σ_{sj∈sel} (dag[si,sj] + dag[sj,si])
```

**3つの greedy スコア設計**:
```
Mode B (plain):
  composite = idf_gain × (0.5 + 0.5 × pca_imp)

Mode C (分岐重視, γ=0.18):
  branching(s) = 1 - max_j A[s,j]   ← 既選択から遠いほど高い
  composite = idf_gain × (0.5 + 0.5 × pca_imp)
            + γ × branching(s) × (1 + pca_imp(s))

Mode D (連続強調・分岐無視, γ=0.15):
  continuity(s) = Σⱼ A[s,j]         ← 既選択に近いほど高い
  composite = idf_gain × (0.5 + 0.5 × pca_imp)
            + γ × continuity(s) × (1 + pca_imp(s))
  M_adj[g,s] = M[g,s] × √(1 + out_deg[s])  ← DAG出次数重み

Mode E (虚構挿入主体, γ=0.12):
  # 1. fiction bits 生成
  fiction_bits = Union_{si: Jac(si,q)≥0.03} sent_bits[si]  AND NOT qb
  # 2. 拡張 SVD: active_grams = query_grams ∪ fiction_grams
  M_adj[g,s] = M[g,s] × √(1 + out_deg[s])
  # 3. Fiction-primary greedy スコア
  composite = (fiction_idf_gain × 1.0 + query_idf_gain × 0.30) × (0.5 + 0.5 × pca_imp)
            + γ × dag_continuity(s) × (1 + pca_imp(s))
  # 4. 双方向終了条件
  終了: fiction_cov ≥ 60% かつ query_cov ≥ cov_threshold
       ※ fiction_bits が空なら query のみで終了 (Mode B 相当)
```

**4クエリ詳細結果 (DAG実装後)**:

| クエリ | B文字 | E文字 | B情保 | E情保 | E虚構Cov | E fiction数 | B→E差分 |
|--------|-------|-------|-------|-------|----------|-------------|---------|
| Q1 比喩談〜傑作 | 28 | 50 | 77% | 77% | 100% | 9 | +s1「子供の時の愛読書」 |
| Q2 失恋 孤独 後悔 別れ | 43 | 43 | 100% | 100% | 0% | 0 | 変化なし (fiction=0) |
| Q3 愛読書は西遊記〜 | 50 | 50 | 94% | 94% | 100% | 9 | 変化なし |
| Q4 金花は仕方なく〜 | 38 | 51 | 86% | **98%** | 100% | 102 | **+s1「そこで金花は今更」** |

**総合統計 (4クエリ平均)**:

| Mode | 圧縮率 | 情報保持 | 連続度 |
|------|--------|---------|--------|
| B: SVD plain | 61.5% | 89.2% | ─ |
| C: 分岐重視 | 72.5% | 89.2% | 0.417 |
| D: 連続強調 | 72.5% | 89.2% | 0.417 |
| **E: 虚構挿入主体** | 77.7% | **92.3%** | **0.458** |

**Mode E の重要発見**:
Q4 で B/C/D が全て見落とす s1「そこで金花は今更のやうに、」を E のみが選択。
fiction tokens (102個) の中に「今更」「やう」等 s1 の特徴語が含まれ、s1 選択で fiction_cov が増加。
→ E は**物語の文脈設定文** (主語・状況提示節) を優先的に引き込む。これが情保率+12.3pp の源泉。

**C=D の理由分析**:

Q2/Q3/Q4 で C=D=B になる。原因: IDF カバレッジが支配的で γ ボーナスが選択を覆せない。

```
Q4 例: s2(後悔)選択後、s3(涼しい視線) の continuity bonus:
  D: composite(s3) = 0 + 0.15 × 0.70 × (1+pca) ≈ 0.16
  D: composite(s4) = idf_gain(仕方なく) × 0.5 + ... > 0.16  → s4 が勝つ
  → s3 は連続強調でも引き込まれない
```

Q1 でのみ C・D が B と異なる (s1 追加):
```
B: s2「比喩談〜」+ s3「西洋には一つもないと思ふ。」← coverage 77% < 90% threshold
C・D: + s1「子供の時の愛読書は西遊記が第一であった。」
      ← coverage threshold 未達 → 分岐/連続ボーナスで s1 が加点され選択
      → 「なぜ傑作か」の前提節が復元される
```

**重要発見: IDF優位性の確認**:
- このコーパスでは IDF カバレッジが隣接ボーナスを常に上回る
- C(分岐) と D(連続) が同じ結果 → 選択の多様性・連続性よりも IDF 情報密度が決定的
- 隣接行列が有効になるのは **coverage threshold 未達** かつ **IDF=0 の隣接節が存在する** とき

**γ が有効な条件**:
```
有効: IDF gain_candidate ≈ 0  AND  continuity/branching > 0
      → threshold 未達で候補が尽きそうな局面のみ
```

---

**LLM応答精度評価 (LLM-as-Judge, gpt-5.2)**:

| 軸 | B:plain | C:branch | D:cont |
|----|---------|----------|--------|
| Faithfulness | **5.00** | **5.00** | 4.75 |
| Relevance | **4.75** | **4.75** | 4.00 |
| Grounding | **5.00** | 4.75 | 4.50 |
| Conciseness | **5.00** | **5.00** | 4.75 |
| Retrieval | **5.00** | 4.75 | **5.00** |
| **総合平均** | **4.95** | **4.85** | 4.60 |

**発見 B > C > D の理由**:
- Mode D (連続強調): Q1 で「今でも」の不在指摘なし・Q4 で主語「金花」を文書にないのに断言 → relevance/grounding 減点
- Mode C (分岐重視): Q1 で「今でも」は文書に記載なしと正しく指摘 (関連性 5.0) · Q4 で「金花」を主語として根拠なく推定 → grounding 軽微減点
- Mode B (plain): シンプルな引用のみ。「金花」主語問題も控えめな提示で減点なし

**Q4「金花」問題の詳細**:
```
取得文書: 「彼女の軽率を後悔しながら、」「仕方なく更にきつぱりと、」「もう一度頭を振つて見せた。」
        ← 主語「金花」は文書中に存在しない
Mode C: 「文書記述に基づく」と正当化 → grounding 4/5
Mode D: 「金花は〜と言える」と断言 → faithfulness 4/5 + grounding 4/5
Mode B: 「参照文書[1]には〜とある」と事実のみ記述 → 全軸 5/5
```

**パラメータ**:
```
α = 0.7 (seq 70% + gram 30%)
γ_C = 0.18 (分岐), γ_D = 0.15 (連続)
文分割: 句点 (。！？…) + 読点 (、 ≥6文字)
rank = 6, power_iteration_iters = 120
```

---

## E152〜E161 全実験横断比較表

全実験を「ルーティング手法 × コーパス × 評価軸」で比較する。
値はすべて OR-route または AND-route の多数決一致率 (16 クエリ中)。

### OR-route 一致率 (法律 / Rust / 混合)

| 実験 | 手法 | K | 法律 OR | Rust OR | 混合 OR | 備考 |
|------|------|---|---------|---------|---------|------|
| E152 | 線形多数決 (dyadic K=8) | 8 | 87.5% | 100.0% | 100.0% | 基盤実験 |
| E153 | 非環状 dyadic | 7 | 87.5% | 100.0% | 100.0% | 最短隣接 = 対蹠 除外 |
| E154 | 環状 dyadic K=8 (sw/2 含む) | 8 | 87.5% | 100.0% | 100.0% | k=7 が孤立ノイズ |
| E155 | 対蹠除外 K=7 | 7 | 87.5% | 100.0% | 100.0% | k=7 除外 |
| E155b | 対蹠置換 (sw/3 or 2sw/5) | 8 | 87.5% | 100.0% | 100.0% | 置換でも同等 |
| E156 | 法律: k=7 相関行列解析 | 8 | — | — | — | 解析実験 (精度なし) |
| E157 | 環状木 w=2 | 8 | **93.8%** | 100.0% | 100.0% | 法律で +6.2pp |
| E158 | 環状木 w=3 | 8 | 87.5% | 100.0% | 100.0% | w=2 より劣化 |
| E159 | 環状木 w スイープ (best) | 8 | **93.8%** (w=2) | 100.0% | 100.0% | w=2 が最良 |
| E160 | 環状木 全オフセット解析 | 8 | — | — | — | 解析実験 (精度なし) |
| E161 | 全演算子比較 (OR_tree greedy) | 7 | 87.5% | **100.0%** | 87.5% | 10ルーティング横断 (NSEG=8) |

### AND-route 一致率 (法律 / Rust / 混合)

| 実験 | 手法 | K | 法律 AND | Rust AND | 混合 AND | 備考 |
|------|------|---|---------|---------|---------|------|
| E152 | 線形多数決 | 8 | **100.0%** | **100.0%** | **100.0%** | AND では最良 |
| E153 | 非環状 dyadic | 7 | **100.0%** | **100.0%** | **100.0%** | E152 と同等 |
| E154 | 環状 dyadic K=8 | 8 | 93.8% | **100.0%** | **100.0%** | 法律で -6.2pp |
| E155 | 対蹠除外 K=7 | 7 | **100.0%** | **100.0%** | **100.0%** | k=7 除外で回復 |
| E155b | 対蹠置換 | 8 | **100.0%** | **100.0%** | **100.0%** | 置換でも回復 |
| E157 | 環状木 w=2 | 8 | 81.2% | 87.5% | **100.0%** | OR 優先設計の影響 |
| E158 | 環状木 w=3 | 8 | 81.2% | 75.0% | 87.5% | w=3 で Rust AND 最低 |
| E159 | 環状木 w スイープ (best) | 8 | 87.5% (w=4) | 87.5% (w=2/4) | 93.8% (w=2/4) | E152 に届かず |
| E161 | 全演算子比較 (AND flat / NOT_Q flat) | 7 | 87.5% / **100.0%** | **100.0%** / **100.0%** | 87.5% / 93.8% | flat scan が tree-greedy を上回る |
| E161 | 全演算子比較 (AND_tree greedy) | 7 | 50.0% | 56.2% | 56.2% | tree-greedy 近似 |

### コーパス別 OR-route トレンド

| コーパス | E152 | E155 | E157 | E159 (best) | 最大値 | 変化幅 |
|---------|------|------|------|-------------|-------|-------|
| 法律 | 87.5% | 87.5% | **93.8%** | **93.8%** | **93.8%** | +6.2pp |
| GitHub Rust | **100.0%** | **100.0%** | **100.0%** | **100.0%** | **100.0%** | 変化なし |
| 混合 | **100.0%** | **100.0%** | **100.0%** | **100.0%** | **100.0%** | 変化なし |

### コーパス別 AND-route トレンド

| コーパス | E152 | E154 | E155 | E157 | E159 (best) | 最大値 | 最悪値 |
|---------|------|------|------|------|-------------|-------|-------|
| 法律 | **100%** | 93.8% | **100%** | 81.2% | 87.5% | **100%** | 81.2% |
| GitHub Rust | **100%** | **100%** | **100%** | 87.5% | 87.5% | **100%** | 75.0% (E158) |
| 混合 | **100%** | **100%** | **100%** | **100%** | 93.8% | **100%** | 87.5% (E158) |

### 主要発見事項

| 番号 | 発見 | 確認実験 |
|-----|------|---------|
| F1 | Rust / 混合コーパスは OR でほぼ全手法 100.0% | E152–E159 全実験 |
| F2 | 法律コーパスの OR は E152–E155 で 87.5%、E157/E159 で 93.8% に改善 | E157, E159 |
| F3 | AND では線形多数決 (E152) が全コーパスで最良 (100.0%) | E152, E153, E155 |
| F4 | 環状 dyadic の k=7 (sw/2 対蹠点) は独立ノイズクラスタ | E154, E156 |
| F5 | 対蹠点の除外 (E155) または置換 (E155b) で AND 精度が E152 に回復 | E155, E155b |
| F6 | 環状木 (E157–E159) は AND において E152 より劣化 | E157–E159 |
| F7 | 法律コーパスは n-gram 密度が高く (popcount 17,000〜24,000)、AND 交差が広範 | E156, E160 |
| F8 | w=2 が OR で最良 (法律 93.8%)、w=4 が AND で Rust/法律の次善 | E159 |
| F9 | NOT_Q flat scan は AND と数学的に等価で全コーパス 93.8〜100.0% | E161 |
| F10 | AND tree-greedy は flat scan より劣化 (50〜56% vs 87〜100%) | E161 |
| F11 | XOR/XNOR/NOR/NOT_AND は qlen=40, n=4 設定でランダム並 (12.5%) | E161 |
| F12 | NAND は argmax 実装により anti-AND として動作し全コーパス 0% | E161 |

### 推奨手法まとめ

| 優先目標 | 推奨実験 | 理由 |
|---------|---------|------|
| OR 精度最優先 | E157/E159 (w=2, 環状木) | 法律 OR を 87.5% → 93.8% に改善 |
| AND 精度最優先 | E152/E153/E155 (線形多数決または対蹠除外) | 全コーパス 100.0% |
| OR+AND バランス | E155 (対蹠除外 K=7) | 両指標とも高水準 (法律 OR 87.5% / AND 100.0%) |
| 最小構成 | E155 K=7 | 7 オフセットで E152 K=8 と同等以上 |
