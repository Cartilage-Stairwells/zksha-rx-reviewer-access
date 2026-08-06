# External Reviewer Guide

## Quick Start

```bash
git clone --branch review-v0.1.8 https://github.com/Cartilage-Stairwells/zksha-rx-reviewer-access
cd zksha-rx-reviewer-access
./validate_release.sh
```

The validation script checks:
- Tag identity (you're on the right release)
- Document integrity (REVIEW_RELEASE.md matches tag)
- Checksum integrity (SHA256SUMS verified)
- Dead path check (no missing files)
- Historical labeling (provenance labels present)

## What You're Looking At

This is a **reviewer snapshot** — a curated, immutable view of the zkSHA-Rx project at a specific point in time. It contains:

1. **Source code** (`src/`) — the NTT implementation with three backends
2. **Formal verification** (`formal/`) — Lean 4 proof descriptions
3. **Tests** (`tests/`) — 102 tests covering backend equivalence, NTT correctness, and round-trip
4. **Benchmarks** (`benches/`) — three-lane benchmark (scalar / AVX2 / AVX-512)
5. **Evidence** (`evidence/`) — benchmark output, CPU info, correctness receipts
6. **Documentation** — claims matrix, measurement methodology, architecture

## What to Review

### If you care about formal verification:
- Read `formal/README.md` for the proof structure
- Read `docs/plonky3/proof-to-code-map.md` for theorem-to-code mapping
- Check: 83 theorems, 0 axioms, 0 sorries — no unproven gaps

### If you care about performance:
- Read `BENCHMARKS.md` for the three-lane methodology
- Read `evidence/avx512_bench_dif_fix_20260804_023129.txt` for raw output
- Key finding: AVX-512 vs Scalar = 1.265×–1.276× geometric mean (historical 2.65× measurement superseded) (measured, not projected)

### If you care about correctness:
- Read `evidence/correctness_receipt_dif_fix.json` for the correctness gate
- The DIT→DIF fix story: verification caught a real implementation bug
- All three backends (reference, scalar, AVX-512) produce identical output

### If you care about the custody/provenance framework:
- Read `EVIDENCE_CHAIN.md` for the five-stage chain
- Read `ARCHITECTURE.md` for the layer structure
- The framework makes verification failures diagnosable

## Build Instructions

```bash
# Requires: Rust 1.97+, x86_64 with AVX-512 for full benchmark
RUSTFLAGS="-Ctarget-cpu=native" cargo build --tests
RUSTFLAGS="-Ctarget-cpu=native" cargo test
```

Note: The snapshot includes a Cargo.toml that references an `iep_runner` binary not present in the snapshot. To run tests, create a stub:

```bash
mkdir -p instrument/runner
echo 'fn main() {}' > instrument/runner/main.rs
cargo test
```

## Benchmark Instructions

```bash
RUSTFLAGS="-Ctarget-cpu=native" cargo bench --bench three_lane_bench
```

This runs the three-lane benchmark with a correctness gate. All three lanes must agree before timing begins.

## Claims and Non-Claims

### What the evidence supports:
- AVX-512 implementation provides measured kernel-level acceleration (1.265×–1.276× canonical, 2.65× historical) on tested workloads
- 102 tests pass with 0 failures
- Three-layer backend equivalence (reference ↔ scalar ↔ AVX-512)
- 83 Lean theorems with 0 axioms and 0 sorries
- The DIT→DIF fix was a real bug caught by verification

### What the evidence does NOT support:
- Universal acceleration across all AVX-512 microarchitectures
- End-to-end proving speedup (kernel speedup ≠ prover speedup)
- Formal verification of the SIMD intrinsics themselves
- Production-readiness or integration into any proving system

## Feedback

Feedback is welcome. Contact: schlagetorren@gmail.com

## Repository Structure

```
├── README.md                    ← Start here
├── docs/
│   ├── plonky3/                 ← Integration proposal
│   └── EXTERNAL_REVIEWER_GUIDE.md ← This file
├── src/                         ← NTT implementation
├── formal/                      ← Lean proof descriptions
├── tests/                       ← 102 tests
├── benches/                     ← Three-lane benchmark
├── evidence/                    ← Benchmark + correctness receipts
├── BENCHMARKS.md                ← Performance details
├── CLAIM_MATRIX.md              ← Every claim, its evidence, its non-claim
├── PROJECT_FACTS.md             ← Canonical source-of-truth
└── validate_release.sh          ← Release validation gate
```
