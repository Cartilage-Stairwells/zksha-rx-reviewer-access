# B2 Execution-Path Evidence — STARK Proof Through the AVX-512 NTT Adapter

**Date:** 2026-09-12
**Branch:** feat/b1-integration-boundary
**Fixture:** tests/stark_prover_fixture.rs
**Claim scope:** execution-path linkage and proof-output invariance ONLY.

## What was run

The canonical Plonky3 0.1.0 Fibonacci STARK (structure identical to
p3-uni-stark's own `tests/fib_air.rs`), with exactly one substitution:

- Plonky3 reference run: `TwoAdicFriPcs` with `Dft = Radix2Dit<BabyBear>`
- This project's run:   `TwoAdicFriPcs` with `Dft = ZkshaDifAdapter` (AVX-512 lane)

Configuration (both runs identical, fully deterministic):
- Trace: 64 rows × 2 columns (Fibonacci)
- Poseidon2-16 permutation: fixed seed 0x5eed_b2e2 (no thread_rng)
- FRI: log_blowup 2, 28 queries, proof-of-work 8 bits
- Challenger: DuplexChallenger (Fiat-Shamir)
- debug build → uni-stark `check_constraints` active during prove

## Results

### 1. Prove path executed through the adapter — PASS
`p3_uni_stark::prove()` completed with `ZkshaDifAdapter` as the PCS DFT backend.

### 2. AVX-512 lane exercised inside proof generation — PASS
Shared tracker (Arc — counters shared with the PCS-owned adapter clones),
read after `prove()` returned:

```
reference_calls = 0
scalar_calls    = 0
avx512_calls    = 12
```

Twelve AVX-512 NTT executions occurred *inside* the prove call
(trace commitment LDE and quotient commitment LDE, each via
`TwoAdicFriPcs::commit → coset_lde_batch → idft_batch/dft_batch`).
Reference lane idle; scalar fallback idle (AVX-512 present on host).

### 3. Proof verified — PASS
`p3_uni_stark::verify()` accepted the adapter-backed proof.

### 4. Proof-output invariance — BYTE-IDENTICAL
Identical fixture through `Radix2Dit`, serialized with serde_json:

```
adapter proof:   124,410 bytes
Radix2Dit proof: 124,410 bytes
byte-for-byte equal: YES
```

The entire Fiat-Shamir transcript — Merkle commitments, sampled challenges,
opened values, FRI opening proof — is identical between the two DFT backends.
This is the strongest valid invariance: no randomized state, no representation
divergence. It follows from B1's bit-identical DFT output plus the
deterministic fixture.

### 5. Regression — full suite green
31 tests, 0 failures: 13 lib unit, 8 plonky3_integration (B1), 7 prover_fixture
(B1), 3 stark_prover_fixture (B2). The adapter instrumentation change below did
not alter any B1 assertion.

## Changes required for this evidence

1. `src/plonky3_adapter.rs`: tracker changed from `RefCell<..>` to
   `Arc<RefCell<..>>`. When the adapter is moved into `TwoAdicFriPcs` and
   cloned internally, all clones now share one counter set — a handle retained
   by the test records calls made inside the proving pipeline. The NTT
   computation is unchanged; only instrumentation wiring changed.
2. `src/lib.rs`: added `use p3_field::AbstractField;` to the `#[cfg(test)]`
   module — a latent B1 issue (the unit test never compiled under
   integration-test-only runs). Mechanical fix; full `cargo test` now passes.

## Claim boundary

NOT claimed: performance, end-to-end prover speedup, production readiness,
compatibility beyond Plonky3 0.1.0, anything about Experiment A.
Experiment A remains sealed. main remains untouched.

## Reproduction

```
cargo test --test stark_prover_fixture -- --nocapture
cargo test   # full suite, 31 tests
```
