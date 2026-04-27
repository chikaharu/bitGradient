// 同型射 (iso) モジュール
//
// 1024 bit ブロック = [u64; 16] = 128 byte は、複数の意味層で読める:
//   ・物理:    [u64; 16]
//   ・文字符号: 128 byte (UTF-8 / nibble4_tokenizer のペイロード)
//   ・意味原子: 256 nibble (4bit × 256)
//   ・論理 ±1: 512 個の Sign2 値 (2bit × 512)
//
// 本モジュールは「同じビット列」を 4 つの言語間で行き来させる純関数群を
// 提供する。可逆性は単体テストで保証する。

use crate::sign2::{Sign2, Sign2Block, BLOCK_BYTES, VALUES_PER_BLOCK};

/// 1 byte = 上位 nibble / 下位 nibble。
#[inline]
pub const fn byte_to_nibbles(b: u8) -> (u8, u8) {
    (b >> 4, b & 0x0f)
}

/// 上位 nibble / 下位 nibble → 1 byte。
#[inline]
pub const fn nibbles_to_byte(hi: u8, lo: u8) -> u8 {
    ((hi & 0x0f) << 4) | (lo & 0x0f)
}

/// 1 nibble (4 bit) → Sign2 2 値。
/// nibble の bit0..1 が値 0、bit2..3 が値 1。
#[inline]
pub fn nibble_to_sign2_pair(n: u8) -> [Sign2; 2] {
    [Sign2::from_bits(n & 0b11), Sign2::from_bits((n >> 2) & 0b11)]
}

/// Sign2 2 値 → 1 nibble。
#[inline]
pub fn sign2_pair_to_nibble(p: [Sign2; 2]) -> u8 {
    (p[0].to_bits() & 0b11) | ((p[1].to_bits() & 0b11) << 2)
}

/// バイト列 (BLOCK_BYTES) → Sign2Block。
pub fn bytes_to_sign2_block(buf: &[u8; BLOCK_BYTES]) -> Sign2Block {
    let mut blk = Sign2Block::ZERO;
    let mut idx = 0usize;
    for &b in buf.iter() {
        let (hi, lo) = byte_to_nibbles(b);
        let p_lo = nibble_to_sign2_pair(lo);
        let p_hi = nibble_to_sign2_pair(hi);
        // 値順序: byte 内の lo の low、lo の high、hi の low、hi の high
        blk.set(idx, p_lo[0]);
        blk.set(idx + 1, p_lo[1]);
        blk.set(idx + 2, p_hi[0]);
        blk.set(idx + 3, p_hi[1]);
        idx += 4;
    }
    debug_assert_eq!(idx, VALUES_PER_BLOCK);
    blk
}

/// Sign2Block → バイト列 (BLOCK_BYTES)。bytes_to_sign2_block の逆射。
pub fn sign2_block_to_bytes(blk: &Sign2Block) -> [u8; BLOCK_BYTES] {
    let mut out = [0u8; BLOCK_BYTES];
    for byte_i in 0..BLOCK_BYTES {
        let base = byte_i * 4;
        let lo = sign2_pair_to_nibble([blk.get(base), blk.get(base + 1)]);
        let hi = sign2_pair_to_nibble([blk.get(base + 2), blk.get(base + 3)]);
        out[byte_i] = nibbles_to_byte(hi, lo);
    }
    out
}

/// [u64; 16] → BLOCK_BYTES バイト列 (big-endian)。
pub fn words_to_bytes(words: &[u64; 16]) -> [u8; BLOCK_BYTES] {
    let mut buf = [0u8; BLOCK_BYTES];
    for i in 0..16 {
        let w = words[i];
        for j in 0..8 {
            buf[i * 8 + j] = ((w >> (56 - j * 8)) & 0xff) as u8;
        }
    }
    buf
}

/// BLOCK_BYTES バイト列 → [u64; 16] (big-endian)。
pub fn bytes_to_words(buf: &[u8; BLOCK_BYTES]) -> [u64; 16] {
    let mut words = [0u64; 16];
    for i in 0..16 {
        let mut w = 0u64;
        for j in 0..8 {
            w = (w << 8) | (buf[i * 8 + j] as u64);
        }
        words[i] = w;
    }
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nibble_byte_roundtrip() {
        for b in 0u8..=255 {
            let (hi, lo) = byte_to_nibbles(b);
            assert_eq!(nibbles_to_byte(hi, lo), b);
        }
    }

    #[test]
    fn nibble_sign2_roundtrip() {
        for n in 0u8..16 {
            let p = nibble_to_sign2_pair(n);
            assert_eq!(sign2_pair_to_nibble(p), n);
        }
    }

    #[test]
    fn bytes_block_roundtrip_random() {
        let mut buf = [0u8; BLOCK_BYTES];
        // LCG で擬似ランダム埋め (純整数演算)
        let mut x: u64 = 0x9E3779B97F4A7C15;
        for b in buf.iter_mut() {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            *b = (x >> 56) as u8;
        }
        let blk = bytes_to_sign2_block(&buf);
        let back = sign2_block_to_bytes(&blk);
        assert_eq!(back, buf);
    }

    #[test]
    fn words_bytes_roundtrip() {
        let words: [u64; 16] = std::array::from_fn(|i| (i as u64) * 0xDEADBEEF_01234567);
        let buf = words_to_bytes(&words);
        let back = bytes_to_words(&buf);
        assert_eq!(back, words);
    }

    #[test]
    fn fourfold_iso_chain() {
        // [u64;16] → bytes → Sign2Block → bytes → [u64;16]
        let words: [u64; 16] = std::array::from_fn(|i| (i as u64).wrapping_mul(0xA5A5_A5A5_A5A5_A5A5));
        let bytes = words_to_bytes(&words);
        let blk = bytes_to_sign2_block(&bytes);
        let bytes2 = sign2_block_to_bytes(&blk);
        let words2 = bytes_to_words(&bytes2);
        assert_eq!(words, words2);
    }
}
