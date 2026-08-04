# Security Model

## What This Document Is

A precise statement of what zkSHA-Rx's formal verification eliminates, what remains outside the proof boundary, and what assumptions the verification depends on. This document exists because experienced cryptographers ask these questions immediately.

## What Formal Verification Eliminates

The 83 Lean theorems prove properties of the **algorithm**, not the **execution substrate**.

### Montgomery Arithmetic (12 theorems — Montgomery.lean)

**Eliminated bug class:** Montgomery multiplication producing incorrect results or out-of-range outputs.

- Bézout identity: R × R_inv = P × NEG_INV + 1 (the constants are correct)
- REDC correctness: Montgomery reduction is algebraically exact
- Modular bounds: montgomeryMul(a,b) < P when a,b < P
- Encoding roundtrip: decode(encode(x)) = x

### Butterfly Algebra (25 theorems — Butterfly.lean)

**Eliminated bug class:** Butterfly operation producing incorrect field arithmetic or breaking algebraic invariants.

- DIF closure: butterfly output stays in [0, P)
- Montgomery↔canonical equivalence: Montgomery DIF = mathematical DIF on encoded values
- Invertibility: DIF butterfly is invertible with correct twiddle inverse
- Additivity: DIF butterfly is additive (f(a1+a2, b1+b2) = f(a1,b1) + f(a2,b2))
- Encoding preservation: encode preserves addition, subtraction, and multiplication

### NTT Stage Composition (8 theorems — NTTStage.lean)

**Eliminated bug class:** Multi-stage composition breaking validity or introducing non-determinism.

- Stage validity preservation: a stage of butterflies preserves the valid range property
- Stage determinism: same input → same output (deterministic)
- Disjoint butterfly commutativity: non-overlapping butterfly operations commute
- Stage concatenation: apply(s1 ++ s2) = apply s2 ∘ apply s1 (composition algebra)

### TSCP Custody Semantics (15 theorems — ReviewerSemantics.lean)

**Eliminated bug class:** Custody/authority plane confusion in the evaluation framework.

- Plane separation: custody plane ≠ authority plane (structurally enforced)
- Completeness gating: complete context → Success or Failure (not Indeterminate)
- Determinism: equal contexts → equal results

## What Remains Outside the Proof Boundary

| Concern | Status | Why |
|---------|--------|-----|
| AVX-512 SIMD intrinsics | Tested, not proven | x86 execution is not modeled in Lean |
| Backend equivalence (ref ↔ scalar ↔ AVX-512) | Tested (102 tests), not proven | Cross-backend equivalence is validated by differential testing |
| Rust compiler code generation | Trusted | The compiler is assumed correct (standard assumption) |
| CPU hardware implementation | Trusted | The CPU is assumed to implement x86-64 correctly |
| Memory safety | Handled by Rust | Rust's type system, not Lean |
| Side-channel resistance | Not addressed | No constant-time guarantees, no timing analysis |
| Twiddle factor generation | Tested, not proven | Twiddle computation is outside the formal model |
| Bit-reversal permutation | Tested, not proven | Post-NTT reordering is tested but not formally verified |

## Assumptions

| Assumption | Type | Justification |
|-----------|------|---------------|
| SHA-256 collision resistance | Cryptographic | Standard assumption for custody framework |
| CPU implements x86-64 correctly | Hardware | Standard trusted computing base |
| Rust compiler is correct | Compiler | Standard assumption (miri addresses some UB) |
| Mathlib `ring` tactic is sound | Mathematical | Mathlib is a community-reviewed library; `ring` is a decision procedure for commutative rings |
| BabyBear prime is correctly chosen | Domain | P = 2^32 - 2^28 + 1 is a well-known NTT-friendly prime |

## Verified vs Tested vs Benchmarked

| Layer | Method | Coverage |
|-------|--------|----------|
| **Verified** (Lean 4) | Formal proof | Montgomery arithmetic, butterfly algebra, NTT stage composition, custody semantics — 83 theorems, 0 axioms, 0 sorries |
| **Tested** (Rust) | Differential testing | 102 tests: three-backend equivalence, NTT correctness vs DFT oracle, round-trip, field operations |
| **Benchmarked** | Criterion measurement | Three-lane benchmark (scalar / AVX2 / AVX-512) with correctness gate, 13 transform sizes 2^8 through 2^20 |

## The Verification Boundary (Precise Statement)

The formal verification covers the **algorithmic specification** of the NTT:
- The mathematical butterfly is correct
- Montgomery arithmetic preserves field properties
- Stage composition preserves validity and determinism

The formal verification does **not** cover:
- The translation from Lean specification to Rust implementation (this is a conformance gap, addressed by testing)
- The translation from Rust source to x86 machine code (this is a compiler trust assumption)
- The execution of x86 instructions on physical hardware (this is a hardware trust assumption)

The conformance gap between Lean and Rust is bridged by:
1. The proof-to-code correspondence map (`docs/plonky3/proof-to-code-map.md`)
2. 102 differential tests across three backends
3. The DIT→DIF fix (a real bug caught by this testing infrastructure)

## The DIT→DIF Fix as Evidence

The verification system caught a real implementation bug: the butterfly used DIT semantics (a+b*w, a-b*w) when the NTT structure requires DIF (a+b, (a-b)*w). This was discovered through NTT correctness testing against a naive DFT oracle, corrected across all three backends, and documented with evidence.

This is the verification boundary working as designed: testing catches what formal verification doesn't cover (the Lean→Rust conformance gap), and the custody framework preserves the evidence trail.
