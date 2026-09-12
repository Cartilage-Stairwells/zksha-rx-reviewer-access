# B2 Dependency-Resolution Record

**Date:** 2026-09-12
**Branch:** feat/b1-integration-boundary
**Operation:** Authorized B2 dependency mutation (execution-path evidence)

## Sequence preserved by this record

> pre-existing B1 state → authorized dependency mutation → compiled pipeline → runtime proving evidence

## Pre-existing B1 state (before this commit)

Manifest dependencies (B1, commit 69be906):
sha2 0.10, serde_json 1, p3-baby-bear 0.1, p3-field 0.1, p3-dft 0.1, p3-matrix 0.1, rand 0.8

Resolved p3-* graph (pre-mutation): 9 crates, all 0.1.0
(p3-baby-bear, p3-dft, p3-field, p3-matrix, p3-maybe-rayon, p3-mds, p3-poseidon2, p3-symmetric, p3-util)

Note: the branch's Cargo.lock was stale relative to the B1 manifest (B1 added
p3-dft/p3-matrix to Cargo.toml without regenerating the lock — cargo regenerated
on demand at build time). This commit replaces it with a lock that matches the
post-mutation manifest exactly.

## Authorized dependency mutation (this commit)

Added the minimum coherent Plonky3 0.1.0 proving-pipeline dependencies:
p3-air 0.1, p3-challenger 0.1, p3-commit 0.1, p3-fri 0.1, p3-merkle-tree 0.1,
p3-poseidon2 0.1, p3-symmetric 0.1, p3-uni-stark 0.1, p3-util 0.1

## Post-mutation resolution (verified three ways)

Resolved p3-* graph: 16 crates, ALL 0.1.0, single coherent version family.
One `p3-dft` in the graph — one `TwoAdicSubgroupDft` trait universe.

1. crates.io dependency metadata: p3-fri 0.1.0 → p3-dft ^0.1.0 (likewise
   p3-commit, p3-merkle-tree, p3-challenger, p3-air, p3-uni-stark — all ^0.1.0).
2. cargo resolution: all 16 p3-* crates resolve to 0.1.0; no family split.
3. cargo build: entire pipeline compiles clean on rustc 1.97.1 (~9s).

Injection point (verified from source): `p3-fri 0.1.0::two_adic_pcs`:
`TwoAdicFriPcs<Val, Dft, InputMmcs, FriMmcs>` holds `dft: Dft` with the sole
bound `Dft: TwoAdicSubgroupDft<Val>` — the exact trait `ZkshaDifAdapter`
implements. No type-universe crossing.

## Superseded concern (recorded for provenance)

A pre-authorization report claimed p3-fri 0.1.0 was tied to a 0.2-family
dependency graph. Verification against crates.io metadata, cargo resolution,
and actual compilation showed all 0.1-era proving crates pin ^0.1.0 families.
The discrepancy traces to the live Plonky3 monorepo (now 0.7-era unified
versioning), which does not describe the published 0.1.0 snapshot that cargo
resolves against. No B2 mutation occurred before this was resolved.
