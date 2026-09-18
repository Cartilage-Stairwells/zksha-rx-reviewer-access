# zkSHA-Rx Evidence Summary — Experiments A, B, C

## Date: 2026-08-18
## Status: ALL THREE EXPERIMENTS SEALED

---

## Experiment A — Kernel Performance (SEALED)

- **Evidence identity:** kernelbench_v1
- **Evidence-only digest:** 50e7282e65e108c61e6e7d48789bb6adba8a3a809b3b65a93f89f23b52cf22c8
- **Repository:** avx512-butterfly (commit d0699e00)
- **CPU at capture:** family 175, model 17 (AMD Zen 5)
- **CPU at verification:** family 191, model 2 (both have AVX-512)
- **Provenance:** CPU change documented; results captured on Zen 5

### Results
| Comparison | Geometric Mean | Range |
|------------|---------------|-------|
| AVX-512 vs Scalar | 2.86× | 2.47×–3.15× |
| AVX2 vs Scalar | 1.86× | 1.71×–1.98× |
| AVX-512 vs AVX2 | 1.54× | 1.37×–1.64× |

- **Correctness gate:** PASS (all three lanes agree, 2^8–2^20)
- **Binary inspection:** 77 zmm instructions (AVX-512 lane), 284 ymm (AVX2 lane), 0 AVX-512 in AVX2
- **Claim scope:** Kernel-level performance only. Does NOT establish end-to-end prover speedup.

### Formal track status
The mathematical butterfly is formally verified in Lean 4.32.2 (B₀–B₂, C₁–C₃, 0 project axioms). The AVX-512 implementation is supported by runtime differential testing and binary inspection — NOT a formal proof that AVX-512 instructions implement the Lean butterfly.

---

## Experiment B — Production Integration Analysis (SEALED)

- **Evidence identity:** experiment_b_v1
- **SHA-256:** 71112896057db7c1bc2e4c6b8e2b38869b8170f21a4bb0762a6af2faab00f569
- **Repository:** tscp-anchor (commit 58208dab)
- **Type:** Observational (source inspection, no runtime modification)

### Principal Finding
The current production proving path does not invoke the NTT/FFT butterfly implementation, and the repository's Avx512Backend is presently a scalar placeholder.

### Evidence
1. `fri_fold_step` is NOT an NTT butterfly (2→1 combine vs 2→2 swap)
2. `Radix2Interpolator` (FFT/NTT) has ZERO production call sites (all in `#[cfg(test)]`)
3. `Avx512Backend` delegates to scalar `Radix2Dit` — no AVX-512 intrinsics
4. The avx512-butterfly kernel is completely disconnected from the prover

---

## Experiment C — FRI Sub-Component Profiling (SEALED)

- **Evidence identity:** experiment_c_profile_v1
- **SHA-256:** d0bfdc44d7b73dbf123d40e551d27da924e8e50789f82b044b08443db319e7e9
- **Repository:** tscp-anchor (commit 58208dab)
- **CPU:** family 191, model 2
- **Type:** Observational profiling (no algorithm change, original code restored)

### Results (Median of 3 iterations)

| Size | Merkle % | Fold % | Query % | Challenger % |
|------|----------|--------|---------|-------------|
| 2^8  | 91.3%    | 2.5%   | 5.2%    | 0.9%        |
| 2^10 | 93.9%    | 4.0%   | 1.8%    | 0.3%        |
| 2^12 | 94.6%    | 4.3%   | 1.1%    | 0.1%        |
| 2^14 | 95.1%    | 4.4%   | 0.4%    | 0.0%        |

### Finding
MerkleTree::build (Poseidon2 hashing) dominated the measured FRI proving workload at 91.3–95.1%, with dominance increasing at larger sizes. This establishes the FRI-level bottleneck. Whether this extends to the complete proving pipeline requires additional profiling beyond fri_prove.

---

## Architecture

```
FORMAL TRACK                          KERNEL TRACK
  Mathematical butterfly               AVX-512 implementation
  Lean / 0 axioms                       differential correctness
       │                                     │
       ▼                                     ▼
  B₀–B₂, C₁–C₃ proven              2.86× vs scalar
                                    1.54× vs AVX2
────────────────────────────────────────────────────────
                         PROVER TRACK
                              │
                         fri_prove
                              │
               ┌──────────────┼──────────────┐
               ▼              ▼              ▼
           MerkleTree      FRI fold       Queries
            /Poseidon
               │
               ▼
          91.3–95.1%
         of measured
          FRI workload
```

No false arrow from AVX-512 butterfly into the prover. The fast kernel and the production bottleneck are different operations.

---

## Governance Disposition

| Experiment | Status | What it establishes |
|------------|--------|---------------------|
| A | SEALED | Kernel performance evidence |
| B | SEALED | Production integration/path analysis |
| C | SEALED | Observational production profiling |

Next research question (OPEN): Can Poseidon2/Merkle construction be accelerated materially while preserving the existing prover's semantics, verification behavior, and evidence boundaries?

This is a new optimization investigation, not a justification for modifying the frozen architecture.
