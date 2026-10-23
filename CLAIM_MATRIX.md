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
| 4 | Backend equivalence | 3 backends: reference oracle, scalar Montgomery bridge, AVX-512 SIMD | `tests/backend_parity/`, `tests/ntt_equivalence.rs` (135 staged comparisons: 3 backends × 45 butterfly stages (log_n 5–10, stages per size = log_n); 90 asserted directly in tests/ntt_equivalence.rs, 45 by transitivity; executed and passed on AVX-512 hardware in the independent run 2026-10-23) | SIMD is tested (three-lane correctness gate), not formally verified |
| 5 | Independent oracle validation | Reference DFT compared against SIMD output | `tests/ntt_equivalence.rs` | Oracle checks correctness of all tested inputs |
| 6 | Formal verification of Montgomery arithmetic | Lean 4: Montgomery multiplication, residue closure, Bézout identity | `formal/README.md`, external Lean repo | Formal verification of SIMD implementation |
| 7 | Reproducible benchmark artifacts | SHA256SUMS, evidence manifest, environment config | `evidence/`, `SHA256SUMS` | Results generalize to all AVX-512 CPUs |

## Reproducibility Status

| Claim | Reproducible from this snapshot? | Notes |
|-------|----------------------------------|-------|
| 1.27× DIF butterfly (canonical) | Yes — three-lane bench (`cargo bench --bench three_lane_bench`) | Canonical: 50 samples, 2s measurement, dual-run verified (0.9% variance). Measured with Criterion on AMD Zen 5 with AVX-512. DIF butterfly, matching Plonky3 DifButterfly. Historical 2.65× was a 10-sample run superseded by canonical protocol. Baseline (clarified 2026-10-23, disassembly-verified): the scalar-source lane compiles to auto-vectorized AVX-512/AVX2 code under documented flags — the canonical ratio is hand kernel vs compiler auto-vectorization on Zen 5; vs true scalar the kernel measures ~7×–9× (evidence/independent/2026-10-23-non-zen5/). |
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
| Test count | 139 tests run and passed at this tag (141 textual `#[test]` declarations). **Counting basis (B-2/EXC-012, stated 2026-09-07; corrected to measured values 2026-10-23):** mechanical census of `#[test]` attributes in the tree as of this commit — 104 in `tests/` (6 in the archived `tests/archive/legacy_montgomery_regression.rs`, which runs as its own test target; 2 occurrences are commented-out code, leaving 96 live + 6 archived = 102 compiled) + 37 unit tests in `src/` = 141 textual / 139 compiled and run across the tree. Independent run 2026-10-23 at review-v0.1.17 (AVX-512 CPU, rustc 1.99.1, default profile): 12 test targets, 139 tests run, 139 passed, 0 failed, 0 ignored. The README's '102 tests' figure is exact for the `tests/` integration suite (102/102 pass); the prior "140 total" was an unstated-basis approximation, corrected to the measured 139 at review-v0.1.18. Zero `#[ignore]`. |

## Important Distinctions

- **Kernel acceleration ≠ end-to-end acceleration.** The 9.15× and 4.58× are isolated butterfly kernel measurements. The 3.94× is a full NTT sweep. None of these imply proportional speedup of a complete proving system.
- **Formal verification covers arithmetic, not SIMD.** The Lean 4 proofs verify Montgomery multiplication properties. The SIMD implementation is validated by differential testing and backend equivalence, not by formal proof.
- **Reproducibility is hardware-specific.** Benchmark numbers were measured on specific AVX-512 hardware. Results on different microarchitectures will differ.

---

## Evidence artifacts — deterministic location pointer (A-9(2) / B-6, 2026-09-07)

Benchmark evidence for this repository is branch-resident, not main-tree:

- **Branch:** `evidence/experiment-a` (head `8df0c247cc503b751ee91245bb952b6994aaf2e6`)
- **Path:** `evidence/experiment_a/` — SHA256SUMS-pinned artifacts
- **Independent resolution (from any clone):**
  ```
  git fetch origin evidence/experiment-a
  git checkout 8df0c247cc503b751ee91245bb952b6994aaf2e6
  sha256sum -c evidence/experiment_a/SHA256SUMS
  ```
