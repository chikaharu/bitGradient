use crate::bitset::DocBits;
use crate::idf::Vocab;
use std::collections::BTreeSet;

/// バイナリ Jaccard で M_doc を構築する
pub fn build_mdoc_binary(doc_bits: &[DocBits], n: usize) -> Vec<f32> {
    let mut m = vec![0.0f32; n * n];
    for i in 0..n {
        for j in i..n {
            let s = doc_bits[i].jaccard_binary(&doc_bits[j]);
            m[i * n + j] = s;
            m[j * n + i] = s;
        }
    }
    m
}

/// IDF² shift+popcount Jaccard で M_doc を構築する
pub fn build_mdoc_idf(doc_bits: &[DocBits], vocab: &Vocab, n: usize) -> Vec<f32> {
    let mut m = vec![0.0f32; n * n];
    for i in 0..n {
        for j in i..n {
            let s = doc_bits[i].jaccard_idf(&doc_bits[j], &vocab.idf_sq);
            m[i * n + j] = s;
            m[j * n + i] = s;
        }
    }
    m
}

/// 高頻度 gram をマスクした IDF² Jaccard で M_doc を構築する
///
/// mask_threshold: この df を超える gram を除外 (0 = マスクなし)
pub fn build_mdoc_idf_masked(
    gram_lists: &[BTreeSet<String>],
    vocab: &Vocab,
    n: usize,
    mask_threshold: usize,
) -> Vec<f32> {
    let masked_idf_sq: Vec<f32> = vocab.idf_sq.iter().enumerate().map(|(i, &v)| {
        if i < vocab.terms.len() {
            let df = vocab.df.get(&vocab.terms[i]).copied().unwrap_or(0);
            if mask_threshold > 0 && df > mask_threshold { 0.0 } else { v }
        } else { 0.0 }
    }).collect();

    let doc_bits: Vec<DocBits> = gram_lists.iter()
        .map(|gl| vocab.to_docbits(gl))
        .collect();

    let mut m = vec![0.0f32; n * n];
    for i in 0..n {
        for j in i..n {
            let s = doc_bits[i].jaccard_idf(&doc_bits[j], &masked_idf_sq);
            m[i * n + j] = s;
            m[j * n + i] = s;
        }
    }
    m
}

/// 行 L1 正規化 (行和を 1 にする)
///
/// PPR の収束保証に必要。
pub fn row_normalize(m: &mut [f32], n: usize) {
    for i in 0..n {
        let s: f32 = m[i * n..(i + 1) * n].iter().sum();
        if s > 1e-12 {
            for j in 0..n { m[i * n + j] /= s; }
        }
    }
}

/// Personalized PageRank (PPR)
///
/// v_k = (1-λ)·M·v_{k-1} + λ·v₀
///
/// 前提: M は行正規化済み (固有値 ≤ 1 → 収束保証)
pub fn ppr(v0: &[f32], m: &[f32], n: usize, lambda: f32, steps: usize) -> Vec<f32> {
    let mut v = v0.to_vec();
    for _ in 0..steps {
        let mut nv = vec![0.0f32; n];
        for i in 0..n {
            if v[i] < 1e-12 { continue; }
            for j in 0..n { nv[j] += v[i] * m[i * n + j]; }
        }
        for (a, b) in nv.iter_mut().zip(v0) {
            *a = (1.0 - lambda) * *a + lambda * b;
        }
        v = nv;
    }
    v
}

/// バイナリ Shift 畳み込みで M_doc を構築する
///
/// M_doc[i][j] = sim_shift(D_i, D_j, max_shift, decay)
pub fn build_mdoc_shift(
    doc_bits: &[DocBits],
    n: usize,
    max_shift: usize,
    decay: f32,
) -> Vec<f32> {
    let mut m = vec![0.0f32; n * n];
    for i in 0..n {
        for j in i..n {
            let s = doc_bits[i].sim_shift(&doc_bits[j], max_shift, decay);
            m[i * n + j] = s;
            m[j * n + i] = s;
        }
    }
    m
}

/// IDF² 加重 Shift 畳み込みで M_doc を構築する
///
/// M_doc[i][j] = sim_shift_idf(D_i, D_j, idf_sq, max_shift, decay)
pub fn build_mdoc_shift_idf(
    doc_bits: &[DocBits],
    vocab: &Vocab,
    n: usize,
    max_shift: usize,
    decay: f32,
) -> Vec<f32> {
    let mut m = vec![0.0f32; n * n];
    for i in 0..n {
        for j in i..n {
            let s = doc_bits[i].sim_shift_idf(
                &doc_bits[j], &vocab.idf_sq, max_shift, decay,
            );
            m[i * n + j] = s;
            m[j * n + i] = s;
        }
    }
    m
}

/// M^k ホップ (生の行列累乗、正規化なし)
///
/// v_k = M^k · v₀
pub fn hop(v: &[f32], m: &[f32], n: usize) -> Vec<f32> {
    let mut nv = vec![0.0f32; n];
    for i in 0..n {
        if v[i] < 1e-12 { continue; }
        for j in 0..n { nv[j] += v[i] * m[i * n + j]; }
    }
    nv
}
