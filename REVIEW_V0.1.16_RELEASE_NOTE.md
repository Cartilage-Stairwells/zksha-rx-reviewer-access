# review-v0.1.16 Release Note

## Purpose

External audit (2026-10-23) found the reviewer package's release identity split: the README and external guide targeted `review-v0.1.15`, while PROJECT_FACTS.md (including its canonical external-reference list) and REVIEWER_QUICKSTART.md still identified `review-v0.1.6` as the current snapshot, and HISTORY.md's Current Release table had not advanced since 2026-07-30. Reviewers following the README would clone a tag whose own files contradicted it.

review-v0.1.16 changes bookkeeping only. **No claim content is modified.** After this release:

- Every reviewer entry document identifies the same current snapshot: `review-v0.1.16`
- `./validate_release.sh` passes at the tag (release-identity match + checksum integrity)
- The out-of-tree benchmark evidence branch (`evidence/experiment-a`, pinned `8df0c247cc503b751ee91245bb952b6994aaf2e6`) has documented verification commands in REVIEW_RELEASE.md and REVIEWER_QUICKSTART.md; the main-tree SHA256SUMS gate is explicitly scoped to main-tree files
- SHA256SUMS regenerated over all tracked files (sealed files were modified on main since the v0.1.15 seal)

## Reviewer impact

Clone with `--branch review-v0.1.16` and run `./validate_release.sh`, then `make test` (AVX-512 hardware for the SIMD lanes). Claims to evaluate are documented in `CLAIM_MATRIX.md` and `VERIFICATION_STATUS.md`.

All prior review tags remain immutable and available for historical comparison.
