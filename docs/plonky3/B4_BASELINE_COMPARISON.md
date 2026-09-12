# B4 — Baseline Comparison (B4-K kernel/DFT-level, B4-P prover-level)

**Date:** 2026-09-12
**Branch:** feat/b4-baseline-comparison (from a03c556e, frozen B2 tip)
**Status:** authorized scope; results recorded in the results commit

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
