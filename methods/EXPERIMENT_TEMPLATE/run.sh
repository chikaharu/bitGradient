#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

# tren scheduler の env.sh を source することで qrun / qsub / qstat / qlog /
# qwait / qwait-mark がフルパスなしで使えるようになる。
#
# tren ソースはリポジトリ内 `artifacts/bitrag/scheduler/` を single source
# of truth とする (Task #21)。下のパスはこのテンプレートが置かれている
# 実験ディレクトリ (artifacts/bitrag/experiment-XXX/) からの相対パス。
#
# shellcheck source=/dev/null
source "$(git rev-parse --show-toplevel)/artifacts/bitrag/scheduler/env.sh"

# ------------------------------------------------------------
# ビルド
# ------------------------------------------------------------
# cargo build --release

# ------------------------------------------------------------
# 実験実行
# ------------------------------------------------------------
mkdir -p results

# 典型的な実行パターン:
#   ./target/release/<experiment-binary> > results/result.txt 2>&1
#
# qrun を使って非同期ジョブとして投入する場合:
#   qrun -- ./target/release/<experiment-binary>
