# Review Responses

## Performance Claim

**Q: What is the measured AVX-512 speedup?**

A:

The supported benchmark result is a kernel-level DIF butterfly measurement:

- AVX-512 vs Scalar: 1.265x-1.276x geometric mean
- AMD Zen 5 with AVX-512
- Criterion benchmark
- 50 samples
- dual-run verified

Earlier measurements, including a 2.65x result, came from different benchmark configurations and are preserved only as historical records.

They are not the current performance characterization.

## End-to-End Speedup

**Q: Does this demonstrate proportional prover acceleration?**

A:

No.

The benchmark measures an NTT kernel component. End-to-end proving performance depends on:

- NTT contribution to total proving time
- memory behavior
- integration overhead
- system architecture

Kernel speedup is not equivalent to prover speedup.

## Formal Verification

**Q: Is the AVX-512 implementation formally verified?**

A:

No.

The formal proofs cover arithmetic and algorithmic properties.

The SIMD implementation is validated through:

- backend equivalence testing
- differential testing
- correctness gates

The distinction between formally verified foundations and tested implementation paths is intentional.
