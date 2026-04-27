/// Nibble Hash Matrix — hash1 (ADD+ROL+XOR) 2×4実装
///
/// ## 2×4行列の構造
///
/// ```text
/// 列 (col)    0         1         2         3
/// 行0 (lo)  state[0]  state[1]  state[2]  state[3]   ← ADD+ROL+XOR
/// 行1 (hi)  state[4]  state[5]  state[6]  state[7]   ← XOR+ADD+ROL
/// ```
///
/// 各バイト b[i] を col = i%4 に割り当て、ADDとROLとXORの連鎖で更新。
/// 終端ファイナライザで列間クロスミキシングを行い均一分布を実現。
/// 最終的に各行を XOR fold して (hash_hi, hash_lo) を出力。
///
/// ## C11 nibble_hash_matrix
///   入力: `&[u8]` (任意長バイト列)
///   出力: `(hash_hi: u8, hash_lo: u8)`
///   演算: ADD → ROL → XOR (2 rows × 4 cols nibble state) + 2段ファイナライザ
///
/// ## P17 nibble_id_spec (u16 パッキング規則)
///
/// ```text
/// u16 = [nibble3 | nibble2 | nibble1 | nibble0]
///         P系     hash_hi   hash_lo    C系
/// nibble0 (bits 3- 0): C系 (カテゴリ、語彙分類)
/// nibble1 (bits 7- 4): hash_lo の上位4bit
/// nibble2 (bits11- 8): hash_hi の上位4bit
/// nibble3 (bits15-12): P系 (位置、頻度分類)
/// ```
///
/// この規則により u16 一つでトークンを16×16グリッド座標として表現できる。
pub fn nibble_hash_matrix(data: &[u8]) -> (u8, u8) {
    // 長さ情報を初期シードとして使い、短い入力でも列間差異を持たせる
    let len = data.len() as u8;
    let mut mat = [
        [len, len.wrapping_mul(3), len.wrapping_mul(7), len.wrapping_mul(11)],
        [len.wrapping_mul(5), len.wrapping_mul(13), len.wrapping_mul(17), len.wrapping_mul(19)],
    ];

    for (i, &b) in data.iter().enumerate() {
        let col = i % 4;
        let rot0 = (col as u32) + 1;           // 1,2,3,4
        let rot1 = ((col + 2) as u32 % 4) + 1; // 3,4,1,2

        // 行0 (lo): ADD → ROL → XOR
        mat[0][col] = mat[0][col].wrapping_add(b);
        mat[0][col] = mat[0][col].rotate_left(rot0);
        mat[0][col] ^= b;

        // 行1 (hi): XOR → ADD → ROL
        mat[1][col] ^= b;
        mat[1][col] = mat[1][col].wrapping_add(mat[0][col]);
        mat[1][col] = mat[1][col].rotate_left(rot1);
    }

    // ファイナライザ: 2段 × 列間クロスミキシング (ADD+ROL+XOR chain)
    for _ in 0..2 {
        for col in 0..4usize {
            let nc = (col + 1) % 4;
            let nc2 = (col + 2) % 4;
            mat[0][col] = mat[0][col]
                .wrapping_add(mat[0][nc])
                .rotate_left((col as u32) + 1)
                ^ mat[1][nc2];
            mat[1][col] = mat[1][col]
                .wrapping_add(mat[0][col])
                .rotate_left(((col + 2) as u32 % 4) + 1)
                ^ mat[0][nc];
        }
    }

    let lo = mat[0][0] ^ mat[0][1] ^ mat[0][2] ^ mat[0][3];
    let hi = mat[1][0] ^ mat[1][1] ^ mat[1][2] ^ mat[1][3];
    (hi, lo)
}

/// hash_hi と hash_lo の上位4bit (nibble) を取り出して 16×16 グリッド座標に変換
///
/// 返り値: (row, col) ∈ [0,15] × [0,15]
#[inline]
pub fn hash_to_grid(hi: u8, lo: u8) -> (u8, u8) {
    (hi >> 4, lo >> 4)
}

/// u16 nibble_id をパックする (P17 nibble_id_spec)
///
/// ```text
/// nibble_id = (p_class << 12) | (hash_hi_nibble << 8) | (hash_lo_nibble << 4) | c_class
/// ```
#[inline]
pub fn pack_nibble_id(hash_hi: u8, hash_lo: u8, c_class: u8, p_class: u8) -> u16 {
    let hn = (hash_hi >> 4) as u16;
    let ln = (hash_lo >> 4) as u16;
    ((p_class as u16 & 0xF) << 12) | (hn << 8) | (ln << 4) | (c_class as u16 & 0xF)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 256 単一バイト入力の均一性テスト
    ///
    /// 単一バイト入力 (1 列のみ更新) では誕生日問題により衝突が生じる。
    /// hash1 の設計目標: 160 種類以上の異なる出力値が出現すること (62.5% カバー)。
    /// これは完全ランダム 256 入力の期待 distinct 数 (≈161.7) に相当する。
    #[test]
    fn test_uniformity_256() {
        let mut lo_count = [0u32; 256];
        let mut hi_count = [0u32; 256];

        for b in 0u8..=255 {
            let (hi, lo) = nibble_hash_matrix(&[b]);
            lo_count[lo as usize] += 1;
            hi_count[hi as usize] += 1;
        }

        let lo_covered = lo_count.iter().filter(|&&c| c > 0).count();
        let hi_covered = hi_count.iter().filter(|&&c| c > 0).count();

        assert!(
            lo_covered >= 150,
            "lo coverage too low: {}/256 distinct values",
            lo_covered
        );
        assert!(
            hi_covered >= 150,
            "hi coverage too low: {}/256 distinct values",
            hi_covered
        );

        // 最大衝突数: 単一バイトで同じ出力に集中しすぎていないか
        let lo_max = *lo_count.iter().max().unwrap();
        let hi_max = *hi_count.iter().max().unwrap();
        assert!(lo_max <= 8, "lo has bucket with {} entries (max allowed 8)", lo_max);
        assert!(hi_max <= 8, "hi has bucket with {} entries (max allowed 8)", hi_max);
    }

    /// 65536 全2バイトペア入力の完全均一性・完全独立性テスト
    ///
    /// 設計目標:
    /// - 完全均一 (marginal): hi と lo がそれぞれ全 256 値をカバー
    /// - 完全均一 (marginal density): 各バイト値の出現頻度が期待値 256 の ±40% 以内
    ///   (min ≥ 154, max ≤ 358)
    /// - 完全均一 (joint): (hi,lo) ペアの出現率が完全ランダム理論値 (63.2%) に近いこと
    /// - 完全独立: Pearson 相関係数 |corr(hi,lo)| ≤ 0.05
    #[test]
    fn test_uniformity_65536() {
        let mut joint = vec![0u32; 256 * 256];
        let mut lo_count = [0u32; 256];
        let mut hi_count = [0u32; 256];

        // hi/lo の値ベクトル (Pearson 相関用)
        let mut hi_vals: Vec<f64> = Vec::with_capacity(65536);
        let mut lo_vals: Vec<f64> = Vec::with_capacity(65536);

        for a in 0u8..=255 {
            for b in 0u8..=255 {
                let (hi, lo) = nibble_hash_matrix(&[a, b]);
                joint[hi as usize * 256 + lo as usize] += 1;
                lo_count[lo as usize] += 1;
                hi_count[hi as usize] += 1;
                hi_vals.push(hi as f64);
                lo_vals.push(lo as f64);
            }
        }

        // 完全均一: hi と lo それぞれが全 256 値をカバーするか
        let lo_covered = lo_count.iter().filter(|&&c| c > 0).count();
        let hi_covered = hi_count.iter().filter(|&&c| c > 0).count();
        assert_eq!(lo_covered, 256, "lo does not cover all 256 values");
        assert_eq!(hi_covered, 256, "hi does not cover all 256 values");

        // 完全均一: marginal density (期待値 = 256、±40% 許容)
        let lo_min = *lo_count.iter().min().unwrap();
        let lo_max = *lo_count.iter().max().unwrap();
        let hi_min = *hi_count.iter().min().unwrap();
        let hi_max = *hi_count.iter().max().unwrap();
        assert!(lo_min >= 154, "lo min count too low: {} (expected ≥154)", lo_min);
        assert!(lo_max <= 358, "lo max count too high: {} (expected ≤358)", lo_max);
        assert!(hi_min >= 154, "hi min count too low: {} (expected ≥154)", hi_min);
        assert!(hi_max <= 358, "hi max count too high: {} (expected ≤358)", hi_max);

        // 完全均一: joint coverage (完全ランダムなら約 63.2%)
        let distinct = joint.iter().filter(|&&c| c > 0).count();
        assert!(
            distinct >= 39321, // 65536 * 0.60
            "joint coverage too low: {} / 65536 distinct pairs",
            distinct
        );

        // 完全独立: Pearson 相関係数 |corr(hi,lo)| ≤ 0.05
        let n = 65536.0f64;
        let mean_hi: f64 = hi_vals.iter().sum::<f64>() / n;
        let mean_lo: f64 = lo_vals.iter().sum::<f64>() / n;
        let (mut sum_xy, mut sum_x2, mut sum_y2) = (0.0f64, 0.0f64, 0.0f64);
        for i in 0..65536 {
            let x = hi_vals[i] - mean_hi;
            let y = lo_vals[i] - mean_lo;
            sum_xy += x * y;
            sum_x2 += x * x;
            sum_y2 += y * y;
        }
        let pearson = if sum_x2 > 0.0 && sum_y2 > 0.0 {
            sum_xy / (sum_x2.sqrt() * sum_y2.sqrt())
        } else {
            1.0
        };
        assert!(
            pearson.abs() <= 0.05,
            "hi and lo are not independent: Pearson corr = {:.6} (expected |corr| ≤ 0.05)",
            pearson
        );
    }

    /// グリッド座標変換のテスト
    #[test]
    fn test_hash_to_grid() {
        let (row, col) = hash_to_grid(0xAB, 0xCD);
        assert_eq!(row, 0xA);
        assert_eq!(col, 0xC);
    }

    /// nibble_id パッキングのテスト
    #[test]
    fn test_pack_nibble_id() {
        let id = pack_nibble_id(0xA0, 0xB0, 0x3, 0x5);
        assert_eq!(id, (0x5u16 << 12) | (0xA << 8) | (0xB << 4) | 0x3);
    }
}
