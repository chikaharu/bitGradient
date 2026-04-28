//! Convergence-speed bench for [`bitgradient::descend_until_local_min`].
//!
//! Custom harness (no criterion dep). Run with `cargo bench`. CI runs
//! `cargo bench --no-run` as a smoke test — to actually time the descent
//! locally, use `cargo bench`.

use std::time::Instant;

use bitgradient::{descend_until_local_min, DgdState, HammingState};

fn run_one(n: usize, repeats: usize) {
    let target: Vec<bool> = (0..n).map(|i| (i * 7 + 3) % 5 < 2).collect();
    let mut total = std::time::Duration::ZERO;
    let mut total_steps = 0;
    for r in 0..repeats {
        let start: Vec<bool> = (0..n).map(|i| (i + r) % 3 == 0).collect();
        let mut state = HammingState::new(start, target.clone());
        let t0 = Instant::now();
        let steps = descend_until_local_min(&mut state, 100 * n);
        total += t0.elapsed();
        total_steps += steps;
        assert_eq!(state.score(), 0, "Hamming descent must reach 0 at n={n}");
    }
    let per_step = total / (total_steps.max(1) as u32);
    println!(
        "n={:>5}  repeats={:>3}  avg_steps={:>5}  total={:?}  per_step={:?}",
        n,
        repeats,
        total_steps / repeats,
        total,
        per_step
    );
}

fn main() {
    println!("# bitgradient: DGD convergence bench (HammingState, 1-bit-flip greedy)");
    println!("# Loss = popcount(state XOR target). Global min = 0 in exactly n steps.");
    for &n in &[64usize, 256, 1024, 4096] {
        let repeats = if n <= 256 { 50 } else { 5 };
        run_one(n, repeats);
    }
}
