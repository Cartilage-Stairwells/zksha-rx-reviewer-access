# review-v0.1.13 Release Note

## Purpose

This release corrects reviewer-facing documentation language and aligns claims with canonical evidence status.

## Corrections

- Replaced obsolete current-performance wording that presented the historical 2.65× measurement as the canonical AVX-512 vs Scalar result.
- Canonical performance claim is now:
  - AVX-512 DIF butterfly: 1.265×–1.276× geometric mean
  - Criterion benchmark
  - AMD Zen 5 with AVX-512
  - 50 samples
  - dual-run verified

## Historical Provenance

The historical 2.65× measurement is retained only where needed for provenance and explicitly identified as a superseded measurement.

Historical records are preserved rather than rewritten to maintain an auditable evidence chain.

## Verification Scope Clarification

The project distinguishes:

- Formal verification of arithmetic foundations:
  - Lean 4 proofs
  - Montgomery arithmetic
  - butterfly algebra
  - NTT stage composition

- Implementation validation:
  - scalar/reference/AVX-512 backend equivalence testing
  - differential testing

The AVX-512 SIMD implementation is validated by testing and equivalence checks, not formally verified at the intrinsic level.

## Release Verification

Verified:

- Clean working tree
- Main branch synchronized with origin
- review-v0.1.13 tag resolves to commit 01db486
- Reviewer-facing documentation aligned with canonical claims
