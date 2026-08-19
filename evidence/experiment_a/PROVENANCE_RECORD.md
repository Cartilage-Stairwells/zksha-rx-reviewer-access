# Experiment A — Kernel Evidence Baseline: experiment_a_afea62bc

> This document attests to the Experiment A evidence package identified as
> **experiment_a_afea62bc**. It does not modify or supersede the evidence artifacts.
> The evidence artifacts are immutable; corrections require a new evidence identity.

---

## What experiment_a_afea62bc Is

A measured AVX-512 kernel-level speedup evaluation of the hand-written 32-bit/16-wide
DIF butterfly implementation against a controlled scalar baseline, captured under
an explicitly constrained build configuration and recorded with cryptographic
provenance. The evidence package establishes the performance characteristics of
the AVX-512 kernel relative to a pure SSE scalar baseline.

## What experiment_a_afea62bc Is Not

- It is **not** an end-to-end prover speedup measurement.
- It is **not** a comparison with the repository's canonical benchmark results (which use target-cpu=native).
- It is **not** a universal AVX-512 performance claim across all CPUs.
- It is **not** an energy efficiency measurement (energy measurement was aborted — gvisor sandbox).
- It is **not** a prover integration validation (FRI prover does not invoke this kernel in this experiment).
- It is **not** a formal verification result (formal proofs are in tscp-anchor, not here).
- It is **not** representative of performance on Intel hardware (only AMD Zen 5 tested).

It establishes **one** measured AVX-512 kernel speedup under a specific scalar baseline constraint.

---

## Evidence Identity

| Field | Value |
|---|---|
| Evidence identity | `experiment_a_afea62bc` |
| Evidence manifest SHA-256 | `afea62bc98276eebefd1fc18ecd9c6fad455630546888f19fd361285b251a9c3` |
| Evidence directory | `evidence/experiment_a/` |
| File count | 13 (12 content files + SHA256SUMS manifest) |
| Date captured | 2026-08-19 |
| Evidence branch | `evidence/experiment-a` |

---

## Source Identity

| Field | Value |
|---|---|
| Repository | `Cartilage-Stairwells/zksha-rx-reviewer-access` |
| Source commit | `46c09eba5d2f99299ec9cb6ded0ec9ef984e6495` |
| Tag relationship | Source files byte-identical to tagged commit `01db486d` / `review-v0.1.13` (4 intervening commits are documentation-only, verified by SHA-256 match) |
| Branch base | `main` @ `46c09eb` |

### Key Source Files (SHA-256 verified against remote)

| File | Local SHA-256 (first 12) | Remote SHA-256 (first 12) | Match |
|---|---|---|---|
| `src/avx512_butterfly_32bit.rs` | `2af3404bc0cb` | `2af3404bc0cb` | ✓ |
| `src/ntt.rs` | `72e1aeefe4b7` | `72e1aeefe4b7` | ✓ |
| `benches/three_lane_bench.rs` | `29a295ebe6ef` | `29a295ebe6ef` | ✓ |

---

## Hardware / Environment

| Field | Value |
|---|---|
| CPU | AuthenticAMD, family 191, model 2 (Zen 5 class, znver5) |
| AVX-512 features | F, DQ, CD, BW, VL, VBMI, VBMI2, VNNI, BITALG, VPOPCNTDQ |
| Cores | 4 |
| Clock | ~4130 MHz |
| Host | modal (gvisor-based container sandbox) |
| OS | Debian 12 (bookworm) |
| Kernel | 4.19.0-gvisor |
| Rust | 1.97.1 (8bab26f4f 2026-07-14) |
| Cargo | 1.97.1 (c980f4866 2026-06-30) |
| Target triple | x86_64-unknown-linux-gnu |
| Build profile | release |
| RUSTFLAGS | `-C target-cpu=x86-64` (prevents auto-vectorization in scalar baseline) |

---

## Methodology

| Field | Value |
|---|---|
| Benchmark framework | Criterion 0.8.2 (pinned in Cargo.lock) |
| Benchmark harness | `benches/three_lane_bench.rs` (three-lane: scalar, AVX2, AVX-512) |
| Targeted run config | 30 samples, 1s warmup, 3s measurement |
| Quick run config | 10 samples, 3s warmup (--quick mode) |
| Sizes | 2^8 through 2^20 (13 sizes) |
| Correctness gate | PASS (all three lanes produce identical output, 5 test sizes) |
| Scalar baseline | `target-cpu=x86-64` (SSE only, no AVX2/AVX-512 auto-vectorization) |
| AVX2 lane | `#[target_feature(enable = "avx2")]` (compiler auto-vectorized, not hand-written) |
| AVX-512 lane | Hand-written intrinsics from `avx512_butterfly_32bit.rs` (16-wide, zmm) |
| Butterfly type | DIF (Decimation-in-Frequency) |
| Field | BabyBear (P = 0x78000001 = 2^31 - 2^27 + 1) |
| Representation | Montgomery R = 2^32, u32 elements |

### Methodology Distinction

Experiment A uses `target-cpu=x86-64` to isolate the hand-written AVX-512 kernel's
contribution by eliminating compiler auto-vectorization from the scalar baseline.

The repository's CANONICAL_RESULTS.md uses `target-cpu=native` which allows
compiler auto-vectorization, producing a faster scalar baseline and lower speedup
ratios (1.265x-1.276x geometric mean).

Both measurements are valid under their respective methodologies:
- Experiment A: isolates AVX-512 kernel contribution (7.23x peak, 4.42x geometric mean)
- Canonical: measures realistic improvement including auto-vectorization (1.265x-1.276x)

---

## Binary Verification

| Lane | zmm count | ymm count | xmm count | ISA confirmed |
|---|---|---|---|---|
| Scalar | 0 | 0 | 96 | SSE only ✓ |
| AVX2 | 0 | 31 | 40 | AVX2 ✓ |
| AVX-512 | 83 | 19 | 13 | AVX-512 ✓ |

Disassembly generated from: `target/release/isa_verify` (ELF64, 456,520 bytes)
Disassembly files: `disasm_scalar.txt`, `disasm_avx2.txt`, `disasm_avx512.txt`

---

## Results

### Headline (2^20, n=1,048,576, 524,288 butterfly operations)

| Lane | Median time | Speedup vs scalar |
|---|---|---|
| Scalar | 2.0164 ms | 1.00x |
| AVX2 | 2.0454 ms | 0.99x |
| AVX-512 | 0.2787 ms | **7.23x** |

### Geometric Mean (2^8 through 2^20)

| Comparison | Geometric mean |
|---|---|
| AVX-512 vs scalar | **4.42x** |
| AVX2 vs scalar | 1.00x |
| AVX-512 vs AVX2 | 4.44x |

### Control: x86-64 vs znver5 Scalar

| Target | Scalar time at 2^20 | Ratio |
|---|---|---|
| x86-64 | 2.0164 ms | 1.00x (baseline) |
| znver5 | 2.1743 ms | 0.93x (slower, code layout effect) |

Conclusion: compiler did NOT auto-vectorize scalar at either target. Scalar baseline is genuinely SSE-only.

---

## Test Status

| Check | Result |
|---|---|
| Correctness gate | PASS (all three lanes agree, 2^8-2^20) |
| ISA verification (scalar) | 0 zmm, 0 ymm ✓ |
| ISA verification (AVX2) | 31 ymm, 0 zmm ✓ |
| ISA verification (AVX-512) | 83 zmm ✓ |
| SHA-256 manifest | All 12 hashes verified |
| Source integrity | Byte-identical to remote 46c09eb |
| Energy measurement | ABORTED (gvisor sandbox, no RAPL/MSR access) |

---

## What Was Established

1. The hand-written AVX-512 DIF butterfly kernel produces output identical to the scalar reference for sizes 2^8 through 2^20.
2. Under the specified hardware (AMD Zen 5) and methodology (Criterion 0.8.2, target-cpu=x86-64), the AVX-512 kernel achieves 7.23x speedup over the SSE-only scalar baseline at 2^20 (geometric mean 4.42x across all sizes).
3. The compiler-auto-vectorized AVX2 lane provides approximately 1.00x speedup (no benefit from auto-vectorization for this kernel).
4. The scalar kernel uses no AVX2 or AVX-512 instructions (binary-verified).
5. The AVX2 kernel uses AVX2 and no AVX-512 (binary-verified).
6. The AVX-512 kernel uses AVX-512 instructions (binary-verified).

## What This Does Not Claim

1. Proving-path integration (FRI prover does not invoke this kernel in this experiment)
2. End-to-end prover speedup
3. Energy efficiency or energy savings
4. Universal performance superiority (only tested on one AMD CPU, one operation)
5. Correctness outside the tested domain (2^8-2^20, random inputs)
6. Performance on Intel hardware (not tested)
7. Formal verification of the SIMD implementation (formal proofs are in tscp-anchor)
8. Correspondence with the admissibility experiment (067fb3ed)

---

## Interpretation Boundary

```
LAYER 1 — EVIDENCE          evidence/experiment_a/ (13 files, SHA-256 pinned)    (immutable)
LAYER 2 — ATTESTATION       this document + Git commit on evidence branch        (describes evidence)
LAYER 3 — INTERPRETATION    future analysis, cross-repository references        (evolves freely)
```

---

## Claims Supported (per CLAIM_LANGUAGE_POLICY.md)

Using the repository's defined epistemic qualifiers:

- **measured**: AVX-512 kernel speedup of 7.23x peak at 2^20, 4.42x geometric mean, under the documented benchmark configuration (target-cpu=x86-64, Criterion 0.8.2, AMD Zen 5).
- **verified**: Correctness gate PASS, ISA identity (scalar=0 zmm, AVX2=31 ymm, AVX-512=83 zmm), SHA-256 manifest integrity, source byte-identity with remote commit 46c09eb.
- **measured**: AVX2 auto-vectorized lane ~1.00x scalar (no speedup from compiler auto-vectorization).

## Claims NOT Supported

- **not claimed**: End-to-end prover acceleration
- **not claimed**: Energy efficiency
- **not claimed**: Universal AVX-512 performance
- **not claimed**: Formal verification
- **not claimed**: Prover integration

---

## Provenance Chain

```
zksha-rx-reviewer-access @ 46c09eb (main, verified source)
    |
    |-- src/avx512_butterfly_32bit.rs (SHA-256 verified)
    |-- src/ntt.rs (SHA-256 verified)
    |-- benches/three_lane_bench.rs (SHA-256 verified)
    |
    |-- Rust 1.97.1 + Criterion 0.8.2 (Cargo.lock pinned)
    |
    |-- RUSTFLAGS="-C target-cpu=x86-64" (clean scalar baseline)
    |
    |-- cargo bench --bench three_lane_bench
    |
    |-- Raw Criterion output (4 benchmark_*.txt files)
    |
    |-- objdump -d target/release/isa_verify (3 disasm_*.txt files)
    |
    |-- SHA-256 manifest (SHA256SUMS, 12 hashes, all verified)
    |
    +-- experiment_a_afea62bc (evidence identity, this document)
```

---

## Audit Trail

1. Source files byte-identical to remote commit 46c09eb (SHA-256 verified, 3 key files)
2. Tag review-v0.1.13 points to 01db486d (4 commits ahead); source identical at both commits
3. EVIDENCE_MANIFEST.md corrected to document tag-to-commit relationship
4. Methodology context added explaining target-cpu=x86-64 vs target-cpu=native distinction
5. All 12 SHA-256 hashes verified (sha256sum -c -> all OK)
6. Disassembly generated from same binary (all 3 files reference isa_verify ELF64)
7. Binary ISA verified: scalar=0 zmm/0 ymm, AVX2=31 ymm, AVX-512=83 zmm
8. No frozen artifacts modified (067fb3ed, firebird_74c6e5f, kernel-archaeology untouched)

All 8 checks: **PASSED**.

---

## Related Documents

- `EVIDENCE_MANIFEST.md` — Experiment metadata and claims
- `PHASE1_KERNEL_SELECTION.md` — Kernel selection rationale
- `BENCHMARK_RESULTS.txt` — Summary of benchmark results
- `HARDWARE_PROFILE.txt` — Hardware identification
- `ENERGY_MEASUREMENT_RESULT.txt` — Energy measurement attempt (aborted)
- `SHA256SUMS` — Hash manifest for all 12 content files

In tscp-anchor:
- `docs/benchmarks/BENCHMARK_PROVENANCE.md` — Evidence identity registry (cross-reference target)
- `docs/benchmarks/FIREBIRD_AVX512_BASELINE.md` — Existing CPU AVX-512 evidence baseline (different hardware, different scope)
- `PROJECT_FACTS.md` — Canonical claims surface (future update target)
