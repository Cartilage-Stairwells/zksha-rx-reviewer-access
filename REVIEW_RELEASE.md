# Review Release — zkSHA-Rx Fly v0.1.0

> This file is the single source for release identity.
> A reviewer should not need to reconcile commit history.
> All references in this repository point to the release below.

## Release Identity

| Field | Value |
|-------|-------|
| Release tag | `review-v0.1.23` |
| Source commit | The commit this tag resolves to — verify with `git rev-parse HEAD` |
| Previous snapshot | `review-v0.1.22` / `7f342ee` (historical) |
| Release signature | Seat identity `CAF6A411CD9AD65E2557E3DA7C65D2F3BA9969AD`, pinned in `gate-amendment/allowlist.pinned.txt` |
| Repository | https://github.com/Cartilage-Stairwells/zksha-rx-reviewer-access |

## Change in this snapshot (review-v0.1.23)

The release signature check is now **enforced**, not advisory. `validate_release.sh`
calls `gate-amendment/check_release_signature.sh`, which fails the release unless all of
the following hold: the tag is an annotated tag object; its OpenPGP signature verifies;
the verifying key's fingerprint is on the pinned release allowlist; the tag target is
reachable from the release branch; and the allowlist itself is configured. Every unmet
condition is reported, and none of them is a warning.

The identity that signs releases from this snapshot on is a **seat** identity, not a
personal one: `TSCP Release Seat (custodian) <seat@solene.local>`, fingerprint
`CAF6A411CD9AD65E2557E3DA7C65D2F3BA9969AD`. Historical signatures made with the two retired personal identities remain valid
for verifying tags that already exist; they can never authorise a release again, and the
gate reports an identity appearing on both lists as a fatal contradiction.

Two honest limitations, recorded rather than implied:

1. **Custody is provisional.** The seat's secret half currently sits in the agent
   sandbox that produced this snapshot, not in owner-controlled storage. A key held by
   whoever operates the release automation is not independently controlled, which is what
   the transition record requires. Rotating to an owner-held key is the follow-on step,
   and the retirement protocol in `KEY-TRANSITION-AND-IDENTITY-RETIREMENT-DRAFT.md`
   already covers how an identity is retired forward.
2. **A signature attests continuity of an authorised identity.** It says nothing about
   authorship by a named person and nothing about the correctness of what it signs.

`gpg` is required to verify the signature. If it is absent the check fails closed by
design rather than passing quietly.

## Change in this snapshot (review-v0.1.22)

`review-v0.1.21` was tagged at `73bf100`. The commit that followed it, `57568f3`
(Apache-2.0 LICENSE), moved the default branch tip past the tag, and the release gate
requires the checked-out commit to be exactly at a tag. A reviewer who cloned and ran
`make reproduce` without first checking out the tag therefore saw `RELEASE VALIDATION
FAILED` on an otherwise intact package. This snapshot tags the tip, so a fresh clone
validates, and pins the toolchain to `rustc 1.99.1`, the compiler the recorded
measurements and the 139-test census were produced with. No source, test, benchmark
result, proof, or measured value is changed.

`SHA256SUMS` is regenerated over all tracked files except itself, which also restores
`LICENSE` to the manifest: it was added at `57568f3` without a manifest update.

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
git checkout review-v0.1.22

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
