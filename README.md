# bitGradient

  **LLM-less 離散勾配降下法 (Discrete Gradient Descent / DGD)** の数学基礎・Rust 実装・実験記録。

  > nibble ビット代数による Rust コンパイルエラー修正の定式化として、
  > 連続勾配を使わず **整数スコアの差分とビット反転** だけで局所探索を行う離散最適化法。
  > 本リポジトリは [bitRAG](https://github.com/chikaharu/bitRAG) の `artifacts/bitrag/` 配下から
  > 離散勾配降下法に関する成果のみを抽出して独立管理するためのものです。

  ## 構成

  ```
  bitGradient/
  ├── docs/                        # 数学・理論ドキュメント
  │   ├── MATH_FOUNDATIONS.md         §10 多目的スコアリング関数と離散勾配降下 (本命)
  │   ├── THEORY_CORE_v1.md           理論コア
  │   ├── THEORY_FORMALIZATION.md     形式化
  │   ├── THEORY_LINEAGE.md           系譜
  │   ├── THEORY_NAND_COMPLETENESS.md NAND 完全性
  │   ├── THEORY_RUST_CODEGEN.md      Rust コード生成
  │   ├── THEORY_EVIDENCE_LEDGER.md   証拠台帳
  │   └── LLMless修正提案_概要原理実装結果.md  概要・原理・実装・結果
  │
  ├── bitrag-core/                 # Rust 実装 (no_std friendly, float 不使用)
  │   ├── Cargo.toml / Cargo.lock
  │   └── src/
  │       ├── lib.rs
  │       ├── gold_cycle.rs         greedy hill climbing 共通骨格 (DGD ドライバ)
  │       ├── sign2.rs              Sign2 多平面
  │       ├── iso.rs                同型判定
  │       ├── eval.rs               評価関数
  │       ├── bitset.rs / corpus.rs / ngram.rs / idf.rs / matrix.rs
  │       ├── nibble4_tokenizer.rs / nibble_hash.rs
  │       ├── object_capsule.rs / fonts.rs
  │
  ├── experiments/                 # 実験結果
  │   ├── EXPERIMENTS.md            E001..E209 詳細記録 (E207 delete hill climb / E208 novelty / E209 1-bit-flip DGD など)
  │   ├── EXPERIMENTS_SUMMARY.md    要約
  │   ├── REPORT_ensemble29.md      ensemble29 レポート
  │   ├── OODA_history.json         OODA ループ履歴
  │   └── OODAテンプレート.md
  │
  └── methods/                     # マテリアル & メソッド要約
      ├── COMPONENT_CATALOG.md       コンポーネント (材料) カタログ
      ├── TOOLS_CATALOG.md           ツール (機材) カタログ
      └── EXPERIMENT_TEMPLATE/       実験テンプレ (条件・仮説・結果フォーマット)
  ```

  ## 離散勾配降下 (DGD) とは

  連続勾配 `∇f` を使わず、

  1. 状態を nibble / bitset で表現し、
  2. 1-bit flip など **離散な近傍** を生成、
  3. **整数スコアの差分** `H(t) − H(s) < 0` のみを採用 (strict/stationary の 2 段)、

  を繰り返す **greedy hill climbing** 系の離散最適化。
  詳細な定義と多目的スコアリング関数は [`docs/MATH_FOUNDATIONS.md` §10](./docs/MATH_FOUNDATIONS.md) を参照。

  ## 抽出元

  - 上流リポジトリ: `chikaharu/bitRAG` の `artifacts/bitrag/` ツリー
  - 抽出日: 2026-04-27
  - 抽出方針: 数学ドキュメント + Rust 実装一式 + 実験結果 + マテリアル/メソッド要約

  ## ビルド

  ```bash
  cd bitrag-core
  cargo build --release
  ```

  ## ライセンス

  未指定 (上流 bitRAG に準拠)。
  