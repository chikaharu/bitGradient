/// gram インデックスを bit 位置とする固定幅 bitset
///
/// ## Shift+popcount の数学的意味
///
/// gram インデックスはアルファベット順に割り当てられており、
/// 文字的に近い gram は語彙空間で近傍に位置する傾向がある。
///
/// ```text
/// k=0: popcount(A & B)          完全一致
/// k=1: popcount(A & (B >> 1))   B を 1 位置右シフト → 語彙隣接 gram の一致
/// k=2: popcount(A & (B >> 2))   2 位置ずれ許容
/// ```
///
/// 合計スコア:
///   sim_shift = Σ_{k=0}^{K} decay^k · [popcount(A & (B>>k)) + popcount(A & (B<<k))]
///
/// これは 1D 畳み込み:
///   conv(A, B)[k] = popcount(A & (B >> k))
///   sim = Σ_k w(k) · conv(A, B)[k]   ← 「論理回路で畳み込み」
///
/// 効果:
///   - OOV gram の"語彙近傍"が hit → 近似OOV補完
///   - IDF との組み合わせで高IDF近傍に集中 → 意味的近似一致
pub struct DocBits {
    pub words: Vec<u64>,
    pub nw: usize,
}

impl DocBits {
    pub fn new(nw: usize) -> Self {
        Self { words: vec![0u64; nw], nw }
    }

    /// gram インデックス i のビットをセットする
    pub fn set(&mut self, i: usize) {
        self.words[i / 64] |= 1u64 << (i % 64);
    }

    /// bitset 全体を右に k bit シフトした新しい words を返す
    ///
    /// word i の結果 = word[i + kw] >> kb | word[i + kw + 1] << (64-kb)
    /// (多倍長整数の右シフトと同等)
    pub fn shift_right(&self, k: usize) -> Vec<u64> {
        let kw = k / 64;
        let kb = k % 64;
        let mut out = vec![0u64; self.nw];
        for i in 0..self.nw {
            let lo = self.words.get(i + kw).copied().unwrap_or(0);
            out[i] = if kb == 0 {
                lo
            } else {
                let hi = self.words.get(i + kw + 1).copied().unwrap_or(0);
                (lo >> kb) | (hi << (64 - kb))
            };
        }
        out
    }

    /// bitset 全体を左に k bit シフトした新しい words を返す
    pub fn shift_left(&self, k: usize) -> Vec<u64> {
        let kw = k / 64;
        let kb = k % 64;
        let mut out = vec![0u64; self.nw];
        for i in kw..self.nw {
            let lo = self.words[i - kw];
            out[i] = if kb == 0 {
                lo
            } else {
                let hi = if i > kw { self.words[i - kw - 1] } else { 0 };
                (lo << kb) | (hi >> (64 - kb))
            };
        }
        out
    }

    // ── 合成代数 bit 演算 (THEORY_RUST_CODEGEN.md §3) ───────────

    /// A ∧ B (要素ごと AND, 新規割当)
    pub fn and(&self, other: &Self) -> DocBits {
        assert_eq!(self.nw, other.nw, "DocBits::and: nw mismatch ({} vs {})", self.nw, other.nw);
        let words = self.words.iter().zip(&other.words)
            .map(|(a, b)| a & b).collect();
        DocBits { words, nw: self.nw }
    }

    /// A ∨ B (要素ごと OR, 新規割当)
    pub fn or(&self, other: &Self) -> DocBits {
        assert_eq!(self.nw, other.nw, "DocBits::or: nw mismatch ({} vs {})", self.nw, other.nw);
        let words = self.words.iter().zip(&other.words)
            .map(|(a, b)| a | b).collect();
        DocBits { words, nw: self.nw }
    }

    /// A ⊕ B (要素ごと XOR, 新規割当)
    pub fn xor(&self, other: &Self) -> DocBits {
        assert_eq!(self.nw, other.nw, "DocBits::xor: nw mismatch ({} vs {})", self.nw, other.nw);
        let words = self.words.iter().zip(&other.words)
            .map(|(a, b)| a ^ b).collect();
        DocBits { words, nw: self.nw }
    }

    /// A ⊖ B := A AND NOT B (差集合, 新規割当)
    pub fn andnot(&self, other: &Self) -> DocBits {
        assert_eq!(self.nw, other.nw, "DocBits::andnot: nw mismatch ({} vs {})", self.nw, other.nw);
        let words = self.words.iter().zip(&other.words)
            .map(|(a, b)| a & !b).collect();
        DocBits { words, nw: self.nw }
    }

    /// mask_idf(τ): idf_sq[i] >= τ となる位置のみ残す masked bitset を返す
    ///
    /// 高IDF gram のみを残して低IDF ノイズを除去する射影演算 (THEORY §3 q_B)。
    pub fn mask_idf(&self, idf_sq: &[f32], tau: f32) -> DocBits {
        let mut out = DocBits::new(self.nw);
        for w in 0..self.nw {
            let base = w * 64;
            let mut bits = self.words[w];
            let mut kept = 0u64;
            while bits != 0 {
                let pos = bits.trailing_zeros() as usize;
                let idx = base + pos;
                if idx < idf_sq.len() && idf_sq[idx] >= tau {
                    kept |= 1u64 << pos;
                }
                bits &= bits - 1;
            }
            out.words[w] = kept;
        }
        out
    }

    // ── 既存メソッド (変更なし) ──────────────────────────────────

    /// バイナリ Jaccard: popcount(A & B) / popcount(A | B)
    pub fn jaccard_binary(&self, other: &Self) -> f32 {
        let (mut inter, mut union) = (0u32, 0u32);
        for (a, b) in self.words.iter().zip(&other.words) {
            inter += (a & b).count_ones();
            union += (a | b).count_ones();
        }
        if union == 0 { 0.0 } else { inter as f32 / union as f32 }
    }

    /// IDF² 加重 Jaccard — trailing_zeros shift+popcount 実装
    ///
    /// ```text
    /// while bits != 0 {
    ///     let pos = bits.trailing_zeros();  // shift で LSB 位置を取得
    ///     sum += idf_sq[base + pos];
    ///     bits &= bits - 1;                // LSB クリア
    /// }
    /// ```
    pub fn jaccard_idf(&self, other: &Self, idf_sq: &[f32]) -> f32 {
        let (mut inter, mut union) = (0.0f32, 0.0f32);
        for w in 0..self.nw {
            let base = w * 64;
            let a = self.words[w];
            let b = other.words[w];
            let mut bits = a & b;
            while bits != 0 {
                let pos = bits.trailing_zeros() as usize;
                inter += idf_sq[base + pos];
                bits &= bits - 1;
            }
            let mut bits = a | b;
            while bits != 0 {
                let pos = bits.trailing_zeros() as usize;
                union += idf_sq[base + pos];
                bits &= bits - 1;
            }
        }
        if union < 1e-9 { 0.0 } else { inter / union }
    }

    /// IDF bin 量子化版 Jaccard (K=8 近似)
    pub fn jaccard_idf_bins(
        bin_a: &[Vec<u64>],
        bin_b: &[Vec<u64>],
        bin_weights: &[f32],
        nw: usize,
    ) -> f32 {
        let (mut inter, mut union) = (0.0f32, 0.0f32);
        for (b, &w) in bin_weights.iter().enumerate() {
            let mut r = 0u32;
            let mut u = 0u32;
            for i in 0..nw {
                r += (bin_a[b][i] & bin_b[b][i]).count_ones();
                u += (bin_a[b][i] | bin_b[b][i]).count_ones();
            }
            inter += w * r as f32;
            union += w * u as f32;
        }
        if union < 1e-9 { 0.0 } else { inter / union }
    }

    // ── 新規: Shift+popcount 畳み込み類似度 ──────────────────────

    /// バイナリ Shift 類似度
    ///
    /// score = Σ_{k=0}^{max_shift} decay^k · [popcount(A & (B>>k)) + popcount(A & (B<<k))]
    ///
    /// 正規化分母: |A| + |B| (k=0 の和集合カウントの近似)
    /// decay=1.0 → 全シフト等重み / decay<1.0 → 遠いシフトを減衰
    pub fn sim_shift(&self, other: &Self, max_shift: usize, decay: f32) -> f32 {
        let sa: f32 = self.words.iter().map(|w| w.count_ones() as f32).sum();
        let sb: f32 = other.words.iter().map(|w| w.count_ones() as f32).sum();
        let denom = sa + sb;
        if denom < 1e-9 { return 0.0; }

        let mut score = 0.0f32;
        let mut w = 1.0f32;

        // k=0: 完全一致 (AND + count_ones)
        let exact: u32 = self.words.iter().zip(&other.words)
            .map(|(a, b)| (a & b).count_ones()).sum();
        score += w * exact as f32 * 2.0;  // ×2 で左右対称に相当

        // k=1..max_shift: 語彙空間シフト
        for k in 1..=max_shift {
            w *= decay;
            // B >> k (高インデックス gram を低インデックス方向へ)
            let shifted_r = other.shift_right(k);
            let inter_r: u32 = self.words.iter().zip(&shifted_r)
                .map(|(a, b)| (a & b).count_ones()).sum();
            // B << k (低インデックス gram を高インデックス方向へ)
            let shifted_l = other.shift_left(k);
            let inter_l: u32 = self.words.iter().zip(&shifted_l)
                .map(|(a, b)| (a & b).count_ones()).sum();
            score += w * (inter_r + inter_l) as f32;
        }

        score / denom
    }

    // ── 相互相関 (Cross-Correlation) ─────────────────────────────

    /// bit 積の相互相関関数 (binary)
    ///
    /// conv[k] = popcount(A & (B << k))  for k = -max_k .. +max_k
    ///
    /// 解釈:
    ///   k=0  → 完全一致 (標準 AND popcount)
    ///   k>0  → B を k bit 右に "ずらして" A と突き合わせる
    ///          ≡ 語彙インデックスが k だけ大きい gram のずれ許容一致
    ///   k<0  → 逆方向シフト
    ///
    /// 返り値: Vec<(k, count)>  k = -max_k..=max_k 順
    pub fn xcorr(&self, other: &Self, max_k: usize) -> Vec<(i32, u32)> {
        let mut result = Vec::with_capacity(2 * max_k + 1);
        for ki in -(max_k as i32)..=(max_k as i32) {
            let shifted = if ki >= 0 {
                other.shift_left(ki as usize)   // B << k → 右シフト方向で一致
            } else {
                other.shift_right((-ki) as usize)
            };
            let count: u32 = self.words.iter().zip(&shifted)
                .map(|(a, b)| (a & b).count_ones()).sum();
            result.push((ki, count));
        }
        result
    }

    /// IDF² 加重 相互相関関数
    ///
    /// conv_idf[k] = Σ_{g ∈ A ∩ shift_k(B)} idf(g)²
    ///
    /// k=0 の値が標準 IDF² 加重 intersection と一致する。
    /// k≠0 では語彙的に隣接する gram も部分スコアとして拾う。
    pub fn xcorr_idf(&self, other: &Self, idf_sq: &[f32], max_k: usize) -> Vec<(i32, f32)> {
        let mut result = Vec::with_capacity(2 * max_k + 1);
        for ki in -(max_k as i32)..=(max_k as i32) {
            let shifted = if ki >= 0 {
                other.shift_left(ki as usize)
            } else {
                other.shift_right((-ki) as usize)
            };
            let mut score = 0.0f32;
            for w in 0..self.nw {
                let base = w * 64;
                let mut bits = self.words[w] & shifted[w];
                while bits != 0 {
                    let pos = bits.trailing_zeros() as usize;
                    score += idf_sq[base + pos];
                    bits &= bits - 1;
                }
            }
            result.push((ki, score));
        }
        result
    }

    /// 相互相関から畳み込みスコアを計算する
    ///
    /// score = Σ_k decay^|k| · conv[k]  / norm
    ///
    /// xcorr の結果をそのまま渡す (binary or IDF)
    pub fn xcorr_to_score(xcorr: &[(i32, f32)], decay: f32, norm: f32) -> f32 {
        if norm < 1e-9 { return 0.0; }
        let score: f32 = xcorr.iter()
            .map(|(k, v)| decay.powi(k.abs()) * v)
            .sum();
        score / norm
    }

    /// IDF² 加重 Shift 類似度
    ///
    /// 各シフト k で AND した bit 列を trailing_zeros で走査し
    /// idf_sq[pos] を積算する。高IDF gram 付近のシフト一致を重視。
    ///
    /// score = Σ_{k} decay^k · Σ_{g ∈ A ∩ shift_k(B)} idf(g)²
    /// / Σ_{g ∈ A ∪ B} idf(g)²   (k=0 union で正規化)
    pub fn sim_shift_idf(
        &self,
        other: &Self,
        idf_sq: &[f32],
        max_shift: usize,
        decay: f32,
    ) -> f32 {
        // 分母: k=0 の IDF² 和集合
        let mut union = 0.0f32;
        for w in 0..self.nw {
            let base = w * 64;
            let mut bits = self.words[w] | other.words[w];
            while bits != 0 {
                let pos = bits.trailing_zeros() as usize;
                union += idf_sq[base + pos];
                bits &= bits - 1;
            }
        }
        if union < 1e-9 { return 0.0; }

        let mut score = 0.0f32;
        let mut wt = 1.0f32;

        // k=0: 完全一致 (trailing_zeros で idf 積算)
        for w in 0..self.nw {
            let base = w * 64;
            let mut bits = self.words[w] & other.words[w];
            while bits != 0 {
                let pos = bits.trailing_zeros() as usize;
                score += wt * idf_sq[base + pos];
                bits &= bits - 1;
            }
        }

        // k=1..max_shift
        for k in 1..=max_shift {
            wt *= decay;
            let shifted_r = other.shift_right(k);
            let shifted_l = other.shift_left(k);
            for w in 0..self.nw {
                let base = w * 64;
                // 右シフト
                let mut bits = self.words[w] & shifted_r[w];
                while bits != 0 {
                    let pos = bits.trailing_zeros() as usize;
                    score += wt * idf_sq[base + pos];
                    bits &= bits - 1;
                }
                // 左シフト
                let mut bits = self.words[w] & shifted_l[w];
                while bits != 0 {
                    let pos = bits.trailing_zeros() as usize;
                    score += wt * idf_sq[base + pos];
                    bits &= bits - 1;
                }
            }
        }

        score / union
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db(nw: usize, bits: &[usize]) -> DocBits {
        let mut d = DocBits::new(nw);
        for &i in bits { d.set(i); }
        d
    }

    #[test]
    fn test_and_or_xor_andnot() {
        let a = db(2, &[0, 5, 70]);       // {0, 5, 70}
        let b = db(2, &[5, 70, 100]);     // {5, 70, 100}

        let collect = |d: &DocBits| -> Vec<usize> {
            let mut v = vec![];
            for w in 0..d.nw {
                let mut bits = d.words[w];
                while bits != 0 {
                    let p = bits.trailing_zeros() as usize;
                    v.push(w * 64 + p);
                    bits &= bits - 1;
                }
            }
            v
        };
        assert_eq!(collect(&a.and(&b)), vec![5, 70]);
        assert_eq!(collect(&a.or(&b)), vec![0, 5, 70, 100]);
        assert_eq!(collect(&a.xor(&b)), vec![0, 100]);
        assert_eq!(collect(&a.andnot(&b)), vec![0]);
        assert_eq!(collect(&b.andnot(&a)), vec![100]);

        // returned DocBits invariants preserved
        let r = a.and(&b);
        assert_eq!(r.nw, a.nw);
        assert_eq!(r.words.len(), a.nw);
    }

    #[test]
    #[should_panic(expected = "nw mismatch")]
    fn test_and_dimension_mismatch_panics() {
        let a = DocBits::new(2);
        let b = DocBits::new(3);
        let _ = a.and(&b);
    }

    #[test]
    fn test_mask_idf_threshold() {
        // nw=1: positions 0..=3 set
        let a = db(1, &[0, 1, 2, 3]);
        let mut idf_sq = vec![0.0f32; 64];
        idf_sq[0] = 0.1; idf_sq[1] = 0.5; idf_sq[2] = 1.0; idf_sq[3] = 2.0;

        let masked = a.mask_idf(&idf_sq, 0.5);
        // keep positions where idf_sq >= 0.5 → {1, 2, 3}
        let expected = (1u64 << 1) | (1u64 << 2) | (1u64 << 3);
        assert_eq!(masked.words[0], expected);
        assert_eq!(masked.nw, a.nw);

        // tau larger than max → empty
        let masked2 = a.mask_idf(&idf_sq, 10.0);
        assert_eq!(masked2.words[0], 0);
    }
}
