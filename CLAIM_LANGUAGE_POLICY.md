# Claim Language Policy

This document defines the epistemic qualifiers used across zkSHA-Rx Fly documentation. Every performance, verification, or architectural claim should use one of these terms.

## Definitions

| Term | Meaning | Use When |
|------|---------|----------|
| **verified** | Directly reproduced in the current environment | Compilation results, test outputs, formal proof compilation, deterministic artifact checks |
| **measured** | Observed benchmark result with documented configuration | Criterion output, hardware-dependent results, kernel-level speedups |
| **reported** | Historical evidence not independently reproduced from this snapshot | Prior benchmark numbers, measurements from unavailable tooling |
| **claimed** | Project assertion requiring external validation | Architecture intent, future work, unsupported boundaries |
| **designed to** | Architecture intent, not yet demonstrated | System goals, correctness patterns, verification strategy |

## Examples

✅ "Lean formalization compiles with zero unsolved goals." → **verified**
✅ "Measured benchmark results report up to 9.15× speedup under the documented benchmark configuration." → **measured**
✅ "Historical benchmark records report a 9.15× kernel speedup; reproduction requires the original hardware and toolchain configuration." → **reported**
✅ "The three-backend approach is designed to close the semantic drift loophole." → **designed to**

❌ "AVX-512 provides 9.15× speedup" (universal claim — not qualified)
❌ "The implementation achieves 9.15× speedup" (universal claim — not qualified)
❌ "The system guarantees correctness" (stronger than evidence supports)

## Scope

This policy applies to all documents in this repository:
- README.md, REVIEWER_QUICKSTART.md, REVIEW_RELEASE.md
- BENCHMARKS.md, MEASUREMENT.md, EVIDENCE_CHAIN.md
- CLAIM_MATRIX.md, PROJECT_FACTS.md, ARCHITECTURE.md
- formal/README.md, HISTORY.md

Any future document added to this repository must follow this policy.
