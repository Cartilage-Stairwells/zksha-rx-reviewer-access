# Release Contract

What a release of this package must be true of, what the repository currently does about it,
and what it does not do. Written against the published tag `review-v0.1.26` (commit `5f8dffc`)
and intended to be true of the repository, not of any claim made about it.

## Status vocabulary

Four levels, deliberately not interchangeable. A row may be at a low level without the
release being wrong; it is wrong to describe a low level as a high one.

| Level | Meaning | What moves a row up |
|-------|---------|---------------------|
| **Implemented** | A check exists in the repository and runs when the validator runs. | Writing the check. |
| **Enforced** | The validation path returns non-zero when the property is violated, demonstrated by a test in `tests/release/`. | A negative test that is observed to refuse. |
| **Release-blocking** | No release can be published without a passing validation, except by a human deliberately choosing to skip or override it. | An automated publication path that runs the gate, such as a protected branch or a CI job that must pass. |
| **Independently verified** | Someone who did not build the release reproduced the behaviour from a clean environment. | An outside party running the documented procedure. |

Two further markers are used in the row notes: **Partial** means the level is met for some
violations of the property and not others, and **None** means no check exists.

## Contract rows

| ID | Property | Implemented | Enforced | Release-blocking | Independently verified | Evidence and gap |
|----|----------|-------------|----------|------------------|------------------------|------------------|
| RC-01 | Source identity: the exact commit | Implemented | Partial | No | No | `validate_release.sh` step 1 prints `Commit: <sha>`; it is displayed and never compared to an expected value, so it informs a reader and decides nothing. |
| RC-02 | Release identity: the tag resolves to the intended commit | Implemented | Enforced | No | No | Step 1 requires an exact-match tag; step 2 requires `REVIEW_RELEASE.md` to name the same tag. Violations are refused. |
| RC-03 | Tag authenticity: signature and signer policy | Implemented | Enforced | No | No | Step 6 calls `gate-amendment/check_release_signature.sh`: annotated tag, verifying signature, fingerprint on the pinned allowlist, target reachable from the release branch, allowlist non-empty. |
| RC-04 | Package integrity: recomputed manifest | Implemented | Enforced | No | No | Step 3 runs `sha256sum -c SHA256SUMS` over 124 entries. A digest mismatch is refused; test T-13. |
| RC-05 | File completeness: no missing and no unlisted files | Partial | Not enforced | No | No | Step 4 checks listed-to-present only. A file present but absent from the manifest is not detected; test T-15 records this as defect D2. |
| RC-06 | License consistency: `LICENSE` and Cargo metadata agree | None | Not enforced | No | No | Both now state Apache-2.0 and the same holder, and nothing checks that they agree. A future edit could desynchronise them silently. |
| RC-07 | Custody disclosure: declared, unambiguous, machine-readable | Implemented | Partial | No | No | Step 7 calls `gate-amendment/check_custody_disclosure.sh`, which refuses a missing line, `independent`, a duplicate declaration, or a missing accepted-deviation disclosure. The step decides on printed text rather than exit status; test T-11 records this as defect D1. |
| RC-08 | Publication enforcement: a failed validation blocks publication | None | Not enforced | No | No | The gate is a script a person elects to run. There is no CI, no hook, and no protected-branch requirement, so a tag can be pushed without validation ever running. Every other row's "Release-blocking: No" is this row's consequence. |
| RC-09 | Reproducibility: the documented procedure succeeds in a clean environment | Partial | Not enforced | No | No | `make reproduce` is `validate verify`: it reproduces the gate and the manifest, not the test suite (`make test`) and not the measurements (`make bench`, needs AVX-512). Rehearsed by the seat only. |
| RC-10 | Historical integrity: earlier tags unchanged | None | Not enforced | No | No | No check compares published tag objects against a recorded value. Immutability so far rests on discipline, not enforcement. |

## Tallies, computed from the ten rows

| Level | Implemented | Enforced | Release-blocking | Independently verified |
|-------|-------------|----------|------------------|------------------------|
| Met for all violations | 5 | 3 | 0 | 0 |
| Partial | 2 | 2 | 0 | 0 |
| None | 3 | 5 | 10 | 10 |

Rows at each level. Implemented: RC-01, RC-02, RC-03, RC-04, RC-07. Implemented partial:
RC-05, RC-09. Implemented none: RC-06, RC-08, RC-10. Enforced: RC-02, RC-03, RC-04.
Enforced partial: RC-01, RC-07. Enforced none: RC-05, RC-06, RC-08, RC-09, RC-10.
Release-blocking: none of the ten. Independently verified: none of the ten.

### Correction of an earlier aggregate

An earlier summary of this inventory reported the split as "five, two, three" without saying
which column it counted and without accounting for all ten rows. That aggregate is superseded.
The counts above are derived from the rows: five implemented, two partial, three absent on the
implemented axis; three enforced, two partial, five not enforced on the enforced axis. The
earlier figure omitted RC-09 entirely and mixed the two axes.

## Known defects

Named, reproducible, and not fixed in `review-v0.1.26`. Both are demonstrated by tests in
`tests/release/` that record the current behaviour, so a correction can be shown to change it.

**D1, exit status discarded at step 7.** The validator runs the custody checker with `|| true`
and decides on whether stdout contains `RESULT: PASS`. A checker that prints that string and
exits non-zero is accepted; the checker's own verdict is not consulted. Test T-11.

**D2, completeness checked in one direction at step 4.** Step 4 verifies every path in
`SHA256SUMS` exists. Nothing verifies that every present file is listed, so an added file
changes the package without changing the manifest verdict. Test T-15.

## The two things the validator does not establish

Stated here because a passing run is routinely read as establishing them.

1. **Independent control of the signing identity.** A release record cannot establish it, which
   is why the custody check refuses `independent` outright.
2. **That publication was blocked.** A validation run that passes establishes that a gate, when
   a person ran it, passed. RC-08 is not met, so nothing in this repository prevents a release
   from being published unvalidated.

## What this document is not

It is not a claim that the implementation is correct, secure, or useful. It is a description of
what the repository checks, what it refuses, and where a reader's confidence should stop.
