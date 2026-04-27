use crate::bitset::DocBits;

/// Gini 係数 (スコア集中度の指標)
pub fn gini(v: &[f32]) -> f32 {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = s.len() as f32;
    let sum: f32 = s.iter().sum();
    if sum < 1e-12 { return 0.0; }
    let w: f32 = s.iter().enumerate()
        .map(|(i, &x)| (2 * i + 1) as f32 * x)
        .sum();
    (w / (n * sum)) - (n + 1.0) / n
}

/// スコア上位 k 件を (score, idx) で返す
pub fn top_k_idx(scores: &[f32], k: usize) -> Vec<(f32, usize)> {
    let mut v: Vec<(f32, usize)> = scores.iter().enumerate()
        .filter(|(_, &s)| s > 1e-9)
        .map(|(i, &s)| (s, i))
        .collect();
    v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    v.truncate(k);
    v
}

/// スコア上位 k 件を (score, text_snippet) で返す
pub fn top_k<'a>(scores: &[f32], texts: &[&'a str], k: usize) -> Vec<(f32, &'a str)> {
    top_k_idx(scores, k).into_iter().map(|(s, i)| (s, texts[i])).collect()
}

/// テキストを先頭 n 文字に切り詰める
pub fn shorten(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// スコアが閾値を超える文書数 (到達数)
pub fn reach_count(scores: &[f32], threshold: f32) -> usize {
    scores.iter().filter(|&&s| s > threshold).count()
}

/// 行和 (接続強度) の統計: (avg, max, argmax)
pub fn row_sum_stats(m: &[f32], n: usize) -> (f32, f32, usize) {
    let sums: Vec<f32> = (0..n).map(|i| m[i * n..(i + 1) * n].iter().sum()).collect();
    let avg = sums.iter().sum::<f32>() / n as f32;
    let (max_idx, &max) = sums.iter().enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap();
    (avg, max, max_idx)
}

// ── 長クエリ × トークン行列スコアリング ────────────────────────

/// IDF² 加重 coverage スコア (長クエリ用)
///
/// Python実装と同等:
///   wsum     = Σ_{g ∈ Q ∩ D} idf(g)²
///   coverage = |Q ∩ D| / |Q|
///   score    = wsum + 0.5·coverage + 0.5·max_idf²(Q∩D)
///
/// 返り値: (score, coverage, max_idf_sq)
pub fn long_query_score(
    qb: &DocBits,
    db: &DocBits,
    idf_sq: &[f32],
    q_size: usize,
) -> (f32, f32, f32) {
    let mut wsum = 0.0f32;
    let mut max_w = 0.0f32;
    let mut inter_count = 0u32;

    for w in 0..qb.nw {
        let base = w * 64;
        let mut bits = qb.words[w] & db.words[w];
        while bits != 0 {
            let pos = bits.trailing_zeros() as usize;
            let v = idf_sq[base + pos];
            wsum += v;
            if v > max_w { max_w = v; }
            inter_count += 1;
            bits &= bits - 1;
        }
    }

    if inter_count == 0 { return (0.0, 0.0, 0.0); }
    let coverage = inter_count as f32 / q_size.max(1) as f32;
    let score = wsum + 0.5 * coverage + 0.5 * max_w;
    (score, coverage, max_w)
}

// ── bit greedy set cover による文再構成 (生成) ──────────────────

/// Greedy Set Cover: クエリ gram を順次カバーする文書を選択
///
/// アルゴリズム:
///   remaining = G_q  (クエリ bitset のコピー)
///   while popcount(remaining) > 0 && 選択数 < max_docs:
///       j* = argmax popcount(remaining & G_j)  ← bit積で新規貢献量
///       remaining &= ~G_{j*}                   ← カバー済みをクリア
///       output.push(j*)
///
/// これは Minimum Set Cover の greedy 近似
/// (最適比率 ln(|Q|) 保証)
///
/// 返り値: (文書インデックス, 新規カバーgram数, 累積カバー率) のリスト
pub fn greedy_cover(
    query_bits: &DocBits,
    doc_bits: &[DocBits],
    max_docs: usize,
) -> Vec<(usize, u32, f32)> {
    let nw = query_bits.nw;
    let q_total: u32 = query_bits.words.iter().map(|w| w.count_ones()).sum();
    if q_total == 0 { return vec![]; }

    // remaining: カバーされていないクエリ gram の bitset
    let mut remaining = query_bits.words.clone();
    let mut covered = 0u32;
    let mut result = Vec::new();
    let mut used = vec![false; doc_bits.len()];

    for _ in 0..max_docs {
        // 各文書の新規貢献量を計算
        let (best_idx, best_gain) = doc_bits.iter().enumerate()
            .filter(|(i, _)| !used[*i])
            .map(|(i, db)| {
                let gain: u32 = remaining.iter().zip(&db.words)
                    .map(|(r, d)| (r & d).count_ones())
                    .sum();
                (i, gain)
            })
            .max_by_key(|(_, g)| *g)
            .unwrap_or((0, 0));

        if best_gain == 0 { break; }

        // remaining から best_idx の gram を除去
        let best_db = &doc_bits[best_idx];
        for i in 0..nw {
            remaining[i] &= !best_db.words[i];
        }

        covered += best_gain;
        used[best_idx] = true;
        let coverage_rate = covered as f32 / q_total as f32;
        result.push((best_idx, best_gain, coverage_rate));

        if coverage_rate >= 1.0 { break; }
    }

    result
}

/// IDF² 加重 Greedy Set Cover
///
/// argmax Σ_{g ∈ remaining ∩ G_j} idf(g)²  ← 高IDF gramを優先的にカバー
///
/// 返り値: (文書インデックス, 新規IDF²和, 累積IDF²カバー率) のリスト
pub fn greedy_cover_idf(
    query_bits: &DocBits,
    doc_bits: &[DocBits],
    idf_sq: &[f32],
    max_docs: usize,
) -> Vec<(usize, f32, f32)> {
    let nw = query_bits.nw;

    // クエリの総IDF²
    let q_total_idf: f32 = (0..nw).map(|w| {
        let base = w * 64;
        let mut s = 0.0f32;
        let mut bits = query_bits.words[w];
        while bits != 0 {
            let pos = bits.trailing_zeros() as usize;
            s += idf_sq[base + pos];
            bits &= bits - 1;
        }
        s
    }).sum();

    if q_total_idf < 1e-9 { return vec![]; }

    let mut remaining = query_bits.words.clone();
    let mut covered_idf = 0.0f32;
    let mut result = Vec::new();
    let mut used = vec![false; doc_bits.len()];

    for _ in 0..max_docs {
        // 各文書の新規IDF²貢献を計算
        let (best_idx, best_gain) = doc_bits.iter().enumerate()
            .filter(|(i, _)| !used[*i])
            .map(|(i, db)| {
                let gain: f32 = (0..nw).map(|w| {
                    let base = w * 64;
                    let mut s = 0.0f32;
                    let mut bits = remaining[w] & db.words[w];
                    while bits != 0 {
                        let pos = bits.trailing_zeros() as usize;
                        s += idf_sq[base + pos];
                        bits &= bits - 1;
                    }
                    s
                }).sum();
                (i, gain)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap_or((0, 0.0));

        if best_gain < 1e-9 { break; }

        let best_db = &doc_bits[best_idx];
        for i in 0..nw {
            remaining[i] &= !best_db.words[i];
        }

        covered_idf += best_gain;
        used[best_idx] = true;
        let coverage_rate = covered_idf / q_total_idf;
        result.push((best_idx, best_gain, coverage_rate));

        if coverage_rate >= 0.999 { break; }
    }

    result
}
