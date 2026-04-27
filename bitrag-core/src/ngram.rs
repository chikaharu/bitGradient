use std::collections::BTreeSet;

/// テキストから n 文字 n-gram を生成する
pub fn ngrams(text: &str, n: usize) -> Vec<String> {
    let ch: Vec<char> = text.chars().collect();
    if ch.len() < n { return vec![]; }
    (0..=ch.len() - n).map(|i| ch[i..i + n].iter().collect()).collect()
}

/// n=2,3,4 の全 n-gram 集合 (BTreeSet) を返す
pub fn gram_set(text: &str) -> BTreeSet<String> {
    let mut g = BTreeSet::new();
    for n in [2usize, 3, 4] {
        for x in ngrams(text, n) { g.insert(x); }
    }
    g
}
