# Claim Matrix — zkSHA-Rx Fly v0.1.0

> Every external claim, its measurement, its evidence, and its non-claim.
> All numbers trace to `PROJECT_FACTS.md`.

## Performance Claims

| # | Claim | Scope | Result | Evidence | Non-Claim |
|---|-------|-------|--------|----------|-----------|
| 1 | AVX-512 DIF butterfly kernel speedup | Three-lane benchmark, Criterion (AMD Zen 5 with AVX-512, 50 samples, dual-run) | 1.265×–1.276× (geo mean, range 1.089×–1.551× per size) | `CANONICAL_RESULTS.md`, `evidence/canonical/` | End-to-end proving speedup |
| 2 | AVX-512 XOR butterfly kernel speedup | Isolated kernel, Criterion (historical 0.5.1; snapshot 0.8.2) | 4.58× | `BENCHMARKS.md` | End-to-end proving speedup |
| 3 | Full NTT sweep speedup | Sizes 2⁸–2²⁰, perf_measure | 3.94× geo mean | `MEASUREMENT.md`, `evidence/` | Kernel speedup ≠ prover speedup |

## Correctness Claims

| # | Claim | Scope | Evidence | Non-Claim |
|---|-------|-------|----------|-----------|
| 4 | Backend equivalence | 3 backends: reference oracle, scalar Montgomery bridge, AVX-512 SIMD | `tests/backend_parity/` (135 staged comparisons) | SIMD is tested (three-lane correctness gate), not formally verified |
| 5 | Independent oracle validation | Reference DFT compared against SIMD output | `tests/ntt_equivalence.rs` | Oracle checks correctness of all tested inputs |
| 6 | Formal verification of Montgomery arithmetic | Lean 4: Montgomery multiplication, residue closure, Bézout identity | `formal/README.md`, external Lean repo | Formal verification of SIMD implementation |
| 7 | Reproducible benchmark artifacts | SHA256SUMS, evidence manifest, environment config | `evidence/`, `SHA256SUMS` | Results generalize to all AVX-512 CPUs |

## Reproducibility Status

| Claim | Reproducible from this snapshot? | Notes |
|-------|----------------------------------|-------|
| 1.27× DIF butterfly (canonical) | Yes — three-lane bench (`cargo bench --bench three_lane_bench`) | Canonical: 50 samples, 2s measurement, dual-run verified (0.9% variance). Measured with Criterion on AMD Zen 5 with AVX-512. DIF butterfly, matching Plonky3 DifButterfly. Historical 2.65× was a 10-sample run superseded by canonical protocol. |
| 4.58× XOR butterfly | Partially — bench harness exists (`cargo bench`) | Same as above. |
| 3.94× full NTT sweep | No — perf_measure harness not in this snapshot | Historical result. Not reproducible from this checkout. |

## Methodology

| Aspect | Value |
|--------|-------|
| Benchmark framework | Criterion (historical 0.5.1; snapshot 0.8.2) |
| Samples | 100 per measurement |
| Warmup | 3 seconds |
| Outlier rejection | Enabled |
| Full NTT framework | perf_measure (historical), 1000 iterations/size |
| Test count | 141 total, all passing as recorded. **Counting basis (B-2/EXC-012, stated 2026-09-07):** mechanical census of `#[test]` attributes in the tree as of this commit — 104 in `tests/` (of which 6 live in the archived `tests/archive/legacy_montgomery_regression.rs`, leaving 98 in the live suite) + 37 unit tests in `src/` = 141 across the tree. The README's separate '102 tests' figure was verified against `tests/` at audit time (VT census); the current-tree census is recorded here so both counts carry stated bases. Zero `#[ignore]`. The prior "140 total" was an unstated-basis approximation. |

## Important Distinctions

- **Kernel acceleration ≠ end-to-end acceleration.** The 9.15× and 4.58× are isolated butterfly kernel measurements. The 3.94× is a full NTT sweep. None of these imply proportional speedup of a complete proving system.
- **Formal verification covers arithmetic, not SIMD.** The Lean 4 proofs verify Montgomery multiplication properties. The SIMD implementation is validated by differential testing and backend equivalence, not by formal proof.
- **Reproducibility is hardware-specific.** Benchmark numbers were measured on specific AVX-512 hardware. Results on different microarchitectures will differ.
