//! # bitgradient
//!
//! **Discrete Gradient Descent (DGD)** — derivative-free, LLM-less local
//! search over bit-vector states with **integer** loss.
//!
//! This crate is **Corollary 4** of the bitRAG main theorem
//! ([MAIN-B](https://github.com/chikaharu/bitrag-theorems)): the optimization
//! dual of the F2 retrieval scaling law. Where MAIN-B says *retrieval*
//! over bit representations is governed by `R = f(Nw/p)`, Cor 4 says
//! *optimization* over the same bit representations is governed by a
//! 1-bit-flip greedy descent on an integer loss — no continuous gradient,
//! no differentiation, no autograd.
//!
//! ## Formal statement (DGD)
//!
//! Let `S` be a finite bit-vector state space and `L : S → ℤ` an integer
//! loss. The **DGD operator** `D : S → S` is defined by
//!
//! ```text
//! D(s) := argmin_{s' ∈ N₁(s) ∪ {s}} L(s'),
//! ```
//!
//! where `N₁(s)` is the 1-bit-flip neighborhood of `s` (Hamming distance 1).
//! A **DGD trajectory** is the sequence `s₀, D(s₀), D²(s₀), …` and
//! terminates at a **local minimum** `s*` when `D(s*) = s*`. Termination is
//! guaranteed because `L(D^k(s₀))` is strictly decreasing in ℤ until a
//! local minimum is reached, and ℤ is bounded below on any finite state
//! space.
//!
//! ## Origin
//!
//! Extracted from [chikaharu/bitRAG](https://github.com/chikaharu/bitRAG)
//! `artifacts/bitrag/` — the *gold_cycle* / *iso* / *sign2* hill-climbers
//! used in experiments E207–E209. Those modules are kept verbatim in
//! [`bitrag-core`](../bitrag-core) as a raw historical extract; this
//! `bitgradient` crate is the **formalized core** with a clean DGD trait,
//! a reference [`HammingState`] implementation, unit tests, and a
//! convergence bench.
//!
//! ## Public API
//!
//! - [`DgdState`] — the trait every state must implement.
//! - [`descend_one_step`] — single greedy 1-bit-flip step.
//! - [`descend_until_local_min`] — iterate to a local minimum.
//! - [`HammingState`] — reference state with Hamming-distance loss.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// A state in a Discrete Gradient Descent search.
///
/// The state space is implicit: it's all bit-vectors of length `bits()`
/// that the implementor's `flip` operation can reach. The loss is an
/// integer (smaller is better, by convention).
pub trait DgdState: Clone {
    /// Number of bits in this state. Stays constant across `flip`.
    fn bits(&self) -> usize;

    /// Flip the `i`-th bit in place. Caller guarantees `i < self.bits()`.
    fn flip(&mut self, i: usize);

    /// Integer loss. Smaller is better.
    fn score(&self) -> i64;
}

/// Try every 1-bit flip; if any strictly lowers the score, apply the best
/// one and return `true`. If no flip improves the score, leave `s`
/// unchanged and return `false` (local minimum).
///
/// Cost: `O(n)` calls to `score` and `2n` flips, where `n = s.bits()`.
pub fn descend_one_step<S: DgdState>(s: &mut S) -> bool {
    let n = s.bits();
    let cur = s.score();
    let mut best_idx: Option<usize> = None;
    let mut best_score = cur;
    for i in 0..n {
        s.flip(i);
        let sc = s.score();
        if sc < best_score {
            best_score = sc;
            best_idx = Some(i);
        }
        s.flip(i);
    }
    if let Some(i) = best_idx {
        s.flip(i);
        true
    } else {
        false
    }
}

/// Repeatedly apply [`descend_one_step`] until the state is a local
/// minimum, or `max_steps` steps have been taken. Returns the actual
/// number of strict-improvement steps performed.
pub fn descend_until_local_min<S: DgdState>(s: &mut S, max_steps: usize) -> usize {
    for k in 0..max_steps {
        if !descend_one_step(s) {
            return k;
        }
    }
    max_steps
}

/// Reference [`DgdState`]: bit-vector with **Hamming-distance** loss to a
/// fixed target.
///
/// `score() = popcount(state XOR target)`. Has a unique global minimum at
/// `state == target` with score `0`. Convergence in at most `bits()`
/// steps from any starting point — useful as a sanity test for the
/// 1-bit-flip greedy operator.
#[derive(Clone, Debug)]
pub struct HammingState {
    state: Vec<bool>,
    target: Vec<bool>,
}

impl HammingState {
    /// Build a new `HammingState`. Panics if `state.len() != target.len()`.
    pub fn new(state: Vec<bool>, target: Vec<bool>) -> Self {
        assert_eq!(
            state.len(),
            target.len(),
            "state and target must have equal length"
        );
        Self { state, target }
    }

    /// Borrow the current bit-vector.
    pub fn state(&self) -> &[bool] {
        &self.state
    }
}

impl DgdState for HammingState {
    fn bits(&self) -> usize {
        self.state.len()
    }

    fn flip(&mut self, i: usize) {
        self.state[i] = !self.state[i];
    }

    fn score(&self) -> i64 {
        self.state
            .iter()
            .zip(self.target.iter())
            .filter(|(a, b)| a != b)
            .count() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alternating(n: usize) -> Vec<bool> {
        (0..n).map(|i| i % 2 == 0).collect()
    }

    #[test]
    fn hamming_score_zero_at_target() {
        let t = alternating(16);
        let s = HammingState::new(t.clone(), t);
        assert_eq!(s.score(), 0);
        assert_eq!(s.bits(), 16);
    }

    #[test]
    fn descend_one_step_finds_single_off_bit() {
        let target = alternating(8);
        let mut state = target.clone();
        state[3] = !state[3];
        let mut s = HammingState::new(state, target);
        assert_eq!(s.score(), 1);
        assert!(descend_one_step(&mut s));
        assert_eq!(s.score(), 0);
    }

    #[test]
    fn descend_one_step_returns_false_at_local_min() {
        let target = alternating(8);
        let mut s = HammingState::new(target.clone(), target);
        assert!(!descend_one_step(&mut s));
        assert_eq!(s.score(), 0);
    }

    #[test]
    fn descend_until_local_min_converges_within_n_steps() {
        let n = 64;
        let target = alternating(n);
        // Start at the bit-wise complement: maximally far from the target.
        let start: Vec<bool> = target.iter().map(|b| !b).collect();
        let mut s = HammingState::new(start, target);
        assert_eq!(s.score(), n as i64);
        let steps = descend_until_local_min(&mut s, 10 * n);
        assert_eq!(s.score(), 0, "must reach global minimum");
        assert_eq!(steps, n, "Hamming descent takes exactly n steps");
    }

    #[test]
    fn loss_strictly_decreases_each_step() {
        let target = alternating(32);
        let start: Vec<bool> = (0..32).map(|i| i % 5 == 0).collect();
        let mut s = HammingState::new(start, target);
        let mut prev = s.score();
        while descend_one_step(&mut s) {
            let cur = s.score();
            assert!(
                cur < prev,
                "DGD loss must strictly decrease at every accepted step"
            );
            prev = cur;
        }
    }

    /// Custom DgdState whose loss is **not** Hamming distance — this
    /// covers the trait against the danger of being secretly tied to one
    /// implementation.
    #[derive(Clone)]
    struct WeightedHammingState {
        state: Vec<bool>,
        weights: Vec<i64>,
    }

    impl DgdState for WeightedHammingState {
        fn bits(&self) -> usize {
            self.state.len()
        }
        fn flip(&mut self, i: usize) {
            self.state[i] = !self.state[i];
        }
        fn score(&self) -> i64 {
            self.state
                .iter()
                .zip(self.weights.iter())
                .filter(|(b, _)| **b)
                .map(|(_, w)| *w)
                .sum()
        }
    }

    #[test]
    fn weighted_loss_descends_to_no_negative_weight_set() {
        let weights: Vec<i64> = vec![3, -2, 5, -1, -7, 4];
        let state = vec![true, true, true, true, true, true];
        let mut s = WeightedHammingState { state, weights };
        descend_until_local_min(&mut s, 100);
        // All bits with positive weight must be off; negative weights on.
        for (b, w) in s.state.iter().zip(s.weights.iter()) {
            assert_eq!(
                *b,
                *w < 0,
                "DGD must select exactly the negative-weight bits"
            );
        }
    }
}
