# History

## Current Release

| Field | Value |
|------|-------|
| Tag | `review-v0.1.21` |
| Commit | This tag's target |
| Date | 2026-10-23 |
| Status | Frozen — VERIFICATION_STATUS reconciliation (per-target test counts aligned to measured census; baseline narrative reconciled; lane-2 codegen verified) |

## Previous Reviewer Snapshots

| Tag | Commit | Date | Notes |
|-----|--------|------|-------|
| `review-v0.1.20` | `84ebf64` | 2026-10-23 | Benchmark baseline-definition clarification (disassembly-verified) + independent non-Zen 5 reproduction evidence |
| `review-v0.1.19` | `612cd89` | 2026-10-23 | Reproducibility mapping: canonical benchmark command identified; staged-comparison count adjudicated |
| `review-v0.1.18` | `9e7d3b1` | 2026-10-23 | Measured count-alignment: test counts aligned to independent run (139/139) |
| `review-v0.1.17` | `df5eaac` | 2026-10-23 | Documentation disposition close: license note, formal-verification census wording, evidence-branch command |
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

## review-v0.1.18 (2026-10-23) — Measured count-alignment: test counts aligned to independent run

**Tag:** review-v0.1.18
**Purpose:** An independent `cargo test` run at review-v0.1.17 (2026-10-23, AVX-512 CPU, rustc 1.99.1, default profile, clean clone) measured 12 test targets, 139 tests run, 139 passed, 0 failed, 0 ignored. The reviewer documents' counts did not match this measurement: README and the guide carried an unscoped "102 tests" (exact for the tests/ integration suite, but ambiguous against the 139 a reviewer actually runs), the quickstart said "140 total" (unstated-basis approximation), EVIDENCE_CHAIN's summary carried "140/140" and "33 Lean 4 theorems" (stale pre-census counts), and CLAIM_MATRIX's counting basis contained two arithmetic slips (live-suite count did not deduct commented-out occurrences).

**Changes (count-wording alignment only; no measured value, benchmark result, or proof content modified):**
- Test-count wording aligned to the measured run: 139 run / 139 pass, composition stated (102 integration + 37 unit; 6 archived tests run as their own target; 2 textual occurrences are commented-out code)
- REVIEWER_QUICKSTART.md "140 total" corrected to 139; README evidence row restated with full breakdown
- EVIDENCE_CHAIN.md summary counts aligned (139/139 suite measurement; formal census wording: 88 theorem declarations across 12 Lean files, Montgomery.lean 12)
- CLAIM_MATRIX.md counting-basis cell corrected to the measured composition
- docs/SECURITY_MODEL.md, docs/plonky3/*: "102 tests" scoped as "102 integration tests" where unscoped
- docs/METRICS.md: 2026-10-23 dated rows appended (tests 139/139; Lean census 88); historical 2026-08-03 rows preserved unaltered
- Release identity advanced to review-v0.1.18 across the entry documents
- SHA256SUMS regenerated over all tracked files except itself

**Supersedes:** review-v0.1.17 for reviewer entry (all prior tags remain immutable).

## review-v0.1.19 (2026-10-23) — Reproducibility mapping: canonical benchmark command identified; staged-comparison count adjudicated

**Tag:** review-v0.1.19
**Purpose:** External audit of review-v0.1.18 (2026-10-23) found a reproducibility gap: the canonical 1.27× result is supported by `cargo bench --bench three_lane_bench` (per CLAIM_MATRIX and the canonical methodology), but REVIEWER_QUICKSTART.md, REVIEW_RELEASE.md, and BENCHMARKS.md instructed reviewers to run `cargo bench --bench butterfly_bench` — a benchmark that measures the scalar reference butterfly only and cannot reproduce the canonical three-lane result. The audit also left the "135 staged pairwise comparisons" claim UNADJUDICATED pending verification against underlying evidence.

**Changes (reproducibility-mapping and count-basis wording only; no measured value, benchmark result, or proof content modified):**
- Canonical benchmark command consistently identified across REVIEWER_QUICKSTART.md, REVIEW_RELEASE.md, BENCHMARKS.md (already present in CLAIM_MATRIX.md, CANONICAL_*.md, and the external guide); `butterfly_bench` explicitly labeled as scalar-reference-only; both bench targets compile-verified (`cargo bench --no-run`, 2026-10-23)
- The 135 staged-comparison count adjudicated with a stated counting basis: 3 backends × 45 butterfly stages (log_n 5–10, stages per size = log_n); 90 comparisons asserted directly in `tests/ntt_equivalence.rs::staged_cross_backend_equivalence`, 45 by transitivity. Independently re-run 2026-10-23 on AVX-512 hardware: 45/45 stage logs captured (`ref==scalar==avx512 PASS`), test passed. CLAIM_MATRIX evidence path corrected to name `tests/ntt_equivalence.rs` alongside `tests/backend_parity/`
- Release identity advanced to review-v0.1.19 across the entry documents
- SHA256SUMS regenerated over all tracked files except itself

**Supersedes:** review-v0.1.18 for reviewer entry (all prior tags remain immutable).

## review-v0.1.20 (2026-10-23) — Benchmark baseline-definition clarification (disassembly-verified) + independent non-Zen 5 reproduction evidence

**Tag:** review-v0.1.20
**Purpose:** An independent reproduction of `three_lane_bench` at review-v0.1.19 (2026-10-23, non-Zen 5 AVX-512 hardware, rustc 1.99.1, canonical protocol configuration) passed the correctness gate on all sizes 2^8–2^20 and measured a 6.99× geometric mean AVX-512 vs scalar — a different ratio regime from the canonical 1.265×–1.276× (Zen 5), which faithfully derives from the published canonical logs (independently re-parsed: 1.279×/1.269×, dual-run delta 0.78%). A disassembly probe of the benchmark's scalar-source lane (compiled verbatim under three flag configurations) verified the root cause: the lane AUTO-VECTORIZES to AVX-512/AVX2 machine code under both documented flag sets (canonical: 16 zmm + 29 ymm; quick-start: 23 zmm + 28 ymm); only a no-flags build is true scalar. The canonical 1.27× is therefore the hand-written kernel vs the compiler's auto-vectorization on Zen 5; the ~7×–9× regime is the kernel vs true scalar code. VERIFICATION_STATUS's recorded "cannot reproduce from public repo" note on the 1.27× is thereby reconciled: the public bench reproduces the ~1.27× regime where compiler auto-vectorization is highly effective (Zen 5), and the ~7× regime on other AVX-512 platforms.

**Changes (baseline-definition wording, flag-set alignment, and new non-canonical evidence; no canonical measured value is modified):**
- Baseline codegen clarification appended at every 1.27× claim site (README, PROJECT_FACTS, REVIEWER_QUICKSTART, BENCHMARKS, CANONICAL_RESULTS, CLAIM_MATRIX, external guide, VERIFICATION_STATUS)
- BENCHMARKS.md quick-start flag set aligned to the canonical flag set (CANONICAL_BENCHMARK_METHODOLOGY.md §4) — previously features-only without target-cpu=native, the same doc-inconsistency class as the review-v0.1.19 bench-command fix
- New non-canonical evidence package `evidence/independent/2026-10-23-non-zen5/`: raw Criterion log, parsed per-lane times, measurement record (with observation/hypothesis separation), and the disassembly probe record; CPU model disclosed as masked ("unknown") by the execution environment
- docs/METRICS.md: 2026-10-23 dated row appended; historical rows preserved
- Release identity advanced to review-v0.1.20 across the entry documents
- SHA256SUMS regenerated over all tracked files except itself

**Supersedes:** review-v0.1.19 for reviewer entry (all prior tags remain immutable).

## review-v0.1.21 (2026-10-23) — VERIFICATION_STATUS reconciliation: per-target counts aligned to measured census; baseline narrative reconciled; lane-2 codegen verified

**Tag:** review-v0.1.21
**Purpose:** External audit of review-v0.1.20 (2026-10-23) accepted the baseline-clarification finding and identified four remaining inconsistencies in VERIFICATION_STATUS.md: (1) the Benchmark Results section still described the canonical 1.27× as "not reproducible from the public repository" without the new codegen evidence; (2) the Scope Corrections row retained the pre-clarification baseline description; (3) the Benchmark Baselines section still described the scalar lane as "no SIMD" and asserted the compiler does not auto-vectorize the AVX2 lane loop; (4) the Verified Claims table reported 136 total tests (rows summing to 128) with three misattributed per-target counts, against the measured census of 139.

**Changes (documentation reconciliation only; no measured value, benchmark result, or proof content modified):**
- Verified Claims table: per-target counts aligned to the independent measured run of 2026-10-23 (babybear_montgomery 37→10, backend_parity 10→14, iep_enforcement 14→11; unit-test row added for the lib target's 37 tests; total corrected 136→139 = 102 integration + 37 unit, doc-test target runs 0) with measurement provenance stated
- Benchmark Results: canonical-runs section retitled and reconciled (raw logs published in-repo and independently re-parsed: run1 1.279×, run2 1.269×, delta 0.78%); reproduction status updated — the canonical runs have not been re-executed from the public repo on Zen 5; the non-Zen 5 independent run measured the true-scalar regime (6.99×, gate PASS); baseline table cells re-described per the disassembly evidence
- Baseline Discrepancy: reconciliation note added (the 8.97×/1.27× distinction is a machine-code regime; v0.1.11 comparison table retained as historical)
- Scope Corrections: the "1.27x acceleration" row extended with the 2026-10-23 machine-code clarification
- Benchmark Baselines: scalar row restated (auto-vectorized codegen under documented flags; true scalar only from a no-flags build); the "compiler does not auto-vectorize this loop" note replaced after a NEW lane-2 disassembly probe (avx2_butterfly_pass: 16 zmm + 29 ymm under canonical flags; 26 ymm with no flags) — both lanes compile near-identically under documented flag sets, explaining the ~1.0× AVX2-vs-Scalar ratio; codegen claims scoped to exact source, flags, compiler (rustc 1.99.1), and platform
- disasm-probe.md: lane-2 probe addendum published
- Release identity advanced to review-v0.1.21 across the entry documents; SHA256SUMS regenerated over all tracked files except itself

**Supersedes:** review-v0.1.20 for reviewer entry (all prior tags remain immutable).
