# Experiment C — FRI Sub-Component Profiling (Observational)

## Evidence Identity: experiment_c_profile_v1
## Date: 2026-08-18
## Type: Observational profiling (no algorithm change)
## SHA-256: d0bfdc44d7b73dbf123d40e551d27da924e8e50789f82b044b08443db319e7e9

## Hardware
- CPU family: 191, model: 2
- AVX-512: present (avx512f, avx512dq, etc.)
- Compilation: RUSTFLAGS="-C target-cpu=x86-64"
- NOTE: Different CPU from Experiment A (family 175/model 17).
  Absolute times are NOT comparable across experiments.
  Percentage breakdowns are the primary evidence.

## Methodology
- Added timing instrumentation to fri_prove (observational only, no algorithm change)
- Instrumented: MerkleTree::build, fri_fold_step, fri_query_round, challenger operations
- 3 iterations per size, sizes 2^8 through 2^14, 20 queries per prove
- Uses std::time::Instant for sub-component timing
- All sub-component timings sum to total (no double-counting)
- Original repository state restored after profiling — no algorithmic modification remains

## Results (Median of 3 iterations)

| Size | Merkle % | Fold % | Query % | Challenger % | Total (ms) |
|------|----------|--------|---------|-------------|------------|
| 2^8  | 91.3%    | 2.5%   | 5.2%    | 0.9%        | 0.971      |
| 2^10 | 93.9%    | 4.0%   | 1.8%    | 0.3%        | 3.830      |
| 2^12 | 94.6%    | 4.3%   | 1.1%    | 0.1%        | 15.217     |
| 2^14 | 95.1%    | 4.4%   | 0.4%    | 0.0%        | 60.985     |

## Key Findings

1. **MerkleTree::build (Poseidon2 hashing) dominated the measured FRI proving workload**
   - Observed range: 91.3–95.1% of FRI proving time (across 4 profiled sizes)
   - Trend: dominance INCREASES with size (91.3% at 2^8 → 95.1% at 2^14)

2. **fri_fold_step is a minor cost**
   - Range: 2.5–4.4% of FRI proving time
   - Stable at ~4% for sizes ≥ 2^10

3. **fri_query_round is negligible at larger sizes**
   - Range: 0.4–5.2% (decreasing with size)
   - <1% at 2^12 and above

4. **Challenger operations are negligible**
   - <0.1% at sizes ≥ 2^12

## Architectural Implications

Given the profiling data:

| Optimization Target | FRI Time Fraction | Potential Impact |
|---------------------|------------------|-----------------|
| Poseidon2/Merkle SIMD (Option C) | 91.3–95.1% | HIGH — dominant cost |
| FRI fold SIMD (Option B) | 2.5–4.4% | LOW — minor fraction |
| NTT integration (Option A) | 0% currently | UNKNOWN — requires architectural change |

The data supports pursuing Option C (Poseidon2/Merkle vectorization) as the
highest-impact SIMD optimization target for the production prover.

Option B (FRI fold SIMD) would address only ~4% of FRI time. Even a 10×
speedup of the fold step would yield <1% end-to-end improvement.

Option A (NTT integration) remains a research question — the prover does not
currently use NTT, and introducing it would require justifying why NTT-based
evaluation is preferable to the current approach.

## What This Profiling Does NOT Establish

- Poseidon2/Merkle is definitively the end-to-end prover bottleneck (Experiment C profiles fri_prove, NOT the complete proving pipeline)
- Absolute proving times (different CPU from firebird evidence)
- Whether Poseidon2 SIMD is feasible or how much speedup it would provide
- Whether the prover's algorithm should be changed to use NTT
- Performance on Ice Lake-SP or other hardware
- Results beyond the 4 profiled sizes (2^8, 2^10, 2^12, 2^14)

## Defensible Statement

> Profiling of the production FRI proving path on AMD Zen 5 (family 191) with
> target-cpu=x86-64 shows that MerkleTree::build (Poseidon2 hashing) accounts
> for 91.3–95.1% of measured FRI proving time across sizes 2^8 through 2^14,
> with dominance increasing at larger sizes. FRI fold operations account for
> approximately 4%. The NTT/FFT butterfly implementation is not invoked.
> Poseidon2/Merkle construction dominated the measured FRI proving workload;
> whether this extends to the complete proving pipeline requires additional
> profiling beyond fri_prove.
