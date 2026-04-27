//! GOLDCYCLE 共通骨格 — function / parameter 分離
//!
//! E71a (二値 {-1,+1}) / E71b (nibble {0..15}) / E72 / E77 / E80 / E81 で
//! 共通する「GOLD 系列 × 循環クロス相関 × nibble ウォーク」骨格。
//!
//! ## function (作用素) と parameter (差し込み点) の対応表
//!
//! | function (このモジュール) | parameter (呼び出し側で差し込む) |
//! |---------------------------|----------------------------------|
//! | `m_seq_10(tap_poly)`      | tap_poly: 原始多項式マスク        |
//! | `gold_seq()`              | (固定: preferred pair 0x09, d=65) |
//! | `circ_xcorr_i64<T>`       | T: スカラ体 (i32 / u8 / ...)      |
//! | `gold_autocorr`           | g: ±1 GOLD 系列                  |
//! | `top_k_shifts`            | k: 上位件数                       |
//! | `build_phi`               | idf_sq, V, L                      |
//! | `gold_encode_indices`     | gram_indices, phi, gold_g         |
//! | `apply_nib_pos`           | (nib_idx, new_nib)                |
//! | `to_bin_nib` / `to_nib_seq` | data: バイト列                  |
//! | `bin_to_nib`              | sign ∈ {-1, +1}                   |
//! | `u4_freq` / `l2_u4`       | nibble ヒスト                     |
//! | `Lcg64::new(seed)`        | seed: 固定シード                  |
//! | `nibble_mutate_rng`       | rng, src                          |
//! | `strip_annotations`       | rustc UI test src                 |
//! | `rustc_run(src, tmp_path)`| tmp_path: 書き込み可能パス        |
//! | `goldcycle_walk<S>`       | S: WalkStrategy, cfg, paths       |
//!
//! ## 数学的不変条件
//! - `gcd(d, 2^n - 1) = 1` (preferred pair の最大長条件) を `gold_seq` 内で静的検証。
//! - `WalkResult` 末尾で `h_min ≤ h_start` を `debug_assert!`。
//! - φ は IDF² 降順割り当てによる弱単射 (V > L のとき rank % L で衝突)。

use std::fs;
use std::process::Command;

// ── m 系列 / GOLD 系列 ──────────────────────────────────────────────────────

/// 次数 10 の Fibonacci LFSR で長さ 1023 の m 系列 ({-1,+1}) を生成する。
///
/// 状態 = [b_9 (MSB) .. b_0 (LSB)]、出力 = MSB、新ビット fb = AND(state, tap_poly)
/// の popcount mod 2、挿入先 = LSB。
///
/// 事前条件: `tap_poly` は次数 10 GF(2) 原始多項式に対応する 10bit マスク。
pub fn m_seq_10(tap_poly: u32) -> Vec<i32> {
    let mut state: u32 = 1;
    let l = (1u32 << 10) - 1;
    let mut seq = Vec::with_capacity(l as usize);
    for _ in 0..l {
        let bit = (state >> 9) & 1;
        seq.push(if bit == 1 { 1i32 } else { -1i32 });
        let fb = (state & tap_poly).count_ones() & 1;
        state = ((state << 1) | fb) & 0x3FF;
    }
    seq
}

#[allow(dead_code)] // const eval only
const fn const_gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 { let t = b; b = a % b; a = t; }
    a
}

// 静的不変条件: preferred pair の最大長条件 gcd(d, L) = 1
const _: () = assert!(const_gcd(65, 1023) == 1, "preferred pair: gcd(d, L) must be 1");

/// GOLD 系列 ({-1,+1}, L=1023) を preferred pair (0x09, decimation=65) で生成する。
///
/// 不変条件: `gcd(65, 1023) = 1` (モジュール先頭の静的 assertion で検証)
/// ⇒ デシメーション後も最大長系列。
pub fn gold_seq() -> Vec<i32> {
    let m1 = m_seq_10(0x09);
    let l = m1.len();
    let d = 65usize;
    let m2: Vec<i32> = (0..l).map(|i| m1[(d * i) % l]).collect();
    (0..l).map(|i| m1[i] * m2[i]).collect()
}

/// GOLD 系列の循環自己相関 corr[k] = Σ G[i]·G[(i+k) mod L]
pub fn gold_autocorr(g: &[i32]) -> Vec<f64> {
    let l = g.len();
    (0..l).map(|k| {
        g.iter().enumerate().map(|(i, &gi)| gi as f64 * g[(i + k) % l] as f64).sum::<f64>()
    }).collect()
}

// ── 循環クロス相関 (汎用 + 特殊化) ─────────────────────────────────────────

/// 循環クロス相関 corr[k] = Σ_i a[i] · b[(i+k) mod m] を i64 で返す総称版。
///
/// スカラ型 T は `Into<i64>` を満たす任意の型 (i32 / u8 / i16 / ...)。
/// 半環 (ℤ, +, ·) 上の作用素として動作する。
pub fn circ_xcorr_i64<T>(a: &[T], b: &[T]) -> Vec<i64>
where
    T: Copy + Into<i64>,
{
    let n = a.len();
    let m = b.len();
    if n == 0 || m == 0 { return vec![]; }
    (0..m).map(|k| {
        a.iter().enumerate()
            .map(|(i, &ai)| ai.into() * b[(i + k) % m].into())
            .sum::<i64>()
    }).collect()
}

/// {-1,+1} 二値特殊化 (E71a 互換)
#[inline]
pub fn circ_xcorr_bin(a: &[i32], b: &[i32]) -> Vec<i64> {
    circ_xcorr_i64(a, b)
}

/// {0..15} nibble 特殊化 (E71b 互換)
#[inline]
pub fn circ_xcorr_nib(a: &[u8], b: &[u8]) -> Vec<i64> {
    circ_xcorr_i64(a, b)
}

/// f64 特殊化 (GOLD 文書空間での xcorr 用)
pub fn circ_xcorr_f64(a: &[f64], b: &[f64]) -> Vec<f64> {
    let n = a.len();
    let m = b.len();
    if n == 0 || m == 0 { return vec![]; }
    (0..m).map(|k| {
        a.iter().enumerate().map(|(i, &ai)| ai * b[(i + k) % m]).sum::<f64>()
    }).collect()
}

/// スコア降順で上位 k 個のシフトを返す (i64 版)。安定ソート。
pub fn top_k_shifts(xcorr: &[i64], k: usize) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..xcorr.len()).collect();
    idx.sort_by(|&a, &b| xcorr[b].cmp(&xcorr[a]));
    idx.into_iter().take(k).collect()
}

// ── 順列 φ / GOLD 文書エンコード ────────────────────────────────────────────

/// IDF² 降順で語彙 → 位相 (mod L) の順列 φ を構築する。
///
/// 事前条件: `l > 0`、`v <= idf_sq.len()`。
/// 弱単射: V > L のとき rank % L で衝突するが、IDF² 降順なので
/// 高 IDF の gram は必ず固有位相を持つ。
pub fn build_phi(idf_sq: &[f32], v: usize, l: usize) -> Vec<usize> {
    debug_assert!(l > 0);
    let mut order: Vec<usize> = (0..v).collect();
    order.sort_by(|&a, &b| {
        idf_sq[b].partial_cmp(&idf_sq[a]).unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut phi = vec![0usize; v];
    for (rank, &gram_idx) in order.iter().enumerate() {
        phi[gram_idx] = rank % l;
    }
    phi
}

/// gram インデックス集合を GOLD 位相空間にエンコードする。
///
/// d[j] = G[j] if ∃ i ∈ gram_indices: φ(i) = j, else 0
pub fn gold_encode_indices(gram_indices: &[usize], phi: &[usize], gold_g: &[i32]) -> Vec<f64> {
    let l = gold_g.len();
    let mut d = vec![0.0f64; l];
    for &i in gram_indices {
        if i < phi.len() {
            let j = phi[i];
            d[j] = gold_g[j] as f64;
        }
    }
    d
}

// ── nibble エンコーダ ──────────────────────────────────────────────────────

/// バイト列 → 二値 nibble 符号 ({-1,+1}^{2N})
pub fn to_bin_nib(data: &[u8]) -> Vec<i32> {
    data.iter().flat_map(|&b| {
        let hi = if (b >> 4) >= 8 { 1i32 } else { -1i32 };
        let lo = if (b & 0x0f) >= 8 { 1i32 } else { -1i32 };
        [hi, lo]
    }).collect()
}

/// バイト列 → nibble 値列 ({0..15}^{2N})
pub fn to_nib_seq(data: &[u8]) -> Vec<u8> {
    data.iter().flat_map(|&b| [(b >> 4), (b & 0x0f)]).collect()
}

/// {-1,+1} → {0x0, 0x8} (E71a の nibble 二値化)
#[inline]
pub fn bin_to_nib(sign: i32) -> u8 {
    if sign >= 0 { 0x8 } else { 0x0 }
}

/// バイト列の `nib_idx` 番目 (上位=偶数) の nibble を `new_nib` で置換した
/// 新しいバッファを返す。
///
/// 事前条件: `new_nib < 16`。`nib_idx / 2 >= src.len()` の場合は src のコピーを返す。
pub fn apply_nib_pos(src: &[u8], nib_idx: usize, new_nib: u8) -> Vec<u8> {
    let byte_pos = nib_idx / 2;
    let upper = (nib_idx % 2) == 0;
    let mut out = src.to_vec();
    if byte_pos >= out.len() { return out; }
    if upper {
        out[byte_pos] = (out[byte_pos] & 0x0f) | (new_nib << 4);
    } else {
        out[byte_pos] = (out[byte_pos] & 0xf0) | new_nib;
    }
    out
}

/// nibble ヒストグラム (16 bin)
pub fn u4_freq(d: &[u8]) -> [u32; 16] {
    let mut h = [0u32; 16];
    for &b in d { h[(b >> 4) as usize] += 1; h[(b & 0xf) as usize] += 1; }
    h
}

/// 16 bin nibble ヒスト間 L2 距離
pub fn l2_u4(a: &[u32; 16], b: &[u32; 16]) -> f64 {
    a.iter().zip(b)
        .map(|(&x, &y)| { let d = x as i64 - y as i64; (d * d) as f64 })
        .sum::<f64>().sqrt()
}

// ── 固定シード LCG ─────────────────────────────────────────────────────────

/// 64bit LCG (Knuth 定数)。固定シードでビット同値の再現を保証する。
pub struct Lcg64 { state: u64 }

impl Lcg64 {
    pub fn new(s: u64) -> Self { Self { state: s.wrapping_add(1) } }
    pub fn next(&mut self) -> u64 {
        self.state = self.state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }
    pub fn next_usize(&mut self, n: usize) -> usize {
        if n == 0 { 0 } else { (self.next() as usize) % n }
    }
    pub fn next_u4(&mut self) -> u8 { (self.next() & 0x0f) as u8 }
    pub fn next_bit(&mut self) -> bool { (self.next() & 1) == 0 }
}

/// 単一 nibble をランダム置換した変異候補を返す。
///
/// 戻り値 = (変異後バイト列, 変異 byte 位置, 上位 nibble か, 新 nibble 値)。
/// 事前条件: `src.len() > 0`。
pub fn nibble_mutate_rng(rng: &mut Lcg64, src: &[u8]) -> (Vec<u8>, usize, bool, u8) {
    let pos = rng.next_usize(src.len());
    let upper = rng.next_bit();
    let nib = rng.next_u4();
    let mut out = src.to_vec();
    if upper { out[pos] = (out[pos] & 0x0f) | (nib << 4); }
    else { out[pos] = (out[pos] & 0xf0) | nib; }
    (out, pos, upper, nib)
}

// ── rustc oracle ───────────────────────────────────────────────────────────

/// rustc UI test 用注釈 (`//@`, `//~`, `// compile-flags` 等) を除去する。
pub fn strip_annotations(src: &str) -> String {
    src.lines()
        .filter(|l| {
            let t = l.trim();
            !t.starts_with("//@") && !t.starts_with("//~")
                && !t.starts_with("// compile-flags")
                && !t.starts_with("// edition")
                && !t.starts_with("// revisions")
                && !t.starts_with("// aux-build")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// ソースを `tmp_path` に書き出して rustc で検証し (エラー数, stderr) を返す。
///
/// 事前条件: `tmp_path` は書き込み可能な絶対 / 相対パス。
pub fn rustc_run(src: &[u8], tmp_path: &str) -> (u32, String) {
    fs::write(tmp_path, src).expect("tmp write");
    let out = Command::new("rustc")
        .args(["--edition=2021", "--error-format=short", tmp_path])
        .output()
        .expect("rustc not found");
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let count = stderr
        .lines()
        .find_map(|l| {
            l.strip_prefix("error: aborting due to ")
                .and_then(|r| r.split_whitespace().next())
                .and_then(|t| t.parse::<u32>().ok())
        })
        .unwrap_or_else(|| {
            stderr.lines().filter(|l| l.starts_with("error[")).count() as u32
        });
    (count, stderr)
}

// ── ウォーク骨格 ───────────────────────────────────────────────────────────

/// ウォーク制御パラメータ
#[derive(Clone, Copy, Debug)]
pub struct WalkConfig {
    pub top_k_shifts: usize,
    pub cands_per_shift: usize,
    pub phase2_trials: usize,
    pub seed: u64,
}

/// ウォーク結果
pub struct WalkResult {
    pub h_start: u32,
    pub h_min: u32,
    pub p1_accepts: usize,
    pub p2_accepts: usize,
    pub p1_trials: usize,
    pub best_bytes: Vec<u8>,
    pub xcorr_peak: i64,
    pub k_star: usize,
    pub accept_details: Vec<(usize, String, u32, u32)>,
}

/// スカラ体 × Phase1 候補選択戦略
///
/// E71a: BinField (Scalar=i32, ±1, gold_nib=bin_to_nib, 順序保存)
/// E71b: NibField (Scalar=u8, 0-15, gold_nib=そのまま, |diff| 降順)
pub trait WalkStrategy {
    type Scalar: Copy;
    /// バイト列 → スカラ列
    fn encode(bytes: &[u8]) -> Vec<Self::Scalar>;
    /// 循環クロス相関 (i64)
    fn xcorr(a: &[Self::Scalar], b: &[Self::Scalar]) -> Vec<i64>;
    /// シフト k に対する Phase1 候補 (nib_idx, new_nib) の優先順序付きリスト
    fn p1_candidates(
        cur: &[u8],
        gold_enc: &[Self::Scalar],
        k: usize,
        max: usize,
    ) -> Vec<(usize, u8)>;
    /// `accept_details` の Phase1 ラベル接頭辞 (例: "Phase1-circ" / "Phase1-circ-nib")
    const PHASE1_LABEL_PREFIX: &'static str;
}

/// GOLDCYCLE 共通ウォーク (Phase1 アライメント誘導 + Phase2 ランダム)
///
/// 不変条件: 末尾で `debug_assert!(h_min <= h_start)`。
pub fn goldcycle_walk<S: WalkStrategy>(
    buggy: &[u8],
    gold: &[u8],
    cfg: &WalkConfig,
    tmp_path: &str,
) -> WalkResult {
    debug_assert!(cfg.top_k_shifts > 0);

    let a = S::encode(buggy);
    let b = S::encode(gold);

    let xcorr = S::xcorr(&a, &b);
    let k_star = xcorr.iter().copied()
        .enumerate().max_by_key(|&(_, v)| v)
        .map(|(i, _)| i).unwrap_or(0);
    let xcorr_peak = xcorr.get(k_star).copied().unwrap_or(0);
    let top_shifts = top_k_shifts(&xcorr, cfg.top_k_shifts);

    let mut rng = Lcg64::new(cfg.seed);
    let (h_start, _err_start) = rustc_run(buggy, tmp_path);
    let mut cur = buggy.to_vec();
    let mut h_cur = h_start;
    let mut h_min = h_start;
    let mut best = cur.clone();
    let mut p1_accepts = 0usize;
    let mut p2_accepts = 0usize;
    let mut accept_details: Vec<(usize, String, u32, u32)> = Vec::new();
    let mut total_p1 = 0usize;

    'phase1: for &k in &top_shifts {
        let cands = S::p1_candidates(&cur, &b, k, cfg.cands_per_shift);
        for (nib_idx, new_nib) in cands {
            total_p1 += 1;
            let cand = apply_nib_pos(&cur, nib_idx, new_nib);
            let (h_cand, _) = rustc_run(&cand, tmp_path);
            if h_cand < h_cur {
                accept_details.push((
                    total_p1,
                    format!("{}(k={k})", S::PHASE1_LABEL_PREFIX),
                    h_cur,
                    h_cand,
                ));
                cur = cand;
                h_cur = h_cand;
                p1_accepts += 1;
                if h_cand < h_min { h_min = h_cand; best = cur.clone(); }
                if h_cur == 0 { break 'phase1; }
            }
        }
    }

    for i in 0..cfg.phase2_trials {
        let (cand, _, _, _) = nibble_mutate_rng(&mut rng, &cur);
        let (h_cand, _) = rustc_run(&cand, tmp_path);
        if h_cand < h_cur {
            accept_details.push((total_p1 + i + 1, "Phase2-random".into(), h_cur, h_cand));
            cur = cand;
            h_cur = h_cand;
            p2_accepts += 1;
            if h_cand < h_min { h_min = h_cand; best = cur.clone(); }
        }
    }

    debug_assert!(h_min <= h_start);

    WalkResult {
        h_start, h_min, p1_accepts, p2_accepts,
        p1_trials: total_p1,
        best_bytes: best,
        xcorr_peak, k_star,
        accept_details,
    }
}

// ── E71a / E71b 既製 WalkStrategy ─────────────────────────────────────────

/// 二値 ({-1,+1}) スカラ体 + 「gold_nib != buggy_nib の順序保存」候補選択 (E71a)
pub struct BinField;
impl WalkStrategy for BinField {
    type Scalar = i32;
    fn encode(bytes: &[u8]) -> Vec<i32> { to_bin_nib(bytes) }
    fn xcorr(a: &[i32], b: &[i32]) -> Vec<i64> { circ_xcorr_bin(a, b) }
    fn p1_candidates(cur: &[u8], gold_enc: &[i32], k: usize, max: usize) -> Vec<(usize, u8)> {
        let n = cur.len() * 2;
        let m = gold_enc.len();
        let mut cands: Vec<(usize, u8)> = Vec::new();
        for i in 0..n {
            let gold_sign = gold_enc[(i + k) % m];
            let gold_nib = bin_to_nib(gold_sign);
            let buggy_nib = if i % 2 == 0 { cur[i / 2] >> 4 } else { cur[i / 2] & 0x0f };
            if gold_nib != buggy_nib { cands.push((i, gold_nib)); }
        }
        cands.into_iter().take(max).collect()
    }
    const PHASE1_LABEL_PREFIX: &'static str = "Phase1-circ";
}

/// nibble ({0..15}) スカラ体 + 「|gold-buggy| 降順」候補選択 (E71b)
pub struct NibField;
impl WalkStrategy for NibField {
    type Scalar = u8;
    fn encode(bytes: &[u8]) -> Vec<u8> { to_nib_seq(bytes) }
    fn xcorr(a: &[u8], b: &[u8]) -> Vec<i64> { circ_xcorr_nib(a, b) }
    fn p1_candidates(cur: &[u8], gold_enc: &[u8], k: usize, max: usize) -> Vec<(usize, u8)> {
        let cur_nibs = to_nib_seq(cur);
        let m = gold_enc.len();
        let mut cands: Vec<(usize, u8, u8)> = Vec::new();
        for i in 0..cur_nibs.len() {
            let gold_nib = gold_enc[(i + k) % m];
            let buggy_nib = cur_nibs[i];
            if gold_nib != buggy_nib {
                let diff = (gold_nib as i32 - buggy_nib as i32).unsigned_abs() as u8;
                cands.push((i, gold_nib, diff));
            }
        }
        cands.sort_by(|x, y| y.2.cmp(&x.2));
        cands.into_iter().take(max).map(|(i, n, _)| (i, n)).collect()
    }
    const PHASE1_LABEL_PREFIX: &'static str = "Phase1-circ-nib";
}

// ── プロット (オプション機能) ─────────────────────────────────────────────

#[cfg(feature = "plot-goldcycle")]
pub mod plot {
    //! GOLDCYCLE プロットヘルパ (plotters 依存)
    use plotters::prelude::*;

    /// 1D xcorr 折れ線 + ピーク赤丸 PNG
    pub fn plot_xcorr_i64(path: &str, xcorr: &[i64], k_star: usize, color: &RGBColor) {
        let root = BitMapBackend::new(path, (800, 300)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let y_min = *xcorr.iter().min().unwrap_or(&0) as f64;
        let y_max = *xcorr.iter().max().unwrap_or(&1) as f64;
        let y_pad = (y_max - y_min) * 0.05 + 1.0;
        let x_max = xcorr.len() as f64;
        let mut chart = ChartBuilder::on(&root)
            .margin(10).x_label_area_size(0).y_label_area_size(0)
            .build_cartesian_2d(0.0..x_max, (y_min - y_pad)..(y_max + y_pad)).unwrap();
        chart.configure_mesh().disable_mesh().draw().unwrap();
        chart.draw_series(LineSeries::new(
            xcorr.iter().enumerate().map(|(i, &v)| (i as f64, v as f64)),
            color,
        )).unwrap();
        if k_star < xcorr.len() {
            chart.draw_series(std::iter::once(Circle::new(
                (k_star as f64, xcorr[k_star] as f64), 5, RED.filled(),
            ))).unwrap();
        }
        root.present().unwrap();
    }

    /// GOLD 自己相関 PNG (上: 全体, 下: k=1..50 サイドローブ拡大)
    pub fn plot_gold_autocorr(path: &str, autocorr: &[f64]) {
        let gold_len = autocorr.len();
        assert!(gold_len > 50, "autocorr.len() must be > 50 for sidelobe plot");
        let root = BitMapBackend::new(path, (900, 400)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let y_max = autocorr.iter().cloned().fold(0.0f64, f64::max);
        let y_min = autocorr.iter().cloned().fold(0.0f64, f64::min);
        let y_pad = (y_max - y_min) * 0.05 + 1.0;
        let (upper, lower) = root.split_vertically(200);
        {
            let mut chart = ChartBuilder::on(&upper)
                .margin(8).x_label_area_size(0).y_label_area_size(0)
                .build_cartesian_2d(0.0..(gold_len as f64), (y_min - y_pad)..(y_max + y_pad)).unwrap();
            chart.configure_mesh().disable_mesh().draw().unwrap();
            chart.draw_series(LineSeries::new(
                autocorr.iter().enumerate().map(|(i, &v)| (i as f64, v)), &BLUE,
            )).unwrap();
            chart.draw_series(std::iter::once(Circle::new((0.0, autocorr[0]), 5, RED.filled()))).unwrap();
        }
        {
            let window = 50.min(gold_len - 1);
            let sl_min = autocorr[1..=window].iter().cloned().fold(f64::INFINITY, f64::min);
            let sl_max = autocorr[1..=window].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let sl_pad = (sl_max - sl_min).abs() * 0.1 + 1.0;
            let mut chart = ChartBuilder::on(&lower)
                .margin(8).x_label_area_size(0).y_label_area_size(0)
                .build_cartesian_2d(1.0..(window as f64 + 1.0), (sl_min - sl_pad)..(sl_max + sl_pad)).unwrap();
            chart.configure_mesh().disable_mesh().draw().unwrap();
            chart.draw_series(LineSeries::new(
                (1..=window).map(|k| (k as f64, autocorr[k])), &GREEN,
            )).unwrap();
        }
        root.present().unwrap();
    }

    /// f64 系列の汎用 xcorr プロット (GOLD 文書空間用)
    pub fn plot_xcorr_f64(path: &str, xcorr: &[f64], k_star: usize) {
        let root = BitMapBackend::new(path, (900, 300)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let y_min = xcorr.iter().cloned().fold(0.0f64, f64::min);
        let y_max = xcorr.iter().cloned().fold(0.0f64, f64::max);
        let y_pad = (y_max - y_min) * 0.05 + 1.0;
        let mut chart = ChartBuilder::on(&root)
            .margin(8).x_label_area_size(0).y_label_area_size(0)
            .build_cartesian_2d(0.0..(xcorr.len() as f64), (y_min - y_pad)..(y_max + y_pad)).unwrap();
        chart.configure_mesh().disable_mesh().draw().unwrap();
        chart.draw_series(LineSeries::new(
            xcorr.iter().enumerate().map(|(i, &v)| (i as f64, v)), &BLUE,
        )).unwrap();
        if k_star < xcorr.len() {
            chart.draw_series(std::iter::once(Circle::new(
                (k_star as f64, xcorr[k_star]), 6, RED.filled(),
            ))).unwrap();
        }
        root.present().unwrap();
    }
}
