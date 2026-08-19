# EXPERIMENT A — Evidence Manifest

## Identity
- Experiment: A (Kernel Evidence Establishment)
- Date: 2026-08-19
- Evidence ID: experiment-a-20260819 (provisional, pending Sean's review)

## Source Identity
- Repository: Cartilage-Stairwells/zksha-rx-reviewer-access
- Commit: 46c09eba5d2f (main branch; same source as tagged commit 01db486d / review-v0.1.13 — 4 intervening commits are documentation-only, verified by SHA-256 match of all source files)
- Source files:
  - src/avx512_butterfly_32bit.rs (295 lines)
  - src/ntt.rs (207 lines)
  - src/field/babybear/constants.rs
  - src/field/babybear/montgomery.rs (194 lines)
  - src/field/babybear/reference.rs
  - benches/three_lane_bench.rs (three-lane benchmark with correctness gate)

## Hardware Identity
- CPU: AMD (AuthenticAMD, family 191, model 2, znver5)
- AVX-512: Full support (avx512f, avx512dq, avx512cd, avx512bw, avx512vl, avx512vbmi, avx512vbmi2, avx512vnni, avx512bitalg, avx512vpopcntdq)
- 4 cores, ~4130 MHz
- OS: Debian 12 (bookworm), Linux 4.19.0-gvisor

## Toolchain Identity
- Rust: 1.97.1 (8bab26f4f 2026-07-14)
- Cargo: 1.97.1 (c980f4866 2026-06-30)
- Target: x86_64-unknown-linux-gnu
- Benchmark build: RUSTFLAGS="-C target-cpu=x86-64" (clean scalar baseline)
- Benchmark harness: Criterion 0.8.2

## Benchmark Methodology
- Harness: Criterion 0.8.2 (--quick mode: 10 samples, 3s warmup for full range)
- Targeted runs: 30 samples, 1s warmup, 3s measurement at key sizes
- Sizes: 2^8 through 2^20 (13 sizes)
- Correctness gate: PASS (all three lanes produce identical output)
- 5 correctness test sizes (2^8, 2^10, 2^12, 2^16, 2^20)

## Binary Verification
- Scalar kernel: 0 ymm (AVX2), 0 zmm (AVX-512) — SSE (128-bit xmm) only
- AVX2 kernel: 31 ymm instructions, 0 zmm — confirmed AVX2
- AVX-512 kernel: Full zmm instruction set — confirmed AVX-512
- Disassembly evidence: disasm_scalar.txt, disasm_avx2.txt, disasm_avx512.txt

## Methodology Context

Experiment A uses `RUSTFLAGS="-C target-cpu=x86-64"` which prevents the compiler
from auto-vectorizing the scalar baseline with AVX2/AVX-512 instructions. This
produces a pure SSE scalar baseline.

The repository's CANONICAL_RESULTS.md uses `target-cpu=native` which allows
compiler auto-vectorization in the scalar baseline. This produces a faster
scalar baseline and consequently lower speedup ratios (1.265x-1.276x geometric
mean vs Experiment A's 4.42x).

Both measurements are valid under their respective methodologies. Experiment A
isolates the hand-written AVX-512 kernel's contribution by eliminating
compiler auto-vectorization from the scalar baseline. The canonical results
measure realistic end-to-end improvement including compiler auto-vectorization.

## Results Summary
- Headline (2^20): AVX-512 7.23x vs scalar, 7.34x vs AVX2
- Geometric mean (2^8-2^20): AVX-512 4.42x vs scalar, 4.44x vs AVX2
- AVX2 vs scalar: ~1.00x (geometric mean) — compiler auto-vectorization provides no speedup
- Control: x86-64 scalar ≈ znver5 scalar (no hidden auto-vectorization)

## Claims Supported
1. The identified scalar, AVX2, and AVX-512 implementations produce equivalent results for the tested domain (2^8 through 2^20).
2. Under the specified hardware (AMD Zen 5) and benchmark methodology (Criterion 0.8.2, target-cpu=x86-64), the identified hand-written AVX-512 implementation achieves 7.23x speedup over scalar at 2^20 (geometric mean 4.42x across all sizes).
3. The tested compiler-vectorized AVX2 lane measured approximately 1.00x scalar under the specified conditions.
4. The scalar kernel uses no AVX2 or AVX-512 instructions (binary-verified).
5. The AVX2 kernel uses AVX2 instructions and no AVX-512 instructions (binary-verified).
6. The AVX-512 kernel uses AVX-512 instructions (binary-verified).

## Claims Explicitly NOT Supported
1. Proving-path integration (FRI prover does not invoke this kernel)
2. FRI invocation
3. End-to-end prover speedup
4. Correspondence with the admissibility experiment (067fb3ed)
5. Formal verification of the SIMD implementation
6. Universal performance superiority (only tested on one AMD CPU, one operation)
7. Correctness outside the tested domain (2^8-2^20, random inputs)
8. Hand-written AVX2 kernel performance (the AVX2 lane is compiler-auto-vectorized, not hand-optimized)
9. Performance on Intel hardware (not tested)
10. Performance of the original 64-bit/8-wide kernel (tscp-pl-phase1 — not benchmarked here)

## Frozen Artifacts NOT Modified
- 067fb3ed (correspondence experiment): NOT touched
- kernel-archaeology/admissibility-experiment-v0.1: NOT touched
- firebird_74c6e5f: NOT touched
- zksha-rx-reviewer-access repository: NOT modified (read-only fetch)
- tscp-anchor master: NOT touched
