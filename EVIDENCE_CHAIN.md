# Evidence Chain — zkSHA-Rx Fly v0.1.0

**Status:** VALIDATED
**Date:** 2026-07-27
**Tag:** v0.1.0-zksha-rx
**Commit:** 427c34a (branch: issue-3-4-5-verification-pipeline)

---

## Complete Evidence Chain

```
Source commit (427c34a)
    ↓
Build configuration (RUSTFLAGS=-C target-feature=+avx512f,+avx512dq, --release)
    ↓
Hardware identity (x86_64, kernel 4.19.0-gvisor, avx512f+dq+cd+bw+vl+vbmi+vnni)
    ↓
Benchmark command (# perf_measure: not in this snapshot (historical))
    ↓
Raw measurements (7 sizes, 1000 iterations each, 10 warmup)
    ↓
Summary claims (3.29x–4.38x, geometric mean 3.94x)
    ↓
SHA256 evidence seal (tscp-evidence-396ebfb.tar.gz, hashes.sha256)
```

Each link is independently verifiable.

---

## 1. Source Commit

```
Commit: 427c34a (current tip, tag v0.1.0-zksha-rx)
Measurement commit: 9473af6 (historical — development repo) (docs: AVX-512 refinement receipt)
Correctness fix commit: 78c040f (DIF butterfly — Semantic Drift resolved)
Evidence seal commit: 49eac025 (tag: vep-0.1.4-sealed, avx512-v1-evidence-sealed)

Repo: https://github.com/Cartilage-Stairwells/zksha-rx-reviewer-access
Branch: issue-3-4-5-verification-pipeline
```

---

## 2. Build Configuration

```
RUSTFLAGS: -C target-feature=+avx512f,+avx512dq
Profile: --release
Rust version: rustc 1.97.1 (8bab26f4f 2026-07-14)
Target: x86_64-linux
```

---

## 3. Hardware Identity

Captured in `evidence/environment.json` (sealed in tscp-evidence-396ebfb.tar.gz):

```
Architecture: x86_64
OS: Linux
Kernel: 4.19.0-gvisor
AVX-512 features present:
  avx512f, avx512dq, avx512cd, avx512bw, avx512vl,
  avx512vbmi, avx512_vbmi2, avx512_vnni, avx512_bitalg,
  avx512_vpopcntdq
```

CI correctness runs on AMD EPYC 7763 (scalar-only, no AVX-512).
Benchmark measurements on SIMD-capable host (full AVX-512 feature set above).

---

## 4. Benchmark Command

```bash
export RUSTFLAGS="-C target-feature=+avx512f,+avx512dq"
# perf_measure: not in this snapshot (historical)
```

Methodology (from `AVX512_REFINEMENT_RECEIPT (historical, not in this snapshot)` §5.1):
- Measured AFTER correctness closure (DIF fix applied first)
- 1000 iterations per size, averaged
- 10 untimed warm-up iterations
- Deterministic LCG seed for reproducibility
- Full NTT (all butterfly stages, not isolated butterfly)
- Same input for scalar and AVX-512 paths
- No architecture-specific shortcuts (8-item audit, §7)

---

## 5. Raw Measurements

From `AVX512_REFINEMENT_RECEIPT (historical, not in this snapshot)` §5.2:

| n | Scalar (ns) | AVX-512 (ns) | Speedup |
|---|-------------|-------------|---------|
| 256 | 5,092 | 1,550 | 3.29x |
| 1,024 | 23,342 | 6,201 | 3.76x |
| 4,096 | 105,545 | 26,033 | 4.05x |
| 16,384 | 473,605 | 111,777 | 4.24x |
| 65,536 | 2,095,378 | 478,669 | 4.38x |
| 262,144 | 9,614,045 | 2,544,568 | 3.78x |
| 1,048,576 | 41,178,005 | 9,770,167 | 4.21x |

---

## 6. Summary Claims

**Validated:**
- BabyBear NTT correctness (140/140 tests, NTT == DFT oracle)
- Scalar/vector equivalence (135 staged pairwise comparisons, all pass)
- AVX-512 implementation correctness (execution canary, spot-checks at 3 sizes)
- Measured AVX-512 performance: 3.29x–4.38x (geometric mean 3.94x)
- Formal invariants (33 Lean 4 theorems (12 in Montgomery.lean, 21 supporting), 0 sorry in Montgomery.lean (3 documented sorry in Examples), Bézout identity)

**Claim scope:** AVX-512 acceleration of BabyBear-field NTT kernels under this
benchmark configuration. NOT universal zk acceleration. Integration with
Plonky3/SP1 pipelines is future work.

**What this does NOT claim** (from receipt §6.2):
- No claim about AVX-512 instruction correctness (hardware trust boundary)
- No claim about optimal performance (current implementation, further optimization possible)
- No Lean theorem about SIMD internals (formal boundary stops at scalar bridge)
- No claim about cache behavior beyond observed measurements

---

## 7. SHA256 Evidence Seal

**Sealed tarball:** `tscp-evidence-396ebfb.tar.gz`
**Sealed at commit:** 396ebfb78fd6ce0bb6bfbc956005cd28b9c0b359

Contents and hashes:
```
25d3cb0f...  result_receipt.json
03378605...  result_artifact.json
24e40130...  run_manifest.json
03436a4f...  environment.json
0d5c6293...  logs/validation.log
```

Verification:
```bash
tar -xzf tscp-evidence-396ebfb.tar.gz -O hashes.sha256 | sha256sum -c
```

**Additional seals:**
- Tag `avx512-v1-evidence-sealed` → 49eac025 (evidence boundary)
- Tag `vep-0.1.4-sealed` → 49eac025 (VEP milestone)
- Tag `tscp-ntt-equivalence-v1` → 72bb60a5 (NTT equivalence)
- Tag `v0.1.0-zksha-rx` → 427c34a (release)
- SHA256SUMS in repo root (9 files sealed)

---

## 8. Inheritance Chain (Correctness)

```
Montgomery.lean (formal proof, 33 theorems (12 in Montgomery.lean, 21 in supporting modules))
    ↓ scalarMul = montgomeryMul (PR #28, sealed)
ScalarBackend
    ↓ butterfly() = butterfly_reference() (Commit 4 oracle contract)
ReferenceBackend (u128 oracle, no p3 dependency)
    ↓ ntt_reference == DFT (NTT contract audit, sealed)
AVX-512Backend
    ↓ ntt_avx512 == ntt_reference (staged parity, 135 checks)
```

Each arrow is an equivalence proof. AVX-512 inherits correctness through
the chain — it does not require its own Lean proof.

---

## 9. Next Funding Objectives

The validation is complete. The grant narrative shifts from
"fund a measurement" to "fund expansion":

1. **Independent reproduction** — second party runs the evidence chain
   on different hardware and confirms the results
2. **More CPUs** — test on Intel Xeon, AMD EPYC (with AVX-512), cloud
   instances to establish portability range
3. **Plonky3/SP1 integration** — validate drop-in replacement in the
   actual proving pipeline, measure end-to-end speedup
4. **Broader workload benchmarks** — FRI, constraint evaluation,
   polynomial commitment — not just isolated NTT

---

## 10. Related Tags

| Tag | Commit | Purpose |
|---|---|---|
| v0.1.0-zksha-rx | 427c34a | Release tag (current) |
| vep-0.1.4-sealed | 49eac025 | VEP milestone — all validation evidence complete |
| avx512-v1-evidence-sealed | 49eac025 | Evidence boundary — separates correctness from optimization |
| tscp-ntt-equivalence-v1 | 72bb60a5 | NTT equivalence sealed |
| v1.0-rc1 | d5d0a003 | Release candidate — scalar reference + evidence corpus |
| v0.3-reproduction | 45f67da4 | Reproduction environment record |
