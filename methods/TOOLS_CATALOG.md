# ツールカタログ (TOOLS_CATALOG.md)

プロジェクト内の再利用可能なツール・ライブラリ・スクリプトを一覧化したドキュメントです。

---

## 目次

1. [bitrag-core（Rust コアライブラリ）](#1-bitrag-core)
2. [tren scheduler（ジョブスケジューラ）](#2-tren-scheduler-旧称-bitrag-sched)
3. [lib/（TypeScript 共有ライブラリ）](#3-lib-typescript-共有ライブラリ)
4. [実験ユーティリティスクリプト（.mjs）](#4-実験ユーティリティスクリプト)
5. [その他ユーティリティ](#5-その他ユーティリティ)

---

## 1. bitrag-core

**場所**: `artifacts/bitrag/bitrag-core/src/`

Rust で書かれた bit 演算ベースのコアライブラリ。高速な文書類似度計算・語彙管理・評価ユーティリティを提供する。

---

### 1.1 `bitset` モジュール

**ファイル**: `artifacts/bitrag/bitrag-core/src/bitset.rs`

**概要**:  
固定幅 bitset (`DocBits`) を中心に、gram インデックスのビット操作・類似度計算・相互相関関数を実装する。`Vec<u64>` を多倍長整数として扱い、シフト+popcount の論理演算で文書間の畳み込み類似度を効率的に計算する。

**主な API**:

| 関数・メソッド | 説明 |
|---|---|
| `DocBits::new(nw)` | `nw` ワード幅の空 bitset を生成 |
| `DocBits::set(i)` | インデックス `i` の gram ビットをセット |
| `DocBits::shift_right(k)` / `shift_left(k)` | bitset の多倍長シフト |
| `jaccard_binary(other)` | バイナリ Jaccard 類似度 |
| `jaccard_idf(other, idf_sq)` | IDF² 加重 Jaccard 類似度 |
| `jaccard_idf_bins(...)` | IDF bin 量子化版 Jaccard（K=8 近似） |
| `sim_shift(other, max_shift, decay)` | バイナリ shift+popcount 畳み込み類似度 |
| `sim_shift_idf(other, idf_sq, ...)` | IDF² 加重 shift 畳み込み類似度 |
| `xcorr(other, max_k)` | bit 積の相互相関関数（binary） |
| `xcorr_idf(other, idf_sq, max_k)` | IDF² 加重相互相関関数 |
| `xcorr_to_score(xcorr, decay, norm)` | 相互相関スコアへの変換 |

**主な用途・再利用シーン**:
- 文書を gram bitset に変換し、Jaccard 類似度でランキングする場面
- OOV gram の「語彙近傍一致」を shift+popcount で近似補完したい場面
- 行列ベース類似度検索のコアとして利用

---

### 1.2 `nibble_hash` モジュール

**ファイル**: `artifacts/bitrag/bitrag-core/src/nibble_hash.rs`

**概要**:  
バイト列を 2×4 nibble 状態行列 (ADD+ROL+XOR) で処理し、`(hash_hi: u8, hash_lo: u8)` を出力するコンパクトなハッシュ関数。出力を nibble (4bit) グリッド座標 (16×16) に変換する補助関数も含む。

**主な API**:

| 関数 | 説明 |
|---|---|
| `nibble_hash_matrix(data: &[u8])` | `(hi, lo)` を出力するメインハッシュ関数 |
| `hash_to_grid(hi, lo)` | `(row, col)` ∈ [0,15]² への変換 |
| `pack_nibble_id(...)` | `u16` nibble_id のパッキング（P17 spec） |

**主な用途・再利用シーン**:
- トークンを 16×16 グリッドに配置するセマンティックマップ構築
- 軽量な埋め込みハッシュとして語彙分類に利用
- nibble_id による位置（P系）×カテゴリ（C系）の 2 次元インデックス生成

---

### 1.3 `matrix` モジュール

**ファイル**: `artifacts/bitrag/bitrag-core/src/matrix.rs`

**概要**:  
文書集合から類似度行列 M_doc を構築し、Personalized PageRank (PPR) やホップ伝播などのグラフアルゴリズムを適用するためのモジュール。

**主な API**:

| 関数 | 説明 |
|---|---|
| `build_mdoc_binary(doc_bits, n)` | バイナリ Jaccard で M_doc を構築 |
| `build_mdoc_idf(doc_bits, vocab, n)` | IDF² Jaccard で M_doc を構築 |
| `build_mdoc_idf_masked(...)` | 高頻度 gram をマスクした IDF² Jaccard で構築 |
| `build_mdoc_shift(...)` | バイナリ shift 畳み込みで M_doc を構築 |
| `build_mdoc_shift_idf(...)` | IDF² shift 畳み込みで M_doc を構築 |
| `row_normalize(m, n)` | 行 L1 正規化（PPR 収束保証に必要） |
| `ppr(v0, m, n, lambda, steps)` | Personalized PageRank 実行 |
| `hop(v, m, n)` | M^k ホップ（行列累乗の単ステップ） |

**主な用途・再利用シーン**:
- グラフ伝播ベースの文書ランキング（クエリ起点 PPR）
- n-hop 到達分析による関連文書探索
- 類似度行列の構築基盤として各実験で再利用

---

### 1.4 `ngram` モジュール

**ファイル**: `artifacts/bitrag/bitrag-core/src/ngram.rs`

**概要**:  
テキストから文字 n-gram を生成するシンプルなユーティリティ。n=2,3,4 の全 gram を `BTreeSet` で返す `gram_set` が主要エントリポイント。

**主な API**:

| 関数 | 説明 |
|---|---|
| `ngrams(text, n)` | 長さ `n` の文字 n-gram を `Vec<String>` で返す |
| `gram_set(text)` | n=2,3,4 の全 n-gram を `BTreeSet<String>` で返す |

**主な用途・再利用シーン**:
- `Vocab::build` や `DocBits` 生成の前処理として利用
- テキストの gram 集合を生成する汎用前処理ステップ

---

### 1.5 `idf` モジュール

**ファイル**: `artifacts/bitrag/bitrag-core/src/idf.rs`

**概要**:  
gram コーパスから語彙・df（文書頻度）・IDF² 重み・bin 量子化・ビット平面表現を一括構築する `Vocab` 構造体と、高速近似 IDF 和計算のための `IdfPlanes` 構造体を提供する。

**主な API**:

| 型・関数 | 説明 |
|---|---|
| `Vocab::build(gram_lists, n_docs, n_bins)` | 語彙・df・idf_sq・bins を一括構築 |
| `Vocab::to_docbits(grams)` | gram 集合 → `DocBits` に変換 |
| `Vocab::to_bins(grams)` | gram 集合 → IDF bin 分解に変換 |
| `Vocab::idf(g)` | gram の IDF 値を返す |
| `Vocab::idf_planes(b_bits)` | B-bit ビット平面表現を構築 |
| `IdfPlanes::sum_bits(s_words)` | bitset 内 gram の IDF 和近似 |
| `IdfPlanes::sum_inter(q, d)` | 積集合の IDF 和近似 |
| `IdfPlanes::pair_jaccard(q, d)` | IDF Jaccard 近似 |

**主な用途・再利用シーン**:
- 検索・ランキングパイプラインの語彙管理と重み付け
- IDF² 加重類似度・greedy cover の基盤
- ビット平面演算による高速近似 IDF 計算

---

### 1.6 `eval` モジュール

**ファイル**: `artifacts/bitrag/bitrag-core/src/eval.rs`

**概要**:  
検索結果の評価・スコアリング・文書選択アルゴリズムを提供する。Gini 係数・上位 k 件取得・IDF² 加重 coverage スコア・Greedy Set Cover など、実験での評価指標計算に使用する。

**主な API**:

| 関数 | 説明 |
|---|---|
| `gini(v)` | Gini 係数（スコア集中度指標） |
| `top_k_idx(scores, k)` | スコア上位 k 件を `(score, idx)` で返す |
| `top_k(scores, texts, k)` | スコア上位 k 件を `(score, text)` で返す |
| `shorten(s, n)` | テキストを先頭 n 文字に切り詰め |
| `reach_count(scores, threshold)` | 閾値を超える文書数を返す |
| `row_sum_stats(m, n)` | 行和の統計（avg, max, argmax） |
| `long_query_score(qb, db, idf_sq, q_size)` | IDF² 加重 coverage スコア（長クエリ用） |
| `greedy_cover(query_bits, doc_bits, max_docs)` | Greedy Set Cover（バイナリ） |
| `greedy_cover_idf(query_bits, doc_bits, idf_sq, max_docs)` | IDF² 加重 Greedy Set Cover |

**主な用途・再利用シーン**:
- 検索品質の自動評価・実験ログ集計
- クエリをカバーするための最小文書集合選択（RAG コンテキスト構築）
- 長クエリに対するスコアリング戦略

---

## 2. tren scheduler (旧称 bitrag-sched)

**本体場所**: `artifacts/bitrag/scheduler/` (in-repo single source of truth, Task #21)

PWD-local の job scheduler。中央デーモンは持たず、最初の `qsub` 呼び出し時に
cwd 配下に `.tren-<uuid>/` を作って UDP listen の wrapper プロセスが貼り付く。
Rust で実装されており、並列実験・バッチ処理を管理するための CLI クライアント群を提供する。

**有効化**: `source artifacts/bitrag/scheduler/env.sh`。`$TREN` と `$PATH`
(`$TREN/target/release` を先頭に追加) がセットされ、`qsub` などをフルパス
無しで呼べる。`target/release/qsub` が存在しなければ初回 source 時に
`cargo build --release` が自動で走る。

**wrapper 本体**: `artifacts/bitrag/scheduler/src/bin/tren_wrapper.rs`

> **NOTE**: 過去 (Task #14) には本体を `~/UTILITY/tren/` に退避してここを
> シム化していたが、Task #21 でその方針を撤回した。task agent の隔離環境
> でも cargo build → qsub がそのまま使えるようにするため、ソースは再び
> リポジトリ内に戻した。`~/UTILITY/tren/` を残している場合は drift の元に
> なるので削除するか、`unset TREN_HOME` した上で本 env.sh を source する。

---

### 2.1 `qsub` — ジョブ投入

**ファイル**: `artifacts/bitrag/scheduler/src/bin/qsub.rs`

**概要**: ジョブをキューに投入するクライアント。依存関係（`--after`）とオーナースコープ（`--owner`）を指定できる。

```sh
qsub "node artifacts/bitrag/experiment-49/ensemble.mjs"
qsub --after 1 2 node foo.mjs       # ジョブ1,2完了後に実行
qsub --owner myexp node foo.mjs     # オーナーに紐づけて投入
```

**主な用途**: 複数実験を依存順に自動実行、並列実験の管理。

---

### 2.2 `qstat` — キュー状態確認

**ファイル**: `artifacts/bitrag/scheduler/src/bin/qstat.rs`

**概要**: キュー内のジョブ一覧と状態を表示するクライアント。オーナーフィルタに対応。

```sh
qstat                    # 全ジョブ表示
qstat --owner myexp      # 特定オーナーのジョブのみ表示
```

**主な用途**: 実行中・待機中のジョブ状況確認。

---

### 2.3 `qlog` — ジョブログ表示

**ファイル**: `artifacts/bitrag/scheduler/src/bin/qlog.rs`

**概要**: 指定したジョブ ID の stdout/stderr ログを表示するクライアント。

```sh
qlog <job_id>
```

**主な用途**: 失敗したジョブのデバッグ、実験結果の確認。

---

### 2.4 `qowner` — オーナー管理

**ファイル**: `artifacts/bitrag/scheduler/src/bin/qowner.rs`

**概要**: ジョブグループのオーナー（スコープ）を管理するクライアント。最大並列数の設定、オーナーのリスト表示・削除・強制終了に対応。

```sh
qowner create myexp --max-parallel 4
qowner list
qowner drop myexp
qowner kill myexp
```

**主な用途**: 実験スコープごとの並列度制御と一括管理。

---

### 2.5 `qwait` — ジョブ完了待機

**ファイル**: `artifacts/bitrag/scheduler/src/bin/qwait.rs`

**概要**: 指定したジョブ ID が完了するまで 300ms ポーリングで待機するクライアント。全完了または任意の 1 つ完了を選択できる。

```sh
qwait 1 2 3             # 全ジョブ完了で exit 0（失敗ジョブがあれば exit 1）
qwait --any 1 2 3       # いずれか1つ完了で exit 0（その1つが FAILED なら exit 1）
```

> **注意**: ジョブが `DONE` で終了すれば exit 0、`FAILED` で終了すれば exit 1 を返す。シェルスクリプトや CI で後続処理を条件分岐する際はこの終了コードを確認すること。

**主な用途**: シェルスクリプトや CI からジョブ完了を待ってから後続処理を実行する場面。

---

### 2.6 `qwait-mark` — 完了待機 + マーカー書き込み

**ファイル**: `artifacts/bitrag/scheduler/src/bin/qwait_mark.rs`

**概要**: `qwait` を内部で実行し、完了後に `processed` マーカーファイルへ結果を追記するラッパー。`ls` や `while sleep` による自前ポーリングループの代替として使うこと。

```sh
qwait-mark <id1> [id2 ...]               # ./processed に 1 または -1 を追記
qwait-mark --marker /tmp/done <id1> ...  # 指定ファイルに追記
qwait-mark --any <id1> [id2 ...]         # qwait の --any モードで待機
```

**processed ファイル規約**:

| 追記値 | 意味 |
|--------|------|
| `1` | 全ジョブ `DONE`（正常完了）|
| `-1` | いずれかのジョブ `FAILED`（失敗）|

> **ポーリング禁止**: 実験スクリプトで `ls` ループや `while sleep` で完了を待つのは禁止。必ず `qwait-mark`（または `qwait`）を使うこと。

**主な用途**: 実験スクリプトからのジョブ完了待ち、`processed` マーカーの自動更新。

---

### 2.7 `qrun` — ジョブ投入 + 完了待機 + マーカー書き込み（1 コマンド）

**ファイル**: `artifacts/bitrag/scheduler/src/bin/qrun.rs`

**概要**: `qsub` でジョブを投入し、完了を待ち、マーカーに書き込むまでを 1 コマンドで行うショートハンド。実験スクリプトの `qsub → ID 取得 → qwait-mark` という定型パターンを 1 行に削減できる。

```sh
qrun node experiment.mjs                           # ./processed に 1 または -1 を追記
qrun --marker /tmp/done node experiment.mjs        # マーカーパスを指定
qrun --any node experiment.mjs                     # --any モードで待機
qrun --owner myexp --after 1 2 node foo.mjs        # qsub 互換フラグも使用可

# --parallel: ::: 区切りで複数コマンドを一括投入し、全完了後にまとめてマーカーを書く
qrun --parallel node exp_a.mjs ::: node exp_b.mjs ::: node exp_c.mjs
qrun --parallel node exp_a.mjs ::: node exp_b.mjs --marker /tmp/done

# --parallel + ブロックごとの --after / --owner: 各ジョブに独立した依存・スコープを設定
qrun --parallel --after 1 node a.mjs ::: --after 2 node b.mjs
qrun --parallel --owner expA --after 1 node a.mjs ::: --owner expB node b.mjs

# --per-block-marker: ブロックごとに "<値>:<job_id>" 形式で個別にマーカーを書く
qrun --parallel --per-block-marker node a.mjs ::: node b.mjs ::: node c.mjs
# → ./processed に 3 行追記例: "1:42\n-1:43\n1:44\n"（ジョブ 43 が失敗した場合）
```

**オプション**:

| フラグ | 説明 |
|--------|------|
| `--marker <path>` | マーカーファイルのパス（省略時: `./processed`）コマンドの前後どちらでも可 |
| `--any` | `qwait --any` モードで待機（いずれか 1 つ完了で進む）コマンドの前後どちらでも可 |
| `--parallel` | `::: ` 区切りで複数コマンドを並列投入し、全ジョブ完了後にマーカーを書く |
| `--per-block-marker` | `--parallel` 専用: ブロックごとに `"<値>:<job_id>"` 形式で個別追記（省略時は全体で 1 行） |
| `--owner <scope>` / `--after <id>...` | `qsub` 互換フラグ（コマンドの一部としてデーモンに転送）|
| `--` | 以降の引数を qrun が解釈せずそのままコマンドへ転送（コマンド自身が `--marker` などを取る場合に使用）|

> **フラグ解釈の範囲**: `--marker` / `--any` / `--parallel` / `--per-block-marker` は `--` 区切りより前であればどこに現れても qrun が解釈し、コマンドへの転送から除外する。`--` 以降の引数はフラグ解釈せずすべてコマンドに転送する。

> **`--parallel` モードの動作**: `::: ` でコマンドを区切ると、それぞれを独立したジョブとして即時投入し、全ジョブ ID をまとめて `qwait` に渡す。全ジョブ完了後にマーカーを追記する。デフォルト（`--per-block-marker` なし）は全 DONE → `1`、いずれか FAILED → `-1` の 1 行。`--per-block-marker` を指定すると各ジョブの成否を `"<値>:<job_id>"` 形式で 1 ブロック 1 行ずつ追記する。グリッドサーチや独立した複数実験の一括投入に便利。

> **ブロックごとの `--owner` / `--after`**: `--parallel` モードでは、各 `:::` ブロックの先頭に `--owner` や `--after` を置くと、そのブロックのジョブにだけ独立して適用される。ブロック内のフラグはコマンド部より前に記述する必要がある。コマンド部が見つかった時点でそのブロック内のフラグ解釈は終了し、残りはすべてコマンド引数として転送される。

> **制約**: `--parallel` と `--any` は同時に指定できません（意味的に矛盾するため）。`--per-block-marker` は `--parallel` なしでは指定できません。

**processed ファイル規約**:

| 追記値 | 意味 | モード |
|--------|------|--------|
| `1` | 全ジョブが `DONE(0)` で正常完了 | 通常・`--parallel`（デフォルト） |
| `-1` | いずれかのジョブが `FAILED` または `DONE(N>0)` で失敗 | 通常・`--parallel`（デフォルト） |
| `1:<job_id>` | そのブロックのジョブが `DONE(0)` で正常完了 | `--parallel --per-block-marker` |
| `-1:<job_id>` | そのブロックのジョブが `FAILED` または `DONE(N>0)` で失敗 | `--parallel --per-block-marker` |

**--per-block-marker の活用例**:

```bash
# グリッドサーチで失敗したジョブ ID を特定する
qrun --parallel --per-block-marker node run.mjs --lr 0.1 ::: node run.mjs --lr 0.01 ::: node run.mjs --lr 0.001

# 失敗したジョブ ID の一覧を取得
grep '^-1:' ./processed | cut -d: -f2

# 特定のジョブが成功したか確認
grep '^1:42$' ./processed
```

**終了コード**: `qwait` の終了コードをそのまま伝搬。失敗検出時は `1` で上書き。

**主な用途**: 実験スクリプトのボイラープレート削減。`qsub → qwait-mark` の 2〜3 行を 1 行に置き換える。`--per-block-marker` でグリッドサーチ等の個別ジョブ成否を追跡できる。

---

### 2.8 `lint-poll` — 禁止ポーリングパターン検出

**ファイル**: `artifacts/bitrag/scheduler/lint-poll.sh`

**概要**: 実験スクリプト内の禁止 `ls` ポーリングパターン（`while sleep`/`while true` + `ls`/`watch ls`）を自動検出する lint ツール。CI や定期チェックから呼び出せる形で、終了コード 0（pass）/ 1（fail）を返す。

```sh
# カレントディレクトリ以下の実験を全スキャン（デフォルトは artifacts/bitrag/）
./scheduler/lint-poll.sh

# 特定ディレクトリだけスキャン
./scheduler/lint-poll.sh artifacts/bitrag/experiment-160
```

**検出パターン**:

| パターン | 対象ファイル | 説明 |
|---------|------------|------|
| `while sleep …` | `.sh` `.bash` `.mjs` `.js` `.ts` | sleep-polling ループ（行頭アンカーで文字列内除外） |
| `while … done` 内の `ls` | `.sh` `.bash` のみ | shell の while ブロック（done で閉じる）内の ls 使用 |
| `watch ls` | `.sh` `.bash` `.mjs` `.js` `.ts` | watch コマンドによる ls 監視（行頭アンカー） |

デフォルトスキャン対象は `experiment-*` ディレクトリ直下（`node_modules/` / `target/` / `scheduler/` は常に除外）。

**終了コード**:

| コード | 意味 |
|--------|------|
| `0` | 違反なし（pass） |
| `1` | 1 件以上の違反を検出（fail） |
| `2` | 引数エラー（ディレクトリ不存在など） |

**主な用途**: CI チェック、新規スクリプトのレビュー前自動検査、既存実験コードの一括監査。

---

### 2.9 `CascadeGuard`（内部ライブラリ）

**ファイル**: `artifacts/bitrag/scheduler/src/cascade_guard.rs`

**概要**: `Drop` 実装によりプロセス終了時に `/tmp/bitrag-cascade.done`（`BITRAG_CASCADE_DONE` 環境変数で変更可）マーカーファイルを書き込む RAII ガード。カスケード実行の完了検知に使用する。

**主な用途**: 実験カスケードのシグナリング、後続プロセスの起動トリガー。

---

### 2.10 `env.sh` — 実験スクリプト用 PATH セットアップヘルパー

**ファイル**: `artifacts/bitrag/scheduler/env.sh`

**概要**: 実験スクリプトの先頭で `source` するだけで `qsub` / `qstat` / `qlog` / `qwait` / `qwait-mark` / `qmap` / `qbind` / `qclone` / `qowner` / `qworkdir` / `qdel` / `qrun` をフルパスなしで呼び出せるようにする Bash ヘルパー。`$TREN` と `$PATH` を自動設定し、`target/release/qsub` が無ければ初回 `source` 時に `cargo build --release` を自動実行する。

```bash
# 実験スクリプト (run.sh) の典型的な使い方
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

# 推奨: リポジトリルートからの相対 source (Task #21 以降の正本パス)
source "$(git rev-parse --show-toplevel)/artifacts/bitrag/scheduler/env.sh"

# 以降は qsub / qrun をフルパスなしで使用できる
qsub bash -c 'node experiment.mjs'
```

**設定される環境変数**:

| 変数 | 内容 |
|------|------|
| `TREN` | `env.sh` が置かれているディレクトリの絶対パス (=`artifacts/bitrag/scheduler/`) |
| `TREN_HOME` | `$TREN` と同値 (互換目的) |
| `PATH` | `$TREN/target/release` を先頭に追加した PATH |

**主な用途**: 実験スクリプトの可読性・移植性向上。フルパス記述を排除し `qsub` / `qrun` / `qstat` などを 1 語で呼び出せるようにする。

---

## 3. lib/（TypeScript 共有ライブラリ）

**場所**: `lib/`

モノレポ全体で共有する TypeScript ライブラリ群。

---

### 3.1 `db` — データベースクライアント

**ファイル**: `lib/db/src/index.ts`

**概要**:  
Drizzle ORM + node-postgres を使用した PostgreSQL クライアント。`DATABASE_URL` 環境変数で接続先を指定する。`db`（Drizzle インスタンス）と `pool`（pg Pool）をエクスポートし、スキーマも再エクスポートする。

```ts
import { db, pool } from "@workspace/db";
```

**主な用途・再利用シーン**:
- API サーバーからのデータベースアクセス
- スキーマ変更・マイグレーションの一元管理

---

### 3.2 `api-zod` — API スキーマ定義

**場所**: `lib/api-zod/`

**概要**:  
Zod スキーマで定義された API リクエスト・レスポンス型の共有ライブラリ。OpenAPI 仕様から自動生成されたコードを含む。フロントエンド・バックエンド間の型安全な契約として機能する。

**主な用途・再利用シーン**:
- リクエスト・レスポンスのバリデーション
- API クライアントと API サーバー双方での型共有

---

### 3.3 `api-client-react` — React 用 API クライアント

**場所**: `lib/api-client-react/`

**概要**:  
OpenAPI 仕様から自動生成された型付き API 関数（`./generated/api`）と Zod スキーマ（`./generated/api.schemas`）、およびベース URL と認証トークンを設定するカスタム fetch 設定ユーティリティをエクスポートするライブラリ。

```ts
import { getDocuments, setBaseUrl, setAuthTokenGetter } from "@workspace/api-client-react";

setBaseUrl("/api");
setAuthTokenGetter(() => localStorage.getItem("token") ?? "");
const docs = await getDocuments();
```

**エクスポート**:

| エクスポート | 説明 |
|---|---|
| 生成 API 関数（`./generated/api`） | OpenAPI 定義から生成された全エンドポイント関数 |
| 生成 Zod スキーマ（`./generated/api.schemas`） | レスポンス型の Zod バリデーションスキーマ |
| `setBaseUrl(url)` | API のベース URL を設定する関数 |
| `setAuthTokenGetter(fn)` | 認証トークン取得コールバックを設定する関数 |
| `AuthTokenGetter` | トークン取得コールバックの型定義 |

**主な用途・再利用シーン**:
- React アプリケーションからの型安全な API 呼び出し
- `api-zod` スキーマと組み合わせたエンドツーエンドのバリデーション

---

### 3.4 `integrations-anthropic-ai` — Anthropic AI クライアント

**ファイル**: `lib/integrations-anthropic-ai/src/index.ts`

**概要**:  
Replit AI Integrations プロキシ経由で Anthropic Claude API にアクセスするクライアントライブラリ。`anthropic`（SDK インスタンス）と、バッチ処理・SSE ストリーミング・レートリミットエラーハンドリングのユーティリティをエクスポートする。

```ts
import { anthropic, batchProcess, batchProcessWithSSE } from "@workspace/integrations-anthropic-ai";
```

**主な用途・再利用シーン**:
- LLM を使った RAG 応答生成
- バッチ評価・自動ジャッジスクリプトからの呼び出し
- `eval_accuracy.mjs` のような LLM-as-judge パイプラインとの連携

---

## 4. 実験ユーティリティスクリプト

各実験ディレクトリに置かれた Node.js ESM スクリプト群。単独で `node <script>.mjs` として実行する。

---

### 4.1 `nibble_analysis.mjs`

**場所**: `artifacts/bitrag/experiment-35/nibble_analysis.mjs`

**概要**:  
テキストを UTF-8 バイト列 → nibble (4bit) 列に変換し、スコープ (`{}`) 内出現カウント付き nibble n-gram を生成する NOT 解析スクリプト。借用・寿命パターンのコーパスと対比し、エラー率（未知 gram 率）を計算してコードの構文異常を検出する。

**主な用途**:
- Rust 借用違反・寿命エラーの静的解析プロトタイプ
- nibble gram 表現の精度評価
- スコープ境界を考慮した重複パターン（`count≥2`）の検出

---

### 4.2 `nibble_full.mjs`

**場所**: `artifacts/bitrag/experiment-35/nibble_full.mjs`

**概要**:  
ディレクトリ以下の全 `.rs` ファイルを対象に nibble gram を生成し、leave-one-out (LOO) 評価でファイルごとのエラー率・重複 alien gram 率を表示するスクリプト。

**主な用途**:
- GitHub コーパス全体での nibble gram 汎化性能の確認
- LOO 評価によるコーパス代表性の測定

---

### 4.3 `nibble_skewer.mjs`

**場所**: `artifacts/bitrag/experiment-35/nibble_skewer.mjs`

**概要**:  
nibble gram を整数 ID にインターンしてコーパス全体の転置インデックス（`BigInt64Array`）を構築するスクリプト。gram の DF 分布確認、gram による文書「串刺し」検索、ファイル間の Jaccard 類似度計算（整数 ID の mergesort で高速実行）を行う。

**主な用途**:
- 転置インデックスを活用した高速 gram 検索の実験
- コーパス内の全共通 gram（df=N）の抽出
- ファイル間類似度行列の構築

---

### 4.4 `type_similarity.mjs`

**場所**: `artifacts/bitrag/experiment-35/type_similarity.mjs`

**概要**:  
Rust ソースから型名を抽出し、nibble n-gram（スコープなし、サイズ 2,4,6,8）ベースの Jaccard 類似度でクラスタリングするスクリプト。型名空間の意味的近傍を可視化する。

**主な用途**:
- 型名の nibble gram 類似度探索
- 関連型のグループ化・類似型の発見
- 型次元での nibble gram 表現の評価

---

### 4.5 `type_ledger.mjs`

**場所**: `artifacts/bitrag/experiment-35/type_ledger.mjs`

**概要**:  
型名（TYPE 次元）と nibble hex 値（VAL 次元）の 2 次元 nibble gram を組み合わせた台帳（ledger）を構築するスクリプト。型情報と値情報を統合した複合 gram 空間を実験する。

**主な用途**:
- 型×値の 2 次元 gram 空間の実験
- トークン識別子の多次元表現プロトタイプ

---

### 4.6 `loo_eval.mjs`

**場所**: `artifacts/bitrag/experiment-35/loo_eval.mjs`

**概要**:  
type_ledger と同様の 2 次元 nibble gram を使って leave-one-out 評価を実施するスクリプト。コーパス全体での汎化性能・類似度スコア分布を測定する。

**主な用途**:
- 2 次元 nibble gram 表現の LOO 評価
- gram 設計のパラメータチューニング

---

### 4.7 `borrow_analysis.mjs`

**場所**: `artifacts/bitrag/experiment-35/borrow_analysis.mjs`

**概要**:  
文字 n-gram ではなくトークン n-gram（n=2,3,4）ベースで借用・寿命パターンを解析するスクリプト。キーワード・プリミティブ型・標準型を正規化したうえで NOT 解析を実施し、借用違反パターンを検出する。

**主な用途**:
- トークンレベル n-gram による借用違反検出の精度評価
- nibble gram と token gram の比較実験

---

### 4.8 `not_analysis.mjs`

**場所**: `artifacts/bitrag/experiment-35/not_analysis.mjs`

**概要**:  
文字 n-gram（n=2,3,4）に XOR bigram スロットカウンタ（pseudo-gram）を加えた gram 集合を構築し、既知コーパスとの差分（alien gram）を計算する NOT 解析スクリプト。pseudo-gram により遷移パターンも捕捉する。

**主な用途**:
- pseudo-gram を含む NOT 解析の実験
- 構文エラー指紋の設計評価

---

### 4.9 `llm_call.mjs`

**場所**: `artifacts/bitrag/experiment-28/llm_call.mjs`

**概要**:  
`contexts/` ディレクトリの JSON コンテキストファイルを読み込み、OpenAI 互換 API（Replit AI Integrations プロキシ）に順次送信して RAG 応答を生成し、結果を `llm_results.json` に保存するスクリプト。`AI_INTEGRATIONS_OPENAI_BASE_URL` / `AI_INTEGRATIONS_OPENAI_API_KEY` 環境変数が必要。

**主な用途**:
- bitRAG が生成したコンテキストを LLM に投入するエンドツーエンドパイプライン
- RAG 応答のバッチ生成

---

### 4.10 `eval_accuracy.mjs`

**場所**: `artifacts/bitrag/experiment-28/eval_accuracy.mjs`

**概要**:  
`llm_results.json` と `contexts/*.json` を読み込み、LLM-as-judge（5 軸: 忠実性・関連性・根拠明示・簡潔性・検索品質）で RAG 応答の品質を自動評価するスクリプト。評価結果を `eval_results.json` に保存する。

**主な用途**:
- RAG パイプラインの品質自動測定
- 検索アルゴリズム変更の効果評価（A/B 比較）

---

## 5. その他ユーティリティ

---

### 5.1 `bitVector.ts`

**場所**: `artifacts/bitrag/src/lib/bitVector.ts`

**概要**:  
フロントエンド（TypeScript/React）向けの bitset ユーティリティ。タグ集合 → bit 配列変換、gram 集合 → 64bit フィンガープリント変換、ハミング重み計算などを提供する。`DocBits`（Rust 側）の TypeScript 対応版に相当する軽量実装。

**主な API**:

| 関数 | 説明 |
|---|---|
| `setToBitArray(tagSet)` | タグ集合 → `Uint8Array` bit 配列に変換 |
| `gramSetToBitArray(grams, size=64)` | gram 集合 → 64bit フィンガープリントに変換（2 本ハッシュ） |
| `bitArrayToString(bits, maxLen)` | bit 配列を文字列表示に変換 |
| `hammingWeight(bits)` | bit 配列のハミング重み（セット数）を計算 |
| `tagIndexFor(tag)` | タグの語彙インデックスを返す |
| `ALL_TAG_LIST()` | 全タグ一覧を返す |

**主な用途・再利用シーン**:
- フロントエンドでの類似度計算・可視化
- タグ・gram のビット表現生成
- UI コンポーネントでの bitset 操作

---

### 5.2 `logger.ts`

**場所**: `artifacts/api-server/src/lib/logger.ts`

**概要**:  
`pino` ベースの構造化ロガー。開発時は `pino-pretty`（カラー表示）、本番時は JSON 出力に自動切り替えされる。認証ヘッダ・Cookie などの機密情報を自動 redact する。

```ts
import { logger } from "./lib/logger";
logger.info({ reqId }, "handler started");
```

**主な用途・再利用シーン**:
- API サーバー全体の統一ロギング
- リクエストトレース・エラーログの構造化出力
- 他の Express/Fastify アプリへの転用
