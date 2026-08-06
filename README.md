# zkSHA-Rx

**A high-performance NTT implementation targeting BabyBear-class fields with scalar, AVX2, and AVX-512 execution paths, built on formally verified algebraic foundations.**

## Why It Matters

NTT (Number Theoretic Transform) correctness and performance are critical components of modern STARK provers — NTT accounts for 30–60% of proving time in systems like Plonky3, SP1, and RISC Zero.

zkSHA-Rx explores a path toward **verified acceleration primitives**: NTT implementations that are both fast and backed by formal proofs of correctness.

## Evidence

| Layer | Evidence |
|-------|----------|
| Formal verification | 83 Lean theorems, 0 axioms, 0 sorries |
| Rust tests | 102 tests, 0 failures |
| SIMD | AVX2 + AVX-512 backends with correctness gate |
| Benchmark | measured AVX-512 kernel speedup (three-lane benchmark, see BENCHMARKS.md for range) |
| Provenance | TSCP custody framework with auditable evidence chain |

## Quick Start

```bash
git clone --branch review-v0.1.8 https://github.com/Cartilage-Stairwells/zksha-rx-reviewer-access
cd zksha-rx-reviewer-access
make reproduce
```

For full review: see `docs/EXTERNAL_REVIEWER_GUIDE.md`

## Performance

**Measured (AMD Zen 5 with AVX-512, three-lane benchmark, canonical dual-run):**
- AVX-512 vs Scalar: **1.27× geometric mean** (range 1.265×–1.276× across dual runs, 50 samples each)
- AVX2 auto-vectorization vs Scalar: **1.07×** (modest compiler speedup)
- Correctness gate: **PASS** (all three lanes agree, 2^8 through 2^20)
- Full details: `CANONICAL_RESULTS.md`

**Key finding:** The compiler cannot auto-vectorize the Montgomery multiplication + DIF butterfly pattern. The hand-written AVX-512 kernel is necessary for the measured speedup.

## Formal Verification

83 theorems across 4 layers:

```
Layer 0: TCP Semantics (15 theorems) — custody/authority plane separation
    ↓
Layer 1: Montgomery Arithmetic (12 theorems) — Bézout, REDC, modular bounds
    ↓
Layer 2: Butterfly Algebra (25 theorems) — DIF closure, encoding, invertibility
    ↓
Layer 3: NTT Stage Composition (8 theorems) — validity, determinism, composition
```

Proof-to-code map: `docs/plonky3/proof-to-code-map.md`

## The DIT→DIF Verification Story

The project's strongest evidence is not the speedup — it's a bug the verification system caught.

During NTT correctness testing against a naive DFT oracle, the butterfly was discovered to use DIT semantics (a+b*w, a-b*w) when the NTT structure requires DIF (a+b, (a-b)*w). This was corrected across all three backends.

This is the verification system working as designed: a real implementation bug, caught by testing, corrected with evidence, and documented for review.

## Plonky3 Integration

zkSHA-Rx's adapter (`ZkshaDifButterfly`) implements Plonky3's `Butterfly<BabyBear>` trait. The adapter:
- Compiles against real Plonky3
- Produces bit-identical output at sizes 17, 64, 256
- Has comparable scalar performance (1.015× within noise)

Full proposal: `docs/plonky3/VERIFIED_NTT_INTEGRATION_PROPOSAL.md`

## Repository Structure

```
├── README.md                              ← You are here
├── docs/
│   ├── plonky3/                           ← Integration proposal + proof map
│   └── EXTERNAL_REVIEWER_GUIDE.md         ← Build/test/bench instructions
├── src/                                   ← NTT implementation (3 backends)
├── formal/                                ← Lean proof descriptions
├── tests/                                  ← 102 tests
├── benches/                               ← Three-lane benchmark
├── evidence/                              ← Benchmark + correctness receipts
├── BENCHMARKS.md                          ← Performance details
├── CLAIM_MATRIX.md                        ← Every claim + evidence + non-claim
├── PROJECT_FACTS.md                       ← Canonical source-of-truth
├── EVIDENCE_CHAIN.md                      ← Five-stage custody chain
└── validate_release.sh                    ← Release validation gate
```

## Claims Discipline

This project maintains strict separation between **measured** and **projected** claims. See `CLAIM_LANGUAGE_POLICY.md` and `CLAIM_MATRIX.md` for the full epistemic framework.

**What we claim:** AVX-512 implementation produces 1.27× kernel-level speedup over scalar (canonical, dual-run verified). See CANONICAL_RESULTS.md for full methodology and evidence.

**What we don't claim:** Universal acceleration across all AVX-512 microarchitectures or end-to-end proving speedup.

## Contact

Sean Christopher Southwick — schlagetorren@gmail.com

## License

See `LICENSE` file.
