# review-v0.1.18 Release Note

## Purpose

An independent verification pass at review-v0.1.17 ran the full test suite on AVX-512 hardware (2026-10-23, rustc 1.99.1, default profile, clean clone of the tag): **12 test targets, 139 tests run, 139 passed, 0 failed, 0 ignored.** The reviewer documents' test-count wording did not match this measurement. review-v0.1.18 aligns every count claim to the measured run. Count-wording only; no measured value, benchmark result, or proof content is modified.

## Measured record

- `cargo test` at review-v0.1.17: 12 test targets, **139 run, 139 passed, 0 failed, 0 ignored**
- Composition: **102 integration tests** across `tests/` targets (including 6 archived regression tests, which run as their own target) + **37 unit tests** in `src/` (lib target)
- Textual census: 141 `#[test]` declarations, of which 2 occurrences are commented-out code
- Run log preserved (2026-10-23); configuration: AVX-512 capable x86_64 CPU, rustc 1.99.1, default profile, `RUSTFLAGS="-Ctarget-cpu=native"`

## Disposition of count findings

| Finding | Disposition at v0.1.18 |
|---|---|
| README/guide "102 tests" | EXACT for the `tests/` integration suite (102/102 pass) — kept where scoped, restated with the full 139 breakdown where unscoped |
| Quickstart "(140 total)" | CORRECTED to the measured 139 with composition stated |
| EVIDENCE_CHAIN "140/140 tests" | CORRECTED to the measured 139/139 suite run |
| EVIDENCE_CHAIN "33 Lean 4 theorems" | CORRECTED to the recorded census wording (88 theorem declarations across 12 Lean files at `formal-corpus-v2026-09-11`; Montgomery.lean: 12) |
| CLAIM_MATRIX counting-basis slips (live-suite arithmetic; commented occurrences) | CORRECTED to the measured composition (96 live + 6 archived = 102 compiled in `tests/`; 37 in `src/`; 141 textual / 139 compiled) |
| METRICS.md dated rows | 2026-08-03 rows preserved as historical records; 2026-10-23 measured rows appended |

## Verification at seal

- `./validate_release.sh` at review-v0.1.17 reproduced 7/7 before changes; the same gate passes 7/7 at review-v0.1.18
- SHA256SUMS regenerated over all tracked files except itself and verified from a clean clone of the tag

## Reviewer impact

Clone with `--branch review-v0.1.18` and run `./validate_release.sh`, then `cargo test` — the suite runs 139 tests (102 integration + 37 unit) and all pass on AVX-512 hardware. All prior review tags remain immutable and available for historical comparison.
