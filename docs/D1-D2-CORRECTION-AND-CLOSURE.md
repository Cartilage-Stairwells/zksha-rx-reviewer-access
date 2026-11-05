# D1 and D2: proposed correction and closure evidence

Defects D1 and D2 are recorded in `docs/RELEASE-CONTRACT.md` and reproduced by the baseline suite.
This document holds the proposed correction and the evidence that it closes them.

## Status boundary, stated first

The correction is **installed and published in `review-v0.1.27`**, on explicit owner authorisation
to edit the validator and to release. It was not in `review-v0.1.26`, which is immutable and still
carries both defects, so a reviewer of that earlier tag receives the defective validator while a
reviewer of `review-v0.1.27` receives the corrected one.

The proposal file `tests/release/candidates/validate_release.proposed.sh` was retired when the
correction entered `validate_release.sh`, so there is one implementation rather than two that can
drift apart.

The closure evidence below was produced by the same harness that produced the baseline, by the seat
that wrote the code. It shows that the correction changes the observed behaviour in the specified
directions. It does **not** constitute independent verification, and this document does not claim
it does.

## The two changes

**D1, exit-status integrity (step 7).** The validator ran the custody checker with `|| true` and
decided on whether stdout contained `RESULT: PASS`, so the checker's own verdict was discarded. The
correction captures the exit status and requires both conditions:

```
CD_RC=0
CD_OUT=$(bash gate-amendment/check_custody_disclosure.sh REVIEW_RELEASE.md 2>&1) || CD_RC=$?
if [ "$CD_RC" -eq 0 ] && printf '%s\n' "$CD_OUT" | grep -q 'RESULT: PASS'; then
```

**D2, completeness in both directions (new step 4b).** Step 4 checked that every listed path
exists. The correction adds the other direction: every present file must be listed, with the
manifest's own self-exclusion preserved.

```
LISTED=$(awk '{print $2}' SHA256SUMS | sort -u)
PRESENT=$(find . -type f -not -path './.git/*' | sed 's|^\./||' | grep -vx 'SHA256SUMS' | sort -u)
UNLISTED=$(comm -13 <(printf '%s\n' "$LISTED") <(printf '%s\n' "$PRESENT") || true)
```

## Evidence

### Before installation, against the validator that `review-v0.1.26` carries

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
  T-21  closure case                                   NOT RUN: no --candidate supplied
  T-22  closure case                                   NOT RUN: no --candidate supplied
  T-23  closure case                                   NOT RUN: no --candidate supplied
  T-24  closure case                                   NOT RUN: no --candidate supplied
  T-25  closure case                                   NOT RUN: no --candidate supplied
  T-26  closure case                                   NOT RUN: no --candidate supplied
  T-27  closure case                                   NOT RUN: no --candidate supplied
  T-28  closure case                                   NOT RUN: no --candidate supplied
  T-29  closure case                                   NOT RUN: no --candidate supplied
```

```
  PASS          18
  KNOWN-DEFECT  2
  FIXED         0
  FAIL          0
  NOT RUN       9
```

Strict mode against the published validator exits 1: `❌ STRICT: 2 known defect(s) still reproduce; a correction has not landed`

### After installation, against the release path

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
  T-11  checker prints PASS and exits 1                observed=fail FIXED
  T-12  checker replaced by a PASS-printing wrapper    observed=pass PASS
  T-13  checker file deleted                           observed=fail PASS
  T-14  step 3 fails, step 7 passes, later cannot override observed=pass PASS
  T-15  one byte appended to a listed file             observed=fail PASS
  T-16  one listed file deleted                        observed=fail PASS
  T-17  one unlisted file added                        observed=fail FIXED
  T-18  tagged clone, no mutation                      observed=pass PASS
  T-19  digest tampered, make reproduce                observed=pass PASS
  T-20  disclosure removed, make reproduce             observed=pass PASS
  T-21  control: honest checker, exits 0 and prints PASS observed=pass PASS
  T-22  D1 closed: prints PASS, exits 1                observed=fail PASS
  T-23  D1 closed: exits 0, prints nothing             observed=fail PASS
  T-24  D1 closed: exits 0, prints REFUSED             observed=fail PASS
  T-25  control: clean tree, nothing added             observed=pass PASS
  T-26  control: file added and correctly listed       observed=pass PASS
  T-27  D2 closed: unlisted file at root               observed=fail PASS
  T-28  D2 closed: unlisted file in a subdirectory     observed=fail PASS
  T-29  no regression: listed file deleted still refused observed=fail PASS
```

```
  PASS          27
  KNOWN-DEFECT  0
  FIXED         2
  FAIL          0
  NOT RUN       0
```

Strict mode against the installed correction exits 0, which is the claim being made: no documented defect
still reproduces.

## Why the negative controls matter

T-11 and T-17 flipping to refused only proves the original reproductions stopped reproducing. A
validator that refused everything would do the same. The controls exist to catch that:

- **T-21** an honest checker that exits 0 and prints PASS must still be accepted. A correction
  that refused on any output fails here.
- **T-25** a clean tree with nothing added must still pass step 4b. A completeness check that
  miscounts the manifest's own self-exclusion fails here.
- **T-26** a file that is added **and** correctly listed must still pass. A correction that
  refused every addition would fail here, and that would make the gate unusable rather than strict.
- **T-23 and T-24** an exit status of 0 with no `RESULT: PASS` line, and with `RESULT: REFUSED`,
  must both be refused, so the correction is not merely "trust the exit status instead".
- **T-28** the same D2 attack from a subdirectory, so the fix is not root-only.
- **T-29** deleting a listed file must still be refused, so step 4 was not displaced by 4b.

## A rig finding worth recording

Dropping the candidate over `validate_release.sh` changes a file that the manifest covers, so the
first candidate run failed step 3 and T-18 recorded a failure that had nothing to do with the
candidate. The harness now re-manifests the candidate before measuring. This is the mitigation
T-12 predicted, confirmed empirically: replacing the gate script is caught by RC-04 before any
property of the gate is exercised. It also means the harness cannot silently measure a mutated
gate.

## What is not established

- That the correction is the right correction. It closes the two defects as described; whether
  step 7 and step 4b are the correct checks remains a review question.
- That no other bypass exists. Twenty-nine cases, not complete coverage.
- That the correction is independent of its author. It is published, and publishing does not
  convert a seat-run suite into an outside result.
  of the public repository sees it yet, and `review-v0.1.26` remains the artifact that carries the
  defects.
- Any independent result: every run here was produced by the seat that wrote the code.
- That installation is the end of the work. It changes the gate on a branch, which is why the
  suite was re-run from the commit rather than from the working tree: a suite split across two
  states of one file is not evidence.

## Note on the plan

`docs/ADVERSARIAL-TEST-PLAN.md` covered T-01 to T-20 when this correction was made, filed with a
recorded digest `a965462a` that binds exactly the bytes this snapshot carries. That is a statement
about this snapshot and not about every later one. `review-v0.1.28` extended the plan to T-33 and
its own manifest binds `b50b05b7`, so both digests remain verifiable, each against the release
that carries it. The supersession is recorded in the private records at commit `ef98b76`. For
published bytes the authoritative binding is the release manifest, because it is the only binding
an independent reader can recompute from a clone.
