use std::collections::{BTreeSet, HashMap};
use crate::bitset::DocBits;

/// 語彙・df・IDF 重み・bins を保持する構造体
pub struct Vocab {
    /// ソート済み gram 一覧
    pub terms: Vec<String>,
    /// gram → インデックス
    pub idx: HashMap<String, usize>,
    /// gram → 文書頻度
    pub df: HashMap<String, usize>,
    /// gram インデックス → idf(g)²  (nw*64 長、未割り当ては 0)
    pub idf_sq: Vec<f32>,
    /// bitset 幅 (u64 ワード数)
    pub nw: usize,
    /// 量子化 IDF bin 数
    pub n_bins: usize,
    /// bin b の代表重み (bin midpoint²)
    pub bin_weights: Vec<f32>,
}

impl Vocab {
    /// gram_lists から語彙・df・idf_sq・bins を一括構築
    ///
    /// n_docs: コーパス文書数 (IDF の分母)
    /// n_bins: 量子化 bin 数 (0 なら bins を構築しない)
    pub fn build(
        gram_lists: &[BTreeSet<String>],
        n_docs: usize,
        n_bins: usize,
    ) -> Self {
        let mut df: HashMap<String, usize> = HashMap::new();
        for gl in gram_lists {
            for g in gl { *df.entry(g.clone()).or_default() += 1; }
        }
        let mut terms: Vec<String> = df.keys().cloned().collect();
        terms.sort();
        let idx: HashMap<String, usize> =
            terms.iter().enumerate().map(|(i, g)| (g.clone(), i)).collect();

        let nw = (terms.len() + 63) / 64;
        let vocab_sz = nw * 64;

        let mut idf_sq = vec![0.0f32; vocab_sz];
        for (g, &d) in &df {
            let idf = ((n_docs as f32 + 1.0) / (d as f32 + 1.0)).ln() + 1.0;
            if let Some(&i) = idx.get(g) { idf_sq[i] = idf * idf; }
        }

        let idf_max = idf_sq.iter().cloned().fold(0.0f32, f32::max).sqrt();
        let bin_width = if n_bins > 0 { idf_max / n_bins as f32 } else { 1.0 };
        let bin_weights: Vec<f32> = (0..n_bins)
            .map(|b| { let mid = (b as f32 + 0.5) * bin_width; mid * mid })
            .collect();

        Self { terms, idx, df, idf_sq, nw, n_bins, bin_weights }
    }

    /// 文書 gram_set から DocBits を作成する
    pub fn to_docbits(&self, grams: &BTreeSet<String>) -> DocBits {
        let mut db = DocBits::new(self.nw);
        for g in grams {
            if let Some(&i) = self.idx.get(g) { db.set(i); }
        }
        db
    }

    /// 文書 gram_set から bins 分解を作成する
    ///
    /// 戻り値: bin_bits[b] = Vec<u64> (長さ nw)
    pub fn to_bins(&self, grams: &BTreeSet<String>) -> Vec<Vec<u64>> {
        let mut bins: Vec<Vec<u64>> = vec![vec![0u64; self.nw]; self.n_bins.max(1)];
        let bin_width = if self.n_bins > 0 {
            self.idf_sq.iter().cloned().fold(0.0f32, f32::max).sqrt() / self.n_bins as f32
        } else { 1.0 };
        for g in grams {
            if let Some(&i) = self.idx.get(g) {
                let idf_v = self.idf_sq[i].sqrt();
                let b = ((idf_v / bin_width) as usize).min(self.n_bins.saturating_sub(1));
                bins[b][i / 64] |= 1u64 << (i % 64);
            }
        }
        bins
    }

    /// idf(g) を返す (未登録語は 0)
    pub fn idf(&self, g: &str) -> f32 {
        self.df.get(g).map(|&d| {
            let n = self.df.len();
            ((n as f32 + 1.0) / (d as f32 + 1.0)).ln() + 1.0
        }).unwrap_or(0.0)
    }

    /// B-bit ビット平面表現を構築する
    ///
    /// IDF値を [0, 2^B-1] の整数に量子化し、各ビット位置 k に対して
    /// plane_k = { g | bit k of quant(idf(g)) = 1 } を返す。
    ///
    /// 使い方:
    ///   let planes = vocab.idf_planes(8);
    ///   sum = planes.dot(Q & D)  ≈  Σ_{g∈Q∩D} idf(g)
    pub fn idf_planes(&self, b_bits: usize) -> IdfPlanes {
        IdfPlanes::build(&self.idf_sq, self.nw, b_bits)
    }
}

// ── IDF ビット平面 ─────────────────────────────────────────────────

/// IDF値を B-bit に量子化したビット平面
///
/// plane[k] は nw 個の u64 ワードからなる bitset。
/// gram g のビット k が 1 ⟺ quant(idf(g)) の bit k が 1
///
/// Σ_{g∈S} idf(g) ≈ Σ_k  scale * 2^k * popcount(S_bits & plane[k])
pub struct IdfPlanes {
    /// plane[k][w] = bitset ワード w のビット平面 k
    pub planes: Vec<Vec<u64>>,
    /// ビット数 B
    pub b_bits: usize,
    /// bitset ワード数
    pub nw: usize,
    /// 量子化スケール (1量子化単位 = scale の idf 値)
    pub scale: f32,
    /// 各文書の事前計算済み sum_idf (オプション)
    pub doc_sum: Vec<f32>,
}

impl IdfPlanes {
    /// idf_sq 配列から IdfPlanes を構築
    pub fn build(idf_sq: &[f32], nw: usize, b_bits: usize) -> Self {
        let levels = (1u32 << b_bits) as f32 - 1.0;
        // idf の最大値
        let max_idf = idf_sq.iter().cloned().fold(0.0f32, f32::max).sqrt();
        let scale = if max_idf > 0.0 { max_idf / levels } else { 1.0 };

        let vocab_sz = nw * 64;
        let mut planes: Vec<Vec<u64>> = vec![vec![0u64; nw]; b_bits];

        for i in 0..vocab_sz.min(idf_sq.len()) {
            let idf = idf_sq[i].sqrt();
            if idf < 1e-9 { continue; }
            let q = ((idf / max_idf * levels).round() as u32).min(levels as u32);
            for k in 0..b_bits {
                if (q >> k) & 1 == 1 {
                    planes[k][i / 64] |= 1u64 << (i % 64);
                }
            }
        }

        Self { planes, b_bits, nw, scale, doc_sum: vec![] }
    }

    /// bitset S に含まれる gram の IDF 和を近似計算
    ///
    ///   sum ≈ scale · Σ_k  2^k · popcount(S & plane_k)
    #[inline]
    pub fn sum_bits(&self, s_words: &[u64]) -> f32 {
        let mut total = 0u32;
        let scale_int: u32 = 1;
        for k in 0..self.b_bits {
            let cnt: u32 = s_words.iter().zip(&self.planes[k])
                .map(|(sw, pk)| (sw & pk).count_ones())
                .sum();
            total += cnt * (scale_int << k);
        }
        total as f32 * self.scale
    }

    /// intersection (Q & D) の IDF 和を直接計算 (Q と D の words を渡す)
    #[inline]
    pub fn sum_inter(&self, q_words: &[u64], d_words: &[u64]) -> f32 {
        let mut total = 0u32;
        for k in 0..self.b_bits {
            let cnt: u32 = q_words.iter().zip(d_words).zip(&self.planes[k])
                .map(|((qw, dw), pk)| (qw & dw & pk).count_ones())
                .sum();
            total += cnt * (1u32 << k);
        }
        total as f32 * self.scale
    }

    /// pair Jaccard (IDF) を近似計算
    ///
    ///   J_pair = S_inter² / (S_q² + S_d² - S_inter²)
    ///   S_inter = Σ_{g∈Q∩D} idf(g)  (近似)
    ///   S_q     = Σ_{g∈Q}   idf(g)  (近似)
    ///   S_d     = Σ_{g∈D}   idf(g)  (近似)
    #[inline]
    pub fn pair_jaccard(&self, q_words: &[u64], d_words: &[u64]) -> f32 {
        let sq = self.sum_bits(q_words);
        let sd = self.sum_bits(d_words);
        let si = self.sum_inter(q_words, d_words);
        let i2 = si * si;
        let denom = sq * sq + sd * sd - i2;
        if denom < 1e-12 { 0.0 } else { i2 / denom }
    }
}
