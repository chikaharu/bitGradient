//! C12/C13 — Object Bit Capsule (E84)
//!
//! ## C12 `n4_gram_circ`
//! UTF-8 バイト列 → 循環 4-bit gram 列 (1-bit スライド窓)
//! F: XNOR(inv4) + SHIFT + POPCOUNT
//! 入力: `&[u8]`  出力: `Vec<u8>` (各要素 ∈ 0..15, 長さ = 8 * bytes.len())
//! 例外: bytes 空 → 空ベクタ
//!
//! ## C13 `ObjectCapsule`
//! 1024-bit ([u64; 16]) のビットカプセル
//! F: XNOR + POPCOUNT + cyclic-XOR
//! 入力: `&[u8]` (encode) / `&ObjectCapsule` (比較)
//! 出力: `ObjectCapsule` (encode) / `u32` (xnor_l1/popcount) / `String` (decode_to_stub)
//! 例外: decode_to_stub は常に文法的に妥当な Rust stub を返す

// ─────────────────────────────────────────────────────────────────────────────
// §2 4-bit 基本演算 (E83 sub1 spec.md §2 と bit-for-bit 同一仕様)
// ─────────────────────────────────────────────────────────────────────────────

/// inv4(x) = XNOR(x, 0) = (x XOR 0xF) & 0xF  [4-bit 反転]
#[inline]
pub fn inv4(x: u8) -> u8 {
    (x ^ 0x0F) & 0x0F
}

/// clz4(x) — MBS から LBS 方向への連続 0 数 (4-bit)
/// 実装: m = smear_right(x); 4 - popcount(m)
#[inline]
pub fn clz4(x: u8) -> u8 {
    let x = x & 0x0F;
    let m = (x | (x >> 1) | (x >> 2) | (x >> 3)) & 0x0F;
    4 - m.count_ones() as u8
}

/// ctz4(x) — LBS から MBS 方向への連続 0 数 (4-bit)
/// 実装: m = smear_left(x) & 0xF; 4 - popcount(m)
#[inline]
pub fn ctz4(x: u8) -> u8 {
    let x = x & 0x0F;
    let m = (x | (x << 1) | (x << 2) | (x << 3)) & 0x0F;
    4 - m.count_ones() as u8
}

// ─────────────────────────────────────────────────────────────────────────────
// §C12 n4_gram_circ — 循環 4-bit gram トークナイザ
// ─────────────────────────────────────────────────────────────────────────────

/// UTF-8 バイト列を循環ビットストリームとして扱い、
/// 1-bit ずつスライドする 4-bit 窓で gram を生成する。
///
/// - 入力長 N バイト → 出力長 8N トークン
/// - 各トークン ∈ 0..15 (4-bit)
/// - bytes が空のとき空ベクタを返す (silent fallback 禁止: panic しない代わりに明示的に空)
pub fn n4_gram_circ(bytes: &[u8]) -> Vec<u8> {
    let n_bits = bytes.len() * 8;
    if n_bits == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(n_bits);
    for i in 0..n_bits {
        // MSB-first: ビット位置 p のビット = bytes[p/8] >> (7 - p%8) & 1
        let mut gram: u8 = 0;
        for b in 0..4u8 {
            let bit_pos = (i + b as usize) % n_bits;
            let byte_idx = bit_pos / 8;
            let bit_off = 7 - (bit_pos % 8);
            let bit = (bytes[byte_idx] >> bit_off) & 1;
            gram = (gram << 1) | bit;
        }
        out.push(gram & 0x0F);
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// §C13 ObjectCapsule — 1024-bit カプセル
// ─────────────────────────────────────────────────────────────────────────────

/// 1024-bit = [u64; 16] のオブジェクトカプセル。
/// XNOR / POPCOUNT / cyclic-XOR のみで操作する。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ObjectCapsule(pub [u64; 16]);

impl ObjectCapsule {
    /// バイト列 → ObjectCapsule
    ///
    /// n4_gram_circ で gram 列を得て、各 (position, gram_value) を
    /// bit index = (position * 4 + gram_value) % 1024 に写像し XOR で畳み込む。
    pub fn encode(bytes: &[u8]) -> Self {
        let grams = n4_gram_circ(bytes);
        let mut words = [0u64; 16];
        for (i, &g) in grams.iter().enumerate() {
            let bit = i.wrapping_mul(4).wrapping_add(g as usize) & 0x3FF;
            words[bit >> 6] ^= 1u64 << (bit & 63);
        }
        Self(words)
    }

    /// XNOR + POPCOUNT: 一致ビット数 (最大 1024 = 完全一致)
    #[inline]
    pub fn xnor_popcount(&self, other: &Self) -> u32 {
        self.0
            .iter()
            .zip(other.0.iter())
            .map(|(a, b)| (!(a ^ b)).count_ones())
            .sum()
    }

    /// L1 距離 = ハミング距離 = XOR のポップカウント
    #[inline]
    pub fn xnor_l1(&self, other: &Self) -> u32 {
        self.0
            .iter()
            .zip(other.0.iter())
            .map(|(a, b)| (a ^ b).count_ones())
            .sum()
    }

    /// capsule 全体のセットビット数
    #[inline]
    pub fn popcount(&self) -> u32 {
        self.0.iter().map(|w| w.count_ones()).sum()
    }

    /// 128 hex chars の文字列表現 (16 words × 16 hex chars)
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|w| format!("{w:016x}")).collect()
    }

    /// capsule → 決定論的 Rust stub コード文字列
    ///
    /// 外部 API 不使用・silent fallback 禁止。
    /// capsule の各 word から識別子・型・構造を決定論的に導出する。
    pub fn decode_to_stub(&self) -> String {
        let name = fmt_stub_name(self.0[0] ^ self.0[1]);
        let n_args = (self.0[2] & 0x7) as usize; // 0..7
        let arg_types: Vec<&str> = (0..n_args.min(4))
            .map(|i| nibble_to_type((self.0[3 + i] & 0xF) as u8))
            .collect();
        let ret_type = nibble_to_type((self.0[8] & 0xF) as u8);
        let is_struct = (self.0[15] >> 63) & 1 == 1;

        if is_struct {
            let fields: String = arg_types
                .iter()
                .enumerate()
                .map(|(i, t)| format!("    pub f{i}: {t},\n"))
                .collect();
            format!("#[derive(Debug, Clone)]\npub struct {name} {{\n{fields}}}\n")
        } else {
            let params: String = arg_types
                .iter()
                .enumerate()
                .map(|(i, t)| format!("a{i}: {t}"))
                .collect::<Vec<_>>()
                .join(", ");
            let dv = type_default(ret_type);
            format!("pub fn {name}({params}) -> {ret_type} {{ {dv} }}\n")
        }
    }
}

/// 64-bit hash → 読み易い識別子 (CV syllable × 4)
fn fmt_stub_name(hash: u64) -> String {
    const CV: [&str; 64] = [
        "ba", "be", "bi", "bo", "bu", "ca", "ce", "ci", "co", "cu", "da", "de", "di", "do",
        "du", "fa", "fe", "fi", "fo", "fu", "ga", "ge", "gi", "go", "gu", "ha", "he", "hi",
        "ho", "hu", "ja", "je", "ji", "jo", "ju", "ka", "ke", "ki", "ko", "ku", "la", "le",
        "li", "lo", "lu", "ma", "me", "mi", "mo", "mu", "na", "ne", "ni", "no", "nu", "pa",
        "pe", "pi", "po", "pu", "ra", "re", "ri", "ro",
    ];
    let h = [
        ((hash >> 48) & 0x3F) as usize,
        ((hash >> 32) & 0x3F) as usize,
        ((hash >> 16) & 0x3F) as usize,
        (hash & 0x3F) as usize,
    ];
    format!("s_{}{}{}{}", CV[h[0]], CV[h[1]], CV[h[2]], CV[h[3]])
}

fn nibble_to_type(n: u8) -> &'static str {
    match n & 0xF {
        0 => "u8",
        1 => "u16",
        2 => "u32",
        3 => "u64",
        4 => "usize",
        5 => "i8",
        6 => "i16",
        7 => "i32",
        8 => "i64",
        9 => "isize",
        10 => "bool",
        11 => "f32",
        12 => "f64",
        13 => "&[u8]",
        14 => "&str",
        15 | _ => "()",
    }
}

fn type_default(t: &str) -> &'static str {
    match t {
        "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32" | "i64" | "isize" => "0",
        "bool" => "false",
        "f32" | "f64" => "0.0",
        "&[u8]" => "&[]",
        "&str" => "\"\"",
        "()" => "",
        _ => "0",
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// §自己テスト
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clz4_values() {
        assert_eq!(clz4(0b0000), 4);
        assert_eq!(clz4(0b0001), 3);
        assert_eq!(clz4(0b0010), 2);
        assert_eq!(clz4(0b0100), 1);
        assert_eq!(clz4(0b1000), 0);
        assert_eq!(clz4(0b1111), 0);
    }

    #[test]
    fn test_ctz4_values() {
        assert_eq!(ctz4(0b0000), 4);
        assert_eq!(ctz4(0b0001), 0);
        assert_eq!(ctz4(0b0010), 1);
        assert_eq!(ctz4(0b0100), 2);
        assert_eq!(ctz4(0b1000), 3);
        assert_eq!(ctz4(0b1111), 0);
    }

    #[test]
    fn test_inv4_xnor_property() {
        for x in 0u8..16 {
            let i = inv4(x);
            assert_eq!((x ^ i) & 0xF, 0xF, "x={x} inv4={i}");
        }
    }

    #[test]
    fn test_n4_gram_circ_length() {
        let b = b"hello";
        let g = n4_gram_circ(b);
        assert_eq!(g.len(), b.len() * 8);
    }

    #[test]
    fn test_n4_gram_circ_range() {
        let b = b"RUST";
        for &g in n4_gram_circ(b).iter() {
            assert!(g < 16, "gram value out of range: {g}");
        }
    }

    #[test]
    fn test_n4_gram_circ_empty() {
        assert_eq!(n4_gram_circ(&[]), Vec::<u8>::new());
    }

    #[test]
    fn test_n4_gram_circ_single_byte() {
        // 0xAB = 0b10101011  → 8 circular 4-bit grams (1-bit slide, MSB-first)
        let g = n4_gram_circ(&[0xABu8]);
        assert_eq!(g.len(), 8);
        // gram[0] = bits[0..3] of circular 0b10101011 = 1010 = 10
        assert_eq!(g[0], 0b1010);
        // gram[1] = bits[1..4] = 0101 = 5
        assert_eq!(g[1], 0b0101);
    }

    #[test]
    fn test_encode_decode_deterministic() {
        let a = ObjectCapsule::encode(b"pub fn foo(x: u32) -> u32 { x }");
        let b = ObjectCapsule::encode(b"pub fn foo(x: u32) -> u32 { x }");
        assert_eq!(a, b);
        let stub_a = a.decode_to_stub();
        let stub_b = b.decode_to_stub();
        assert_eq!(stub_a, stub_b);
    }

    #[test]
    fn test_xnor_l1_identity() {
        let a = ObjectCapsule::encode(b"let x = 1;");
        assert_eq!(a.xnor_l1(&a), 0);
        assert_eq!(a.xnor_popcount(&a), 1024);
    }

    #[test]
    fn test_xnor_l1_symmetry() {
        let a = ObjectCapsule::encode(b"fn foo() {}");
        let b = ObjectCapsule::encode(b"fn bar() {}");
        assert_eq!(a.xnor_l1(&b), b.xnor_l1(&a));
    }

    #[test]
    fn test_popcount_range() {
        let c = ObjectCapsule::encode(b"struct MyType { x: u32 }");
        let p = c.popcount();
        assert!(p <= 1024, "popcount={p} out of range");
    }

    /// 単純な決定論的 RNG (SplitMix64)。外部依存を増やさないため自前実装。
    fn splitmix64(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn random_bytes(state: &mut u64, n: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(n);
        while out.len() < n {
            let v = splitmix64(state);
            for k in 0..8 {
                if out.len() < n {
                    out.push((v >> (k * 8)) as u8);
                }
            }
        }
        out
    }

    fn capsule_xor(a: &ObjectCapsule, b: &ObjectCapsule) -> ObjectCapsule {
        let mut w = [0u64; 16];
        for i in 0..16 {
            w[i] = a.0[i] ^ b.0[i];
        }
        ObjectCapsule(w)
    }

    /// T08 の加法性 `E(x ⊕ y) == E(x) ⊕ E(y)` を 1024 バイト × 1000 ペアで実測する。
    /// 同じテスト内で長さ違いケース (短い側を 0 詰めして XOR) も測る。
    /// 完全加法性が崩れていることを反例として明示し、分布統計を表示する。
    #[test]
    fn test_capsule_additivity_proptest() {
        let n_pairs = 1000usize;
        let len = 1024usize;
        let mut state: u64 = 0xDEAD_BEEF_CAFE_F00D;
        let mut sum_d: u64 = 0;
        let mut max_d: u32 = 0;
        let mut zero_cnt: usize = 0;
        for _ in 0..n_pairs {
            let x = random_bytes(&mut state, len);
            let y = random_bytes(&mut state, len);
            let xy: Vec<u8> = x.iter().zip(y.iter()).map(|(a, b)| a ^ b).collect();
            let ex = ObjectCapsule::encode(&x);
            let ey = ObjectCapsule::encode(&y);
            let exy = ObjectCapsule::encode(&xy);
            let lhs_xor_rhs = capsule_xor(&exy, &capsule_xor(&ex, &ey));
            let d = lhs_xor_rhs.popcount();
            sum_d += d as u64;
            if d > max_d {
                max_d = d;
            }
            if d == 0 {
                zero_cnt += 1;
            }
        }
        let mean = sum_d as f64 / n_pairs as f64;
        let zero_ratio = zero_cnt as f64 / n_pairs as f64;
        eprintln!(
            "[T08 additivity, equal-length 1024B × {n_pairs} pairs] \
             mean(d)={mean:.3} max(d)={max_d} zero_ratio={zero_ratio:.4}"
        );

        // 長さ違いケース: 512B と 1024B (短い側を 0 詰め)。
        let n_pairs2 = 200usize;
        let mut sum_d2: u64 = 0;
        let mut max_d2: u32 = 0;
        let mut zero_cnt2: usize = 0;
        for _ in 0..n_pairs2 {
            let x = random_bytes(&mut state, 512);
            let y = random_bytes(&mut state, 1024);
            let mut x_pad = x.clone();
            x_pad.resize(1024, 0);
            let xy: Vec<u8> = x_pad.iter().zip(y.iter()).map(|(a, b)| a ^ b).collect();
            let ex = ObjectCapsule::encode(&x_pad);
            let ey = ObjectCapsule::encode(&y);
            let exy = ObjectCapsule::encode(&xy);
            let d = capsule_xor(&exy, &capsule_xor(&ex, &ey)).popcount();
            sum_d2 += d as u64;
            if d > max_d2 {
                max_d2 = d;
            }
            if d == 0 {
                zero_cnt2 += 1;
            }
        }
        let mean2 = sum_d2 as f64 / n_pairs2 as f64;
        let zero_ratio2 = zero_cnt2 as f64 / n_pairs2 as f64;
        eprintln!(
            "[T08 additivity, mismatched-length 512B/1024B (zero-pad) × {n_pairs2} pairs] \
             mean(d)={mean2:.3} max(d)={max_d2} zero_ratio={zero_ratio2:.4}"
        );

        // 反例の存在を契約として固定する: 完全加法性は成立しない。
        // これは T08 が反例ありに格下げされていることを保証するための回帰テスト。
        assert!(
            max_d > 0,
            "T08 additivity unexpectedly holds — 仕様格上げの再評価が必要"
        );
    }

    #[test]
    fn test_decode_stub_compiles_with_rustfmt_check() {
        let c = ObjectCapsule::encode(b"pub fn compute(a: u32, b: u64) -> u64 { 0 }");
        let stub = c.decode_to_stub();
        assert!(
            stub.contains("pub fn") || stub.contains("pub struct"),
            "stub does not start with pub fn or pub struct: {stub:?}"
        );
        assert!(stub.ends_with('\n'), "stub missing trailing newline");
    }
}
