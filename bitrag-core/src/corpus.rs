use std::fs;

/// ファイルから空行を除いてテキスト行を読み込む
pub fn load_corpus(path: &str) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("corpus 読み込み失敗 {path}: {e}"))
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}
