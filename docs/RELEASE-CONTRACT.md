# Release Contract

What a release of this package must be true of, what the repository currently does about it,
and what it does not do. Written against branch `assurance-baseline`, where defects D1 and D2 are corrected at commit
`532799a`. The published tag `review-v0.1.26` still carries both, because published tags are
immutable, so the statuses below describe the branch and not the artifact a reviewer of that tag
receives. Intended to be true of the repository, not of any claim made about it.

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
| RC-04 | Package integrity: recomputed manifest | Implemented | Enforced | No | No | Step 3 runs `sha256sum -c SHA256SUMS` over every listed entry. A digest mismatch is refused; test T-13. |
| RC-05 | File completeness: no missing and no unlisted files | Implemented | Enforced | No | No | Step 4 refuses a listed path that is absent, and step 4b refuses a present file with no manifest entry. Both directions are tested: T-16 and T-29 for the first, T-17, T-27 and T-28 for the second. |
| RC-06 | License consistency: `LICENSE` and Cargo metadata agree | None | Not enforced | No | No | Both now state Apache-2.0 and the same holder, and nothing checks that they agree. A future edit could desynchronise them silently. |
| RC-07 | Custody disclosure: declared, unambiguous, machine-readable | Implemented | Enforced | No | No | Step 7 calls `gate-amendment/check_custody_disclosure.sh` and requires exit status 0 **and** the `RESULT: PASS` line, so a checker that prints the string and fails is refused (T-11, T-22), one that prints nothing useful is refused (T-23), and an honest pass is still accepted (T-21). |
| RC-08 | Publication enforcement: a failed validation blocks publication | None | Not enforced | No | No | The gate is a script a person elects to run. There is no CI, no hook, and no protected-branch requirement, so a tag can be pushed without validation ever running. Every other row's "Release-blocking: No" is this row's consequence. |
| RC-09 | Reproducibility: the documented procedure succeeds in a clean environment | Partial | Not enforced | No | No | `make reproduce` is `validate verify`: it reproduces the gate and the manifest, not the test suite (`make test`) and not the measurements (`make bench`, needs AVX-512). Rehearsed by the seat only. |
| RC-10 | Historical integrity: earlier tags unchanged | None | Not enforced | No | No | No check compares published tag objects against a recorded value. Immutability so far rests on discipline, not enforcement. |

## Tallies, computed from the ten rows

| Level | Implemented | Enforced | Release-blocking | Independently verified |
|-------|-------------|----------|------------------|------------------------|
| Met for all violations | 6 | 5 | 0 | 0 |
| Partial | 1 | 1 | 0 | 0 |
| None | 3 | 4 | 10 | 10 |

Rows at each level. Implemented: RC-01, RC-02, RC-03, RC-04, RC-05, RC-07. Implemented partial:
RC-09. Implemented none: RC-06, RC-08, RC-10. Enforced: RC-02, RC-03, RC-04, RC-05, RC-07.
Enforced partial: RC-01. Enforced none: RC-06, RC-08, RC-09, RC-10. Release-blocking: none of the
ten. Independently verified: none of the ten.

### Correction of an earlier aggregate

An earlier summary of this inventory reported the split as "five, two, three" without saying
which column it counted and without accounting for all ten rows. That aggregate is superseded.
The counts above are derived from the rows: five implemented, two partial, three absent on the
implemented axis; three enforced, two partial, five not enforced on the enforced axis. The
earlier figure omitted RC-09 entirely and mixed the two axes.

## Defects found, and their correction

Both were live in `review-v0.1.26` and both are corrected on this branch at commit `532799a`. The
published tag keeps them, because published tags are immutable, which is the point of the tag.

**D1, exit status discarded at step 7.** The validator ran the custody checker with `|| true` and
decided on whether stdout contained `RESULT: PASS`, so the checker's own verdict was never
consulted. Step 7 now requires exit status 0 and the line. Refused: a checker that prints the
string and exits 1 (T-11, T-22), a checker that exits 0 and prints nothing (T-23), and one that
exits 0 and prints `RESULT: REFUSED` (T-24). Accepted: an honest checker (T-21).

**D2, completeness checked in one direction at step 4.** Step 4 verified that every listed path
exists, and nothing verified that every present file is listed. New step 4b does, preserving the
manifest's own self-exclusion. Refused: an unlisted file at the root (T-17, T-27) and in a
subdirectory (T-28). Accepted: a clean tree (T-25) and a file added with its manifest entry (T-26).

The negative controls are the load-bearing part. A validator that simply refused everything would
satisfy T-22 and T-27 while being useless, so T-21, T-25 and T-26 exist to make that failure
visible, and they pass.

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
