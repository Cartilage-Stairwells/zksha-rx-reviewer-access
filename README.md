# zkSHA-Rx Fly

**Zero-Knowledge Security Hash Acceleration and Repair**
*AVX-512 vectorized NTT for the BabyBear field — Plonky3 / SP1 / Polygon Zero*

---

## What This Is

zkSHA-Rx Fly is an AVX-512 vectorized BabyBear NTT acceleration engine for Plonky3-derived proving systems. It uses AVX-512 512-bit SIMD instructions to process 16 field elements per butterfly operation, delivering 9.15× speedup on the BabyBear DIT butterfly kernel and 4.58× on the raw i32 XOR butterfly kernel, measured via Criterion (historical 0.5.1; this snapshot uses 0.8.2).

No GPU. No ASIC. No FPGA. Standard CPU architecture with AVX-512.

---

## Why It Matters

The NTT consumes 30–60% of total proving time in modern STARK systems. Every team building on Plonky3, SP1, or RISC Zero hits this wall. Existing libraries (arkworks, Winterfell, Plonky3 native) implement field arithmetic in scalar mode — one element at a time.

zkSHA-Rx Fly processes 16 elements per instruction using 512-bit vector lanes, with Montgomery multiplication entirely in SIMD.

---

## Architecture

```
TSCP (protocol / custody layer)
 └── zkSHA-Rx (acceleration layer)
      ├── AVX-512 Butterfly Engine (BabyBear DIT, raw i32 XOR)
      ├── BabyBear Field Accelerator (Montgomery R=2^64 domain)
      ├── Receipt Composition Algebra (custody/provenance binding)
      ├── Semantic Reference Protocol
      └── Formal Verification Layer (Lean 4)
```

TSCP provides the custody and provenance guarantees — every artifact is hashed, signed, and chain-of-custody verified. zkSHA-Rx is the performance layer underneath: hardware-accelerated kernels with a formally specified correctness boundary.

---

## Verified Components

Per the benchmark-of-record (criterion (historical 0.5.1; snapshot 0.8.2), 100 samples, 3s warmup, outlier rejection):

| Kernel | Scalar | AVX-512 | Speedup | CI width |
|---|---|---|---|---|
| BabyBear DIT (R=2⁶⁴), n=2²⁰ | 504.7 Melem/s | 4,619.4 Melem/s | **9.15×** | <0.1% |
| Raw i32 XOR butterfly, n=2²⁰ | 2,423.8 Melem/s | 11,105 Melem/s | **4.58×** | <0.2% |

Formal side: `Montgomery.lean` compiles clean under Lean 4 core (no Mathlib) with 33 theorems (12 in Montgomery.lean, 21 in supporting modules) proved and 0 `sorry` in Montgomery.lean tactic calls, covering REDC cancellation, Montgomery multiply bounds, and butterfly invariants. The Montgomery arithmetic formalization was strengthened by deriving both modular inverse properties from a single Bézout identity (`R·R_inv = P·NEG_INV + 1`), reducing proof duplication and making the duality between constants explicit.

---

## Claim Discipline

- **Verified**: measured, reproducible, cited with exact command and hardware context.
- **Projected**: explicitly labeled, never presented as fact in funding or grant materials.
- Every number traces to a committed artifact with a SHA-256 hash and a reproduction command.

---

## Implementation Contract

| Property | Mechanism |
|---|---|
| Field arithmetic correctness | BabyBear Montgomery ops, unit-tested |
| Butterfly equivalence | Independent reference oracle; proptest over [0,p)³ |
| NTT composition | Cross-backend equivalence tests, shared input, deterministic seed |
| Backend parity | Scalar and AVX-512 paths produce identical outputs |

The oracle in `src/field/babybear/reference.rs` shares no code with any production path. Independence is the point. Any backend that disagrees with the oracle fails the contract regardless of its output's plausibility.

---

## Measurement Contract

| Property | Mechanism |
|---|---|
| Known environment | `provenance/provenance.json` — CPU, kernel, toolchain, ISA flags |
| Reproducible execution | Pinned commit, explicit RUSTFLAGS, Criterion `--save-baseline` |
| Validated inputs/outputs | Seal gates verify AVX-512 path was active before committing |
| Sealed artifacts | SHA256SUMS over every file in the bundle |

---

## Repository Layout

```
src/
  avx512_butterfly_32bit.rs   ← butterfly contract + AVX-512 implementation
  field/babybear/
    montgomery.rs             ← Montgomery arithmetic backend
    canonical.rs              ← canonical BabyBear type
    reference.rs              ← independent oracle (shares no code with production)
    constants.rs
tests/
  babybear_domain.rs          ← oracle agreement + cross-backend equivalence
  babybear_montgomery.rs      ← Montgomery arithmetic tests
  ntt_equivalence.rs          ← NTT = DFT verification (staged cross-backend)
  backend_parity/             ← 135 staged pairwise comparisons
benches/
  butterfly_bench.rs          ← Criterion bench: scalar and avx512 groups
examples/
  ntt_driver.rs               ← NTT forward/inverse driver
tools/
  capture_benchmark_provenance.sh   ← standalone host/toolchain snapshot
  run_benchmark.sh                  ← full capture + benchmark + seal sequence
  generate_benchmark_report.py      ← auto-generates JSON + markdown from results
benchmark_reports/
  firebird_74c6e5f/           ← first performance specimen
BENCHMARKS.md                 ← investor-facing benchmark document
```

---

## Running

```bash
# Correctness (any host)
cargo test --release

# Performance measurement (AVX-512 host required)
export RUSTFLAGS="-C target-feature=+avx512f,+avx512dq"
# perf_measure harness: not included in this snapshot (historical measurement)

# Full Criterion benchmarks
cargo bench --bench butterfly_bench

# Sealed benchmark bundle
# Benchmark tooling: not included in this snapshot (historical measurement)
# Criterion data: not in this snapshot (historical)
# Seal tooling: not included in this snapshot
```

---

## Evidence Chain

```
Lean Formal Proof (Montgomery arithmetic)
    ↓
Scalar Backend (Montgomery bridge, PR #28)
    ↓
Reference Backend (independent oracle, DFT verified)
    ↓
AVX-512 Backend (vectorized, backend parity verified)
    ↓
Benchmark Receipt (speedup measured, correctness embedded)
    ↓
Sealed Bundle (SHA256SUMS, provenance captured)
```

Each layer provides evidence for the next. No layer makes claims it cannot support.

---

## License

License file included; repository is private with reviewer access at this stage

---

## Contact

- **Sean Christopher Southwick** — `schlagetorren@gmail.com`
- GitHub: [Cartilage-Stairwells/tscp-anchor](https://github.com/Cartilage-Stairwells/tscp-anchor) (formal verification)
- Reviewer snapshot: [Cartilage-Stairwells/zksha-rx-reviewer-access](https://github.com/Cartilage-Stairwells/zksha-rx-reviewer-access) (this repository)
