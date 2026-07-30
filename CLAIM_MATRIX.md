# Claim Matrix — zkSHA-Rx Fly v0.1.0

> Every external claim, its measurement, its evidence, and its non-claim.
> All numbers trace to `PROJECT_FACTS.md`.

## Performance Claims

| # | Claim | Scope | Result | Evidence | Non-Claim |
|---|-------|-------|--------|----------|-----------|
| 1 | AVX-512 DIT butterfly kernel speedup | Isolated kernel, Criterion 0.5.1 | 9.15× | `BENCHMARKS.md` | End-to-end proving speedup |
| 2 | AVX-512 XOR butterfly kernel speedup | Isolated kernel, Criterion 0.5.1 | 4.58× | `BENCHMARKS.md` | End-to-end proving speedup |
| 3 | Full NTT sweep speedup | Sizes 2⁸–2²⁰, perf_measure | 3.94× geo mean | `MEASUREMENT.md`, `evidence/` | Kernel speedup ≠ prover speedup |

## Correctness Claims

| # | Claim | Scope | Evidence | Non-Claim |
|---|-------|-------|----------|-----------|
| 4 | Backend equivalence | 3 backends: reference oracle, scalar Montgomery bridge, AVX-512 SIMD | `tests/backend_parity/` (135 staged comparisons) | SIMD is formally verified |
| 5 | Independent oracle validation | Reference DFT compared against SIMD output | `tests/ntt_equivalence.rs` | Oracle proves correctness of all inputs |
| 6 | Formal verification of Montgomery arithmetic | Lean 4: Montgomery multiplication, residue closure, Bézout identity | `formal/README.md`, external Lean repo | Formal verification of SIMD implementation |
| 7 | Reproducible benchmark artifacts | SHA256SUMS, evidence manifest, environment config | `evidence/`, `SHA256SUMS` | Results generalize to all AVX-512 CPUs |

## Methodology

| Aspect | Value |
|--------|-------|
| Benchmark framework | Criterion 0.5.1 |
| Samples | 100 per measurement |
| Warmup | 3 seconds |
| Outlier rejection | Enabled |
| Full NTT framework | perf_measure, 1000 iterations/size |
| Test count | 140 total (all passing) |

## Important Distinctions

- **Kernel acceleration ≠ end-to-end acceleration.** The 9.15× and 4.58× are isolated butterfly kernel measurements. The 3.94× is a full NTT sweep. None of these imply proportional speedup of a complete proving system.
- **Formal verification covers arithmetic, not SIMD.** The Lean 4 proofs verify Montgomery multiplication properties. The SIMD implementation is validated by differential testing and backend equivalence, not by formal proof.
- **Reproducibility is hardware-specific.** Benchmark numbers were measured on specific AVX-512 hardware. Results on different microarchitectures will differ.
