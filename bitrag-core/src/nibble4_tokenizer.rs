// UTF-8 ニブル4 1024bit トークナイザ + デコーダ
//
// 設計:
//   - 1 ブロック = 1024 bit = [u64; 16] = 128 byte = 256 nibble
//   - byte 0..=126 がペイロード (最大 127 byte)
//   - byte 127 が長さタグ:
//       bit 7 (0x80): 終端フラグ (1 = 終端ブロック / 0 = 中間ブロック)
//       bit 0..=6   : このブロック内のペイロード使用バイト数 (0..=127)
//   - エンコードは 1 文字 (UTF-8 1〜4 byte) ずつ進め、残ペイロードに収まらない
//     文字が来たら現ブロックを中間タグで閉じて次ブロックへ。
//     文字はブロック境界を跨がない。
//   - 空文字列は終端フラグのみ立った (タグ = 0x80) ブロック 1 個を返す。
//   - 各 byte は内部的に [上位 4bit, 下位 4bit] のニブル列として扱うが、
//     byte 列としての完全性は保たれるため、エンコーダ/デコーダの実装は
//     byte 単位で書ける (ニブルへの分解は to_nibbles で観測可能)。
//
// 可逆性: encode → decode で UTF-8 文字列がビット完全に復元される。
//
// 公理 A0 (Logic-Only) との関係: 本トークナイザは byte 配列の論理的な分割
// (高位/低位 4bit shift と mask) と純粋な配列コピーのみで構成され、
// 解析演算・浮動小数を一切含まない。NAND 完全性証明 (Task #114) における
// 可逆ビット表現の最小単位。

use std::fmt;

/// 1024 bit = [u64; 16] のトークン 1 ブロック。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nibble4Token(pub [u64; 16]);

/// 1 ブロックのバイト容量 (タグを含む)。
pub const BLOCK_BYTES: usize = 128;
/// 1 ブロックのペイロード容量 (タグを除く)。
pub const PAYLOAD_BYTES: usize = 127;
/// タグの終端フラグビット。
pub const FINAL_FLAG: u8 = 0x80;
/// タグの使用バイト数マスク。
pub const USED_MASK: u8 = 0x7f;

/// デコード時のエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// 中間ブロックの後に終端ブロックが来ない (ストリーム不正)。
    UnterminatedStream,
    /// ペイロード末尾を超える used 値。
    UsedOutOfRange(u8),
    /// ペイロードが UTF-8 として不正。
    InvalidUtf8,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::UnterminatedStream => write!(f, "unterminated stream"),
            DecodeError::UsedOutOfRange(u) => write!(f, "used out of range: {}", u),
            DecodeError::InvalidUtf8 => write!(f, "invalid utf-8"),
        }
    }
}

impl std::error::Error for DecodeError {}

impl Nibble4Token {
    /// 全 0 のブロック。
    pub const ZERO: Nibble4Token = Nibble4Token([0u64; 16]);

    /// 128 byte → [u64; 16] (big-endian 各 8 byte → u64)。
    fn from_bytes(buf: &[u8; BLOCK_BYTES]) -> Self {
        let mut words = [0u64; 16];
        for i in 0..16 {
            let mut w = 0u64;
            for j in 0..8 {
                w = (w << 8) | (buf[i * 8 + j] as u64);
            }
            words[i] = w;
        }
        Nibble4Token(words)
    }

    /// [u64; 16] → 128 byte (big-endian)。
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

    /// 観測用: 256 nibble に分解 (各要素 0..=15)。
    pub fn to_nibbles(&self) -> [u8; 256] {
        let bytes = self.to_bytes();
        let mut nibbles = [0u8; 256];
        for i in 0..BLOCK_BYTES {
            nibbles[i * 2] = (bytes[i] >> 4) & 0x0f;
            nibbles[i * 2 + 1] = bytes[i] & 0x0f;
        }
        nibbles
    }

    /// このブロックの生タグ (1 byte)。
    pub fn tag(&self) -> u8 {
        self.to_bytes()[BLOCK_BYTES - 1]
    }

    /// このブロックが終端ブロックか。
    pub fn is_final(&self) -> bool {
        (self.tag() & FINAL_FLAG) != 0
    }

    /// このブロック内のペイロード使用バイト数。
    pub fn used(&self) -> u8 {
        self.tag() & USED_MASK
    }
}

/// UTF-8 文字列をニブル4 1024bit ブロック列にエンコードする。
///
/// 文字 (UTF-8 1〜4 byte) はブロック境界を跨がない。空文字列は
/// 終端フラグのみ立ったブロック 1 個を返す。
pub fn encode(s: &str) -> Vec<Nibble4Token> {
    let mut blocks: Vec<Nibble4Token> = Vec::new();
    let mut buf = [0u8; BLOCK_BYTES];
    let mut used: usize = 0;

    for ch in s.chars() {
        let mut tmp = [0u8; 4];
        let bytes = ch.encode_utf8(&mut tmp).as_bytes();
        if used + bytes.len() > PAYLOAD_BYTES {
            // 現ブロックを中間ブロックとして確定 (終端フラグ無し)
            buf[BLOCK_BYTES - 1] = used as u8; // bit 7 = 0
            blocks.push(Nibble4Token::from_bytes(&buf));
            buf = [0u8; BLOCK_BYTES];
            used = 0;
        }
        buf[used..used + bytes.len()].copy_from_slice(bytes);
        used += bytes.len();
    }

    // 終端ブロック (終端フラグ立て + used)
    buf[BLOCK_BYTES - 1] = FINAL_FLAG | (used as u8);
    blocks.push(Nibble4Token::from_bytes(&buf));
    blocks
}

/// ニブル4 ブロック列を UTF-8 文字列にデコードする。
///
/// 終端フラグの立ったブロックで終わる必要がある。
pub fn decode(blocks: &[Nibble4Token]) -> Result<String, DecodeError> {
    if blocks.is_empty() {
        return Ok(String::new());
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(blocks.len() * PAYLOAD_BYTES);
    let last = blocks.len() - 1;
    for (i, blk) in blocks.iter().enumerate() {
        let raw = blk.to_bytes();
        let tag = raw[BLOCK_BYTES - 1];
        let used = (tag & USED_MASK) as usize;
        let is_final = (tag & FINAL_FLAG) != 0;
        if used > PAYLOAD_BYTES {
            return Err(DecodeError::UsedOutOfRange(used as u8));
        }
        if i < last && is_final {
            return Err(DecodeError::UnterminatedStream);
        }
        if i == last && !is_final {
            return Err(DecodeError::UnterminatedStream);
        }
        bytes.extend_from_slice(&raw[..used]);
    }
    String::from_utf8(bytes).map_err(|_| DecodeError::InvalidUtf8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(s: &str) {
        let enc = encode(s);
        let dec = decode(&enc).expect("decode");
        assert_eq!(dec, s, "round-trip mismatch (len={})", s.len());
    }

    #[test]
    fn empty_string() {
        let enc = encode("");
        assert_eq!(enc.len(), 1);
        assert!(enc[0].is_final());
        assert_eq!(enc[0].used(), 0);
        roundtrip("");
    }

    #[test]
    fn ascii_short() {
        roundtrip("Hello, bitRAG!");
    }

    #[test]
    fn japanese_multibyte() {
        roundtrip("にぶる4のトークナイザ");
    }

    #[test]
    fn emoji_4byte() {
        roundtrip("🦀🚀🧠💡");
    }

    #[test]
    fn cross_block_boundary_ascii() {
        // 300 byte = 127 + 127 + 46
        let s: String = std::iter::repeat('a').take(300).collect();
        let enc = encode(&s);
        assert_eq!(enc.len(), 3);
        assert!(!enc[0].is_final());
        assert_eq!(enc[0].used(), 127);
        assert!(!enc[1].is_final());
        assert_eq!(enc[1].used(), 127);
        assert!(enc[2].is_final());
        assert_eq!(enc[2].used(), 46);
        roundtrip(&s);
    }

    #[test]
    fn char_boundary_no_split() {
        // 126 byte ASCII の後に 4 byte 絵文字 → 残 1 byte に収まらず次ブロックへ。
        // ブロック0: used=126 (非終端), ブロック1: used=4 (終端)
        let mut s: String = std::iter::repeat('a').take(126).collect();
        s.push('🦀');
        let enc = encode(&s);
        assert_eq!(enc.len(), 2);
        assert!(!enc[0].is_final());
        assert_eq!(enc[0].used(), 126);
        assert!(enc[1].is_final());
        assert_eq!(enc[1].used(), 4);
        roundtrip(&s);
    }

    #[test]
    fn exactly_one_block() {
        // 127 byte ASCII でちょうど 1 ブロック (終端タグ込み)
        let s: String = std::iter::repeat('z').take(127).collect();
        let enc = encode(&s);
        assert_eq!(enc.len(), 1);
        assert!(enc[0].is_final());
        assert_eq!(enc[0].used(), 127);
        roundtrip(&s);
    }

    #[test]
    fn large_alphanumeric() {
        let s: String = (0..1000).map(|i| ((b'a' + (i % 26) as u8) as char)).collect();
        roundtrip(&s);
    }

    #[test]
    fn mixed_scripts() {
        roundtrip("ABCあいう🦀漢字とemojiが混在 — テスト 12345");
    }

    #[test]
    fn nibble_count_is_256() {
        let blk = encode("x")[0];
        assert_eq!(blk.to_nibbles().len(), 256);
    }

    #[test]
    fn unterminated_stream_detected() {
        // 中間タグだけのブロックで終わる → エラー
        let mut buf = [0u8; BLOCK_BYTES];
        buf[BLOCK_BYTES - 1] = 10; // 中間タグ (FINAL_FLAG なし)
        let bad = Nibble4Token::from_bytes(&buf);
        assert_eq!(decode(&[bad]), Err(DecodeError::UnterminatedStream));
    }
}
