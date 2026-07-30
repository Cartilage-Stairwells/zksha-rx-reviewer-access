# Reviewer Quickstart — zkSHA-Rx Fly v0.1.0

> **This repository is the frozen review entry point.**
> All artifacts referenced below are contained here.
> No private repository access is required.

## What This Is

An AVX-512 vectorized Number Theoretic Transform (NTT) for the BabyBear field (P = 0x78000001), the native field of Plonky3-style proving systems.

This is a **research artifact**, not a production library. You're reviewing whether the claims and evidence boundaries are honest.

## What to Review

Pick the scope that matches your expertise:

| Expertise | What to read | Key question |
|-----------|-------------|--------------|
| Benchmark methodology | `CLAIM_MATRIX.md`, `BENCHMARKS.md`, `MEASUREMENT.md`, `EVIDENCE_CHAIN.md` | Are the numbers measured correctly? |
| NTT/SIMD implementation | `src/avx512_butterfly_32bit.rs`, `src/lib.rs`, `tests/` | Is the three-backend approach valid? |
| Formal verification | `formal/README.md`, `PROJECT_FACTS.md` | Is the proof/claim boundary honest? |
| Claim/evidence alignment | `CLAIM_MATRIX.md`, `PROJECT_FACTS.md` | Do claims exceed evidence? |

## What IS Claimed

| Measurement | Scope | Result | Harness |
|-------------|-------|--------|---------|
| DIT butterfly kernel | isolated kernel | 9.15× (historical) | Criterion (0.5.1 → 0.8.2) |
| XOR butterfly kernel | isolated kernel | 4.58× (historical) | Criterion (0.5.1 → 0.8.2) |
| Full NTT sweep | sizes 2⁸–2²⁰ | 3.94× geo mean (historical) | perf_measure (not in snapshot) |

## What is NOT Claimed

- End-to-end proving-system acceleration (kernel-level only)
- Formally verified SIMD implementation (formal verification covers arithmetic, not SIMD)
- Production readiness (research artifact)
- Drop-in replacement for Plonky3/SP1 (integration is future work)

## How to Reproduce

```bash
# This repository is self-contained — clone THIS repo, not any other.
# Review tag: review-v0.1.4

# Build
cargo build --release

# Run tests (140 total)
cargo test --release

# Run benchmarks (requires AVX-512 CPU)
cargo bench --bench butterfly_bench
```

Requires: Rust 1.97.1+, AVX-512 capable CPU (for benchmarks only).

## Key Files

| File | Purpose |
|------|---------|
| `CLAIM_MATRIX.md` | Every claim, its measurement, evidence, and non-claim |
| `REVIEW_RELEASE.md` | Release tag, commit SHA, artifact hashes, reproduction |
| `PROJECT_FACTS.md` | Canonical source of truth for all external claims |
| `BENCHMARKS.md` | Benchmark methodology and results |
| `MEASUREMENT.md` | Raw measurement documentation |
| `EVIDENCE_CHAIN.md` | Evidence provenance chain |
| `ARCHITECTURE.md` | System architecture and backend design |
| `formal/README.md` | Lean 4 formal verification artifact info |
| `src/avx512_butterfly_32bit.rs` | AVX-512 SIMD implementation (real kernel) |
| `src/lib.rs` | Scalar reference + public API |

## Reviewer Guidelines

- You are not being asked to endorse, validate, or approve anything
- We want to know: are the boundaries honest? Are the claims backed by evidence?
- If you find a problem, tell us. That's the whole point.
- Expected time: 30 min (quick), 2-3 hours (full), 4-6 hours (reproduction)

## Contact

Sean Christopher Southwick — schlagetorren@gmail.com
