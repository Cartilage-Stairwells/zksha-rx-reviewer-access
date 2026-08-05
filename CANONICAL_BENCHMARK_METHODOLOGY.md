# Canonical Benchmark Methodology — zkSHA-Rx Three-Lane Benchmark

## Status: FROZEN — canonical benchmark protocol v1.0
## Date frozen: August 5, 2026
## Date: August 5, 2026

---

## 0. Scope and Status

This document defines the canonical protocol for three-lane butterfly benchmark measurements. **No canonical run has been conducted yet under this protocol.** All measurements in the Historical Measurements section (Section 9) predate this methodology and were conducted under varying configurations. They are included as historical evidence only.

A "canonical run" under this protocol requires:
- Exact commit hash recorded
- CPU info archived at time of measurement
- Criterion configuration as specified in Section 5 (or higher sample count)
- RUSTFLAGS as specified in Section 4
- Correctness gate verified
- Evidence files named with timestamp

Once a canonical run is completed, it becomes the reference measurement. All prior runs remain historical context.

## 1. Workload

**Operation:** DIF radix-2 butterfly pass on Montgomery-encoded BabyBear field elements.

This corresponds to Plonky3's `DifButterfly` operation — the single-butterfly-pass primitive used in recursive NTT implementations. The benchmark measures one butterfly pass, not a full NTT.

**Field:** BabyBear (P = 2^31 - 1, Mersenne prime used in Plonky3)

**Data:** Random u32 values in [0, P), generated with `rand::Rng` (`thread_rng`). Seed is not fixed — individual iterations are not bit-reproducible, but the statistical distribution of timings is reproducible within noise bounds.

**Sizes:** 2^8 through 2^20 (13 sizes). Each size benchmarks one butterfly pass on n elements with n/2 twiddle factors.

**Butterfly operation per pair (i, i+n/2):**
```
sum = (a + b) mod P
diff = (a - b) mod P
result = (diff * w) mod P  // Montgomery multiply
output[i] = sum
output[i + n/2] = result
```

## 2. Three Lanes

| Lane | Description | SIMD level |
|------|-------------|------------|
| Scalar | Pure u32 arithmetic, no SIMD, scalar reference implementation | None (SSE2 baseline) |
| AVX2 | Identical scalar code, compiled with `#[target_feature(enable = "avx2")]` — compiler auto-vectorizes | Compiler-generated AVX2 |
| AVX-512 | Hand-written AVX-512 intrinsic kernel (`avx512_butterfly_pass_32`) | Explicit AVX-512F + AVX-512DQ |

All three lanes operate on identical raw u32 Montgomery-encoded data. Lanes 1 and 2 use the same algorithmic code; the only difference is the compiler's auto-vectorization. Lane 3 uses direct AVX-512 intrinsics.

The correctness of the AVX-512 lane against the scalar lane is the engineering claim. The AVX2 lane exists to show what the compiler can achieve without hand-written intrinsics.

## 3. Correctness Gate

Before any timing begins, the benchmark verifies that all three lanes produce byte-identical output on sizes 2^8, 2^10, 2^12, 2^16, and 2^20. If any lane disagrees, the benchmark prints "Correctness gate: FAIL" and aborts. No timing data is collected if the correctness gate fails.

A benchmark run is invalid unless the output contains "Correctness gate: PASS".

## 4. Software Configuration

| Parameter | Value |
|-----------|-------|
| Repository | avx512-butterfly (private development repo — not the public reviewer-access repo) |
| Commit | Must be recorded per run (see Section 8) |
| Rust version | rustc 1.97.1 (8bab26f4f 2026-07-14) |
| Criterion version | 0.5 |
| Release profile | opt-level=3, lto=true, codegen-units=1 |
| Bench profile | opt-level=3, lto=true, codegen-units=1 |
| RUSTFLAGS | `-C target-cpu=native -C target-feature=+avx512f,+avx512dq,+avx2` |

**Note on RUSTFLAGS:** These flags are fixed for all canonical runs. Historical runs may have used different flags. The `target-cpu=native` flag enables whatever the current CPU supports; the explicit `target-feature` flags ensure AVX-512 codegen is not omitted even if the compiler's CPU detection is conservative in the virtualized environment.

## 5. Criterion Configuration

| Parameter | Value (current code) | Value (recommended for canonical runs) |
|-----------|---------------------|---------------------------------------|
| Sample size | 10 | 50+ (higher reduces variance) |
| Warm-up time | 500ms per benchmark | 500ms |
| Measurement time | 1s per benchmark | 2s+ (longer reduces variance) |
| Statistical output | Criterion default (median, IQR, change detection) | Same |

**Current code configuration** (in `benches/three_lane_bench.rs`):
```rust
Criterion::default()
    .sample_size(10)
    .warm_up_time(std::time::Duration::from_millis(500))
    .measurement_time(std::time::Duration::from_secs(1));
```

**Why 10 samples is insufficient for canonical claims:** The sandbox is a virtualized environment with high noise. Observed variance across 10-sample runs is large (AVX-512 vs Scalar geometric mean ranges from 1.06× to 1.13× across same-day runs). 50+ samples would reduce the confidence interval and produce more stable median timings.

**For the first canonical run:** Increase `sample_size` to 50 and `measurement_time` to 2s in the benchmark code before running. This increases total benchmark time from ~3 minutes to ~20 minutes, which is acceptable for a one-time canonical measurement.

## 6. Hardware Configuration

| Parameter | Value |
|-----------|-------|
| CPU vendor | AuthenticAMD |
| CPU family | 175 (Zen 5) |
| CPU model | 17 |
| Model name | unknown (virtualized — hypervisor does not expose model name) |
| Cores | 4 |
| Base clock | ~3.7 GHz (observed, varies by session) |
| Cache | 8192 KB L2 (per core) |
| Architecture | x86_64 |
| Kernel | 4.19.0-gvisor (Google Cloud sandbox) |

**AVX-512 instruction set support:**
avx512f, avx512dq, avx512cd, avx512bw, avx512vl, avx512vbmi, avx512_vbmi2, avx512_vnni, avx512_bitalg, avx512_vpopcntdq

**Important architectural note:** AMD Zen 5 implements AVX-512 using dual 256-bit execution units. 512-bit operations take 2 cycles rather than 1. This means AVX-512 throughput on Zen 5 is lower than on Intel CPUs with dedicated 512-bit execution units (Ice Lake, Sapphire Rapids). Results should not be generalized to all AVX-512 microarchitectures.

**Sandbox hardware variability:** The sandbox CPU configuration is non-deterministic across sessions. AVX-512 availability, core count, and clock speed may change between sandbox restarts. All benchmark runs must capture and archive `/proc/cpuinfo` alongside timing data. Runs where AVX-512 is unavailable should be discarded.

## 7. Reproduction Steps

```bash
# 1. Clone the repository (private development repo — not publicly accessible)
# The benchmark code is in the avx512-butterfly development repo.
# For reviewer-access reproduction, use the public snapshot:
# git clone --branch review-v0.1.10 https://github.com/Cartilage-Stairwells/zksha-rx-reviewer-access
# The benchmark source is at benches/three_lane_bench.rs in that snapshot.

cd /path/to/workspace/avx512-butterfly

# 2. Record the exact commit
git rev-parse HEAD > evidence/commit-$(date +%Y%m%d_%H%M%S).txt

# 3. Verify AVX-512 availability
grep -c 'avx512f' /proc/cpuinfo
# Must return > 0 — if 0, AVX-512 is unavailable, discard this session

# 4. Archive CPU state
cat /proc/cpuinfo > evidence/cpu-info_$(date +%Y%m%d_%H%M%S).txt

# 5. Set compiler flags (fixed — do not modify)
source $HOME/.cargo/env
export RUSTFLAGS="-C target-cpu=native -C target-feature=+avx512f,+avx512dq,+avx2"

# 6. (Optional but recommended) Increase sample size for canonical runs
# Edit benches/three_lane_bench.rs:
# Change sample_size(10) to sample_size(50)
# Change measurement_time(Duration::from_secs(1)) to Duration::from_secs(2)

# 7. Run the benchmark
cargo bench --bench three_lane_bench 2>&1 | tee evidence/avx512_bench_$(date +%Y%m%d_%H%M%S).txt

# 8. Verify correctness gate passed
grep "Correctness gate: PASS" evidence/avx512_bench_*.txt
# If not found, the run is INVALID — discard all timing data
```

## 8. Required Evidence Per Run

Each benchmark run must produce:
1. **Commit hash** — exact git commit at time of measurement
2. **CPU info file** — `/proc/cpuinfo` at time of measurement
3. **Benchmark output file** — Full Criterion output including correctness gate result
4. **Timestamp** — Embedded in all filenames (YYYYMMDD_HHMMSS)
5. **RUSTFLAGS confirmation** — either in the benchmark output or archived separately

A run is incomplete (and cannot be cited as canonical) if any of the above is missing.

## 9. Historical Measurements

**All measurements below predate this methodology.** They were conducted under varying configurations and should not be cited as canonical results. They are included to document the measurement history and explain why a canonical protocol is needed.

| Run | Date (UTC) | Commit | Samples | AVX-512 vs Scalar (geo mean) | AVX2 vs Scalar (geo mean) | Correctness | Evidence file |
|-----|-----------|--------|---------|------------------------------|---------------------------|------------|---------------|
| 1 | Aug 1-2 | not archived | 50 (per prior records) | 2.65× | 1.00× | PASS | (recorded in BENCHMARKS.md; evidence file not preserved) |
| 2 | Aug 2, 13:03 | not archived | 50 (per prior records) | 3.08× | 2.37× | PASS | avx512_bench_20260802_130339.txt (file not in current archive) |
| 3 | Aug 4, 07:18 | ~41df345 | 10 | 1.122× | 1.025× | PASS | avx512_bench_20260804_071823.txt |
| 4 | Aug 4, 19:02 | ~f82aa73 | 10 | ~1.06× | ~1.0× | PASS | avx512_bench_20260804_190203.txt |
| 5 | Aug 5, 01:05 | ~7dbe9fe | 10 | ~1.13× avg | ~1.06× avg | PASS | avx512_bench_20260805_010514.txt |
| 6 | Aug 5, 07:05 | ~1ddb401 | 10 | 1.061× | 0.992× | PASS | avx512_bench_20260805_070503.txt |
| 7 | Aug 5, 13:01 | ~ee14742 | 10 | ~1.06× | ~1.0× | PASS | avx512_bench_20260805_130147.txt |

**Observed range across all runs:** AVX-512 vs Scalar: 1.06× to 3.08×
**Observed range across all runs:** AVX2 vs Scalar: 0.99× to 2.37×

**Why the range is so wide:**

1. **Sample count:** Runs 1-2 (50 samples) show higher speedups than runs 3-7 (10 samples). This is counterintuitive — more samples should give more accurate results, not higher speedups. The likely explanation is that the 50-sample runs captured a different CPU/cache/thermal state than the 10-sample runs. The runs are not directly comparable because the CPU state is non-deterministic across sessions.

2. **The 1.00× AVX2 result in Run 1 is anomalous.** All subsequent runs show AVX2 providing modest speedup (1.0× to 2.37×). The 1.00× result may reflect a session where compiler auto-vectorization was ineffective for unknown reasons (different LLVM version behavior, different CPU state).

3. **Runs 1-2 evidence files are not preserved.** The August 2 benchmark output file is not in the current evidence archive. The numbers for runs 1-2 are from prior session records, not from archived evidence files. This is a gap that the canonical protocol prevents by requiring evidence archiving.

4. **Commits for runs 1-2 are not archived.** The exact code state for the August 2 run is unknown. Runs 3-7 have approximate commit hashes inferred from auto-commit timestamps, not from explicit recording.

5. **The correctness gate passes on every run.** This is the most important finding: regardless of performance variance, the AVX-512 implementation produces identical output to the scalar reference across all tested configurations and sizes.

## 10. Claim Language

**For historical results (pre-methodology):**
> "Historical three-lane benchmark measurements on AMD Zen 5 with AVX-512 (virtualized sandbox, Criterion) show AVX-512 vs Scalar geometric mean speedup ranging from 1.06× to 3.08× across 7 runs with varying sample counts. Results are sensitive to virtualization noise, sample count, and CPU state. The correctness gate (all three lanes produce identical output) passed on every run. These results predate the canonical benchmark methodology and should not be cited as canonical performance."

**For canonical results (once conducted):**
> "Canonical three-lane benchmark on AMD Zen 5 with AVX-512 (virtualized sandbox, Criterion, [N] samples per size, commit [hash]): AVX-512 vs Scalar geometric mean speedup [X.XX]× (range [min]× to [max]×). Correctness gate: PASS. Full evidence: [evidence file names]."

**NOT claimed:**
- Universal AVX-512 speedup (results are hardware-specific to AMD Zen 5)
- Stable single-number speedup (variance is high in the virtualized environment)
- End-to-end proving speedup (this is a kernel-level benchmark only — see Section 11)
- Reproducibility on different hardware without re-measurement
- Performance based on runs 1-2 (evidence files not preserved)

## 11. Separation from Full-Pipeline Benchmarks

This benchmark measures the **butterfly kernel in isolation**. It does not include:
- Memory allocation and initialization
- Twiddle factor generation
- Multi-stage NTT composition
- Proof generation overhead (hashing, Merkle trees, FRI, etc.)

The Phase 2.5 Plonky3 integration benchmark (39% recursive DFT speedup, 11% total proof speedup at 2^15, commit cdfccf035850a08ee91d236bf1234035a772f739) measures the **full proving pipeline** and is a separate measurement under a separate methodology. The Phase 2.5 benchmark is the stronger claim because it demonstrates real-world impact, not just kernel throughput.

Kernel microbenchmarks and full-pipeline benchmarks must not be conflated. A reviewer who sees "2.65× kernel speedup" and "39% DFT speedup" in the same document must understand that these measure different things at different levels of the stack.

## 12. Canonical Run Protocol

### Prerequisites
Before conducting the first canonical run:
1. **Increase sample_size to 50** in the benchmark code (reduces variance)
2. **Increase measurement_time to 2s** (more stable medians)
3. **Record the exact commit hash** in the evidence directory before running
4. **Verify AVX-512 availability** and archive `/proc/cpuinfo`
5. **Archive the RUSTFLAGS** used for the run

### Dual-Run Requirement
The canonical benchmark must be run **twice** under identical conditions (same commit, same session, same RUSTFLAGS). Both runs must pass the correctness gate. The purpose is to verify that the methodology itself is stable — if both runs produce reasonably consistent results, they can be confidently presented as the baseline for future comparisons.

**Run 1:** Conducted under full canonical protocol. Archive all evidence.
**Run 2:** Conducted immediately after Run 1 (same session to minimize CPU state drift). Archive all evidence.

**Consistency check:** Compare the geometric mean speedups across both runs. If the AVX-512 vs Scalar geometric means differ by more than 15%, the variance is too high for a canonical claim — note this in the results and consider increasing sample_size to 100. If they are within 15%, report both runs and their range as the canonical baseline.

### After Canonical Runs Complete
Once both canonical runs are completed and the consistency check passes:
1. Update Section 9 with a "Canonical Results" subsection containing both runs
2. The historical table remains for context but is superseded by the canonical results
3. All future performance claims cite the canonical runs, not historical measurements
4. Tag review-v0.1.10 with the methodology, corrected hardware documentation, canonical results, and supporting evidence
