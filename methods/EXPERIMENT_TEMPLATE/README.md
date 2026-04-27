# 実験テンプレート — 科学的実験フロー

```
┌─────────────────────────────────────────────────────────────┐
│                  ★ 実験開始前の儀式 ★                        │
│                                                             │
│   experiment-class.md                                       │
│   ・この実験は何のためか（目標を1文で）                        │
│   ・goal: / status: / method: タグ付け                       │
│   ・関連実験の確認                                            │
│   ・チェックリスト完了                                        │
│                                                             │
│   ※ タグ定義は ../tag.md を参照                              │
│   ※ 目標が書けない実験は走らせない                            │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                     実験前 (事前記述)                         │
│                                                             │
│   仮説.md ──────────────────→ conditions.md                 │
│   ・何を検証するか              ・どう実験するか              │
│   ・なぜそう思うか              ・パラメータ・環境             │
│   ・成功の定義                  ・実行コマンド                 │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼  cargo run --release
┌─────────────────────────────────────────────────────────────┐
│                      実験実行                                │
│                                                             │
│   stdout.md                    stderr.md                    │
│   ・プログラムの出力            ・警告・エラー・パニック       │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                     実験後 (事後記述)                         │
│                                                             │
│   result.md                                                 │
│   ・数値まとめ表                                             │
│   ・グラフ (result.png 等)                                   │
│   ・仮説との照合・判定                                        │
│                                                             │
│        ├── novelKnowledge.md   ← 確認・発見された事実        │
│        ├── wasteHypothesis.md  ← 棄却された仮説と理由        │
│        └── novelHypothesis.md  ← 次に検証すべき新仮説        │
│                                                             │
│   最後に experiment-class.md の                             │
│   status: → done / result: を記入する                       │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼  次の実験へ
              novelHypothesis.md → 次の experiment-class.md
```

## ファイル一覧

| ファイル | 書くタイミング | 内容 |
|--------|--------------|------|
| `experiment-class.md` | **実験前・必須** | 目標・タグ・関連実験・チェックリスト |
| `仮説.md` | 実験前 | 検証命題・根拠・成功条件 |
| `conditions.md` | 実験前 | パラメータ・環境・コマンド |
| `stdout.md` | 実験中/後 | 標準出力ログ |
| `stderr.md` | 実験中/後 | エラー・警告ログ |
| `result.md` | 実験後 | 数値表・グラフ・判定 |
| `novelKnowledge.md` | 実験後 | 得られた新知見 |
| `wasteHypothesis.md` | 実験後 | 棄却仮説と棄却理由 |
| `novelHypothesis.md` | 実験後 | 次の実験候補仮説 |

## scheduler を使う場合の完了待ち規約

scheduler（`qsub`/`qwait`）を使って実験を走らせる場合は **`qrun`** または **`qwait-mark`** で完了を待つこと。`ls` や `while sleep` によるポーリングループを自前で書くのは禁止。

### processed ファイル規約

| 値 | 意味 |
|----|------|
| `1` | 全ジョブ DONE（正常完了） |
| `-1` | いずれかのジョブ FAILED（失敗） |

`qrun` / `qwait-mark` は `<marker_path>`（省略時 `./processed`）に上記の値を **追記** する。

### qrun — 1 行で投入〜完了待ちまで（推奨）

`qrun` は `qsub` と `qwait-mark` を 1 コマンドに統合したショートハンドです。

```bash
# ジョブ投入・完了待ち・マーカー書き込みを 1 行で実行
qrun node experiment.mjs

# マーカーパスを指定（コマンドの前でも後でも可）
qrun --marker /tmp/done node experiment.mjs
qrun node experiment.mjs --marker /tmp/done    # 後置も有効

# --owner / --after など qsub 互換フラグはコマンドの一部として転送される
qrun --owner myexp node experiment.mjs
qrun --owner myexp --after 1 2 node foo.mjs

# コマンド自身が --marker や --any を受け取る場合は -- 区切りを使う
qrun -- node tool.mjs --marker out.txt    # out.txt は tool.mjs へ転送

# 完了後の処理
if grep -q '^1$' ./processed; then
  echo "成功"
else
  echo "失敗"
fi
```

#### --parallel — 複数ジョブを並列投入してまとめて待つ

グリッドサーチや独立した複数実験を 1 コマンドで一括投入できます。  
コマンドを `:::` で区切ると、それぞれ独立したジョブとして即時投入され、全ジョブ完了後にマーカーへ 1 行だけ書き込まれます。

```bash
# 3 つの実験を並列投入し、全完了を待つ（./processed に結果を書く）
qrun --parallel node exp_a.mjs ::: node exp_b.mjs ::: node exp_c.mjs

# マーカーパスを指定する場合（--marker は --parallel の前後どちらでも可）
qrun --parallel node exp_a.mjs ::: node exp_b.mjs --marker /tmp/done

# 完了後の処理
if grep -q '^1$' ./processed; then
  echo "全ジョブ成功"
else
  echo "いずれかのジョブ失敗"
fi
```

##### ブロックごとの --after / --owner

各 `:::` ブロックの**先頭**に `--after` や `--owner` を置くと、そのブロックのジョブにだけ独立して適用されます。依存関係を持つジョブ群を並列投入したい場合に使います。

```bash
# ジョブ 1 完了後に a.mjs、ジョブ 2 完了後に b.mjs を実行（それぞれ独立した依存）
qrun --parallel --after 1 node a.mjs ::: --after 2 node b.mjs

# ブロックごとに --owner を指定（異なるスコープに紐づける）
qrun --parallel --owner expA node a.mjs ::: --owner expB node b.mjs

# --owner と --after を組み合わせる
qrun --parallel --owner expA --after 1 node a.mjs ::: --owner expB --after 2 node b.mjs

# マーカーパスも同時に指定できる
qrun --parallel --after 1 node a.mjs ::: --after 2 node b.mjs --marker /tmp/done
```

> **フラグの位置**: `--owner` / `--after` はブロックの**先頭**（コマンド名より前）に書く必要があります。コマンド名が現れた時点でそのブロックのフラグ解釈は終了し、残りはすべてコマンド引数として転送されます。
>
> **マーカー規約**: 全ジョブ `DONE` → `1`、いずれかのジョブ `FAILED` → `-1` を追記。  
> **制約**: `--parallel` と `--any` は同時に指定できません。`--parallel` は全ジョブ完了を保証しますが、`--any` はいずれか 1 つで処理を進めるため意味的に矛盾します（指定するとエラーで終了します）。

##### --per-block-marker — ブロックごとに個別マーカーを記録する

`--per-block-marker` を追加すると、全体での 1 行まとめ書きの代わりに、ブロックごとに `"<値>:<job_id>"` 形式で個別に追記します。グリッドサーチで**どのパラメータセットが失敗したか**を特定したい場合に便利です。

```bash
# グリッドサーチ: 3 条件を並列実行し、各ジョブの成否を個別に記録する
qrun --parallel --per-block-marker \
  node run.mjs --lr 0.1   ::: \
  node run.mjs --lr 0.01  ::: \
  node run.mjs --lr 0.001

# ./processed の記録例（ジョブ 43 が失敗した場合）:
#   1:42
#   -1:43
#   1:44

# 失敗したジョブ ID を取得する
grep '^-1:' ./processed | cut -d: -f2

# 特定のジョブが成功したか確認する
grep '^1:42$' ./processed && echo "JOB 42 succeeded"

# --per-block-marker は --parallel と組み合わせて使う（単体では使用不可）
# 他のオプション（--marker, --after, --owner）とも組み合わせ可能
qrun --parallel --per-block-marker --marker /tmp/results \
  --owner expA node a.mjs ::: --owner expB node b.mjs
```

**--per-block-marker のマーカー規約**:

| 追記値 | 意味 |
|--------|------|
| `1:<job_id>` | そのブロックのジョブが `DONE(0)` で正常完了 |
| `-1:<job_id>` | そのブロックのジョブが `FAILED` または `DONE(N>0)` で失敗 |

> **従来の動作との違い**: `--per-block-marker` なしでは全ジョブのまとめ結果を 1 行（`1` または `-1`）だけ書きます。`--per-block-marker` ありではブロック数と同じ行数が追記されます。

### qwait-mark — ID 取得済みの場合に使う

ジョブ ID を先に取得しておく必要がある場合（複数ジョブの並列投入など）は `qwait-mark` を使う。

```bash
# ジョブ投入
JOB_ID=$(qsub "node experiment.mjs" | grep -oP '(?<=JOB )\d+')

# 完了を待ち、./processed に 1 または -1 を追記する
qwait-mark "$JOB_ID"
# マーカーパスを変えたい場合は位置引数か --marker で指定できる:
#   qwait-mark "$JOB_ID" /tmp/my_marker
#   qwait-mark --marker /tmp/my_marker "$JOB_ID"

# 完了後の処理
if grep -q '^1$' ./processed; then
  echo "成功"
else
  echo "失敗"
fi
```

複数ジョブをまとめて待つ場合:

```bash
ID1=$(qsub "node exp_a.mjs" | grep -oP '(?<=JOB )\d+')
ID2=$(qsub "node exp_b.mjs" | grep -oP '(?<=JOB )\d+')

# 両方完了まで待機し /tmp/my_marker に書き込む（位置引数の末尾を非数値にすればマーカー扱い）
qwait-mark "$ID1" "$ID2" /tmp/my_marker
```

> **注意**: 実行中タスクの内部では `ls` ループでの待機が残っている場合があるが、
> 新規実験では必ず `qrun` または `qwait-mark` を使うこと。

## ポーリング禁止チェックリスト

新規スクリプトを書いた後・PR レビュー前に `lint-poll.sh` を実行して違反がないことを確認すること。

```bash
# artifacts/bitrag/ 以下を全スキャン（終了コード 0 = pass、1 = fail）
"$TREN/lint-poll.sh"               # source ~/UTILITY/tren/env.sh で $TREN がセット済み

# 特定実験ディレクトリのみスキャン
"$TREN/lint-poll.sh" artifacts/bitrag/experiment-XXX
```

| チェック項目 | 対応 |
|-------------|------|
| `while sleep …` ループがない | `qwait-mark` / `qwait` に置換する |
| `while` 内で `ls` を使っていない | `qwait-mark` / `qwait` に置換する |
| `watch ls` パターンがない | `qwait-mark` / `qwait` に置換する |

> `lint-poll.sh` が終了コード 1 を返した場合は、報告されたファイル・行番号を確認して修正すること。
> 詳細は `TOOLS_CATALOG.md` の `lint-poll` セクションを参照。

## タグ参照

タグの定義・一覧は `../tag.md` を見ること。
