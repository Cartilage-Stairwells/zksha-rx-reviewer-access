# B5 — Twiddle Amortization (Adapter-Boundary Attribution Recovery)

> Experiment question: **How much of the 1.70x adapter-boundary attribution
> is recoverable by amortizing twiddle generation?**
> Branch: `feat/b5-twiddle-amortization`, derived from preserved B4 tip
> `a9055a74`. B4 remains CLOSED_FROZEN; B1/B2 evidence untouched.

---

## Change (the entire diff)

`src/plonky3_adapter.rs` only:
- `compute_twiddles(log_n)` now returns `Arc<Vec<Vec<u32>>>` from a
  `thread_local` cache keyed by `log_n` (twiddles depend only on `log_n` and
  the fixed two-adic generator).
- The uncached path is preserved verbatim as `compute_twiddles_uncached`.
- A `clear_twiddle_cache()` helper was added (used by measurement examples;
  the committed bench harness is byte-identical to B4's and calls nothing new).
- Incidental: a malformed, duplicated doc-comment block in B4's committed
  version (a stray "B2: shared tracker" paragraph inserted mid-sentence) was
  removed. Cosmetic only; zero code effect. Disclosed rather than reverted.

No other file was modified. All other tracked files verified byte-identical
to B4 tip `a9055a74`. New measurement instruments (this branch only):
`examples/measure_twiddles.rs`, `examples/measure_cold_vs_steady.rs`,
`compare_before_b4.py`.

## Method

- Config S: stable rustc 1.97.1 (8bab26f4f), no RUSTFLAGS, Criterion 0.5,
  unmodified B4-K bench harness (`benches/b4k_lanes.rs`, byte-identical).
- Same-day before/after pair on the same branch: before = unmodified B5
  branch point (identical to B4 tip), after = cache diff applied. Archived
  B4 numbers remain the baseline of record; the same-day pair is the primary
  controlled comparison.
- Gates: `assert_identical` bit-identity vs the reference backend runs inside
  the bench before timing every lane/size point. The after-run completed all
  78 points, so every timed point passed its gate. Full test suite: 31/31
  PASS (re-verified 2026-09-27 on the restored instance).
- All 78 points measured before AND after (w=1 and w=4, 2^8–2^20, 3 lanes),
  with Criterion 95% CIs, in `b5_results_consolidated.json`.

## Results (w=1, 2^20, medians)

| Lane | Before | After | Delta |
|---|---|---|---|
| zksha_avx512 | 14.045 ms ±0.16% | 11.571 ms ±0.44% | **−2.474 ms (−17.6%)** |
| zksha_scalar | 25.189 ms ±4.46% | 21.211 ms ±0.79% | **−3.978 ms (−15.8%)** |
| plonky3_radix2dit (control) | 38.164 ms ±0.85% | 38.428 ms ±0.22% | +0.264 ms (+0.7%, noise) |

- The control lane (Plonky3's own DFT, no shared code) is flat: the measured
  effect is isolated to our adapter's per-call twiddle path.
- **Attribution ratio (scalar/avx512): 1.79x → 1.83x** (same-day).
  Against archived B4 (1.70x): the ratio recovers only ~2% relative.
- vs Plonky3 Radix2Dit at 2^20 w1: **2.72x → 3.32x faster** (our lane
  accelerates; theirs does not).

## w=4 (2^20)

zksha_avx512 −1.251 ms (−2.3%), zksha_scalar +0.475 ms, plonky3 control
+3.650 ms (+5.1%). The control lane's own drift (±5–7% CI on these runs)
exceeds every w=4 delta. **No w=4 claim is made**; w=4 effects are within
run-to-run noise on this instance.

## Direct twiddle measurement (T_direct) — disclosed disagreement

`examples/measure_twiddles.rs` (standalone, non-Criterion, 50 runs):
log_n=20 median 10.614 ms (min 7.776, max 26.421 — high variance).

The controlled before/after delta is 2.474 ms (avx lane, tight CIs).
T_direct disagrees with T_delta by ~4x. The cold-vs-steady example
(cold 22.748 ms, steady 17.030 ms, example-internal delta 5.7 ms) also does
not reconcile with the Criterion absolutes. Per the experiment's rule 10,
this disagreement is reported, not resolved: the standalone measurement does
not represent the in-bench cost of the same computation (allocator state,
page warmth, and its own 3.4x min-max spread). Consequently:

- **SUPPORTED (controlled, gated, tight-CI):** the cache removes
  ~2.5 ms (avx lane) / ~4.0 ms (scalar lane) per call at 2^20 w1 — about
  16–18% of the adapter dft_batch cost — moving the attribution ratio only
  1.79x → 1.83x.
- **UNESTABLISHED:** the precise causal split between twiddle *computation*
  and twiddle *allocation* within that recovered time, and any precise
  percentage attribution of the standalone-measured twiddle cost.

## Answer to the B5 question

**Only a small portion.** Twiddle amortization recovers ~2.5–4 ms per call
(~16–18% of the adapter boundary at 2^20 w1), but because it benefits the
scalar lane comparably, the avx512:scalar attribution ratio barely moves
(1.79x → 1.83x same-day; archived B4 1.70x → 1.83x). The gap to the
kernel-level 7.23x is **not** recoverable via twiddles. The dominant
adapter-boundary dilution remains **unidentified** in this experiment:
bit-reversal, per-column loop overhead, strided data movement (w>1), and
residual allocation costs. The experiment does not license any claim about
those components beyond naming them as unmeasured.

## Prover sanity (non-claim)

Single after-state run, deterministic Fibonacci STARK fixture, 2^14:
zksha 505.44 ms vs plonky3 510.63 ms (arms within ~1%). Consistent with
B4's finding that the DFT backend is nearly invisible end-to-end at this
fixture scale. No end-to-end claim is made.

## Claim boundaries

- Claimed: measured adapter-boundary (dft_batch) steady-state improvement
  from twiddle caching, config S, this fixture, all lanes reported.
- The after-state timing is the amortized steady state (cache warm); cold
  first-call cost is higher (approx. +6–10 ms per the non-Criterion example,
  disclosed as approximate).
- Not claimed: energy, end-to-end/prover acceleration, kernel-level ratios,
  w=4 effects, cross-toolchain comparisons, or a causal decomposition of the
  recovered time into computation vs allocation.

## Evidence

- Raw Criterion estimates: `/app/b5-results/{before,after,sanity_b4p}/`
- Consolidated: `docs/plonky3/b5_results_consolidated.json`
- Environment record: `docs/plonky3/b5_environment.json`
- Note: before/after runs executed 2026-09-17 on a prior sandbox instance;
  the workspace (including raw estimates and the adapter diff) was restored
  to a recycled instance on 2026-09-27, at which point the diff was
  re-verified against B4 tip `a9055a74` and the 31/31 test suite re-run.

## Custody

- B4 (`feat/b4-baseline-comparison` @ `a9055a74`): untouched, re-verified.
- B2 (`feat/b1-integration-boundary` @ `a03c556e`): untouched.
- `main`: observed at `b17b4127` (owner's own docs-only adjudication commit
  of 2026-09-17, outside this experiment; the B4 freeze point `ac90de5d`
  remains the experiment's recorded parent context).
- No merge, no tag, no release, no push to main.
