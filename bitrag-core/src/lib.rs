pub mod corpus;
pub mod ngram;
pub mod bitset;
pub mod idf;
pub mod matrix;
pub mod eval;
pub mod nibble_hash;
pub mod gold_cycle;
pub mod object_capsule;
pub mod nibble4_tokenizer;
pub mod sign2;
pub mod iso;

#[cfg(feature = "plot-fonts")]
pub mod fonts;

pub use corpus::load_corpus;
pub use ngram::{ngrams, gram_set};
pub use bitset::DocBits;
pub use idf::{Vocab, IdfPlanes};
pub use matrix::{
    build_mdoc_binary, build_mdoc_idf, build_mdoc_idf_masked,
    build_mdoc_shift, build_mdoc_shift_idf,
    row_normalize, ppr, hop,
};
pub use eval::{
    gini, top_k, top_k_idx, shorten, reach_count, row_sum_stats,
    long_query_score, greedy_cover, greedy_cover_idf,
};
