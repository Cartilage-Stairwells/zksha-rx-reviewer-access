# review-v0.1.19 Release Note

## Purpose

External audit of review-v0.1.18 (2026-10-23) verified the release identity and count correction, and found two open items: (1) a documentation mismatch in the benchmark instructions — the canonical 1.27× claim is reproduced by `cargo bench --bench three_lane_bench`, but the quickstart, release doc, and BENCHMARKS.md pointed reviewers at `cargo bench --bench butterfly_bench`, which measures the scalar reference butterfly only; (2) the "135 staged pairwise comparisons" claim left REPORTED / UNADJUDICATED. review-v0.1.19 closes both. Reproducibility-mapping and count-basis wording only; no measured value, benchmark result, or proof content is modified.

## Disposition of audit items

| Audit item | Disposition at v0.1.19 |
|---|---|
| Release tag / target commit / count reconciliation | VERIFIED at v0.1.18 — unchanged; the 139 = 102 integration + 37 unit record is preserved |
| Canonical benchmark command mismatch | RESOLVED — `three_lane_bench` consistently identified as the command that reproduces the canonical 1.27× result (quickstart, REVIEW_RELEASE, BENCHMARKS); `butterfly_bench` labeled as scalar-reference-only with an explicit note that it does not reproduce the canonical result. Both targets compile-verified (`cargo bench --no-run`, 2026-10-23) |
| 135 staged comparisons UNADJUDICATED | ADJUDICATED — counting basis derived from source and verified by live re-run: `tests/ntt_equivalence.rs::staged_cross_backend_equivalence` stages log_n 5–10 (stages per size = log_n; total 45 stages) across 3 backends = 135 staged pairwise relations — 90 asserted directly (ref==scalar, ref==AVX-512 per stage), 45 established by transitivity. Independent re-run 2026-10-23 on AVX-512 hardware: 45/45 stage logs captured (`ref==scalar==avx512 PASS`), test passed |
| Historical measurement rows | PRESERVED — dated 2026-08-03 rows and historical release records are unaltered |
| Checksum seal | SHA256SUMS regenerated over all tracked files except itself; gate re-verified from a clean clone of this tag |

## Verification at seal

- `./validate_release.sh` passes 7/7 at this tag (from a clean clone)
- `cargo bench --bench three_lane_bench --no-run` and `cargo bench --bench butterfly_bench --no-run` both compile (2026-10-23, rustc 1.99.1)
- Staged-comparison capture: 45 `ref==scalar==avx512 PASS` lines; evidence log preserved (2026-10-23)

## Reviewer impact

Clone with `--branch review-v0.1.19` and run `./validate_release.sh`. To reproduce the canonical benchmark claim, run `cargo bench --bench three_lane_bench` (AVX-512 required). All prior review tags remain immutable and available for historical comparison.
