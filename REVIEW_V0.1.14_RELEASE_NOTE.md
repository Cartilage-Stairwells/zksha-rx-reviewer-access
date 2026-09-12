# review-v0.1.14 Release Note

## Purpose

This release re-seals reviewer-facing release integrity. The v0.1.10–v0.1.13 releases improved and aligned claim documentation (BENCHMARKS.md, CLAIM_MATRIX.md, MEASUREMENT.md, PROJECT_FACTS.md, and related docs) but did not regenerate `SHA256SUMS` or advance the release-identity document, so the newest claims sat outside a passing mechanical validation gate.

review-v0.1.14 changes bookkeeping only. **No claim content is modified.** After this release:

- `./validate_release.sh` passes at the `review-v0.1.14` tag (release-identity match + checksum integrity + dead-path + historical-labeling checks)
- The documented quick-start entry point (`README.md`, `docs/EXTERNAL_REVIEWER_GUIDE.md`) targets the current release instead of the historical `review-v0.1.8` snapshot

## Reviewer impact

Clone with `--branch review-v0.1.14` and run `./validate_release.sh`, then `make test` (AVX-512 hardware for the SIMD lanes). Claims to evaluate are documented in `CLAIM_MATRIX.md` and `VERIFICATION_STATUS.md`.

All prior review tags remain immutable and available for historical comparison.
