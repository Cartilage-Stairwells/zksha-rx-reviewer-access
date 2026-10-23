# ARIA Final Audit — review-v0.1.21

**Date:** 2026-10-23 | **Target:** commit 73bf100, tag review-v0.1.21, Cartilage-Stairwells/zksha-rx-reviewer-access
**Mode:** Read-only, no mutation. Tagged artifacts untouched.

## Auditor independence boundary (read first)

This audit was EXECUTED independently (all reproductions run fresh, from a clean clone, against the published tag) but AUTHORED by the same seat that sealed review-v0.1.21. The repository's own documentation was treated as claims under audit, not as evidence. A fully independent countersignature (external Aria or any third party) remains open; this report provides the reproduction procedures and raw outputs so any third party can re-execute them.

## Environment and limitations

- x86_64 AVX-512 sandbox, 4 cores, CPU model masked as "unknown" — hardware identity cannot be independently confirmed; all timing-sensitive verifications were cross-checked against published logs rather than re-timed.
- Toolchain: rustc 1.99.1. Build quirk: an UNTRACKED stub `instrument/runner/main.rs` (`fn main() {}`) is required for `cargo test`; disclosed here as an environment necessity, never committed.
- Two prior attempts to delegate this audit to a fresh background worker stalled at a processing checkpoint with no findings; the audit was then executed directly.

## False alarm recorded and resolved (audit-methodology incident)

The first manifest recompute reported 101/110 with nine "FAILED open or read" entries (including absolute paths under /app/avx512-butterfly/). Root cause: the audit command piped the tracked-file list through `xargs` into `sha256sum -c SHA256SUMS`, so the repo's JSON evidence files were consumed as ADDITIONAL checksum lists, and their internally-referenced paths (absolute paths from the upstream project's original machine) were checked as if listed in the manifest. Falsified by direct blob comparison: the committed SHA256SUMS exactly equals regeneration from `git ls-files` (empty diff; 111 tracked files = 110 covered + SHA256SUMS itself). Corrected check: **110/110 match, 0 mismatches**. No defect in the sealed artifact; defect was in the audit command. Recorded per house discipline: observation → hypothesis → falsification → resolution.

## Findings

### Finding 1 — Benchmark provenance
1. **Claim:** canonical 1.265×–1.276× (Zen 5) derives from published logs; reproduction status is precisely worded (canonical runs not re-executed from the public repo on Zen 5; independent non-Zen 5 run = true-scalar regime 6.99×, gate PASS).
2. **Evidence:** `evidence/canonical/canonical_run1/run2_*.txt`; `evidence/independent/2026-10-23-non-zen5/`; VERIFICATION_STATUS.md §Benchmark Results.
3. **Reproduction:** canonical logs re-parsed independently: run1 geo = 1.2790×, run2 geo = 1.2690×, dual-run delta 0.78% (documented 0.9% — consistent, computed on rounded values). Package's 6.99× re-derived from its published `bench-parsed.json`: 6.988× over 13 sizes, exact match to its README. Gate-PASS line present verbatim in the raw log.
4. **Evidence boundary:** canonical Zen 5 values are supported by published logs only (not re-executed here — different hardware). The 6.99× run was executed on hardware whose model is masked; its hardware identity is unconfirmed.
5. **Unestablished remainder:** a Zen 5 re-execution of the public repo's bench; non-virtualized hardware behavior.
6. **Status: VERIFIED** (as documentation-provenance claims: values faithfully derived from published evidence; wording claims no unperformed reproduction — exactly one qualifying line found, correctly worded).

### Finding 2 — Lane-2 SIMD evidence
1. **Claim:** the disassembly probe (lane 1 + lane 2 addendum) documents that both benchmark lanes auto-vectorize under documented flag sets, with conditions and scoping declared.
2. **Evidence:** `evidence/independent/2026-10-23-non-zen5/disasm-probe.md` (three-config table for lane 1; addendum table for lane 2).
3. **Reproduction:** internal consistency verified: addendum present; declared conditions present (rustc 1.99.1, both flag sets, "probe platform"); per-config register counts present in table form (canonical: 16 zmm / 29 ymm both lanes; no-flags: lane 1 = 0 vector regs, lane 2 = 26 ymm); boundary language present ("do not assert" about other execution paths).
4. **Evidence boundary:** the probe outputs are published records from the sealing seat's own execution on the masked-CPU sandbox; this audit did NOT re-run the disassembly (machine-code claims rest on the published probe record).
5. **Unestablished remainder:** independent third-party re-execution of the disasm probe; Zen 5 codegen verification.
6. **Status: SUPPORTED-PARTIAL** (internally consistent and properly scoped as published evidence; not independently re-executed in this audit).

### Finding 3 — The 139-test census
1. **Claim:** 139 tests = 102 integration + 37 unit; doc-test target 0; specific per-target counts.
2. **Evidence:** VERIFICATION_STATUS.md §Verified Claims.
3. **Reproduction:** full `cargo test` run from the clean clone: **139 passed, 0 failed, 0 ignored**. Per-target measured: lib 37, babybear_domain 12, babybear_montgomery 10, backend_parity 14, core_domain_blindness 8, gate_enforcement 24, iep_enforcement 11, archive/legacy_montgomery_regression 6, ntt_equivalence 4, raw_u32_audit 6, representation_audit 7, doc-tests 0. **All match the documented table exactly.**
4. **Evidence boundary:** executed on this sandbox's hardware (count is hardware-independent for correctness tests).
5. **Unestablished remainder:** none for the census.
6. **Status: VERIFIED** (independently executed in this audit).

### Finding 4 — Historical/tag integrity
1. **Claim:** prior review tags remain immutable and resolvable; HISTORY.md snapshot rows match actual commits.
2. **Evidence:** git refs; HISTORY.md.
3. **Reproduction:** `git ls-remote` — review-v0.1.17 → df5eaac, v0.1.18 → 9e7d3b1, v0.1.19 → 612cd89, v0.1.20 → 84ebf64; HISTORY.md rows list exactly these pairs. Tag identity for v0.1.21: local clone == committed SHA == remote tag target == 73bf100.
4. **Evidence boundary:** current remote state; no historical server logs consulted.
5. **Unestablished remainder:** none observed.
6. **Status: VERIFIED.**

## Seal reproductions

| Item | Result |
|---|---|
| Tag identity | VERIFIED — 73bf100 local, remote, and tag-target all equal |
| Manifest | 110/110 recomputed hashes match (after correcting the audit command's own xargs bug; false alarm documented above) |
| Release gate | Executed: 7 passed, 0 failed, 1 warning — "Tag is not signed — additional trust layer, not required for review" (benign, disclosed) |
| Census re-run | 139/139 passed |

## Final disposition

| Finding | Status |
|---|---|
| Benchmark provenance | VERIFIED |
| Lane-2 SIMD evidence | SUPPORTED-PARTIAL (published-record verification; probe not re-executed here) |
| 139-test census | VERIFIED (independently executed) |
| Historical/tag integrity | VERIFIED |
| Seal (tag, manifest, gate) | VERIFIED (executed) |

**Reproducible discrepancies:** none against the sealed artifact. One audit-methodology false alarm (xargs-induced manifest misread) documented and falsified by blob comparison.

**Overall:** review-v0.1.21's documented state matches the independently reproduced state at every point tested. Open items are all of the "not re-executed on Zen 5 / no third-party countersignature" class — evidence-boundary disclosures, not contradictions. Recommend: hand this report + the tag to the external Aria seat for countersignature; optional future event: a public-repo Zen 5 bench re-execution to close the canonical-reproduction gap.
