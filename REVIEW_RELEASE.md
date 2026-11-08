# Review Release — zkSHA-Rx Fly v0.1.0

> This file is the single source for release identity.
> A reviewer should not need to reconcile commit history.
> All references in this repository point to the release below.

## Release Identity

| Field | Value |
|-------|-------|
| Release tag | `review-v0.1.29` |
| Source commit | The commit this tag resolves to — verify with `git rev-parse HEAD` |
| Previous snapshot | `review-v0.1.28` / `14032e1` (historical) |
| Release signature | Seat identity `CAF6A411CD9AD65E2557E3DA7C65D2F3BA9969AD`, pinned in `gate-amendment/allowlist.pinned.txt` |
| Repository | https://github.com/Cartilage-Stairwells/zksha-rx-reviewer-access |

## Custody Disclosure

CUSTODY-DISCLOSURE: joint

Machine-readable form of the disclosure recorded in full under "Change in this snapshot
(review-v0.1.23)" below: custody is joint by owner decision on 2026-11-04, an accepted
deviation from the transition record's non-negotiable rather than compliance with it. From
this snapshot the disclosure is enforced rather than described: `validate_release.sh` calls
`gate-amendment/check_custody_disclosure.sh`, which refuses a record that omits the line,
asserts independent control, or declares the mode more than once.

## Change in this snapshot (review-v0.1.29)

Documentary accuracy only. The validator, the gate, the custody disclosure and the signature path are
unchanged, and `review-v0.1.28` remains immutable and keeps every defect corrected here.

**D-01, a false claim of permanence.** `docs/D1-D2-CORRECTION-AND-CLOSURE.md` stated that the adversarial
plan was "unchanged, because it was filed with a recorded digest and its bytes must stay those bytes".
That is a universal claim, and it is false in any package published after `review-v0.1.27`, because this
line of snapshots extended the plan to T-34. The sentence now names both digests and what each binds:
`a965462a` for the bytes carried by `review-v0.1.27`, `b50b05b7` for the bytes carried by
`review-v0.1.28` and later. The supersession is recorded in the private records at commit `ef98b76`.

**D-02, a false description of a case.** Plan row T-14 described a fixture in which step 7 passes.
Measured, the fixture appends a prohibited custody declaration, so step 7 refuses. Measured separately
against the pre-correction validator with only the README tampered, step 7 passes while step 3 refuses,
which is why the fixture cannot attribute its refusal to the custody step at all.

**D-03, an unreachable verdict.** T-22 declared the pre-correction behaviour as refusal when it was
acceptance, so the case could never report a correction even though it discriminates. Corrected.

**D-04, controls presented as evidence.** T-14, T-23 and T-24 are refused by both the pre-correction and
the corrected validator, so none of them can attribute a refusal to a correction. They were reported as
PASS, which reads as closure evidence. The suite now carries a CONTROL verdict for exactly this situation
and prints the count on its own line, so a passing control is never counted as a corrected defect.

**D-05, an unscoped count.** The results document reported that a run "covers all 29 cases". That was
true at the commit it records and is not true of this suite, which runs 34. The figure is now scoped to
the commit that measured it.

Nothing here changes what the validator does. Each correction makes a claim accurate, and the behavioural
evidence for D1 to D4 continues to rest on the isolated probes run against `review-v0.1.27` and
`review-v0.1.28`, not on a case count.

## Change in this snapshot (review-v0.1.28)

Two further validator defects and one harness defect are corrected. All three were found by a
third-party run against `review-v0.1.27`, and each was confirmed here by probe before anything was
changed. `review-v0.1.27` is immutable and keeps them.

**D3, the completeness comparison failed open.** Step 4b read

    UNLISTED=$(comm -13 <(...) <(...) || true)

so a `comm` that errored produced an empty unlisted set and a printed pass. Measured against
`review-v0.1.27`: with a `comm` on PATH that exits 1 and prints nothing, the validator printed
"Every present file is listed in SHA256SUMS" and exited 0. The comparison is now fail-closed, and a
non-zero `comm` is a failed release that names the cause. This is the same silent-degradation class
as D1, one line below the D1 correction.

**D4, only regular files were enumerated.** `find . -type f` cannot see a symlink, fifo, socket or
device, so an unlisted entry of those kinds rode along unlisted. Measured against `review-v0.1.27`:
an unlisted symlink and an unlisted fifo each produced a pass. A digest manifest has no way to express
them, so they are now refused by name rather than silently skipped. Empty directories are deliberately
not enumerated, because git cannot carry one, so refusing them would fail a working copy over a state
the artifact cannot express.

**Harness: step 4b was folded into step 4.** `step_block` opened on `^<n>\.` and closed on
`/^[0-9]+\. /`, so an alphanumeric identifier neither opened its own block nor closed its
predecessor's. That is fixed. Case T-17 now measures the validator's own verdict rather than a
single step's internals, because its earlier discrimination came from the fold rather than from the
check it named.

Not changed in this snapshot: the validator's other steps, the custody disclosure, the signature
gate, and the published artifacts of every earlier tag.

## Change in this snapshot (review-v0.1.27)

Two defects in the release validator are corrected, and the gate now has its first committed test
suite. Both defects were live in `review-v0.1.26`, which is immutable and keeps them.

**D1, the custody check's verdict was ignored.** Step 7 ran the checker with `|| true` and passed
the release if the checker's output merely *contained* the string `RESULT: PASS`, so a checker that
printed that string and exited non-zero was accepted. Step 7 now requires exit status 0 **and** the
line. A checker that prints nothing useful, or prints `RESULT: REFUSED`, is refused too.

**D2, completeness was checked in one direction.** Step 4 verified that every path in
`SHA256SUMS` exists; nothing verified that every present file is listed, so a file could be added
to the package without the manifest changing its verdict. New step 4b closes that direction, with
the manifest's own self-exclusion preserved.

Neither correction was made because it seemed better. Each was demonstrated failing first, by a
committed test that reproduced it against this package's previous validator, and each is now
demonstrated refused, with negative controls so that a validator which refused everything would
fail the suite rather than pass it.

**The gate now has tests.** `tests/release/run_release_tests.sh` runs 29 cases, each beginning
from a fresh fixture and changing exactly one property, and `make release-test` runs it. It records
what the validator does, not what it ought to do: a case whose behaviour matches the documented
state is PASS, one that matches a documented defect is KNOWN-DEFECT, and `--strict` refuses while
any known defect reproduces. On this snapshot: 27 PASS, 2 FIXED, 0 KNOWN-DEFECT, 0 FAIL.

**The contract is written down.** `docs/RELEASE-CONTRACT.md` states what must be true of a release
and separates four things that are routinely confused: a check existing, the validation path
refusing violations, publication being blocked without validation, and an outside party having
reproduced the result. On this snapshot the first two are substantially met and the last two are
not met at all: **0 of 10 properties are release-blocking and 0 of 10 are independently verified.**
Nothing in this repository prevents a tag from being pushed without validation ever running. That
is recorded as RC-08 and it is not fixed by this release.

`docs/D1-D2-CORRECTION-AND-CLOSURE.md` holds the correction and the evidence for it, and
`docs/ADVERSARIAL-TEST-PLAN.md` holds the case list. No test, source file, or measured value
changes, and `make reproduce` keeps its existing meaning: the gate and the manifest, not the Rust
suite and not the benchmarks.

## Change in this snapshot (review-v0.1.26)

Three corrections, none of which touches a test, a source file, or a measured value.

**The custody disclosure is now enforced.** The frozen position of 2026-11-04 requires every
release record to keep the joint-custody deviation visible, but until now that requirement was
prose: `validate_release.sh` did not read for it, so a future record could have omitted the
disclosure and still passed every check. The validator now calls
`gate-amendment/check_custody_disclosure.sh`, and a refusal counts as a failed release. The
check is ported from the reference implementation filed at `61c011b` in
`tscp-workstream-records`, including its whitespace handling, with one addition: a duplicate
declaration is refused. The reference took the first declaration and ignored later ones, so a
record could carry `joint` and then `independent` and pass while asserting exactly what the
check exists to prevent.

**The package metadata now declares its licence and author.** `Cargo.toml` carried no
`license` and no `authors` field, so the only statement of either was prose in `README.md`.
It now declares `license = "Apache-2.0"` and `authors = ["Sean Christopher Southwick"]`,
matching the copyright notice restored at `review-v0.1.25`. This closes the separate open
finding recorded in that snapshot.

**The reproduction command was stale.** The instructions told a reviewer to check out
`review-v0.1.22`, three releases behind, so following them produced an older snapshot than
the one being reviewed. It now checks out this tag.

## Change in this snapshot (review-v0.1.25)

Licence notice correction. The Apache-2.0 `LICENSE` shipped since `review-v0.1.22` carried
the unfilled Apache template notice, so the package stated no copyright holder anywhere:
`LICENSE` was the only file in the tree mentioning copyright, there was no `NOTICE` file, and
the README pointed solely at `LICENSE`. The notice is restored to the form used from v0.1.0
through v0.1.15:

    Copyright 2025-2026 Sean Christopher Southwick

Scope is that one line. The licence terms, sections 1 through 9, are unmodified, and no test,
source file, or measured value changes. Three separate findings are recorded rather than one
verdict: the licence file was absent from `review-v0.1.16` through `review-v0.1.21`
(historical), the limitation-of-liability clause was repaired from "be Iiable" when the file
returned at `review-v0.1.22` (historical), and the named holder was lost in that same
restoration.

`Cargo.toml` still declares no `license` and no `authors` field. That is a separate open
finding, deliberately not bundled into this correction.

The erratum text for `review-v0.1.24` remains inside that tag and is not reproduced here.

## Change in this snapshot (review-v0.1.23)

The release signature check is now **enforced**, not advisory. `validate_release.sh`
calls `gate-amendment/check_release_signature.sh`, which fails the release unless all of
the following hold: the tag is an annotated tag object; its OpenPGP signature verifies;
the verifying key's fingerprint is on the pinned release allowlist; the tag target is
reachable from the release branch; and the allowlist itself is configured. Every unmet
condition is reported, and none of them is a warning.

The identity that signs releases from this snapshot on is a **seat** identity, not a
personal one: `TSCP Release Seat (custodian) <seat@solene.local>`, fingerprint
`CAF6A411CD9AD65E2557E3DA7C65D2F3BA9969AD`. Historical signatures made with the two retired personal identities remain valid
for verifying tags that already exist; they can never authorise a release again, and the
gate reports an identity appearing on both lists as a fatal contradiction.

Two honest limitations, recorded rather than implied:

1. **Custody is joint, by owner decision on 2026-11-04.** The seat's secret half sits in
   two places: the agent keyring that signs releases, and a backup held by the owner, so
   that losing the sandbox does not end the identity and either party can restore it. This
   departs from the transition record's non-negotiable that the private half is not held by
   whoever operates the release automation. The departure is recorded here as an accepted
   deviation decided by the owner, not as compliance with that requirement. Residual copies
   exist and cannot be withdrawn: two auto-commit entries in the platform's workspace
   repository history, and one object in private file storage behind an expired link. Those
   are platform-side rather than GitHub, and they are part of this key's history.
2. **A signature attests continuity of an authorised identity.** It says nothing about
   authorship by a named person and nothing about the correctness of what it signs.

`gpg` is required to verify the signature. If it is absent the check fails closed by
design rather than passing quietly.

## Change in this snapshot (review-v0.1.22)

`review-v0.1.21` was tagged at `73bf100`. The commit that followed it, `57568f3`
(Apache-2.0 LICENSE), moved the default branch tip past the tag, and the release gate
requires the checked-out commit to be exactly at a tag. A reviewer who cloned and ran
`make reproduce` without first checking out the tag therefore saw `RELEASE VALIDATION
FAILED` on an otherwise intact package. This snapshot tags the tip, so a fresh clone
validates, and pins the toolchain to `rustc 1.99.1`, the compiler the recorded
measurements and the 139-test census were produced with. No source, test, benchmark
result, proof, or measured value is changed.

`SHA256SUMS` is regenerated over all tracked files except itself, which also restores
`LICENSE` to the manifest: it was added at `57568f3` without a manifest update.

## Artifact Inventory

| Artifact | Location | Hash/Ref |
|----------|----------|----------|
| AVX-512 SIMD kernel | `src/avx512_butterfly_32bit.rs` | in release commit |
| Scalar reference | `src/lib.rs` | in release commit |
| Benchmark harnesses | `benches/three_lane_bench.rs` (canonical three-lane), `benches/butterfly_bench.rs` (scalar reference) | in release commit |
| Test suite | `tests/` (14 files) | in release commit |
| Evidence bundle | `evidence/` | in release commit |
| SHA256 sums | `SHA256SUMS` | in release commit |
| Benchmark docs | `BENCHMARKS.md` | in release commit |
| Measurement docs | `MEASUREMENT.md` | in release commit |
| Evidence chain | `EVIDENCE_CHAIN.md` | in release commit |
| Architecture | `ARCHITECTURE.md` | in release commit |
| Canonical facts | `PROJECT_FACTS.md` | in release commit |
| Claim matrix | `CLAIM_MATRIX.md` | in release commit |
| Formal verification | `formal/README.md` | external repo ref |

## Formal Verification Reference

The Lean 4 formalization is in a separate repository:
- **Repository:** Cartilage-Stairwells/tscp-anchor (public)
- **Tag:** `formal-corpus-v2026-09-11` (previous reference `v0.1.0-zksha-rx` does not exist in tscp-anchor; repaired 2026-09-11 — see HISTORY)
- **Lean version:** 4.32.1
- **Build command:** `lake build`
- **Theorem count:** 87 theorem declarations across 12 Lean files (census at tag `formal-corpus-v2026-09-11`, 2026-09-11; method: theorem-declaration grep: 83 build roots plus 4 scaffold declarations outside them). The earlier 88 counted one line of block-comment prose in `Core.lean` as a declaration and is withdrawn.
- **See:** `formal/README.md` in this repo for details

## Reproduction Commands

```bash
# Clone this repository
git clone https://github.com/Cartilage-Stairwells/zksha-rx-reviewer-access.git
cd zksha-rx-reviewer-access
git checkout review-v0.1.29

# Build
cargo build --release

# Run tests
cargo test --release

# Run the canonical three-lane benchmark (reproduces the 1.27x kernel claim; requires AVX-512 CPU)
cargo bench --bench three_lane_bench

# Verify evidence integrity
sha256sum -c SHA256SUMS
```

## Out-of-Tree Benchmark Evidence (Branch-Resident)

The raw benchmark evidence artifacts are **not in the main tree**; they live
on a separately pinned evidence branch and are verified with their own
SHA256SUMS manifest. `sha256sum -c SHA256SUMS` above validates only the
main-tree files; complete evidence verification also requires:

```bash
git fetch origin evidence/experiment-a
git checkout 8df0c247cc503b751ee91245bb952b6994aaf2e6
cd evidence/experiment_a && sha256sum -c SHA256SUMS
```

- **Branch:** `evidence/experiment-a`
- **Pinned commit:** `8df0c247cc503b751ee91245bb952b6994aaf2e6`
- **Path:** `evidence/experiment_a/` — SHA256SUMS-pinned artifacts
- **Note:** the evidence manifest paths are relative to its own directory; the verification command must run from inside `evidence/experiment_a/` (as shown above)

See `BENCHMARKS.md` and `CLAIM_MATRIX.md` for the claims this evidence supports.

## Relationship to Source Repository

This repository is a **frozen reviewer snapshot**. It is not the development repository.

- The development repository (`avx512-butterfly`) is private and contains additional operational documents not relevant to technical review.
- This snapshot contains everything needed to evaluate the technical claims.
- The snapshot will not change. If the project evolves, a new review tag (e.g., `review-v0.2.0`) will be created.
