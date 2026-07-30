# History

## Current Release

| Field | Value |
|------|-------|
| Tag | `review-v0.1.4` |
| Commit | This tag's target |
| Date | 2026-07-30 |
| Status | Frozen — stale checkout command fixed |

## Previous Reviewer Snapshots

| Tag | Commit | Date | Notes |
|-----|--------|------|-------|
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
