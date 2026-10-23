# review-v0.1.20 Release Note

## Purpose

An independent reproduction of the canonical benchmark at review-v0.1.19 surfaced a reproducibility gap in the 1.27× claim's baseline definition. review-v0.1.20 clarifies the baseline with machine-code evidence, aligns the quick-start flag set with the canonical methodology, and publishes the independent run as non-canonical evidence. **No canonical measured value is modified.** The 1.265×–1.276× claim itself is unchanged and remains Zen 5-scoped.

## What was found (2026-10-23)

1. **Independent reproduction (non-Zen 5 AVX-512 hardware, rustc 1.99.1, canonical protocol configuration, clean clone of review-v0.1.19):** correctness gate PASS on all sizes 2^8–2^20; AVX-512 vs scalar geometric mean **6.99×** — a different ratio regime from the canonical 1.27×.
2. **Canonical logs re-parsed independently:** run1 = 1.279×, run2 = 1.269×, dual-run delta 0.78% — the published 1.265×–1.276× faithfully derives from the evidence; the canonical claim is internally consistent.
3. **Disassembly probe (decisive):** the benchmark's scalar-source lane, compiled verbatim under the documented flag sets, contains AVX-512/AVX2 vector instructions (canonical flags: 16 zmm + 29 ymm; quick-start flags: 23 zmm + 28 ymm). Only a no-flags build produces true scalar machine code. **The "scalar" baseline is compiler-auto-vectorized code, not scalar machine code.**
4. Consequently both published regimes are now explained and reconciled: the canonical **1.27× = hand-written kernel vs the compiler's auto-vectorization on Zen 5** (where auto-vectorization is highly effective); the **~7×–9× regime = hand kernel vs true scalar code** (this run: 6.99×; the earlier single-size Zen 5 experiment on the evidence branch: 8.67×). VERIFICATION_STATUS's prior note that the 1.27× "cannot reproduce from public repo" is updated: the public bench reproduces the 1.27× regime on Zen 5-class codegen and the ~7× regime on other AVX-512 platforms.

## Changes

- Baseline-definition clarification appended at every 1.27× claim site; the claim's wording now states what the baseline's machine code actually is
- BENCHMARKS.md quick-start flags aligned to the canonical flag set (same doc-inconsistency class as the review-v0.1.19 bench-command fix)
- New non-canonical evidence: `evidence/independent/2026-10-23-non-zen5/` (raw log, parsed times, measurement record with observation/hypothesis separation, disassembly probe record; CPU model disclosed as masked by the execution environment)
- Release identity advanced to review-v0.1.20; all prior tags remain immutable

## Verification at seal

- `./validate_release.sh` passes 7/7 at this tag (from a clean clone); SHA256SUMS regenerated over all tracked files except itself and verified from the clean clone

## Reviewer impact

The reviewer package now distinguishes the two comparison regimes explicitly. The canonical claim is unchanged: hand-written AVX-512 kernel vs compiler auto-vectorization on AMD Zen 5, 1.265×–1.276× geometric mean, dual-run verified. Against true scalar code the kernel measures ~7×–9× (evidence included). Universal acceleration across AVX-512 microarchitectures remains a non-claim.
