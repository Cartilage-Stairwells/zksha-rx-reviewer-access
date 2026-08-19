# Phase 1 — Kernel Selection

## Selected Implementation
- Repository: zksha-rx-reviewer-access
- Commit: 46c09eba5d2f (main branch, tag review-v0.1.13)
- Source files:
  - src/avx512_butterfly_32bit.rs (295 lines, 32-bit/16-wide AVX-512 kernel)
  - src/ntt.rs (207 lines, three NTT backends: reference, scalar, AVX-512)
  - src/field/babybear/constants.rs (BabyBear prime constants)
  - src/field/babybear/montgomery.rs (194 lines, scalar Montgomery backend)
  - src/field/babybear/reference.rs (reference oracle)
  - benches/three_lane_bench.rs (three-lane benchmark with correctness gate)
- Kernel name: avx512_butterfly (Cargo package name)
- Field/modulus: BabyBear, p = 0x78000001 = 2^31 - 2^27 + 1
- Representation: Montgomery R = 2^32, u32 elements
- SIMD width: 16 lanes of 32-bit values per __m512i
- Target architecture: x86_64
- Butterfly type: DIF (decimation-in-frequency)
- Formula: x' = a + b mod p, y' = (a - b) * w mod p

## Why This Implementation Was Selected
1. It is a complete, buildable Cargo project with proper structure
2. It has three NTT backends (reference, scalar, AVX-512) built in
3. It has an existing three-lane benchmark (three_lane_bench.rs) with correctness gate
4. It uses the same mathematical algorithm (DIF, same constants) as the original kernel
5. It was specifically created as a reviewer-facing evidence snapshot
6. It builds and all 139 tests pass on this hardware
7. It uses Criterion 0.8.2 (modern benchmarking)

## Alternatives Considered
- tscp-pl-phase1/babybear_avx512_butterfly.rs: 64-bit/8-wide, flat file, no Cargo project structure, would require wrapping. Has both DIT and DIF.
- tscp-anchor/crates/tscp-backends/src/avx512.rs: Placeholder (scalar delegate, no AVX-512). Not a real implementation.

## AVX2 Lane Characterization
The AVX2 lane in three_lane_bench.rs is NOT a hand-optimized AVX2 kernel.
It is scalar code compiled with #[target_feature(enable = "avx2")], which
allows the compiler to auto-vectorize using AVX2 instructions. This is a
legitimate approach but differs from a hand-written AVX2 butterfly kernel.

## Hardware Note
This machine is AMD (family 191, model 2, identified as znver5 by rustc).
Different from firebird_74c6e5f's Ice Lake-SP. Experiment A gets its own
evidence identity, so this is acceptable.
