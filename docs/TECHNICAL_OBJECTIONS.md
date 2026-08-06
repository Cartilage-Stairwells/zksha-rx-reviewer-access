# Technical Objections

## Objection: "What is the actual performance improvement?"

**Answer:**

The supported measured result is:

AVX-512 vs Scalar DIF butterfly kernel:
1.265x-1.276x geometric mean

This is a kernel-level measurement performed on AMD Zen 5 hardware with AVX-512 using the canonical benchmark protocol.

Historical benchmark numbers are retained as provenance records but are not used as current performance claims.

## Objection: "Does this imply end-to-end proving acceleration?"

**Answer:**

No.

The benchmark isolates a kernel operation. Full proving performance depends on the proportion of runtime spent in the measured operation and on integration effects.

The project explicitly separates:

- kernel acceleration
- full NTT performance
- complete prover performance

## Objection: "Is the SIMD implementation formally verified?"

**Answer:**

No.

The Lean formalization verifies arithmetic foundations and algorithmic properties.

The SIMD implementation is validated through differential testing and backend equivalence.
