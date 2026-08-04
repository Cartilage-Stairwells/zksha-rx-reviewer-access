# History

## Current Release

| Field | Value |
|------|-------|
| Tag | `review-v0.1.6` |
| Commit | This tag's target |
| Date | 2026-07-30 |
| Status | Frozen — validated clean snapshot (supersedes local v0.1.5 tag drift) |

## Previous Reviewer Snapshots

| Tag | Commit | Date | Notes |
|-----|--------|------|-------|
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
2. **Benchmark evidence updated:** Historical 9.15× DIT numbers replaced with measured 2.65× DIF three-lane benchmark results (AVX-512 vs scalar, Intel AVX-512, geometric mean).
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

