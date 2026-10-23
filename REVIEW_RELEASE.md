# Review Release — zkSHA-Rx Fly v0.1.0

> This file is the single source for release identity.
> A reviewer should not need to reconcile commit history.
> All references in this repository point to the release below.

## Release Identity

| Field | Value |
|-------|-------|
| Release tag | `review-v0.1.19` |
| Source commit | The commit this tag resolves to — verify with `git rev-parse HEAD` |
| Previous snapshot | `review-v0.1.18` / `9e7d3b1` (historical) |
| Repository | https://github.com/Cartilage-Stairwells/zksha-rx-reviewer-access |

## Artifact Inventory

| Artifact | Location | Hash/Ref |
|----------|----------|----------|
| AVX-512 SIMD kernel | `src/avx512_butterfly_32bit.rs` | in release commit |
| Scalar reference | `src/lib.rs` | in release commit |
| Benchmark harnesses | `benches/three_lane_bench.rs` (canonical three-lane), `benches/butterfly_bench.rs` (scalar reference) | in release commit |
| Test suite | `tests/` (14 files) | in release commit |
| Evidence bundle | `evidence/` | in release commit |
| SHA256 sums | `SHA256SUMS` | in release commit |
| Benchmark docs | `BENCHMARKS.md` | in release commit |
| Measurement docs | `MEASUREMENT.md` | in release commit |
| Evidence chain | `EVIDENCE_CHAIN.md` | in release commit |
| Architecture | `ARCHITECTURE.md` | in release commit |
| Canonical facts | `PROJECT_FACTS.md` | in release commit |
| Claim matrix | `CLAIM_MATRIX.md` | in release commit |
| Formal verification | `formal/README.md` | external repo ref |

## Formal Verification Reference

The Lean 4 formalization is in a separate repository:
- **Repository:** Cartilage-Stairwells/tscp-anchor (public)
- **Tag:** `formal-corpus-v2026-09-11` (previous reference `v0.1.0-zksha-rx` does not exist in tscp-anchor; repaired 2026-09-11 — see HISTORY)
- **Lean version:** 4.32.1
- **Build command:** `lake build`
- **Theorem count:** 88 theorem declarations across 12 Lean files (census at tag `formal-corpus-v2026-09-11`, 2026-09-11; method: theorem-declaration grep)
- **See:** `formal/README.md` in this repo for details

## Reproduction Commands

```bash
# Clone this repository
git clone https://github.com/Cartilage-Stairwells/zksha-rx-reviewer-access.git
cd zksha-rx-reviewer-access
git checkout review-v0.1.19

# Build
cargo build --release

# Run tests
cargo test --release

# Run the canonical three-lane benchmark (reproduces the 1.27x kernel claim; requires AVX-512 CPU)
cargo bench --bench three_lane_bench

# Verify evidence integrity
sha256sum -c SHA256SUMS
```

## Out-of-Tree Benchmark Evidence (Branch-Resident)

The raw benchmark evidence artifacts are **not in the main tree**; they live
on a separately pinned evidence branch and are verified with their own
SHA256SUMS manifest. `sha256sum -c SHA256SUMS` above validates only the
main-tree files; complete evidence verification also requires:

```bash
git fetch origin evidence/experiment-a
git checkout 8df0c247cc503b751ee91245bb952b6994aaf2e6
cd evidence/experiment_a && sha256sum -c SHA256SUMS
```

- **Branch:** `evidence/experiment-a`
- **Pinned commit:** `8df0c247cc503b751ee91245bb952b6994aaf2e6`
- **Path:** `evidence/experiment_a/` — SHA256SUMS-pinned artifacts
- **Note:** the evidence manifest paths are relative to its own directory; the verification command must run from inside `evidence/experiment_a/` (as shown above)

See `BENCHMARKS.md` and `CLAIM_MATRIX.md` for the claims this evidence supports.

## Relationship to Source Repository

This repository is a **frozen reviewer snapshot**. It is not the development repository.

- The development repository (`avx512-butterfly`) is private and contains additional operational documents not relevant to technical review.
- This snapshot contains everything needed to evaluate the technical claims.
- The snapshot will not change. If the project evolves, a new review tag (e.g., `review-v0.2.0`) will be created.
