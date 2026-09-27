# TSCP Transmutation Inventory — v3

**Status:** working inventory. Evidence instrument, not a sales document.
**Version:** v3 (2026-09-27). Supersedes v2 (2026-09-27).
**Scope:** Cartilage-Stairwells/TSCP body of work. Not part of the frozen
reviewer snapshot's claims; no claim in this file extends the review scope
of zkSHA-Rx Fly.

---

## Signal taxonomy

Every asset is classified into one of four states. The distinction between
external contact and external use is mandatory; Tier 1 means survivability
under external interaction, not purchase intent.

| State | Meaning |
|-------|---------|
| Externally exercised | Someone outside the project interacted with the artifact/workflow in a substantive way |
| Externally scrutinized | Outside technical expert/reviewer examined or responded to the work |
| Externally exposed | Publicly available/distributed; no meaningful uptake observed |
| Unexercised | No meaningful outside interaction known |

A fifth working state is used for assets demonstrated internally against
externally exercised history but not yet validated on external material:

| State | Meaning |
|-------|---------|
| Internally demonstrated, externally unvalidated | Works reproducibly on the project's own material; has not yet been exercised on systems the project does not control |

---

## Inventory

| # | Asset | Economic form | Signal | Defensible claim |
|---|-------|--------------|--------|------------------|
| 1 | Evidence bundles (reviewer package) | Customer deliverable | Externally exercised | External review survived contact; corrections were generated against concrete package artifacts. |
| 2 | Claim-scope machinery | Software primitive | Externally exercised | Externally exercised as part of a review workflow; standalone product demand not yet demonstrated. |
| 3 | SHA/reproducibility discipline | Integrity infrastructure | Externally exercised | Externally exercised infrastructure; demand for standalone tooling not yet demonstrated. |
| 4 | Byte census tooling | Developer tool | Externally exercised (as correction instrument) | Demonstrated utility: yes (140→141 correction). External demand: not yet. |
| 5 | Correction history | **Machine-checkable correction corpus** (formerly: adjudication corpus / evaluation-data seed) | Externally exercised (emerging corpus) | Seed corpus: 6 extracted events from 7 examined commits; 9 correction-bearing commits remain unextracted. Each event is a genuine claim→artifact→observation→discrepancy→correction→rule instance. Not yet a mature evaluation dataset. |
| 6 | Benchmark methodology + experiments | Benchmark/evaluation capability | Externally scrutinized | Technical expert responses received (AVX-512/NTT work). No demonstrated third-party execution of the methodology. |
| 7 | Evidence doctrine | Methodology / IP | Embedded in own work; no independent uptake | — |
| 8 | Evidence vocabulary | Schema / API | Externally exposed | Public availability is exposure, not uptake. |
| 9 | TSCP store | Commercial interface | Externally exposed | No purchase/inquiry signal observed. Instrumentation pending (Experiment 4). |
| 10 | Public GitHub experiments | Demonstration corpus | Externally exposed / planned experiment | Experiment 2 pending. |
| 11 | Discovery framework | Sales methodology | Unexercised | No action until a Track A/B conversation occurs. |
| 12 | Case-study machinery | Consulting product | Unexercised | Fold into #1's template. |
| 14 | **Machine-checkable rule library** | Validation/regression-test specification | **Internally demonstrated, externally unvalidated** | Six rules extracted from externally exercised correction history (AC-001..AC-006): count basis, repository identity, scope precision, measurement status, controlled vocabulary, symbol existence, cross-document consistency, provenance witnesses. Next experiment: implement 2–3 predicates against an unrelated repository. |

(#13 is the store's measurable event — folded into #9 above.)

---

## Working product hypothesis

External review has repeatedly created value at the boundary where human
claims become mechanically inspectable, and the correction history is
beginning to specify the automation required at that boundary.

**Product boundary (emerging, from the corpus):**

> A claim-integrity system that mechanically tests whether externally
> presented claims remain correctly bound to their stated artifacts, scope,
> provenance, and observable repository state.

Not a theorem prover. Not an AI truth machine. Not a replacement for expert
adjudication. It operates around the boundary conditions of claims.

**Precision requirement on "mechanically checkable":** every extracted rule
has a mechanically checkable component that can be evaluated without
resolving the underlying scientific domain question. A program can determine
that a symbol does not exist, that a document says 140 while a census produces
141, that a stated arithmetic relationship does not hold, or that a claimed
verification scope exceeds the listed verified files. It does not thereby
determine whether the underlying scientific proposition is true. That boundary
is a feature of the hypothesis, not a limitation.

---

## What the corpus is not

- Not evidence of demand. No purchase intent has been demonstrated anywhere
  in this inventory.
- Not a mature AI-evaluation dataset. It is a machine-checkable correction
  corpus seed with stated coverage (6 of 16 correction-bearing commits).
- Not a product architecture. The next corpus operation is
  extract → classify → test schema → stop. Premature transmutation (corpus →
  ontology → platform) is explicitly out of scope until the four experiments
  return.

---

## Approved experiments (hard cutoff after four)

| # | Action | What it tests | Status |
|---|--------|----------------|--------|
| 1 | Extract adjudication corpus | Whether correction events become a reusable structured asset | **Done 2026-09-27. 6 events, 8 rule categories.** |
| 2 | Census one deliberately boring unrelated public repo, full evidence recording (repo/commit, question, counting rule, raw output, independent check, result, discrepancy) | Whether the mechanical primitive survives outside the originating environment. Null result is still an evidence event. | Pending |
| 3 | Ask the reviewer one question: what was missing or hardest to verify | External requirements interview. "Nothing" is useful negative evidence. | Pending |
| 4 | Instrument TSCP store with a measurable inquiry event | Convert future attention into observable events: visit → request → contact → conversation → proposal. None interpreted as demand until they occur. | Pending |

**Experiment 1 outcome (recorded):** the six events produce heterogeneous,
mechanically checkable predicates — not six variations of one hard-coded
check. Each event can become a regression fixture: input claim → expected
binding → mechanical predicate → observed result → violation → disposition.

---

*No provenance, equivalence, or demand claim should be inferred from this
inventory beyond what each signal state explicitly supports.*
