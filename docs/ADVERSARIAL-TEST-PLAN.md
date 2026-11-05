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
| T-14 | Boundary | step 3 fails while step 7 passes | overall exit non-zero | overall exit non-zero |
| T-15 | Integrity | one byte appended to a listed file | refuse | refuse |
| T-16 | Integrity | one listed file deleted | refuse | refuse |
| T-17 | Integrity | one unlisted file added | accept (defect D2) | refuse |
| T-18 | Pipeline | none, on a tagged clone | full pass, exit 0 | full pass, exit 0 |
| T-19 | Pipeline | digest tampered, then `make reproduce` | exit non-zero | exit non-zero |
| T-20 | Pipeline | disclosure removed, then `make reproduce` | exit non-zero | exit non-zero |

## The four properties that matter most

T-11, T-12, T-14 and T-17 test the enforcement boundary rather than the checker's internals, and
they are the reason this phase exists. Each is expected to expose something:

- **T-11, exit-status integrity.** The validator discards the checker's exit status and trusts
  printed text, so a failing checker can pass by printing one string. This is defect D1.
- **T-12, output authenticity.** A wrapper that prints the magic string satisfies step 7 whatever
  it does. The only thing standing between that and a passing release is step 3: the checker is
  in the manifest, so replacing it changes a digest. That defence belongs to RC-04 and should not
  be mistaken for RC-07 working.
- **T-14, no override.** A later step cannot erase an earlier failure, because the validator
  counts failures rather than short-circuiting on success.
- **T-17, file completeness.** An added file is invisible to step 4, which checks one direction
  only. This is defect D2.

## What this plan does not cover

- Publication enforcement (RC-08). No automated path exists, so no test can demonstrate that a
  bad release is blocked from being published. Tests here prove a script refuses, nothing more.
- Independent execution (RC-09, the "Independently verified" column). Every result recorded here
  was produced by the seat that built the release.
- Whether the checks are the right checks. That is the contract's question, not the suite's.
- Benchmark reproduction, which needs AVX-512 hardware and is out of scope for the gate.
