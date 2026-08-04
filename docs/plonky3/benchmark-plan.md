# Benchmark Plan

## Three-Lane Methodology

1. **Scalar baseline**: pure Rust, no SIMD
2. **AVX2 auto-vectorization**: compiler-generated, `-Ctarget-feature=avx2`
3. **AVX-512 kernel**: hand-written intrinsics, `avx512_radix2_butterfly_32`

## Correctness Gate

All three lanes must produce identical output on all test sizes before timing begins.

## Current Results (Intel AVX-512)

| Metric | Value |
|--------|-------|
| AVX-512 vs Scalar | 2.65× geometric mean (range 1.97×–3.97×) |
| AVX2 vs Scalar | 1.00× (no speedup from compiler) |
| Correctness gate | PASS (all sizes 2^8 through 2^20) |
| Test suite | 102 tests, 0 failures |

## Planned Comparison

Direct comparison against Plonky3's AVX-512 kernel on identical hardware, same transform sizes.

## Reproduction

```bash
git clone --branch review-v0.1.7 https://github.com/Cartilage-Stairwells/zksha-rx-reviewer-access
cd zksha-rx-reviewer-access
RUSTFLAGS="-Ctarget-cpu=native" cargo bench --bench three_lane_bench
```

## Hardware Requirements

- x86_64 with AVX-512F + AVX-512DQ
- Intel: Ice Lake or later recommended
- AMD: Zen 4 or later (note: Zen 5 uses 256-bit execution units for 512-bit ops)
