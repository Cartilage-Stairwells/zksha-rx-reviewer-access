# Independent non-Zen 5 reproduction of three_lane_bench — 2026-10-23

**Status:** non-canonical evidence. This is an independent reproduction run, not part of the dual-run canonical measurement. It is recorded to document the benchmark's behavior on AVX-512 hardware other than AMD Zen 5.

## Run configuration
- Repository state: review-v0.1.19 (commit 612cd89), clean clone
- Command: `RUSTFLAGS="-Ctarget-cpu=native" cargo bench --bench three_lane_bench` (per docs/EXTERNAL_REVIEWER_GUIDE.md)
- Toolchain: rustc 1.99.1; Criterion as configured in the bench source (50 samples, 500 ms warmup, 2 s measurement — same shape as the canonical protocol)
- Hardware: x86_64, 4 cores, AVX-512 (f/bw/cd/dq/vbmi/vl). **CPU model masked as "unknown" by the execution environment** — a disclosed limitation of this evidence; absolute times should not be compared across machines, and the platform identity cannot be independently confirmed.

## Results
- Correctness gate: **PASS — all three lanes agree on sizes 2^8 through 2^20**
- AVX-512 vs scalar geometric mean: **6.988×** (range 5.666×–7.854× across 13 sizes)
- AVX-512 vs avx2 geometric mean: 7.057×
- Observation: scalar-lane and avx2-lane times are equal within noise (as in the canonical Zen 5 runs)

## Files
- `bench-raw.log` — full Criterion output
- `bench-parsed.json` — parsed per-lane, per-size times
- `MEASUREMENT-RECORD.md` — full record including the disassembly probe
- `disasm-probe.md` — machine-code verification of the scalar-lane codegen under three flag configurations

## Interpretation boundary (see MEASUREMENT-RECORD.md for the full observation/hypothesis separation)
This run does **not** contradict the canonical 1.265×–1.276× claim (AMD Zen 5). The disassembly probe (this date) shows the benchmark's scalar-source lane compiles to AVX-512/AVX2 vector machine code under both documented flag sets — so the canonical ratio is hand-written kernel vs the compiler's auto-vectorization on Zen 5, where that auto-vectorization is highly effective. On the (unknown, non-Zen 5) hardware of this run, the same auto-vectorized baseline is far less effective relative to the hand kernel, yielding the ~7× regime. Correctness generalizes across both platforms; the speedup ratio's magnitude is platform-dependent, consistent with the package's non-claim of universal acceleration.
