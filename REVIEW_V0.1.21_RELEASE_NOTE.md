# review-v0.1.21 Release Note

## Purpose

External audit of review-v0.1.20 found four remaining VERIFICATION_STATUS.md inconsistencies lagging the v0.1.20 baseline clarification. review-v0.1.21 reconciles them in a narrowly scoped documentation-only correction. **No canonical measured value, benchmark result, or proof content is modified.**

## Disposition of audit items

| Audit item | Disposition at v0.1.21 |
|---|---|
| Benchmark Results: "cannot be reproduced from public repository" narrative | RECONCILED — section retitled (canonical runs executed in the private repo; raw logs published in `evidence/canonical/` and independently re-parsed 2026-10-23: 1.279×/1.269×, delta 0.78%). Reproduction status stated precisely: canonical runs not re-executed from the public repo on Zen 5; the regime explanation is disassembly-verified; the independent non-Zen 5 run (gate PASS 2^8–2^20, 6.99×) is published |
| Scope Corrections row retains pre-clarification wording | EXTENDED with the 2026-10-23 machine-code clarification; original corrected wording preserved |
| Benchmark Baselines: "Scalar backend (no SIMD)" + "compiler does not auto-vectorize" note | CORRECTED — new lane-2 disassembly probe (published as addendum to disasm-probe.md): `avx2_butterfly_pass` also auto-vectorizes under documented flag sets (16 zmm + 29 ymm canonical; 26 ymm with no flags). Both lanes compile near-identically under documented flags, explaining the ~1.0× AVX2/Scalar ratio. Codegen claims scoped to the exact probe source, flags, compiler (rustc 1.99.1), and platform; no assertion about other execution paths |
| Verified Claims: 136 total (rows summing to 128), misattributed counts | ALIGNED TO MEASURED CENSUS — per-target counts from the independent 2026-10-23 run (babybear_montgomery 37→10, backend_parity 10→14, iep_enforcement 14→11); unit-test row added (lib target, 37 tests); total 139 = 102 integration + 37 unit; doc-test target (0 tests) noted; measurement provenance stated |
| Historical measurements | PRESERVED — v0.1.11 comparison table and Scope Corrections history retained, explicitly labeled historical |

## Verification at seal

- `./validate_release.sh` passes 7/7 at this tag (from a clean clone); SHA256SUMS regenerated over all tracked files except itself and verified from the clean clone

## Reviewer impact

VERIFICATION_STATUS.md now agrees with the measured census and the machine-code evidence at every claim site. The canonical 1.265×–1.276× claim is unchanged and remains Zen 5-scoped; the ~7×–9× true-scalar regime is documented with published evidence. All prior review tags remain immutable.
