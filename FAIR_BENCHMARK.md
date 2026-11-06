# FAIR_BENCHMARK.md — The Controlled Scalar-vs-SIMD Measurement

**Status:** Measured, triple-run verified, bit-exactness gated. Result JSONs in `evidence/fair_bench_20260907/`.
**Environment:** `evidence/fair_bench_20260907/environment.json` (AMD family 191, AVX-512F+DQ, gvisor sandbox, rustc 1.97.1, commit in file)
**Companion harness:** `benches/fair_lane_bench.rs` (4 lanes) and `benches/kernel_ab_bench.rs` (kernel A/B)

---

## 1. Why this benchmark exists

The historical three-lane benchmark compares the AVX-512 kernel against the
u128 reference oracle (`babybear_mul_reference`). The oracle is a
**correctness reference**, deliberately simple:

```
(a as u128 * b as u128 * R_INV) % p
```

Its cost relative to any fast implementation varies by microarchitecture and
sandbox state, so a kernel-vs-oracle ratio is not a stable property of the
kernel. Independent measurements of that ratio on identical hardware produced
4.68× and 9.74× one month apart — 108% apart, failing the 15% dual-run
consistency criterion in CANONICAL_BENCHMARK_METHODOLOGY.md.

The fair benchmark replaces the oracle lane's role as *performance* baseline
with the project's own optimized scalar CIOS implementation
(`ScalarBackend::mul_raw`, the verified CIOS reduction) while keeping the
oracle in the suite for its actual job: the correctness gate.

## 2. Design

Four lanes, identical loop structure, identical mod-p add/sub, only the
multiply differs:

| Lane | Multiply | Role |
|---|---|---|
| `oracle_u128` | `babybear_mul_reference` | Correctness reference only |
| `scalar_cios` | `ScalarBackend::mul_raw` (CIOS) | **Fair performance baseline** |
| `avx2` | compiler, `#[target_feature(avx2)]` | Continuity with three_lane_bench |
| `avx512` | hand-written kernel | Candidate |

- Criterion config identical to CANONICAL_BENCHMARK_METHODOLOGY.md
  (50 samples, 500ms warmup, 2s measurement).
- Correctness gate before timing: all four lanes bit-identical on 2^8–2^20.
- **Two build modes are meaningful** (both documented, both measured):
  - *Baseline build* (no `target-cpu=native`): scalar CIOS compiles to true
    scalar code; the AVX-512 lane still executes real intrinsics via its
    `#[target_feature]` attributes. This measures SIMD vs scalar.
  - *Native build* (`-C target-cpu=native`): LLVM auto-vectorizes the CIOS
    loop itself with AVX-512 (`zmm`, `vpmuludq` — confirmed by disassembly).
    This measures the hand-written kernel against the compiler's own SIMD.
- The scalar CIOS lane is the project's own verified implementation — nothing
  new was written for the baseline; only the benchmark wiring.

## 3. Results — baseline build (SIMD vs true scalar)

| Run | Geometric mean (AVX-512 / scalar CIOS), 2^8–2^20 |
|---|---|
| Run 1 | 2.98× |
| Run 2 | 2.92× |
| Run 3 | 2.97× |
| **Spread** | **1.9% — passes the 15% consistency criterion** |

Per-size (run 1): 2.67× (2^8) → 3.16× (2^16–2^18) → 2.79× (2^20).

## 4. Results — native build (hand-written vs compiler-vectorized)

Hand-written kernel vs compiler's own AVX-512 auto-vectorization of the same
CIOS arithmetic: **1.12×** geometric mean. This is the number that justifies
hand-written intrinsics: the compiler *can* vectorize this pattern with
native codegen, and the hand kernel still beats it.

## 5. Decomposition of the historical oracle-relative ratio

On the same machine, same session (baseline build, run 1):

| Comparison | Geometric mean | Meaning |
|---|---|---|
| AVX-512 vs scalar CIOS | 2.98× | Real engineering improvement |
| scalar CIOS vs u128 oracle | 3.13× | Cost of the oracle (the confound) |
| AVX-512 vs u128 oracle | 9.32× | What the oracle-relative benchmark measured |

Check: 2.98 × 3.13 = 9.32 — the historical ratio decomposes multiplicatively
into the real improvement and the oracle's cost. The instability of the
historical number (4.68× → 9.74× across days) tracks the confound term, not
the kernel.

## 6. Recommended canonical claim

> The hand-written AVX-512 BabyBear DIF butterfly kernel is **~3× faster than
> the project's verified scalar CIOS implementation** (geometric mean over
> stages 2^8–2^20; 2.92–2.98× across three runs, 1.9% spread, passing the
> dual-run consistency criterion) and **12% faster than the compiler's own
> AVX-512 auto-vectorization** of the same arithmetic on the reference test
> system. Oracle-relative ratios measure the cost of the u128 correctness
> oracle and are unsuitable as performance claims; the oracle remains in the
> suite as the correctness gate.

Hardware caveat: single test system (AMD family 191, virtualized). The fair
ratio was stable to ~2% here, so it is expected to transfer far better than
the oracle-relative ratio ever did; bare-metal confirmation recommended.

## 7. Kernel A/B and optimization log (honest negative results)

Two optimization candidates were implemented, gated bit-exact against the
original kernel by `tests/kernel_opt_equivalence.rs` (all sizes 2^8–2^20,
non-multiple-of-16 tails, boundary values, 20 random rounds), and measured
with the same Criterion config:

| Candidate | Change | Geometric mean vs original |
|---|---|---|
| `avx512_butterfly_pass_32_opt` | even/odd interleave: 7 ops → 2 ops (`inserti32x8` + `vpermd`) | **0.929× — slower** |
| `avx512_butterfly_pass_32_unroll` | 2× unroll, original arithmetic | **0.940× — slower** (1.067× at 2^20 only) |

Both candidates are **retained in `src/avx512_butterfly_opt.rs` as
experiments, not wired into any public API path** (only the A/B bench and the
equivalence test reference them).

Conclusion: the original kernel's unpack-chain interleave overlaps better
than a single `vpermd` on this microarchitecture, and the out-of-order engine
already extracts the ILP that explicit unrolling targets. **The original
kernel survived an adversarial optimization pass by an external reviewer —
its ~3× fair advantage is the practical ceiling for this algorithm class on
this hardware** (no AVX-512 IFMA52 available here; the even/odd product split
is required without it).

## 8. Reproduction

```
# Baseline build (true scalar CIOS vs SIMD):
cargo bench --bench fair_lane_bench

# Native build (hand-written vs compiler-vectorized):
RUSTFLAGS="-C target-cpu=native" cargo bench --bench fair_lane_bench

# Kernel A/B (bit-exact gated):
cargo bench --bench kernel_ab_bench

# Admission gate for any future kernel change:
cargo test --release --test kernel_opt_equivalence
```

Results artifacts: `evidence/fair_bench_20260907/*.json` (seven runs across
the two benches and two build modes).
