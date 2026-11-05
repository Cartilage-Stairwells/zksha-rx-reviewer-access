# Adversarial Test Plan

Attack cases for the release gate, and what each one is expected to produce. Written against
`review-v0.1.26`. The plan and the harness are kept in step: every case ID below appears in the
output of `tests/release/run_release_tests.sh`, and a case that runs without being listed here,
or is listed without running, is a defect in this document.

## What these tests are, and are not

They are the first committed test infrastructure for the release checker. Before this phase the
checker had been exercised only by ad hoc commands in a sandbox, which left no regression
protection: the duplicate-declaration hardening added at `review-v0.1.26` could be removed by a
later edit with nothing to notice.

They test two different things, and the difference matters:

1. **The checker's logic.** Given a record, does `check_custody_disclosure.sh` refuse what it
   should refuse and accept what it should accept.
2. **The enforcement boundary.** Does the release path actually stop when the checker refuses.
   A checker that refuses perfectly is worth nothing if the validator ignores it.

## Mutation protocol

Every case starts from a passing fixture and changes exactly one property, so a refusal can be
attributed to the change rather than to accumulated drift.

1. Build a fixture from the repository under test.
2. Confirm the unmutated fixture produces the expected passing result.
3. Apply one mutation.
4. Run the relevant check and record the observed result and exit status.
5. Discard the fixture. No fixture is reused between cases.

## Result vocabulary

| Marker | Meaning |
|--------|---------|
| PASS | The observed behaviour matches what a correct gate must do. |
| KNOWN-DEFECT | The observed behaviour matches what `review-v0.1.26` actually does, and that behaviour is wrong. Recorded, not excused. |
| FIXED | The observed behaviour differs from `review-v0.1.26` and matches what a corrected gate must do. |
| FAIL | The observed behaviour matches neither. The baseline itself is unreliable and must be investigated before any conclusion is drawn. |

A run exits non-zero if any case is FAIL. `--strict` also treats KNOWN-DEFECT as failure, which
is how a correction is demonstrated: the same suite, re-run, must stop producing KNOWN-DEFECT.

## Cases

| ID | Group | Mutation | Expected at v0.1.26 | Expected when corrected |
|----|-------|----------|---------------------|--------------------------|
| T-01 | Disclosure | none, clean record | accept | accept |
| T-02 | Disclosure | declaration removed | refuse | refuse |
| T-03 | Disclosure | value changed to `independent` | refuse | refuse |
| T-04 | Disclosure | second declaration appended, `joint` then `independent` | refuse | refuse |
| T-05 | Disclosure | accepted-deviation wording removed everywhere | refuse | refuse |
| T-06 | Disclosure | declaration indented, so not at line start | refuse | refuse |
| T-07 | Disclosure | value changed to `joint-ish` | refuse | refuse |
| T-08 | Disclosure | whole record rewritten with CRLF endings | accept | accept |
| T-09 | Disclosure | tab inserted inside the required wording | accept | accept |
| T-10 | Disclosure | declaration moved out of `REVIEW_RELEASE.md` | refuse | refuse |
| T-11 | Boundary | checker prints `RESULT: PASS` and exits 1 | accept (defect D1) | refuse |
| T-12 | Boundary | checker replaced by a wrapper that prints PASS without reading | step 7 accepts, step 3 refuses | unchanged, defence is RC-04 |
| T-13 | Boundary | checker file deleted | refuse | refuse |
| T-14 | Ordering | tampered README only, so step 3 fails while step 7 passes (exercises the ordering; still a control, since the same mutation is caught independently) | overall exit non-zero | overall exit non-zero |
| T-15 | Integrity | one byte appended to a listed file | refuse | refuse |
| T-16 | Integrity | one listed file deleted | refuse | refuse |
| T-17 | Integrity | one unlisted file added | accept (defect D2) | refuse |
| T-18 | Pipeline | none, on a tagged clone | full pass, exit 0 | full pass, exit 0 |
| T-19 | Pipeline | digest tampered, then `make reproduce` | exit non-zero | exit non-zero |
| T-20 | Pipeline | disclosure removed, then `make reproduce` | exit non-zero | exit non-zero |

### Cases T-21 to T-34

The table above covers T-01 to T-20. The suite runs through T-34. The rows below are generated
from the harness itself, so this section cannot drift from what is executed again.

| Case | Group | Mutation | Pre-correction | Post-correction |
|---|---|---|---|---|
| T-21 | Closure | control: honest checker, exits 0 and prints PASS | accept | accept |
| T-22 | Closure | D1 closed: prints PASS, exits 1 | accept | refuse |
| T-23 | Control | D1 closed: exits 0, prints nothing (control, cannot discriminate) | refuse | refuse |
| T-24 | Control | D1 closed: exits 0, prints REFUSED (control, cannot discriminate) | refuse | refuse |
| T-25 | Closure | control: clean tree, nothing added | accept | accept |
| T-26 | Closure | control: file added and correctly listed | accept | accept |
| T-27 | Closure | D2 closed: unlisted file at root | refuse | refuse |
| T-28 | Closure | D2 closed: unlisted file in a subdirectory | refuse | refuse |
| T-29 | Closure | no regression: listed file deleted still refused | refuse | refuse |
| T-30 | Completeness | unlisted symlink (D4) | accept | refuse |
| T-31 | Completeness | unlisted fifo (D4) | accept | refuse |
| T-32 | Completeness | failing comm fails closed (D3) | accept | refuse |
| T-33 | Completeness | parser keeps step 4b out of step 4 | step 4 absorbs 4b | 4b isolated from step 4 |
| T-34 | Aggregation | prohibited declaration appended, manifest re-cut so step 3 stays clean | not present | overall refuses, step 3 clean |
