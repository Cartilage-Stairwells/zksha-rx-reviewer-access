# Verification Status

> This document is the reference for what is verified, what is
> partially verified, and what remains pending. It is maintained alongside
> the evidence artifacts and updated whenever the verification state changes.
>
> **Last updated:** August 5, 2026 (review-v0.1.11)
> **Reviewer:** Independent cold-read verification completed August 5, 2026

---

## Verified Claims

Claims that can be independently reproduced from a clean checkout of this
repository on AVX-512-capable hardware.

| Claim | Status | Evidence |
|-------|--------|----------|
| AVX-512 execution path runs | ✅ Verified | `is_avx512_supported()` canary + benchmark execution |
| Scalar / AVX2 / AVX-512 lane equivalence | ✅ Verified | Correctness gate in `three_lane_bench.rs` (PASS, 2^8–2^20) |
| NTT stage equivalence (scalar vs AVX-512) | ✅ Verified | `tests/ntt_equivalence.rs` (4 tests, 0 failures) |
| Montgomery arithmetic correctness | ✅ Verified | `tests/babybear_montgomery.rs` (37 tests, 0 failures) |
| Montgomery arithmetic formalization | ✅ Verified | `tscp-anchor` repo, Montgomery.lean (12 theorems, 0 sorry, 0 axioms) |
| BabyBear field domain correctness | ✅ Verified | `tests/babybear_domain.rs` (12 tests, 0 failures) |
| Backend parity (scalar vs reference) | ✅ Verified | `tests/backend_parity/` (10 tests, 0 failures) |
| Representation audit (u32 Montgomery) | ✅ Verified | `tests/representation_audit.rs` (7 tests, 0 failures) |
| Raw u32 audit (no implicit modular ops) | ✅ Verified | `tests/raw_u32_audit.rs` (6 tests, 0 failures) |
| Gate enforcement (evidence contracts) | ✅ Verified | `tests/gate_enforcement.rs` (24 tests, 0 failures) |
| Core domain blindness | ✅ Verified | `tests/core_domain_blindness.rs` (8 tests, 0 failures) |
| Legacy Montgomery regression | ✅ Verified | `tests/archive/legacy_montgomery_regression.rs` (6 tests, 0 failures) |
| IEP enforcement | ✅ Verified | `tests/iep_enforcement.rs` (14 tests, 0 failures) |

**Total: 136 tests, 0 failures, 0 ignored**

---

## Benchmark Results

### Independently Reproduced from Public Repo (v0.1.11)

The three-lane benchmark was run from the public repository using the benchmark
protocol (50 samples, 2s measurement, Criterion 0.8.2).

| Metric | Value | Baseline |
|--------|-------|----------|
| AVX-512 vs Scalar (geometric mean) | 8.97x | Reference arithmetic (`babybear_mul_reference`) |
| AVX2 vs Scalar (geometric mean) | 1.00x | Reference arithmetic (no auto-vectorization benefit) |
| Correctness gate | PASS | All three lanes agree, 2^8 through 2^20 |
| Per-size AVX-512 speedup range | 7.21x to 10.13x | Against reference arithmetic |

### Results — Optimized Scalar Baseline (from private development repo, not reproducible from public repo)

The following results were produced from the private development repository
(commit daf2a74, crate version 0.2.0) and are included as evidence artifacts
in `evidence/`. They CANNOT be reproduced from the public repository
because the scalar baseline differs (see Baseline Discrepancy below).

| Metric | Value | Baseline |
|--------|-------|----------|
| AVX-512 vs Scalar (geometric mean) | 1.265x–1.276x | Optimized scalar (private repo) |
| AVX2 vs Scalar (geometric mean) | 1.062x–1.075x | Optimized scalar (private repo) |
| Dual-run variance | 0.9% | — |

### Baseline Discrepancy

The public repository and the private development repository use different scalar
baselines for the three-lane benchmark:

| Implementation | Public repo (v0.1.11) | Private repo (v0.2.0) |
|---------------|----------------------|----------------------|
| Scalar baseline | `babybear_mul_reference` (64-bit modular multiply) | Optimized scalar (details in private repo) |
| Scalar time at 2^8 | 433.6 ns | 158.1 ns |
| AVX-512 time at 2^8 | 60.1 ns | 53.9 ns |
| AVX-512/Scalar ratio | 7.21x | 2.93x |

The AVX-512 kernel times are similar (~54–60 ns at 2^8), confirming the same
kernel is being measured. The speedup ratio differs because the scalar
baseline is ~2.7x slower in the public repo.

**The 8.97x and 1.27x numbers are both valid measurements — they compare
against different baselines and must not be conflated.** The 8.97x measures
the kernel against a naive reference; the 1.27x measures it against an
optimized scalar implementation.

---

## Reproducibility Notes

| Item | Status | Notes |
|------|--------|-------|
| Public clean checkout | ✅ Repaired (v0.1.11) | Module declarations added to `lib.rs`; `[[bench]]` declaration added for `three_lane_bench` |
| Benchmark reproduction | ✅ Repaired (v0.1.11) | Benchmark compiles and runs from public repo; config updated to benchmark protocol (50 samples, 2s) |
| SHA256SUMS manifest | ✅ Repaired (v0.1.11) | Regenerated to include all files at current commit |
| Missing binary source | ✅ Repaired (v0.1.11) | `iep_runner` `[[bin]]` declaration removed from `Cargo.toml` |
| 1.27x reproduction (optimized scalar baseline) | ⚠️ Cannot reproduce from public repo | Scalar baseline differs between public and private repos |
| Phase 2.5 Plonky3 integration results | ⚠️ Internal | 39% DFT / 11% total proof speedup; evidence in private repo pending IP review |
| tscp-anchor tag reference | ✅ Repaired (v0.1.11) | `formal/README.md` updated to remove non-existent tag reference |
| Theorem count reference | ✅ Repaired (v0.1.11) | Updated from "33" to actual count (12 in Montgomery.lean) |
| Hardware validation (non-virtualized) | ⚠️ Pending | All measurements on virtualized AMD Zen 5 |

---

## Scope Corrections

Previous wording → corrected wording:

| Previous | Corrected |
|----------|-----------|
| "The formal codebase contains zero axioms and zero sorries." | "The Montgomery arithmetic formalization contains zero axioms and zero sorry declarations." |
| "AVX-512 gives 1.27x acceleration." | "Under the documented reference-arithmetic baseline, the AVX-512 implementation measured an 8.97x geometric mean improvement. Against an optimized scalar implementation (private repo), the speedup was 1.265x-1.276x. These compare against different baselines." |
| "Phase 2.5 frozen at commit 8a24a7c" | "Phase 2.5 frozen at signed Git tag `phase-2.5`, resolving to commit `cdfccf035850a08ee91d236bf1234035a772f739`." |
| "The full source code is at commit 8a24a7c (tag phase-2.5)" | "The full source code is at commit `cdfccf035850a08ee91d236bf1234035a772f739` (tag `phase-2.5`)." |
| "zero-risk for existing Plonky3 users" | "zero behavioral impact when disabled" |
| "Signed multiply keeps the product small." | "Signed multiplication preserves the required intermediate range during quotient reconstruction and avoids the unsigned interpretation that can overflow the signed 64-bit intermediate." |

---

## Benchmark Baselines

The three-lane benchmark compares four execution paths:

| Baseline | Purpose | Configuration |
|----------|---------|---------------|
| Reference arithmetic (`babybear_mul_reference`) | Correctness oracle — 64-bit modular multiply | Used in correctness gate and scalar/AVX2 lanes |
| Scalar backend (no SIMD) | Practical CPU comparison — pure Rust, no vectorization | Lane 1 in three-lane benchmark |
| AVX2 backend (compiler auto-vectorized) | SIMD comparison — compiler-generated with AVX2 target feature | Lane 2 (note: current code does not vectorize — see below) |
| AVX-512 backend (hand-written intrinsics) | Target implementation — explicit `__m512i` SIMD kernel | Lane 3 in three-lane benchmark |

> **Note on AVX2 lane:** The current `avx2_butterfly_pass` function applies the
> `#[target_feature(enable = "avx2")]` attribute but uses the same scalar code
> as Lane 1. The compiler does not auto-vectorize this loop. This is why the
> AVX2 vs Scalar ratio is ~1.0x. A future improvement would be to use
> `std::arch::x86_64::_mm256_*` intrinsics for a true AVX2 comparison.

---

## Phase 2.5 Plonky3 Integration Results

> These results are referenced in `CANONICAL_RESULTS.md` as a separate
> methodology. The supporting evidence (benchmark output, proof hashes,
> verification results) is in a private repository pending IP review.
> A public artifact is pending.

| Metric | Value | Verification Status |
|--------|-------|---------------------|
| Recursive DFT speedup | 39% at trace size 2^15 | ⚠️ Referenced in public docs; evidence artifact pending |
| Total proof speedup | 11% at trace size 2^15 | ⚠️ Referenced in public docs; evidence artifact pending |
| Proof artifact preservation | Reported as preserved | ⚠️ Evidence in private repo |
| System compatibility | Reported as verified | ⚠️ Evidence in private repo |

---

## Formal Verification Boundary

### What Lean Proves

✅ Montgomery multiplication closure (Montgomery product of two residues is a valid residue)
✅ Residue laws (addition, subtraction, multiplication under Montgomery encoding)
✅ Bézout identity (R × R_inv ≡ 1 mod P)
✅ Zero axioms, zero sorry declarations in Montgomery.lean

### What Lean Does NOT Prove

❌ AVX-512 assembly/intrinsics correctness directly
❌ Butterfly operations (validated by differential testing)
❌ NTT correctness (validated by differential testing)
❌ End-to-end proof system correctness

SIMD correctness comes from:
1. Differential testing (correctness gate: scalar ↔ AVX2 ↔ AVX-512)
2. Backend parity tests (reference oracle vs each backend)
3. Benchmark validation (timing measured only after correctness gate passes)

---

## Known Limitations

Measured results are limited to:
- AMD Zen 5 processor (virtualized environment)
- AVX-512 execution path
- BabyBear field configuration (P = 0x78000001, R = 2^32)
- Rust 1.97.1, Criterion 0.8.2

No claim is made regarding:
- Universal acceleration across all AVX-512 microarchitectures
- Performance on non-AMD hardware
- Non-virtualized hardware behavior
- Formal correctness of the SIMD implementation (validated by differential testing, not formal proof)
- The 8.97x speedup represents comparison against a naive reference; comparison against an optimized scalar implementation yields a different ratio (1.27x)

---

## Audit Trail

1. **Independent cold-read verification** (August 5, 2026): Cloned public repo at `review-v0.1.10`, attempted to build and verify all claims
2. **Identified weaknesses**: 5 missing module declarations, stale SHA256SUMS, missing `[[bench]]` declaration, incorrect benchmark configuration (10 samples/1s vs benchmark protocol 50/2s), missing binary source, broken tscp-anchor tag reference, baseline discrepancy between public and private repos
3. **Corrected claims**: Updated scope language (see Scope Corrections table), documented baseline discrepancy
4. **Regenerated evidence**: Added module declarations, fixed benchmark config, added `[[bench]]` declaration, fixed formal/README.md, independently reproduced benchmark from public repo (136 tests pass, 50-sample benchmark runs, correctness gate PASS)
5. **This document**: Created as first-class verification status reference

This audit trail demonstrates the behavior expected from serious cryptographic engineering: independent verification → identified weaknesses → corrected claims → regenerated evidence.
