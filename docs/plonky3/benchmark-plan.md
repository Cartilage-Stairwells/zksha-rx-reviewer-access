# Benchmark Plan

## Three-Lane Methodology

1. Scalar baseline
2. AVX2 compiler vectorization
3. AVX-512 hand-written SIMD kernel

## Correctness Gate

All implementations must produce identical outputs before timing begins.

## Current Results

| Metric | Result |
|---|---|
| AVX-512 vs Scalar DIF butterfly | 1.265x-1.276x geometric mean |
| AVX2 vs Scalar | 1.07x |
| Correctness gate | PASS |
| Test coverage | 102 tests, 0 failures |

## Methodology

- AMD Zen 5 with AVX-512
- Criterion benchmark
- 50 samples
- dual-run verification

Historical measurements are documented separately and are not used as current performance claims.

## Planned Comparison

Direct comparison against Plonky3 AVX-512 implementation on identical hardware.
