# review-v0.1.17 Release Note

## Purpose

External audit (2026-10-23) of review-v0.1.16 confirmed the release-identity correction but found the reviewer package not yet clean: (1) PROJECT_FACTS.md claimed a license file that does not exist in the tree or manifest, contradicting the recorded licensing deferral; (2) docs/EXTERNAL_REVIEWER_GUIDE.md claimed "83 theorems, 0 axioms, 0 sorries — no unproven gaps," overstating the documented evidence; (3) REVIEWER_QUICKSTART.md called the repository self-contained while raw benchmark evidence lives on a separately pinned branch. Mechanical pre-flight of the documented verification commands found two further defects: the out-of-tree evidence check fails as written, and the review-v0.1.16 release note was not covered by the manifest it shipped with.

review-v0.1.17 closes all of the above. Claim-scope and documentation corrections only. **No measured value, benchmark result, or proof content is modified.**

## Disposition of audit findings

| Finding | Disposition at v0.1.17 |
|---|---|
| Missing LICENSE vs "It contains a license file" (PROJECT_FACTS.md) | RESOLVED — documentation aligned to the recorded licensing deferral (PROVENANCE_AND_LICENSING.md, commit 87eb2e3). The licensing decision is unchanged: no repository-wide license file; licensing deferred pending file-level provenance; upstream-derived material retains its applicable license. |
| "83 theorems, 0 axioms, 0 sorries — no unproven gaps" (guide) | RESOLVED — wording scoped to the documented evidence: Montgomery.lean core is 12 theorems, 0 axioms, 0 sorries (compile-verified per formal/README.md); corpus census at `formal-corpus-v2026-09-11` counts 88 theorem declarations across 12 Lean files (independently reproduced by theorem-declaration grep at this release); 3 documented sorries exist in example modules. "No unproven gaps" is withdrawn — corpus-wide sorry-freedom is not established (deferred skeleton work exists in corpus modules outside the reviewer package's claim scope). |
| "Self-contained" wording vs external evidence branch (quickstart) | RESOLVED — quickstart now states review-entry artifacts are in-tree and raw benchmark evidence is branch-resident, matching REVIEW_RELEASE.md. |
| 83 vs 88 theorem count unreconciled | RESOLVED — the release's single documented census is the 88-declaration count (formal-corpus-v2026-09-11, method: theorem-declaration grep), independently reproduced at this release. The 83 figure is removed from the reviewer guide (historical pre-census count, superseded by the review-v0.1.15 theorem-count alignment). |

## Mechanical findings closed en route (pre-flight of the documented commands, 2026-10-23)

- The out-of-tree evidence verification command `sha256sum -c evidence/experiment_a/SHA256SUMS`, run from the repository root as documented, fails for all 12 entries: the manifest's paths are relative to the manifest's own directory. Corrected in REVIEW_RELEASE.md, PROJECT_FACTS.md, and REVIEWER_QUICKSTART.md. The evidence manifest itself verifies **12/12** from the manifest directory at pinned commit `8df0c247cc503b751ee91245bb952b6994aaf2e6` (independently verified at this release).
- `REVIEW_V0.1.16_RELEASE_NOTE.md` was tracked at review-v0.1.16 but not covered by SHA256SUMS (the v0.1.13–v0.1.15 release notes are covered). Brought under the manifest at this release. Recurrence of the missed-coverage class documented in HISTORY.md (2026-09-27 entry); no prior tag is altered.

## Verification at seal

- `./validate_release.sh` at review-v0.1.16 reproduced **7/7 passed** (1 warning: unsigned tag, consistent with the advisory-signature policy) before changes.
- Main-tree SHA256SUMS at review-v0.1.16 verified **99/99** against a clean checkout before changes.
- Formal corpus census independently reproduced: **88 theorem declarations across 12 Lean files**; `Montgomery.lean`: **12 theorems, 0 axioms, 0 sorries** (textual census; compile status as documented in formal/README.md).
- SHA256SUMS at this release regenerated over all tracked files except itself and re-verified from a clean clone of the tag.

## Reviewer impact

Clone with `--branch review-v0.1.17` and run `./validate_release.sh`, then `make test` (AVX-512 hardware for the SIMD lanes). Claims to evaluate are documented in `CLAIM_MATRIX.md` and `VERIFICATION_STATUS.md`.

All prior review tags remain immutable and available for historical comparison.
