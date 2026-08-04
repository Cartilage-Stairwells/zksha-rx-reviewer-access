# Proof-to-Code Correspondence Map

This document maps the 83 Lean 4 theorems in zkSHA-Rx's formal verification stack to their corresponding Rust implementations.

## Overview

| Lean File | Theorems | Rust Counterpart | Layer |
|-----------|----------|------------------|-------|
| ReviewerSemantics.lean | 15 | TSCP custody framework | Layer 0 — Semantics |
| Montgomery.lean | 12 | src/field/babybear/montgomery.rs | Layer 1 — Arithmetic |
| Butterfly.lean | 25 | src/avx512_butterfly_32bit.rs, src/field/babybear/reference.rs | Layer 2 — Butterfly |
| NTTStage.lean | 8 | src/ntt.rs | Layer 3 — Stage Composition |
| Core.lean | 2 | Core verification | Layer 4 — Soundness |
| BridgePreservation.lean | 8 | Bridge lemmas | Cross-layer |
| Evidence/ManifestBinding.lean | 5 | Evidence binding | Evidence |
| Examples/NormalizationBridge.lean | 4 | Normalization | Examples |
| Examples/PropositionalKernel.lean | 3 | Propositional evaluation | Examples |
| TSCP_Formal_Backbone.lean | 1 | Backbone structure | Structure |
| **Total** | **83** | | |

## Layer 1 — Montgomery Arithmetic

### Montgomery.lean → src/field/babybear/montgomery.rs

| Lean Theorem | What It Proves | Rust Function |
|-------------|---------------|---------------|
| bezout_identity | R * R_inv = P * NEG_INV + 1 | Montgomery constants (R, R_inv, NEG_INV) |
| R_inv_correct | (R * R_inv) % P = 1 | R_inv field constant |
| montgomery_radix_coprime | gcd(R, P) = 1 | Ensures Montgomery form is valid |
| neg_inv_correct | (P * NEG_INV + 1) % R = 0 | REDC inverse constant |
| decode_encode_roundtrip | decode(encode(x)) = x | Montgomery encode/decode pair |
| montgomeryMul_lt_p | montgomeryMul(a,b) < P when a,b < P | montgomeryMul bounds check |
| mul_mod_congr | a ≡ a' (mod P) → a*c ≡ a'*c (mod P) | Congruence preservation |
| montgomeryMul_congr | a ≡ a' → montgomeryMul(a,b) ≡ montgomeryMul(a',b) | Montgomery congruence |
| cond_sub_eq_mod | u < 2P → conditional_sub(u,P) = u % P | Conditional subtraction |
| P_neg_inv_mod_R | (P * NEG_INV) % R = R - 1 | Magic constant verification |
| cios_exact | CIOS Montgomery reduction is exact | Montgomery reduction algorithm |
| montgomeryMul_scalar_correct | montgomeryMul(a,b) = (a*b*R_inv) mod P | Scalar Montgomery correctness |

## Layer 2 — Butterfly Algebra

### Butterfly.lean → src/avx512_butterfly_32bit.rs, src/field/babybear/reference.rs

| Lean Theorem | What It Proves | Rust Function |
|-------------|---------------|---------------|
| two_inv_correct | (2 * 2_inv) % P = 1 | Inverse of 2 constant |
| mod_add_lt | mod_add(a,b) < P when a,b < P | babybear_add_reference |
| mod_sub_lt | mod_sub(a,b) < P when a,b < P | babybear_sub_reference |
| mod_mul_lt | mod_mul(a,b) < P when a,b < P | babybear_mul_reference |
| mod_add_comm | mod_add(a,b) = mod_add(b,a) | Addition commutativity |
| mul_P_add_mod | (P*q + r) % P = r when r < P | Modular reduction identity |
| mod_eq_of_lt_of_congr | a ≡ b (mod P), a,b < P → a = b | Canonical form uniqueness |
| add_mod_lemma | (x%P + y%P) % P = (x+y) % P | Modular addition identity |
| mod_sub_congr | mod_sub(a,b) % P = (a + P - b) % P | Subtraction congruence |
| mod_sub_bridge_left | a ≡ a' → (a+P-b)%P = (a'+P-b)%P | Bridge lemma for subtraction |
| decompose_pos | r1 ≥ r2 → decomposition holds | Positive case decomposition |
| decompose_neg | r1 < r2 → decomposition holds | Negative case decomposition |
| decompose_rhs_pos | RHS positive case | Decomposition verification |
| decompose_rhs_neg | RHS negative case | Decomposition verification |
| mod_sub_congr_transport | x≡x', y≡y' → (x+P-y)%P = (x'+P-y')%P | Congruence transport |
| div_mod_diff_le | n/m ≤ 1 → n - n%m ≤ m | Subtraction bound |
| dif_closure | DIF butterfly output stays in [0, P) | butterfly_reference (DIF) |
| encode_lt | encode(x) < P when x < P | Montgomery encoding bound |
| mont_dif_closure | Montgomery DIF output stays in [0, P) | scalar_butterfly_32 (Montgomery) |
| encode_add | encode(a+b mod P) = mod_add(encode a, encode b) | Montgomery addition encoding |
| encode_sub | encode(a-b mod P) = mod_sub(encode a, encode b) | Montgomery subtraction encoding |
| encode_mul | encode(a*b mod P) = mod_mul(encode a, encode b) | Montgomery multiplication encoding |
| mont_dif_equivalence | Montgomery DIF = mathematical DIF on encoded values | scalar_butterfly_32 correctness |
| mod_mul_cancel | mod_mul(mod_mul(x,w), w_inv) = x when w*w_inv ≡ 1 | Inverse cancellation |
| dif_invertible | DIF butterfly is invertible with correct twiddle inverse | Butterfly invertibility |
| dit_closure | DIT butterfly output stays in [0, P) | butterfly_reference (DIT) |
| dif_additive | DIF butterfly is additive: f(a1+a2, b1+b2) = f(a1,b1) + f(a2,b2) | Butterfly additivity |

## Layer 3 — NTT Stage Composition

### NTTStage.lean → src/ntt.rs

| Lean Theorem | What It Proves | Rust Function |
|-------------|---------------|---------------|
| applyButterfly_untouched | Butterfly on (i,j) leaves other indices unchanged | ntt_scalar_stage (index handling) |
| applyButterfly_at_i | Butterfly updates index i correctly | ntt_scalar_stage (i update) |
| applyButterfly_at_j | Butterfly updates index j correctly | ntt_scalar_stage (j update) |
| butterfly_preserves_validity | Single butterfly preserves valid range | butterfly_reference |
| stage_preserves_validity | Stage of butterflies preserves valid range | ntt_reference_stage / ntt_scalar_stage |
| stage_deterministic | Stage.apply is deterministic (same input → same output) | All NTT stage functions |
| disjoint_butterflies_commute | Disjoint butterfly operations commute | ntt_scalar_stage (pair independence) |
| stage_concat | apply(s1 ++ s2) = apply s2 ∘ apply s1 | Multi-stage composition |

## Layer 0 — TSCP Semantics

### ReviewerSemantics.lean → TSCP custody framework (Rust)

| Lean Theorem | What It Proves | Rust Implementation |
|-------------|---------------|---------------------|
| plane_disjoint | Custody plane ≠ Authority plane | PlaneAssignment type |
| completeness_gating | Complete context → Success or Failure (not Indeterminate) | evaluate function |
| incompleteness_gating | Incomplete context → Indeterminate | evaluate function |
| determinism | Same input → same result (Lean functions are deterministic) | Evaluate contract |
| success_ne_failure | Success ≠ Failure | EvalResultType (sum type) |
| success_ne_indeterminate | Success ≠ Indeterminate | EvalResultType |
| failure_ne_indeterminate | Failure ≠ Indeterminate | EvalResultType |
| indeterminate_implies_incomplete | Indeterminate → context incomplete | Evaluate logic |
| evaluation_preserves_equality | Equal contexts → equal results | Evaluate contract |
| initial_reachable | Initial state is reachable | Reachability (base case) |
| authority_unreachability | Cannot reach Authority from Custody | Plane separation |
| state_equiv_refl | State equivalence is reflexive | StateEquiv |
| state_equiv_symm | State equivalence is symmetric | StateEquiv |

## Verification Properties

| Property | Formalized? | Tested? | Evidence |
|----------|-------------|---------|----------|
| Montgomery multiplication correct | ✅ (Lean) | ✅ (102 tests) | Montgomery.lean |
| DIF butterfly correct | ✅ (Lean) | ✅ (three-lane) | Butterfly.lean |
| NTT stage preserves validity | ✅ (Lean) | ✅ (stage tests) | NTTStage.lean |
| Multi-stage composition | ✅ (Lean) | ✅ (full NTT) | NTTStage.lean (stage_concat) |
| Backend equivalence | ❌ (not formalized) | ✅ (102 tests) | tests/ntt_equivalence.rs |
| AVX-512 = scalar = reference | ❌ (not formalized) | ✅ (three-lane) | evidence/correctness_receipt_dif_fix.json |
| DFT correctness | ❌ (not formalized) | ✅ (oracle test) | tests/ntt_equivalence.rs |

## What Is NOT Formally Verified

- The AVX-512 SIMD intrinsics themselves (x86 execution is not modeled in Lean)
- Backend equivalence (reference vs scalar vs AVX-512) — tested but not proven
- The Rust compiler's code generation — trusted, not verified
- The CPU's hardware implementation — trusted, not verified

The formal verification covers the **algorithm**, not the **execution substrate**.
