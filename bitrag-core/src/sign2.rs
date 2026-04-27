// 2bit サイン埋込
//
// 1 値 = 2bit:
//   01 (=1) → +1
//   00 (=0) →  0
//   10 (=2) → -1
//   11 (=3) → スペア (予約)
//
// 1024bit = [u64;16] = 128byte に 512 値が詰まる。

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sign2 {
    Plus,
    Zero,
    Minus,
    Spare,
}

impl Sign2 {
    /// 2bit 表現に変換 (0..=3)。
    #[inline]
    pub const fn to_bits(self) -> u8 {
        match self {
            Sign2::Plus => 0b01,
            Sign2::Zero => 0b00,
            Sign2::Minus => 0b10,
            Sign2::Spare => 0b11,
        }
    }

    /// 2bit (下位 2bit のみ参照) から復元。
    #[inline]
    pub const fn from_bits(b: u8) -> Self {
        match b & 0b11 {
            0b01 => Sign2::Plus,
            0b00 => Sign2::Zero,
            0b10 => Sign2::Minus,
            _ => Sign2::Spare,
        }
    }

    /// 整数 -1 / 0 / +1 への投影 (Spare は 0)。
    #[inline]
    pub const fn to_i8(self) -> i8 {
        match self {
            Sign2::Plus => 1,
            Sign2::Zero => 0,
            Sign2::Minus => -1,
            Sign2::Spare => 0,
        }
    }
}

impl fmt::Display for Sign2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Sign2::Plus => '+',
            Sign2::Zero => '0',
            Sign2::Minus => '-',
            Sign2::Spare => '*',
        };
        write!(f, "{c}")
    }
}

/// 1024bit = 512 値のサイン2bit ブロック。
pub const VALUES_PER_BLOCK: usize = 512;
pub const BLOCK_BYTES: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sign2Block(pub [u64; 16]);

impl Sign2Block {
    pub const ZERO: Sign2Block = Sign2Block([0u64; 16]);

    /// 値 i (0..512) を取り出す。byte = i/4, シフト = (i%4)*2 (LSB から)。
    #[inline]
    pub fn get(&self, i: usize) -> Sign2 {
        debug_assert!(i < VALUES_PER_BLOCK);
        let byte = self.byte_at(i / 4);
        let shift = (i % 4) * 2;
        Sign2::from_bits(byte >> shift)
    }

    /// 値 i (0..512) を書き込む。
    #[inline]
    pub fn set(&mut self, i: usize, v: Sign2) {
        debug_assert!(i < VALUES_PER_BLOCK);
        let bi = i / 4;
        let shift = (i % 4) * 2;
        let mut bytes = self.to_bytes();
        let cleared = bytes[bi] & !(0b11u8 << shift);
        bytes[bi] = cleared | (v.to_bits() << shift);
        *self = Self::from_bytes(&bytes);
    }

    fn byte_at(&self, bi: usize) -> u8 {
        let w = self.0[bi / 8];
        let j = bi % 8;
        ((w >> (56 - j * 8)) & 0xff) as u8
    }

    fn to_bytes(&self) -> [u8; BLOCK_BYTES] {
        let mut buf = [0u8; BLOCK_BYTES];
        for i in 0..16 {
            let w = self.0[i];
            for j in 0..8 {
                buf[i * 8 + j] = ((w >> (56 - j * 8)) & 0xff) as u8;
            }
        }
        buf
    }

    fn from_bytes(buf: &[u8; BLOCK_BYTES]) -> Self {
        let mut words = [0u64; 16];
        for i in 0..16 {
            let mut w = 0u64;
            for j in 0..8 {
                w = (w << 8) | (buf[i * 8 + j] as u64);
            }
            words[i] = w;
        }
        Sign2Block(words)
    }
}

/// Sign2 列を Sign2Block 列にパック (1 ブロック 512 値、末尾は Spare 埋め)。
pub fn pack(values: &[Sign2]) -> Vec<Sign2Block> {
    let mut blocks = Vec::with_capacity(values.len().div_ceil(VALUES_PER_BLOCK));
    for chunk in values.chunks(VALUES_PER_BLOCK) {
        let mut blk = Sign2Block::ZERO;
        for (i, v) in chunk.iter().enumerate() {
            blk.set(i, *v);
        }
        for i in chunk.len()..VALUES_PER_BLOCK {
            blk.set(i, Sign2::Spare);
        }
        blocks.push(blk);
    }
    blocks
}

/// Sign2Block 列を `len` 個の Sign2 値にアンパック。
pub fn unpack(blocks: &[Sign2Block], len: usize) -> Vec<Sign2> {
    let mut out = Vec::with_capacity(len);
    'outer: for blk in blocks {
        for i in 0..VALUES_PER_BLOCK {
            if out.len() == len {
                break 'outer;
            }
            out.push(blk.get(i));
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────
// NAND 一族
// ─────────────────────────────────────────────────────────────────────
//
// 目的: {-1, 0, +1} 上の演算を NAND だけで構成する (公理 A0 の構成的実例)。
// 全演算は f32/f64 を使わず、bit 操作のみで完結する。
//
// 慣習:
//   ビット対 (h, ℓ) で h=上位 / ℓ=下位、値 v = ℓ - h ∈ {-1,0,+1}。
//   Spare(11) は「第二の零」: to_i8 = 0 として零に同化させる演算と、
//   入力に Spare があれば結果も Spare にする保守的な演算を分けて提供。

#[inline]
const fn nand(a: u8, b: u8) -> u8 {
    !(a & b) & 1
}

/// XOR を NAND 4 個で合成 (古典的恒等)。
///   xor(a,b) = nand( nand(a, nand(a,b)), nand(b, nand(a,b)) )
#[inline]
const fn xor_via_nand(a: u8, b: u8) -> u8 {
    let t = nand(a, b);
    nand(nand(a, t), nand(b, t))
}

/// AND を NAND 2 個で合成: and(a,b) = nand( nand(a,b), nand(a,b) )
#[inline]
const fn and_via_nand(a: u8, b: u8) -> u8 {
    let t = nand(a, b);
    nand(t, t) // NOT (NOT (a AND b)) = a AND b
}

impl Sign2 {
    /// 否定: -v = (h, ℓ) → (ℓ, h)。Spare/Zero は不動点。
    #[inline]
    pub const fn neg(self) -> Self {
        let b = self.to_bits();
        let h = (b >> 1) & 1;
        let l = b & 1;
        // 新しい上位 = 旧 ℓ、新しい下位 = 旧 h
        Self::from_bits((l << 1) | h)
    }

    /// 乗算 ({-1,0,+1} の乗法モノイド)。NAND 完備性で実装。
    /// Spare * x は「Spare を零の代理」とみなして 0 を返す。
    pub const fn mul(self, other: Self) -> Self {
        let a = self.to_bits();
        let b = other.to_bits();
        let (ah, al) = ((a >> 1) & 1, a & 1);
        let (bh, bl) = ((b >> 1) & 1, b & 1);
        // 大きさ m = (al XOR ah) AND (bl XOR bh)  (Spare→0 同化)
        let ma = xor_via_nand(al, ah);
        let mb = xor_via_nand(bl, bh);
        let m = and_via_nand(ma, mb);
        // 符号 (上位ビット) = ah XOR bh、ただし結果が 0 のときは符号も 0
        let s_raw = xor_via_nand(ah, bh);
        let s = and_via_nand(s_raw, m);
        // (h, ℓ) = (s, m XOR s)  ⇒  値 ℓ - h = (m XOR s) - s
        //   m=0 → (0,0)=Zero
        //   m=1, s=0 → (0,1)=Plus
        //   m=1, s=1 → (1,0)=Minus
        let h = s;
        let l = xor_via_nand(m, s);
        Self::from_bits((h << 1) | l)
    }

    /// 飽和加算 ({-1,0,+1} の足し算を [-1,1] に押し込める)。
    /// 表:
    ///   (-1)+(-1) = -1 (飽和)、(-1)+0 = -1、(-1)+(+1) = 0
    ///   ( 0)+x    = x、(+1)+x   は対称
    /// Spare はゼロ同化。
    pub const fn add_saturating(self, other: Self) -> Self {
        let va = self.to_i8();
        let vb = other.to_i8();
        let s = va + vb;
        if s > 0 {
            Sign2::Plus
        } else if s < 0 {
            Sign2::Minus
        } else {
            Sign2::Zero
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_roundtrip() {
        for v in [Sign2::Plus, Sign2::Zero, Sign2::Minus, Sign2::Spare] {
            assert_eq!(Sign2::from_bits(v.to_bits()), v);
        }
        assert_eq!(Sign2::Plus.to_bits(), 0b01);
        assert_eq!(Sign2::Zero.to_bits(), 0b00);
        assert_eq!(Sign2::Minus.to_bits(), 0b10);
        assert_eq!(Sign2::Spare.to_bits(), 0b11);
    }

    #[test]
    fn to_i8_projection() {
        assert_eq!(Sign2::Plus.to_i8(), 1);
        assert_eq!(Sign2::Zero.to_i8(), 0);
        assert_eq!(Sign2::Minus.to_i8(), -1);
        assert_eq!(Sign2::Spare.to_i8(), 0);
    }

    #[test]
    fn block_get_set_each_position() {
        let mut blk = Sign2Block::ZERO;
        for i in 0..VALUES_PER_BLOCK {
            let v = match i % 4 {
                0 => Sign2::Plus,
                1 => Sign2::Zero,
                2 => Sign2::Minus,
                _ => Sign2::Spare,
            };
            blk.set(i, v);
        }
        for i in 0..VALUES_PER_BLOCK {
            let expected = match i % 4 {
                0 => Sign2::Plus,
                1 => Sign2::Zero,
                2 => Sign2::Minus,
                _ => Sign2::Spare,
            };
            assert_eq!(blk.get(i), expected, "pos {i}");
        }
    }

    #[test]
    fn pack_unpack_roundtrip() {
        let mut values: Vec<Sign2> = Vec::new();
        for i in 0..1500 {
            values.push(match i % 4 {
                0 => Sign2::Plus,
                1 => Sign2::Zero,
                2 => Sign2::Minus,
                _ => Sign2::Spare,
            });
        }
        let packed = pack(&values);
        assert_eq!(packed.len(), 3); // 1500 < 3*512
        let back = unpack(&packed, values.len());
        assert_eq!(back, values);
    }

    #[test]
    fn capacity_512_per_block() {
        assert_eq!(VALUES_PER_BLOCK, 512);
        assert_eq!(BLOCK_BYTES * 4, VALUES_PER_BLOCK);
    }

    // ── NAND 一族 ──

    #[test]
    fn neg_is_bit_swap() {
        assert_eq!(Sign2::Plus.neg(), Sign2::Minus);
        assert_eq!(Sign2::Minus.neg(), Sign2::Plus);
        assert_eq!(Sign2::Zero.neg(), Sign2::Zero);
        assert_eq!(Sign2::Spare.neg(), Sign2::Spare);
    }

    #[test]
    fn mul_full_table_excluding_spare() {
        let xs = [Sign2::Minus, Sign2::Zero, Sign2::Plus];
        for a in xs {
            for b in xs {
                let got = a.mul(b).to_i8();
                let want = a.to_i8() * b.to_i8();
                assert_eq!(got, want, "{:?} * {:?}", a, b);
            }
        }
    }

    #[test]
    fn mul_spare_is_zero_absorbing() {
        for a in [Sign2::Minus, Sign2::Zero, Sign2::Plus, Sign2::Spare] {
            assert_eq!(Sign2::Spare.mul(a).to_i8(), 0);
            assert_eq!(a.mul(Sign2::Spare).to_i8(), 0);
        }
    }

    #[test]
    fn add_saturating_table() {
        let xs = [Sign2::Minus, Sign2::Zero, Sign2::Plus];
        for a in xs {
            for b in xs {
                let s = (a.to_i8() + b.to_i8()).clamp(-1, 1);
                assert_eq!(a.add_saturating(b).to_i8(), s, "{:?} + {:?}", a, b);
            }
        }
    }
}
