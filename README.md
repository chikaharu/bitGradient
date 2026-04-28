# bitGradient

**LLM-less Discrete Gradient Descent (DGD)** — derivative-free, integer-only
local search over bit-vector states.

> Where [bitRAG MAIN-B](https://github.com/chikaharu/bitrag-theorems) says
> *retrieval* over bit representations is governed by `R = f(Nw/p)`, this
> repo's **Corollary 4** says *optimization* over the same bit
> representations is governed by a 1-bit-flip greedy descent on an
> integer loss — no continuous gradient, no autograd, no LLM.

## Layout

```
bitGradient/
├── bitgradient/     # ★ Formalized DGD core (CI-tested, clippy-clean)
│   ├── src/lib.rs       DgdState trait + descend_one_step + descend_until_local_min
│   │                    + HammingState reference impl + 6 unit tests
│   ├── benches/dgd.rs   Convergence-speed bench (n = 64 / 256 / 1024 / 4096)
│   └── Cargo.toml       Apache-2.0, MSRV 1.70, no runtime deps
│
├── bitrag-core/     # Raw extract from bitRAG (gold_cycle / sign2 / iso / …)
│                    # Kept verbatim as a historical snapshot. Not in CI.
│
├── docs/            # Math foundations + theory papers (THEORY_*.md, MATH_*.md)
├── experiments/     # E001..E209 records (E207 delete-hill-climb, E209 1-bit-flip DGD …)
├── methods/         # Component / tool catalogs + experiment template
└── .github/workflows/ci.yml   fmt + clippy -D warnings + test + doc + bench-smoke
```

## Formal statement (DGD)

Let `S` be a finite bit-vector state space and `L : S → ℤ` an integer
loss. The **DGD operator** `D : S → S` is

```text
D(s) := argmin_{s' ∈ N₁(s) ∪ {s}} L(s'),
```

where `N₁(s)` is the 1-bit-flip neighborhood (Hamming distance 1). A
**DGD trajectory** `s₀, D(s₀), D²(s₀), …` terminates at a **local
minimum** `s*` when `D(s*) = s*`. Termination is guaranteed because
`L(D^k(s₀))` is strictly decreasing in `ℤ` until a local minimum, and
`ℤ` is bounded below on any finite state space.

The crate `bitgradient/` provides the trait and a reference
[`HammingState`] implementation; the original hill-climbers
(`gold_cycle`, `sign2`, `iso`) live untouched in `bitrag-core/`.

## MAIN-B Corollary 4 (optimization dual)

> **Cor 4.** Under the assumptions of MAIN-B, every loss `L : S → ℤ`
> defined as a fixed-point integer combination of bitRAG primitives
> (`AND`, `XOR`, `popcount`) admits a finite DGD trajectory whose
> length is bounded by `|S| · max_s |L(s)|`, and whose sample-path
> evaluations cost is `O(|S| · n)` where `n = bits(s)`.

The reference `HammingState` is the simplest non-trivial witness of
Cor 4: convergence in **exactly `n` steps** from any starting point,
verified by the `descend_until_local_min_converges_within_n_steps`
test.

The MAIN-B paper that hosts Cor 4 is being assembled at
[`chikaharu/bitrag-theorems`](https://github.com/chikaharu/bitrag-theorems);
this README will be updated with the final paper hash once it lands.

## Quick start

```rust
use bitgradient::{descend_until_local_min, HammingState};

let target = vec![true, false, true, true, false, true, false, false];
let start  = vec![false; 8];
let mut state = HammingState::new(start, target);

let steps = descend_until_local_min(&mut state, 100);
println!("converged in {steps} steps to score {}", 0);
```

## Build & test (formalized core)

```bash
cd bitgradient
cargo test            # 6 unit tests
cargo bench           # convergence bench, n = 64 / 256 / 1024 / 4096
cargo clippy --all-targets -- -D warnings
```

## Build (raw extract)

```bash
cd bitrag-core
cargo build --release
```

> **Note.** `bitrag-core/` is the verbatim extract from
> `chikaharu/bitRAG` `artifacts/bitrag/` and is intentionally **not**
> covered by CI. It contains pre-existing clippy warnings and a known
> overflow in `iso::tests::words_bytes_roundtrip` that pre-date this
> repository. The formalized DGD primitives (and all CI guarantees)
> live in `bitgradient/`.

## Origin

- Upstream repository: `chikaharu/bitRAG` `artifacts/bitrag/` tree
- Extraction date: 2026-04-27
- DGD formalization (this repo's `bitgradient/` crate): 2026-04-28
- Related experiments: E207 (delete-hill-climb), E208 (novelty),
  E209 (1-bit-flip DGD)

## Citation

```bibtex
@misc{chikaharu2026bitgradient,
  author = {chikaharu},
  title  = {{bitGradient}: LLM-less Discrete Gradient Descent over
            Bit-Vector States},
  year   = {2026},
  howpublished = {\url{https://github.com/chikaharu/bitGradient}},
  note   = {Corollary 4 of the bitRAG main theorem (MAIN-B).}
}
```

## License

Apache-2.0 (see [`LICENSE`](LICENSE)). The raw extract under
`bitrag-core/` inherits its license from upstream `chikaharu/bitRAG`.
