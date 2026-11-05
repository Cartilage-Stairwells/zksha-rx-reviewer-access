# Release Assurance Results

Recorded observations, not claims. Every line below was produced by running
`tests/release/run_release_tests.sh` against this branch, and the case rows are copied from that
output verbatim rather than retyped.

## What was run

| Field | Value |
|-------|-------|
| Suite | `tests/release/run_release_tests.sh` (also `make release-test`) |
| Cases | 20, listed in `docs/ADVERSARIAL-TEST-PLAN.md` |
| Base | tag `review-v0.1.26`, commit `5f8dffc` |
| Branch | `assurance-baseline`, released nowhere |
| Run by | the seat that built the release, not an independent party |
| Date | 2026-11-05 |

## Results

```
  T-01  clean record, no mutation                      observed=pass PASS
  T-02  declaration line removed                       observed=fail PASS
  T-03  value changed to independent                   observed=fail PASS
  T-04  second declaration appended                    observed=fail PASS
  T-05  accepted-deviation wording removed             observed=fail PASS
  T-06  declaration indented off line start            observed=fail PASS
  T-07  value changed to joint-ish                     observed=fail PASS
  T-08  record rewritten with CRLF endings             observed=pass PASS
  T-09  tab inside the required wording                observed=pass PASS
  T-10  declaration moved to another file              observed=fail PASS
  T-11  checker prints PASS and exits 1                observed=pass KNOWN-DEFECT
  T-12  checker replaced by a PASS-printing wrapper    observed=pass PASS
  T-13  checker file deleted                           observed=fail PASS
  T-14  step 3 fails, step 7 passes, later cannot override observed=pass PASS
  T-15  one byte appended to a listed file             observed=fail PASS
  T-16  one listed file deleted                        observed=fail PASS
  T-17  one unlisted file added                        observed=pass KNOWN-DEFECT
  T-18  tagged clone, no mutation                      observed=pass PASS
  T-19  digest tampered, make reproduce                observed=pass PASS
  T-20  disclosure removed, make reproduce             observed=pass PASS
```

```
  PASS          18
  KNOWN-DEFECT  2
  FIXED         0
  FAIL          0
```

preflight: steps 3, 4 and 7 parse, and observe 'pass' on a clean fixture

## What the baseline establishes

The gate currently refuses every disclosure mutation in Group A, including the duplicate
declaration and the whitespace variants, and it stops the pipeline in Group D. Those are observed
refusals, not reasoned ones: each case began from a passing fixture and changed exactly one
property.

Both known defects reproduce on demand:

- **D1 (T-11).** A checker that prints `RESULT: PASS` and exits 1 is accepted at step 7, because
  the validator discards the exit status and trusts the printed string.
- **D2 (T-17).** A file added without a manifest entry is not detected at step 4, because
  completeness is checked in one direction only.

T-12 records a mitigation worth not confusing with a defence: when the checker is replaced
wholesale, step 7 accepts it, but step 3 refuses the release because the checker is in the
manifest. That protection belongs to RC-04. It is not the custody check working.

## Limitations, stated rather than implied

- **No publication enforcement was tested, because none exists.** RC-08 has no automated path, so
  no test here can demonstrate that a bad release is blocked from being published. What is
  demonstrated is that a script refuses when run.
- **No result here is independent.** Every case was executed by the seat that built the release.
  The contract's "Independently verified" column stays at zero for all ten rows.
- **RC-06 and RC-10 have no cases** because no check exists for either. Their absence from the
  suite is not evidence that they hold.
- **The checks themselves are not evaluated.** This suite establishes that the gate does what the
  documentation says. It says nothing about whether those checks are the right ones, whether the
  implementation is correct, or whether the claims the package makes are true.
- **`make bench` was not run.** It needs AVX-512 hardware and is outside the gate.
- The suite has not been run against any other release. `RELEASE_TAG` selects the tag under test;
  a different tag changes which baseline is recorded.
