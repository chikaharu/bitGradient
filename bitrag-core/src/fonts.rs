//! 共有フォントロード: `BITRAG_FONT_DIR` 環境変数から
//! plotters の `register_font` 経由で日本語/欧文フォントを登録する。
//!
//! - `BITRAG_FONT_DIR` 未設定や必須フォント欠落は **panic** (silent fallback 禁止)。
//! - 同一プロセスから複数回呼んでも安全 (plotters の `register_font` は冪等的に上書き)。
//! - フォント family 名定数 (`FAM_JP_SANS` 等) を呼び出し側に提供する。
//!
//! 参考: `artifacts/bitrag/experiment-font-smoke/src/main.rs`,
//!       `artifacts/bitrag/assets/fonts/README.md`

use plotters::style::{register_font, FontStyle};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub const FAM_JP_SANS: &str = "Noto Sans JP";
pub const FAM_JP_SERIF: &str = "Noto Serif JP";
pub const FAM_LATIN: &str = "Liberation Sans";

fn font_dir() -> PathBuf {
    let raw = env::var("BITRAG_FONT_DIR").unwrap_or_else(|_| {
        panic!(
            "BITRAG_FONT_DIR is not set. \
             Set it to the absolute path of artifacts/bitrag/assets/fonts \
             (see assets/fonts/README.md). silent fallback is forbidden."
        )
    });
    let p = PathBuf::from(raw);
    if !p.is_dir() {
        panic!("BITRAG_FONT_DIR={} is not a directory", p.display());
    }
    p
}

fn load_static(p: &Path) -> &'static [u8] {
    let bytes = fs::read(p)
        .unwrap_or_else(|e| panic!("required font missing: {}: {}", p.display(), e));
    Box::leak(bytes.into_boxed_slice())
}

fn register_required(family: &str, style: FontStyle, file: &Path) {
    let bytes = load_static(file);
    let style_label = match style {
        FontStyle::Normal => "Normal",
        FontStyle::Bold => "Bold",
        FontStyle::Italic => "Italic",
        FontStyle::Oblique => "Oblique",
    };
    register_font(family, style, bytes).unwrap_or_else(|_| {
        panic!(
            "register_font failed for family={} style={} file={}",
            family,
            style_label,
            file.display()
        )
    });
}

/// 日本語サンセリフ/明朝/欧文 (Liberation Sans) を一括登録する。
/// 失敗は loud panic。
pub fn register_all_fonts() {
    let d = font_dir();
    register_required(FAM_JP_SANS, FontStyle::Normal, &d.join("NotoSansJP-400.ttf"));
    register_required(FAM_JP_SANS, FontStyle::Bold, &d.join("NotoSansJP-700.ttf"));
    register_required(FAM_JP_SERIF, FontStyle::Normal, &d.join("NotoSerifJP-400.ttf"));
    register_required(FAM_JP_SERIF, FontStyle::Bold, &d.join("NotoSerifJP-700.ttf"));
    register_required(FAM_LATIN, FontStyle::Normal, &d.join("LiberationSans-Regular.ttf"));
    register_required(FAM_LATIN, FontStyle::Bold, &d.join("LiberationSans-Bold.ttf"));
}
