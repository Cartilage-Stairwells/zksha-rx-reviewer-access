# B4 — Baseline Comparison (B4-K kernel/DFT-level, B4-P prover-level)

**Date:** 2026-09-12
**Branch:** feat/b4-baseline-comparison (from a03c556e, frozen B2 tip)
**Status:** RESULTS RECORDED (2026-09-12). 252 measured points; all correctness gates passed; full data in b4_results_consolidated.json, environment in b4_environment.json

## Scope (authorized 2026-09-12)

Two separately-conceived measurements. Neither is a production-prover
benchmark; no extrapolation beyond the measured fixture is permitted.

- **B4-K** — kernel/DFT comparative performance: isolates the adapter/backend
  contribution at the `dft_batch` boundary.
- **B4-P** — end-to-end performance of this deterministic Fibonacci STARK
  fixture at 2^10/2^12/2^14 trace rows, with all other proving costs included.

## Lanes and build configurations

BabyBear's `Packing` in Plonky3 0.1.0 is compile-time cfg-selected, so each
configuration is a separate build/run. Differing toolchains do NOT constitute
a controlled single-variable experiment across configurations — every lane
is reported with its exact toolchain and target features, and cross-
configuration comparisons are disclosed as such.

| Lane | Toolchain | Target features | Role |
|---|---|---|---|
| zksha_avx512 | stable 1.97.1 | none (runtime detection) | subject |
| zksha_scalar | stable 1.97.1 | none | attribution baseline |
| plonky3_radix2dit (scalar packing) | stable 1.97.1 | none | oracle |
| plonky3_radix2dit (AVX2 packing) | stable 1.97.1 | +avx2 | oracle, separate build |
| plonky3_radix2dit (AVX-512 packing) | nightly 1.100.0 | +avx512f,+avx512dq + nightly-features | oracle, separate toolchain |

Every timed lane passes a bit-identical correctness gate against the
reference backend at every size before it is timed. Any gate failure aborts
the run. Criterion 0.5 statistics; CIs recorded. No energy measurements
(gvisor sandbox: no RAPL/MSR access — established limitation).

## Methodology

- Criterion 0.5, sample_size 30 (B4-K) / 15 (B4-P), CIs from estimates.json
- B4-K sizes 2^8–2^20, shapes: single-column (w=1) and prover batch (w=4)
- B4-P: prove() wall-time, fresh challenger+trace per sample, proof verified
  and byte-identity-checked between arms before timing
- B4-P is measured in the bench (release) profile: p3-uni-stark's debug-only
  check_constraints is compiled out — the timed path is the real prover
- Deterministic inputs throughout (no thread_rng)

## Run commands (the official runs)

```
# Config S (scalar packing)
CRITERION_HOME=<results>/config-scalar cargo bench --bench b4k_lanes
CRITERION_HOME=<results>/config-scalar cargo bench --bench b4p_prover

# Config A (AVX2 packing)
RUSTFLAGS="-C target-feature=+avx2" CRITERION_HOME=<results>/config-avx2 \
  cargo bench --bench b4k_lanes
RUSTFLAGS="-C target-feature=+avx2" CRITERION_HOME=<results>/config-avx2 \
  cargo bench --bench b4p_prover

# Config N (AVX-512 packing, nightly toolchain)
RUSTFLAGS="-C target-feature=+avx512f,+avx512dq" \
  CRITERION_HOME=<results>/config-avx512-nightly \
  cargo +nightly bench --features nightly-features --bench b4k_lanes
RUSTFLAGS="-C target-feature=+avx512f,+avx512dq" \
  CRITERION_HOME=<results>/config-avx512-nightly \
  cargo +nightly bench --features nightly-features --bench b4p_prover
```

## Claim boundaries

- Reported: measured comparative performance under each disclosed build
  configuration, with CIs, all lanes reported regardless of direction.
- Not claimed: production-prover performance, energy, superiority across
  toolchain boundaries as a single controlled variable, or anything beyond
  this deterministic fixture.
- B1/B2 artifacts untouched; frozen B2 64-row fixture is NOT a timing
  subject (it is the integration/equivalence gate); main untouched.


## Results (2026-09-12)

All 252 points measured; every correctness gate passed before its point was
timed (bit-identity vs reference for every B4-K lane/size; proof verification,
tracker engagement, and proof-output byte identity for every B4-P arm/size —
byte identity held at 2^10, 2^12, and 2^14, extending the frozen B2 finding).
CIs are 95% confidence intervals on the Criterion median.

### B4-K — dft_batch, single-column (w=1), median ± half-CI

| Lane | Config S (stable, scalar pkg) | Config A (stable, +avx2) | Config N (nightly, +avx512f) |
|---|---|---|---|
| zksha_avx512 | 15.097 ms ±0.03% | 15.223 ms ±0.06% | 26.893 ms ±0.21% |
| zksha_scalar | 25.661 ms ±0.06% | 21.725 ms ±0.04% | 21.411 ms ±0.06% |
| plonky3_radix2dit | 40.416 ms ±0.06% | 41.374 ms ±0.03% | 41.615 ms ±0.06% |

Ratios (medians, same-config comparisons):
- zksha_avx512 vs plonky3_radix2dit: **2.68× (S)**, 2.72× (A), 1.55× (N)
- zksha_avx512 vs zksha_scalar: **1.70× (S)**, 1.43× (A), **0.80× (N — see anomaly)**
- plonky3 packing is idle at w=1 (nothing to pack), hence near-identical
  across S/A/N — consistent with the compile-time packing selection.

### B4-K — dft_batch, prover batch (w=4), median ± half-CI

| Lane | Config S | Config A | Config N |
|---|---|---|---|
| zksha_avx512 | 56.852 ms ±0.19% | 55.198 ms ±0.15% | 110.024 ms ±1.06% |
| zksha_scalar | 98.460 ms ±0.16% | 80.571 ms ±0.19% | 85.549 ms ±1.07% |
| plonky3_radix2dit | 73.930 ms ±0.38% | 98.612 ms ±0.28% | 95.582 ms ±1.06% |

Ratios at 2^20:
- zksha_avx512 vs plonky3_radix2dit: **1.30× (S)**, 1.79× (A), 0.87× (N)
- Plonky3's own packing HURTS at w=4 (S 73.9 ms is their fastest w=4
  configuration; A/N are slower at 4/8 and 4/16 lane utilization).
- Crossover: zksha_avx512 is slower than plonky3_radix2dit below ~2^14 at
  w=4 and below ~2^12 at w=1 (fixed per-call overheads dominate small sizes).
  Full per-size data in the consolidated JSON.

### Attribution analysis (verified from source, not inferred)

The kernel-level 7.23× (Experiment A, butterfly stage) does NOT survive to
the adapter boundary: full dft_batch shows 1.70× avx512/scalar at 2^20 w=1
(config S). Verified mechanism — per `src/plonky3_adapter.rs`:
- `compute_twiddles` runs on every call (scalar BabyBear Montgomery
  arithmetic per twiddle; ~n scalar mults per call)
- `bit_reverse` runs per column
- multi-column shapes pay strided gather/scatter twice per column
- the AVX-512 kernel accelerates only the butterfly-stage portion

Consequence: the adapter's per-call twiddle computation is the dominant
dilution source at large sizes. This is an OBSERVATION, not an optimization
claim — twiddle caching/amortization was NOT performed (not authorized by
the B4 scope).

### B4-P — prove() wall-time, median ± half-CI

| Size | Arm | Config S | Config A | Config N |
|---|---|---|---|---|
| 2^10 | zksha_avx512 | 34.95 ms ±0.32% | 14.45 ms ±0.25% | 10.43 ms ±0.27% |
| 2^10 | plonky3_radix2dit | 34.69 ms ±0.01% | 14.35 ms ±0.06% | 9.88 ms ±0.10% |
| 2^12 | zksha_avx512 | 138.46 ms ±0.08% | 56.60 ms ±0.03% | 40.53 ms ±0.35% |
| 2^12 | plonky3_radix2dit | 138.55 ms ±0.04% | 57.02 ms ±0.05% | 39.16 ms ±0.22% |
| 2^14 | zksha_avx512 | 553.27 ms ±0.08% | 225.35 ms ±0.77% | 161.36 ms ±0.37% |
| 2^14 | plonky3_radix2dit | 554.54 ms ±0.12% | 228.86 ms ±0.12% | 157.72 ms ±0.59% |

Findings:
- **The DFT backend is nearly invisible end-to-end at this fixture scale.**
  Adapter-vs-Radix2Dit delta: S: −0.2% to +0.7% (statistically indistinguishable);
  A: −1.5% (adapter faster) at 2^14; N: +2.3% (Radix2Dit faster) at 2^14.
  Merkle hashing and FRI dominate; they are identical infrastructure in both
  arms and scale with the config's packing, not with the DFT choice.
- **The dominant prover-side variable in this experiment is the MMCS
  packing** (Plonky3's `FieldMerkleTreeMmcs` uses `BabyBear::Packing`):
  prove() at 2^14 drops 553→225 ms (A) and 553→161 ms (N) for BOTH arms.
  This is a configuration effect of the host prover, not a property of the
  adapter, and is disclosed as such.
- No extrapolation: these numbers describe this deterministic Fibonacci
  STARK fixture only. Production-prover behavior is not claimed.

### Observed anomaly (disclosed, unexplained)

Under config N (nightly 1.100.0), the zksha AVX-512 lane is 1.78× SLOWER
than under config S (26.893 vs 15.097 ms at 2^20 w=1; same direction at
w=4), while the zksha scalar lane is ~17% FASTER under N. Source inspection
rules out a cfg-selected alternate code path (the kernel compiles from
identical source in all configs; `#[target_feature(enable=...)]` functions,
no `cfg(target_feature)` variant). The variance is attributed to nightly-
compiler codegen differences and is flagged for follow-up. The subject
lane's canonical record remains its documented project toolchain (stable
1.97.1, runtime detection — config S).

## Final claim boundaries

- Claimed: measured comparative performance of the named lanes under the
  named build configurations, with CIs, all lanes and sizes reported
  including unfavorable ones.
- Not claimed: cross-toolchain single-variable comparisons; production
  prover performance; energy; any optimization of the adapter (twiddle
  amortization is noted as observed future work, NOT performed).
- Continuity note: the 2026-07-23 comparison (p3-baby-bear 0.6.1, butterfly
  kernel level, found Plonky3 scalar 2.77× faster than ours) differs in
  crate version and measurement scope from today's dft_batch-level results
  (p3-dft 0.1.0); both stand as separate records under their own conditions.
