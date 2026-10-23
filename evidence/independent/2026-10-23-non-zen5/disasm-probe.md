# Scalar-lane codegen probe — 2026-10-23

## Question
Does the benchmark's "scalar baseline" lane (`scalar_butterfly_pass`, benches/three_lane_bench.rs, pure-u32 source) compile to scalar machine code under the documented flag sets?

## Method
The function `scalar_butterfly_pass` was copied verbatim (with its `babybear_mul_reference` dependency, itself copied verbatim from src/field/babybear/reference.rs) into an isolated probe crate and compiled in release mode under three flag configurations; the function's disassembly (objdump) was analyzed for vector registers. `#[inline(never)]` was added to preserve the symbol; the loop body is unmodified.

## Results
| Config | RUSTFLAGS | insn count | zmm | ymm | xmm | Codegen |
|---|---|---|---|---|---|---|
| A (canonical) | `-C target-cpu=native -C target-feature=+avx512f,+avx512dq,+avx2` | 452 | 16 | 29 | 72 | auto-vectorized |
| B (BENCHMARKS.md quick-start, pre-clarification) | `-C target-feature=+avx512f,+avx512dq` | 541 | 23 | 28 | 70 | auto-vectorized |
| C (no flags) | (none) | 180 | 0 | 0 | 40 | true scalar |

## Conclusions (machine-code-verified on the probe platform, rustc 1.99.1)
1. Under both documented flag sets, the scalar-source lane is **compiler-auto-vectorized** (AVX-512/AVX2 vector registers present). It is not scalar machine code.
2. Only a build with no AVX-512 target features produces true scalar machine code.
3. Therefore the canonical 1.27× (Zen 5) is best read as: **hand-written AVX-512 kernel vs the compiler's auto-vectorization of the same operation**. The ~7×–9× regime (this run: 6.99×; earlier single-size Zen 5 experiment on the evidence branch: 8.67×) is the hand kernel vs true scalar code.

## Boundary
The probe establishes codegen on the probe platform and toolchain. Zen 5 codegen was not directly disassembled here; the canonical logs' scalar ≈ avx2 lane equality at every size (in several cases identical to the nanosecond) is consistent with the same auto-vectorization on Zen 5, and is recorded as corroborating observation, not independent verification of Zen 5 codegen.

## Addendum — lane 2 (`avx2_butterfly_pass`) probe, same date

The same probe method was applied to lane 2 (identical loop shape, with the `#[target_feature(enable = "avx2")]` hint):

| Config | RUSTFLAGS | insn count | zmm | ymm | Codegen |
|---|---|---|---|---|---|
| A (canonical) | `-C target-cpu=native -C target-feature=+avx512f,+avx512dq,+avx2` | 602 | 16 | 29 | auto-vectorized (AVX-512 codegen) |
| B (quick-start, pre-clarification) | `-C target-feature=+avx512f,+avx512dq` | 613 | 23 | 28 | auto-vectorized (AVX-512 codegen) |
| C (no flags) | (none) | 345 | 0 | 26 | AVX2 codegen (from the target-feature hint alone) |

Conclusion: lanes 1 and 2 compile to near-identical auto-vectorized machine code under both documented flag sets, which explains the ~1.0× AVX2-vs-Scalar ratio in the canonical runs. With no flags, lane 1 is true scalar and lane 2 is AVX2-vectorized. Both probes are tied to the exact probe source, flags, compiler (rustc 1.99.1), and platform; they do not assert anything about scalar or AVX2 execution paths elsewhere in the codebase.
