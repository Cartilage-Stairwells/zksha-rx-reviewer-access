# Prepared Responses to Technical Objections

This document contains prepared responses to the most likely technical objections from reviewers. Each response is precise, scoped, and references the evidence.

## Objection 1: "Are the Lean proofs connected to the optimized SIMD implementation?"

**Imprecise answer (avoid):** "The NTT is formally verified."

**Precise answer:** "The verified specification covers the algorithmic level — Montgomery arithmetic (12 theorems), butterfly algebra (25 theorems), and NTT stage composition (8 theorems). The AVX-512 SIMD backend equivalence is validated through 102 differential tests across three backends (reference, scalar, AVX-512), not formal proof. The formal model proves the algorithm correct; the tests prove the implementations agree. The conformance gap between Lean and Rust is documented in `docs/plonky3/proof-to-code-map.md` and `docs/SECURITY_MODEL.md`."

**Key distinction:** formal verification of the algorithm ≠ formal verification of the implementation.

## Objection 2: "Why integrate another NTT instead of improving Plonky3's?"

**Answer:** "The integration is intentionally backend-level. The adapter implements Plonky3's `Butterfly<BabyBear>` trait without modifying Plonky3's existing NTT path. This allows evaluation without architectural disruption. The value proposition is not kernel speed — Plonky3 already has a performant AVX-512 NTT. The value is the formal verification layer (83 Lean theorems) and the custody framework."

## Objection 3: "What is the actual end-to-end proving speedup?"

**Answer:** "We do not claim end-to-end proving speedup. The 2.65× is a kernel-level measurement (AVX-512 vs scalar, geometric mean across 13 transform sizes). End-to-end impact depends on NTT's share of total proving time (30–60% in STARK systems) and integration overhead, which we have not yet measured. We explicitly separate kernel speedup from prover speedup in `CLAIM_MATRIX.md`."

## Objection 4: "Has anyone independently reproduced these results?"

**Answer:** "Not yet. Independent reproduction is the highest-priority next step. The benchmark is reproducible via `make bench` on any AVX-512 capable machine. All evidence artifacts (benchmark output, CPU info, correctness receipt) are in the `evidence/` directory with SHA256 checksums."

## Objection 5: "The 2.65× speedup is modest compared to GPU acceleration."

**Answer:** "Correct. GPU acceleration achieves 10–100× for NTT. zkSHA-Rx targets a different deployment profile: zero-cost deployment on existing cloud infrastructure, no GPU dependency. The 2.65× is measured on commodity CPU hardware. The differentiator is not speed — it is the formal verification layer and custody framework, which no GPU implementation provides."

## Objection 6: "Lean proofs with 0 axioms and 0 sorries — is that real?"

**Answer:** "Yes. The 83 theorems are verified with `lake build` producing zero errors, zero axioms, and zero sorries. The proof uses Mathlib's `ring` tactic for algebraic identity discharge (commutative ring normalization), which is a well-established decision procedure. No custom axioms are introduced. The build is reproducible — clone tscp-anchor and run `lake build`."

## Objection 7: "What stops someone from just taking this code?"

**Answer:** "The work is open-source (see LICENSE). The value is not in the code alone — it is in the verification stack (83 Lean theorems), the custody framework (TSCP), and the evidence chain. These represent months of work that cannot be replicated by copying source files."
