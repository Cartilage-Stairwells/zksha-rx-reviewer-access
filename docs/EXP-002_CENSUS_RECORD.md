# EXP-002 — Boring-Repo Census: Experiment Record

**Experiment ID:** EXP-002 (TSCP transmutation inventory v3, Experiment 2)
**Date:** 2026-09-27 (observation 2026-09-27T00:15Z–00:20Z UTC)
**Status:** Complete — reproducibility/portability result.

---

## Question

How many tests does this repository actually contain under a precisely stated
counting rule?

## Repository and pin

- **Repository:** rust-lang/regex (public, Rust, unrelated to
  TSCP/zksha-rx/crypto-proof work)
- **Pinned commit:** `72d650cb0a880a01ab6dc2137c0888e8f89740f7` (branch master,
  committer date 2026-08-10T12:02:10Z, "automata: replace uses of deprecated
  module integer constants")
- **Selection rationale:** public; substantial test material in a mature Rust
  crate; `#[test]` attributes mechanically identifiable without domain
  knowledge. Not selected because of any expected error; no prior knowledge of
  any stated test count in this repository.

## Frozen counting rule (P1)

Frozen after selection/pinning, before any census execution or inspection of
count-related content.

Count all occurrences of the exact literal string `#[test]` (case-sensitive,
no intervening whitespace) in all files whose name ends in `.rs` within the
repository tree at the pinned commit.

Accepted limitations, recorded at freeze time: occurrences inside comments or
string literals are counted; variants (`#[test_case(...)]`, doctests,
benchmarks) are NOT counted; no files excluded.

## Exact mechanical predicate (Method A)

    grep -rhoF --include='*.rs' '#[test]' . | wc -l

Run at the root of the pinned tree, acquired via codeload tarball of the
pinned commit.

## Method A result

    478

Breakdown (same frozen rule): 45 in `tests/` subtree; 369 in crate source
subtrees; 45 + 369 = 478.

Top files: regex-automata/src/meta/strategy.rs (57), regex-syntax/src/hir/translate.rs (53),
regex-syntax/src/ast/parse.rs (29), regex-syntax/src/hir/mod.rs (21),
regex-lite/src/hir/parse.rs (21), regex-automata/src/util/look.rs (19).

## Independent verification (Method B)

Materially different method: Python script (os.walk traversal + str.count),
executed on an independently downloaded copy of the same pinned commit
(codeload zip archive, separate acquisition path, extracted via Python
zipfile).

## Method B result

    Total occurrences of literal '#[test]' in *.rs: 478
    Files containing at least one: 66

Per-file top-10 identical to Method A.

## Agreement/discrepancy

**Agreement: 478 = 478.** Two independent implementations on two independent
acquisitions of the same pinned tree produce identical counts.

No human-facing test-count claim was found in the repository's README.md or
RELEASE.md. No claim comparison was possible; this is recorded as an absence,
not as agreement with a claim.

## Acquisition friction (recorded, not a predicate failure)

Method B's first acquisition attempt used an invalid codeload path
(`/zip.gz/<sha>`, 14-byte 404 response) and `unzip` was unavailable in the
environment. Corrected by using the valid `/zip/<sha>` path and extracting
with Python's zipfile module. The frozen predicate was never modified. This
friction is an observation about environment tooling, not about the census.

## Evidence status (kept separate)

- **Reproducibility of the mechanical census:** demonstrated (two methods,
  two acquisitions, identical result).
- **Implementation agreement for P1:** demonstrated — two independent
  implementations on independent acquisitions of the same pinned tree produced
  identical results (478 = 478). This establishes reproducibility of the
  computation. It does not independently establish that P1 is semantically the
  right way to define "test"; P1 does not capture doctests, test-macro
  variants, or commented-out distinctions, per its frozen scope.
- **Agreement with an existing human-facing claim:** not applicable (no
  stated claim found).
- **Evidence of market demand:** none. Nothing in this experiment bears on
  demand.

## Limitations

- Single repository, single predicate, single pinned commit (snapshot).
- Literal-text matching counts commented/string occurrences; this is P1's
  recorded scope, not a defect discovered post hoc.
- The count answers the frozen question only; it is not asserted to equal
  "the number of tests" under any other rule.

## Impact on product hypothesis

**Strengthens, narrowly.** The mechanical census primitive (rule category:
count basis) reproduced exactly outside its originating repository, on a
project the TSCP work does not control, under a predicate frozen before
execution. This establishes portability and reproducibility of one predicate.
It establishes nothing about demand, and does not promote any inventory
signal beyond what the evidence supports.

---

*Observation ≠ interpretation ≠ demand. This record establishes what the
artifacts establish.*
