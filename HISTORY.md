# History

## Current Release

| Field | Value |
|------|-------|
| Tag | `review-v0.1.17` |
| Commit | This tag's target |
| Date | 2026-10-23 |
| Status | Frozen — documentation disposition close (license note, census wording, evidence command; no measured values changed) |

## Previous Reviewer Snapshots

| Tag | Commit | Date | Notes |
|-----|--------|------|-------|
| `review-v0.1.16` | `904423b` | 2026-10-23 | Integrity re-seal: release identity aligned across reviewer docs; out-of-tree evidence commands added |
| `review-v0.1.15` | `ac90de5` | 2026-09-12 | docs(review-v0.1.15): theorem-count alignment to stated census + phant |
| `review-v0.1.14` | `11c1690` | 2026-09-12 | Integrity re-seal for reviewer release review-v0.1.14: regenerate SHA2 |
| `review-v0.1.13` | `01db486` | 2026-08-05 | review-v0.1.13: documentation corrections |
| `review-v0.1.12` | `cbbc19d` | 2026-08-05 | review-v0.1.12: terminology clarification |
| `review-v0.1.11` | `4789695` | 2026-08-05 | review-v0.1.11: verification repair |
| `review-v0.1.10` | `116eef0` | 2026-08-05 | Add canonical AVX-512 benchmark methodology and results |
| `review-v0.1.9` | `767a52a` | 2026-08-04 | release: review-v0.1.9 — evidence compounding layer |
| `review-v0.1.8` | `5cbb669` | 2026-08-04 | release: review-v0.1.8 — researcher-consumable release |
| `review-v0.1.7` | `fe19b1b` | 2026-08-04 | release: review-v0.1.7 — DIF verification snapshot |
| `review-v0.1.6` | `7ff9825` | 2026-07-30 | release: review-v0.1.6 validated clean snapshot |
| `review-v0.1.5` | `0ea4eff` | 2026-07-30 | Claims language normalization + CLAIM_LANGUAGE_POLICY.md |
| `review-v0.1.4` | `bda165b` | 2026-07-30 | Stale checkout command fixed |
| `review-v0.1.3` | `4e2d404` | 2026-07-30 | Identity alignment + validation gate |
| `review-v0.1.2` | `6f7cfd8` | 2026-07-30 | Evidence closure: SHA256SUMS, dead paths, Criterion version |
| `review-v0.1.1` | `9edec5f` | 2026-07-30 | Evidence symbol fix (stale `avx512_radix2_butterfly` → `avx512_radix2_butterfly_32`) |
| `review-v0.1.0` | `e3d49fc` | 2026-07-29 | Initial frozen reviewer snapshot |

## Historical Development Artifacts (Not In This Tree)

These artifacts exist in the development repository and are referenced for provenance only. They are not reproducible from this snapshot.

| Artifact | Commit/Tag | Status |
|----------|------------|--------|
| v0.1.0-zksha-rx | `eb5533d` | Development repo tag (pre-review snapshots) |
| vep-0.1.4-sealed | `e41096f` | TSCP formal seal |
| AVX-512 refinement receipt | `9473af6` | Benchmark measurements (Criterion 0.5.1) |
| Montgomery bridge (PR #28) | `68911609` | Scalar CIOS correspondence |
| NTT semantic drift fix | `78c040f` | DIT→DIF butterfly correction |
| Custody plane closure | `49d52bd` | Acceptance harness + firewall + Lean invariants |

## Benchmark Version History

| Version | Context |
|---------|---------|
| Criterion 0.5.1 | Historical measurements (9.15×, 4.58×, 3.94×) |
| Criterion 0.8.2 | This snapshot's Cargo.toml — bench harness available for re-measurement on AVX-512 hardware |

## review-v0.1.7 (2026-08-04) — DIF verification snapshot

**Tag:** review-v0.1.7
**Supersedes:** review-v0.1.6 (kept immutable as historical provenance)

### Changes from v0.1.6

1. **Documentation corrected:** All "DIT" references updated to "DIF" across README, CLAIM_MATRIX, PROJECT_FACTS, REVIEWER_QUICKSTART, BENCHMARKS.
2. **Benchmark evidence updated:** Historical 9.15× DIT numbers replaced with measured 2.65× DIF three-lane benchmark results (AVX-512 vs scalar, AMD Zen 5 with AVX-512, geometric mean).
3. **New evidence artifacts:** Correctness receipt, three-lane benchmark output, CPU info.
4. **Three-lane benchmark added:** `benches/three_lane_bench.rs` with correctness gate.
5. **Verification story:** The DIT→DIF mismatch was a development-repo regression (not present in the v0.1.6 implementation code, but present in v0.1.6 documentation). The implementation code in v0.1.6 already used DIF semantics. The development repo (avx512-butterfly) had a regression to DIT that was discovered via NTT correctness testing, corrected, and merged via PR #14 (commit 71bcfe7, GitHub-verified).

### What didn't change

- Core implementation code (src/) — already DIF in v0.1.6
- Formal verification (formal/) — unchanged
- Evidence chain structure — same model, updated numbers


## review-v0.1.8 (2026-08-04) — Researcher-consumable release

**Tag:** review-v0.1.8
**Supersedes:** review-v0.1.7 (kept immutable)

### Changes from v0.1.7

1. **Plonky3 integration proposal** (`docs/plonky3/VERIFIED_NTT_INTEGRATION_PROPOSAL.md`) — integration path, compatibility matrix, benchmark plan, proof-to-code map
2. **External reviewer guide** (`docs/EXTERNAL_REVIEWER_GUIDE.md`) — build/test/bench instructions, claims vs non-claims, repository structure
3. **Proof-to-code correspondence map** (`docs/plonky3/proof-to-code-map.md`) — 83 Lean theorems mapped to Rust implementations across 4 layers
4. **README rewritten** as researcher landing page
5. **Makefile** added: `make reproduce` (validate + verify), `make test`, `make bench`
6. **Repository structure** reorganized for external consumption

### Goal

A researcher should understand the project in 10 minutes without needing the author present.


## review-v0.1.9 (2026-08-04) — Evidence compounding layer

**Tag:** review-v0.1.9
**Supersedes:** review-v0.1.8 (kept immutable)

### What v0.1.8 says

> "Here is the artifact."

### What v0.1.9 says

> "Here is how the artifact will be evaluated, challenged, and maintained."

### New documents

- `docs/SECURITY_MODEL.md` — precise verification boundary: what formal verification eliminates, what remains outside the proof, assumptions, verified vs tested vs benchmarked
- `docs/REVIEW_RESPONSES.md` — response tracking protocol with prepared answers to likely questions
- `docs/EXTERNAL_EVALUATION_LOG.md` — structured evaluation timeline
- `docs/INTEGRATION_QUESTIONS.md` — open + anticipated integration questions
- `docs/METRICS.md` — external/technical/integration/performance tracking
- `docs/TECHNICAL_OBJECTIONS.md` — 7 prepared responses to likely technical objections

### Narrative

v0.1.8 is the artifact. v0.1.9 is the evaluation infrastructure. Together they constitute the minimum surface a serious cryptography project presents to external reviewers.


## review-v0.1.14 (2026-09-11) — Integrity re-seal

**Tag:** review-v0.1.14
**Purpose:** Mechanical release-gate alignment following the 2026-09-11 public boundary audit. The v0.1.10–v0.1.13 releases updated claim documents without regenerating the integrity manifest, leaving the newest claims outside a passing mechanical gate.

**Changes (bookkeeping only — no claim content modified):**
- `SHA256SUMS` regenerated over all tracked files
- `REVIEW_RELEASE.md` release identity advanced to `review-v0.1.14` (previous: `review-v0.1.13` / `01db486`, historical)
- Quick-start pointers in `README.md` and `docs/EXTERNAL_REVIEWER_GUIDE.md` updated from `review-v0.1.8` to `review-v0.1.14`
- `VERIFICATION_STATUS.md` last-updated line advanced to `review-v0.1.14`
- This release note added

**Claims:** unchanged from `review-v0.1.13` (canonical 1.265×–1.276× AVX-512 DIF butterfly; 102 tests; formal verification per `CLAIM_MATRIX.md`).

**Supersedes:** `review-v0.1.13` for reviewer entry (all prior tags remain immutable).

## review-v0.1.15 (2026-09-11) — Theorem-count alignment to stated census + phantom-tag repair

**Tag:** review-v0.1.15
**Purpose:** The release carried three mutually inconsistent theorem counts (README headline 83 vs its own layer table summing 60; PROJECT_FACTS "33 across 7 modules"; REVIEW_RELEASE "33") and referenced a non-existent tscp-anchor tag (`v0.1.0-zksha-rx`). Counts are now aligned to a stated, reproducible census.

**Census (2026-09-11, by Solene, Base44 Superagent):** theorem-declaration grep at tscp-anchor commit `4a4d6064` (pinned as tag `formal-corpus-v2026-09-11`): **88 declarations across 12 Lean files**. Core layers: ReviewerSemantics.lean 15, Montgomery.lean 12 (compile-verified, 0 axioms, 0 sorries), Butterfly.lean 27, NTTStage.lean 8. The prior "83" figure matches no census and is superseded; "33 across 7 modules" matched no current corpus.

**Changes:**
- `README.md`, `PROJECT_FACTS.md`, `REVIEW_RELEASE.md`: theorem counts replaced with the census-backed statement (method + reference stated)
- `REVIEW_RELEASE.md`: phantom tag `v0.1.0-zksha-rx` replaced with the real pinned tag `formal-corpus-v2026-09-11` (same repair class as the v0.1.11 non-existent-tag fix, recorded in VERIFICATION_STATUS)
- `VERIFICATION_STATUS.md`: "What Lean Does NOT Prove" corrected — butterfly/NTT Lean declarations exist (27/8); compile verification is asserted only for Montgomery.lean
- Release identity advanced to `review-v0.1.15`; SHA256SUMS regenerated over all tracked files

**Claims otherwise unchanged from review-v0.1.14** (canonical 1.265×–1.276× AVX-512 DIF butterfly; 102 tests; formal verification per CLAIM_MATRIX.md). No new verification claims are made by this release — compile status beyond Montgomery.lean is deliberately not asserted.

**Supersedes:** review-v0.1.14 for reviewer entry (all prior tags remain immutable).

## Integrity re-seal (2026-09-27) — CANONICAL_BENCHMARK_METHODOLOGY.md manifest repair

**Not a release.** Post-release custody correction; no claim content modified.

**What happened:** Commit `b17b4127` (owner-approved documentation adjudication, 2026-09-17) amended `CANONICAL_BENCHMARK_METHODOLOGY.md` (methodology status line and Phase 2.5 scope labeling) but omitted the corresponding `SHA256SUMS` update. The manifest consequently carried the pre-commit hash (`bbc0aff5`, the file at `b17b4127^`) for the post-commit file (`3835ccae`) from 2026-09-17 onward. No exception was documented; the mechanical release gate (`validate_release.sh`, `sha256sum -c SHA256SUMS`) never ran in that window because no release tag followed `b17b4127`.

**Detection:** 2026-09-27, by a full manifest-vs-tree recomputation (98 entries; 97 matched) during the EXP-002 audit. The recomputation was performed independently of the release gate. Investigation record preserved; classification: ordinary missed re-seal, recurring the failure class documented at review-v0.1.14 (v0.1.10–v0.1.13 instance, discovered by external audit 2026-09-11). This is the first instance of the class detected mechanically.

**Repair:** Commit `ae4e62bc` (2026-09-27) updated the single stale manifest entry to the current file hash. Bookkeeping only. No scientific or benchmark claim was changed by the repair. `b17b4127` is not altered; all prior tags remain immutable. Full manifest recomputation after repair: 98/98 entries match.

**Adjudicated consequence (recorded, not yet implemented):** the same-commit re-seal invariant — a commit that modifies a manifest-covered file must update `SHA256SUMS` in the same commit set — is SUPPORTED by two observed instances of this failure class (v0.1.10–v0.1.13; b17b4127). Enforcement architecture is a separate design decision.


## review-v0.1.16 (2026-10-23) — Release-identity re-seal: reviewer docs alignment + out-of-tree evidence discoverability

**Tag:** review-v0.1.16
**Purpose:** External audit (2026-10-23) found the reviewer entry documents inconsistent: README targeted `review-v0.1.15` while PROJECT_FACTS.md, its canonical external-reference list, and REVIEWER_QUICKSTART.md still identified `review-v0.1.6`; HISTORY.md's Current Release table was likewise stale. Intermediate fixes on main also touched sealed files without regenerating SHA256SUMS. This release re-seals the reviewer package.

**Changes (bookkeeping only, no claim changes):**
- Release identity advanced to `review-v0.1.16` across README.md, docs/EXTERNAL_REVIEWER_GUIDE.md, REVIEWER_QUICKSTART.md, PROJECT_FACTS.md, REVIEW_RELEASE.md, VERIFICATION_STATUS.md, HISTORY.md
- Approved repository-access wording aligned with the file's own do-not-use policy ("publicly accessible" → "accessible to anyone")
- REVIEW_RELEASE.md + REVIEWER_QUICKSTART.md: out-of-tree benchmark evidence branch (`evidence/experiment-a` @ `8df0c24`) verification commands added — the release gate validates main-tree files only
- HISTORY.md snapshot table completed (review-v0.1.6 through review-v0.1.15 rows)
- SHA256SUMS regenerated over all tracked files

**Supersedes:** review-v0.1.15 for reviewer entry (all prior tags remain immutable).

## review-v0.1.17 (2026-10-23) — Documentation disposition close: license note, census wording, out-of-tree evidence command

**Tag:** review-v0.1.17
**Purpose:** External audit (2026-10-23) of review-v0.1.16 confirmed the release-identity correction but left two unresolved defects: PROJECT_FACTS.md claimed a license file absent from the tree and manifest, and docs/EXTERNAL_REVIEWER_GUIDE.md claimed "83 theorems, 0 axioms, 0 sorries — no unproven gaps," overstating the documented evidence. Mechanical pre-flight of the documented verification commands additionally found the out-of-tree evidence check fails as written (manifest paths are relative to the manifest's own directory) and the review-v0.1.16 release note was not covered by the SHA256SUMS it shipped with.

**Changes (claim-scope and documentation corrections; no measured value, benchmark result, or proof content modified):**
- PROJECT_FACTS.md license note aligned to the recorded licensing deferral (PROVENANCE_AND_LICENSING.md): no repository-wide license file is carried; licensing deferred pending file-level provenance
- docs/EXTERNAL_REVIEWER_GUIDE.md: "83 theorems, 0 axioms, 0 sorries — no unproven gaps" replaced with evidence-scoped wording — Montgomery.lean core: 12 theorems, 0 axioms, 0 sorries (compile-verified); corpus census at formal-corpus-v2026-09-11: 88 theorem declarations across 12 Lean files (independently reproduced by theorem-declaration grep at this release); 3 documented sorries in example modules; corpus-wide sorry-freedom not claimed
- REVIEWER_QUICKSTART.md: "self-contained" wording reconciled with the branch-resident evidence; review-tag reference advanced
- REVIEW_RELEASE.md, PROJECT_FACTS.md, REVIEWER_QUICKSTART.md: out-of-tree evidence verification command corrected to run from the manifest directory (12/12 entries verify at pinned commit 8df0c247 — independently verified at this release)
- REVIEW_V0.1.16_RELEASE_NOTE.md brought under SHA256SUMS coverage (recurrence of the missed-coverage class documented 2026-09-27; no prior tag altered)
- Release identity advanced to review-v0.1.17 across README.md, docs/EXTERNAL_REVIEWER_GUIDE.md, REVIEWER_QUICKSTART.md, PROJECT_FACTS.md, REVIEW_RELEASE.md, VERIFICATION_STATUS.md, HISTORY.md
- SHA256SUMS regenerated over all tracked files except itself

**Supersedes:** review-v0.1.16 for reviewer entry (all prior tags remain immutable).
