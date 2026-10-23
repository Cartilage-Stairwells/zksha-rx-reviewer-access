# Measurement record: three_lane_bench independent reproduction
**Date:** 2026-10-23 | **Operator:** Solene (custodian seat) | **Tag under test:** review-v0.1.19 (commit 612cd89)
**Command:** RUSTFLAGS="-Ctarget-cpu=native" cargo bench --bench three_lane_bench (per docs/EXTERNAL_REVIEWER_GUIDE.md)
**Hardware:** x86_64, 4 cores, AVX-512 (f/bw/cd/dq/vbmi/vl), model masked as "unknown" (sandboxed cloud VM)
**Toolchain:** rustc 1.99.1, Criterion as configured in bench (50 samples, 500ms warmup, 2s measurement — matches canonical protocol)

## Measured results (this machine)
- Correctness gate: PASS — all three lanes agree on sizes 2^8 through 2^20
- AVX-512 vs scalar geometric mean: **6.988x** (range 5.666x–7.854x across 13 sizes)
- AVX-512 vs avx2 geometric mean: 7.057x
- Observation: scalar lane and avx2 lane times are equal within noise on this platform (e.g., 2^20: 1,226,800 vs 1,244,500 ns)
- Raw log: three-lane-bench-20261023.log; parsed values: three-lane-bench-parsed-20261023.json

## Comparison against published canonical evidence (parsed from evidence/canonical/canonical_run1+2, Zen 5)
- canonical_run1 geo 512/scalar = 1.279x; canonical_run2 = 1.269x; dual-run delta 0.78% (consistent with the published 1.265x–1.276x claim and 0.9% variance)
- canonical runs: scalar lane times ≈ avx2 lane times at every size, in several cases identical to the nanosecond (2^18 run1: 44,401 vs 44,402)
- evidence/experiment_a/benchmark_znver5_2_20.txt (same three-lane harness, earlier Zen 5 experiment, single size 2^20): 512/scalar = 8.67x

## Separation of observation and hypothesis
OBSERVED: (1) gate PASS on all 13 sizes on this non-Zen5 AVX-512 machine; (2) the 512/scalar ratio differs by ~5.5x between this run (6.99x) and the canonical Zen 5 runs (1.27x); (3) on both platforms the scalar and avx2 lanes time identically, but at different absolute levels; (4) the project's own evidence contains both regimes on Zen 5 (8.67x in the earlier experiment log vs 1.27x canonical).
HYPOTHESIS VERIFIED (disassembly probe, 2026-10-23): the scalar-lane loop (copied verbatim from benches/three_lane_bench.rs into an isolated probe crate) was compiled under three flag configurations and disassembled:
- canonical flags (-C target-cpu=native -C target-feature=+avx512f,+avx512dq,+avx2): 452 insns, 16 zmm + 29 ymm — AUTO-VECTORIZED
- quick-start flags (-C target-feature=+avx512f,+avx512dq, no native): 541 insns, 23 zmm + 28 ymm — AUTO-VECTORIZED
- no flags: 180 insns, 0 zmm, 0 ymm — TRUE SCALAR
Conclusion (machine code verified on this platform): under both documented flag sets, the "scalar baseline" lane is compiler-auto-vectorized AVX-512/AVX2 code, not scalar machine code. The canonical 1.27x is therefore: hand-written kernel vs the compiler's auto-vectorization of the same operation on Zen 5. The earlier experiment's 8.67x regime (and its disasm_scalar.txt showing no vector registers) corresponds to a no-AVX-512-features build. The cross-platform spread (1.27x on Zen 5 vs 6.99x here) follows from auto-vectorization effectiveness: on Zen 5 the compiler-vectorized baseline nearly matches the hand kernel; on this machine it achieves little (this machine's criterion-measured scalar lane at 2^20: 1,226,800 ns vs Zen 5 canonical: 193,650 ns, while the hand kernel differs only ~1.4x: 187,570 vs 131,470 ns).

## Disposition
No release action taken. This record is evidence for a claim-interpretation decision (the 1.27x figure is honestly measured but its baseline's machine-code definition under canonical flags is not documented; the same harness yields ~7x where the baseline is true scalar). Flagged to operator and audit seat for disposition.
