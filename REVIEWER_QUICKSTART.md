# Reviewer Quickstart — zkSHA-Rx Fly v0.1.0

## What This Is

An AVX-512 vectorized Number Theoretic Transform (NTT) for the BabyBear field (P = 0x78000001), the native field of Plonky3-style proving systems.

This is a **research artifact**, not a production library. You're looking at it because we asked you to review whether the claims and evidence boundaries are honest.

## What to Review

Pick the scope that matches your expertise:

- **Benchmark methodology:** Is the Criterion config sound? Are the speedup numbers measured correctly? See `BENCHMARKS.md`, `MEASUREMENT.md`, `EVIDENCE_CHAIN.md`
- **NTT/SIMD implementation:** Is the three-backend approach valid? Is the backend equivalence methodology adequate? See `src/`, `ARCHITECTURE.md`
- **Formal verification boundary:** Does "formally verified arithmetic foundations" accurately describe the Lean 4 proof scope without overstating? See the Lean 4 source at [tscp-anchor](https://github.com/Cartilage-Stairwells/tscp-anchor) (tag v0.1.0-zksha-rx), `PROJECT_FACTS.md`
- **Claim/evidence alignment:** Do the external claims exceed the evidence? See `PROJECT_FACTS.md` (canonical source of truth)

## What IS Claimed

- Kernel-level speedup: 9.15× (DIT butterfly), 4.58× (XOR butterfly) — measured via Criterion 0.5.1, 100 samples, 3s warmup, outlier rejection
- Backend equivalence: staged pairwise comparisons across reference oracle, scalar Montgomery bridge, and AVX-512 SIMD
- Formal verification: Montgomery arithmetic for BabyBear field in Lean 4, proven by `decide`, 0 sorry in core module

## What is NOT Claimed

- End-to-end proving-system acceleration (kernel-level only)
- Formally verified SIMD implementation (formal verification covers arithmetic, not SIMD)
- Production readiness (research artifact)
- Drop-in replacement for Plonky3/SP1 (integration is future work)

## How to Reproduce

```bash
git clone https://github.com/Cartilage-Stairwells/avx512-butterfly.git
cd avx512-butterfly
git checkout v0.1.0-zksha-rx

# Build
cargo build --release

# Run tests (140 total)
cargo test --release

# Run benchmarks
cargo bench --bench butterfly_bench
```

Requires: Rust 1.97.1+, AVX-512 capable CPU.

## Key Files

| File | Purpose |
|------|---------|
| `PROJECT_FACTS.md` | Canonical source of truth for all claims |
| `BENCHMARKS.md` | Benchmark methodology and results |
| `EVIDENCE_CHAIN.md` | Evidence provenance chain |
| `ARCHITECTURE.md` | System architecture and backend design |
| `README.md` | Project overview |
| `GRANT_PITCH.md` | Grant application (not asking you to fund — context for scope) |

## Commit Reference

- **Tag:** `v0.1.0-zksha-rx`
- **Commit:** `eb5533d` (frozen, GPG signed)
- **Branch tip:** `3dd6583` (identity metadata fix, GPG signed)

## Reviewer Guidelines

- You are not being asked to endorse, validate, or approve anything
- We want to know: are the boundaries honest? Are the claims backed by evidence?
- If you find a problem, tell us. That's the whole point.
- Expected time: 30 min (quick), 2-3 hours (full), 4-6 hours (reproduction)

## Contact

Sean Christopher Southwick — schlagetorren@gmail.com
