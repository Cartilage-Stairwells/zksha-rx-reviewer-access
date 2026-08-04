# Verified NTT Integration Proposal — Plonky3

## What

This document proposes an integration path for incorporating zkSHA-Rx's formally verified NTT backend into the Plonky3 proving system.

zkSHA-Rx provides:
- A DIF radix-2 NTT with three execution paths (reference, scalar, AVX-512 SIMD)
- Formal verification in Lean 4 (83 theorems, 0 axioms, 0 sorries)
- A custody/provenance framework (TSCP) for verification evidence
- Three-lane benchmark with correctness gate

## Current State

### Adapter Feasibility (A2 Survey + Prototype)

A prototype adapter (`ZkshaDifButterfly`) implements Plonky3's `Butterfly<BabyBear>` trait using zkSHA-Rx's butterfly implementation.

Results:
- **Compiles** against the real Plonky3 codebase (Plonky3 v0.x, `p3-baby-bear`, `p3-field`)
- **Bit-identical output** vs Plonky3's `DifButterfly` at sizes 17, 64, and 256
- **Scalar performance**: geometric mean 1.015× vs Plonky3's scalar path (within noise)
- **AVX-512 benchmark**: pending dedicated hardware availability

### Binary Compatibility

BabyBear field types are binary-compatible between zkSHA-Rx and Plonky3:
- Same prime: P = 0x78000001 (2^32 - 2^28 + 1)
- Same Montgomery R constant: R = 2^32 mod P
- Same DIF butterfly semantics: (a + b, (a - b) * w)

### Integration Path

```
Plonky3 DFT trait (TwoAdicSubgroupDft)
        ↓
ZkshaDifButterfly adapter
        ↓
zkSHA-Rx butterfly (reference / scalar / AVX-512)
        ↓
Formal verification (Lean 4) + custody evidence (TSCP)
```

## What This Provides

This provides an integration path toward a formally verified NTT backend for Plonky3.

The formal verification covers:
1. **Montgomery arithmetic**: Bézout identity, REDC correctness, modular multiplication bounds
2. **Butterfly algebra**: DIF closure, Montgomery↔canonical equivalence, invertibility, additivity
3. **NTT stage composition**: stage validity preservation, determinism, disjoint butterfly commutativity, stage concatenation

## What This Does NOT Claim

- This does not claim to make Plonky3 "the first verified STARK prover"
- This does not claim end-to-end proving soundness — only NTT kernel correctness
- This does not claim the AVX-512 kernel is faster than Plonky3's existing implementation
- This does not claim formal verification of the SIMD intrinsics themselves (the formal model covers the algorithm, not the x86 execution)

## Value Proposition

The differentiator is not kernel speed — Plonky3 already has a performant AVX-512 BabyBear NTT. The differentiator is:

1. **Formal verification**: 83 Lean theorems proving the NTT algorithm is correct (not just tested)
2. **Custody framework**: verification evidence as auditable, tamper-evident artifacts
3. **Verification discipline**: the DIT→DIF fix demonstrates the verification system catching a real implementation bug

## Next Steps

1. **AVX-512 benchmark**: compare zkSHA-Rx's kernel against Plonky3's on identical hardware
2. **End-to-end test**: run a full Plonky3 proof using the adapter as the NTT backend
3. **Conformance vectors**: generate test vectors from the Lean model and verify against Rust output
4. **Independent reproduction**: have an external party reproduce the benchmarks

## References

- Plonky3 repository: https://github.com/Plonky3/Plonky3
- zkSHA-Rx reviewer snapshot: `git clone --branch review-v0.1.7 https://github.com/Cartilage-Stairwells/zksha-rx-reviewer-access`
- Proof-to-code correspondence: `docs/plonky3/proof-to-code-map.md`
- Benchmark evidence: `evidence/avx512_bench_dif_fix_20260804_023129.txt`
- Correctness receipt: `evidence/correctness_receipt_dif_fix.json`
