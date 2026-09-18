# Experiment B — Observational Finding (Sealed)

## Evidence Identity: experiment_b_v1
## Date: 2026-08-18
## Type: Observational (source inspection, no runtime modification)

## Repository State
- Repo: tscp-anchor
- Branch: master
- HEAD: 58208dab19156424d0b6d9e532f805732ed7be47
- Tag: benchmark/firebird_74c6e5f (at 653ce061)
- Clone: https://github.com/Cartilage-Stairwells/tscp-anchor.git

## Methodology
Static source inspection and call-path tracing. No code modification, no runtime change.
Production code = all code NOT inside `#[cfg(test)]` modules.

## Principal Finding

**The current production proving path does not invoke the NTT/FFT butterfly implementation, and the repository's Avx512Backend is presently a scalar placeholder.**

## Evidence

### 1. Production Prover Path

```
prove_instrumented_internal (oracle_bridge.rs:145)
  → fri_prove (fri_query.rs)
    → MerkleTree::build × (log2(n) + 1) — Poseidon2 hashing
    → fri_fold_step × log2(n) rounds (fri.rs)
    → fri_query_round — Merkle opening proofs
```

### 2. fri_fold_step is NOT an NTT butterfly

NTT butterfly: `(a + b, (a - b) * w)` — 2 inputs → 2 outputs (swap operation)
FRI fold: `(p(x) + p(-x))/2 + β * (p(x) - p(-x))/(2x)` — 2 inputs → 1 output (combine operation)

These are structurally different operations. The AVX-512 kernel accelerates the butterfly; the prover performs the fold.

Source: fri.rs, `pub fn fri_fold_step`

### 3. Radix2Interpolator (FFT/NTT) has ZERO production uses

All calls to `Radix2Interpolator::evaluate_coeffs`, `Radix2Interpolator::fft`, `Radix2Interpolator::ifft` are inside `#[cfg(test)]` modules in:
- fri_protocol.rs (6 calls, all after line 158 `#[cfg(test)]`)
- fri_query.rs (4 calls, all after line 243 `#[cfg(test)]`)
- quotient.rs (1 call, after line 65 `#[cfg(test)]`)

Production (non-test) calls: **ZERO** (verified by Python scan stripping test modules)

### 4. Avx512Backend is a placeholder

Source: tscp-backends/src/avx512.rs

Module documentation explicitly states:
> "This is NOT an AVX-512 implementation. It does not use any SIMD intrinsics."

The struct delegates to `p3_dft::Radix2Dit` — the same scalar implementation used by `ScalarBackend`.
The `name()` method returns `"placeholder-radix2-dit"`.

The test `placeholder_backend_matches_scalar` is documented as tautological:
> "Both backends delegate to the same `Radix2Dit` implementation, so this test is tautological."

### 5. avx512-butterfly kernel is disconnected

The standalone avx512-butterfly repository contains a real AVX-512 DIF butterfly kernel using 16 `_mm512_*` intrinsics. This kernel is NOT referenced by, imported by, or connected to the tscp-anchor prover in any way.

## Interpretation

The evidence does NOT establish that AVX-512 is useless to the prover. It establishes that the current prover architecture does not use NTT/FFT butterfly operations, so the existing AVX-512 kernel cannot improve prover performance without architectural change.

The prover's dominant cost components are:
1. MerkleTree::build (Poseidon2 hashing) — called log2(n)+1 times
2. fri_fold_step (element-wise field arithmetic) — called log2(n) times
3. fri_query_round (Merkle path openings)

Which component dominates requires profiling measurement (Experiment C).

## What This Finding Does NOT Establish

- Poseidon2/Merkle is definitively the dominant cost (plausible but unmeasured)
- The AVX-512 kernel has no value to the project (it may have value if NTT is added to the prover path)
- End-to-end prover speedup from any SIMD optimization (not measured)

## Defensible Statement

> The current production proving path in tscp-anchor does not invoke the NTT/FFT butterfly implementation. The repository's Avx512Backend is a scalar placeholder. The FRI folding operation performed by the prover is structurally distinct from an NTT butterfly. Integration of the AVX-512 kernel would require architectural change, not a drop-in replacement. The dominant cost component within the prover requires profiling measurement to establish.

## Files Inspected

- crates/tscp-verifier/src/oracle_bridge.rs (prove_instrumented_internal)
- crates/oracle-layer/src/fri_query.rs (fri_prove, fri_query_round)
- crates/oracle-layer/src/fri.rs (fri_fold_step, fold_domain)
- crates/oracle-layer/src/fft.rs (Radix2Interpolator — production vs test usage)
- crates/oracle-layer/src/fri_protocol.rs (fri_commit — production vs test usage)
- crates/oracle-layer/src/quotient.rs (production vs test usage)
- crates/oracle-layer/src/sumcheck.rs (sumcheck — exists but not exercised by benchmark)
- crates/oracle-layer/src/merkle.rs (MerkleTree::build)
- crates/tscp-backends/src/avx512.rs (placeholder backend)
- crates/tscp-backends/src/lib.rs (backend module structure)
- crates/tscp-verifier/benches/evidence_baseline.rs (firebird benchmark harness)
