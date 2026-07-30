use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use std::hint::black_box;
use avx512_butterfly::scalar_radix2_butterfly;
use p3_baby_bear::BabyBear;
use p3_field::AbstractField;
use rand::Rng;

// LOG_N_RANGE: 2^8 (256) → 2^20 (~1M elements).
// Covers small-size latency, L1/L2/L3 cache transitions, and sustained throughput.
const LOG_N_RANGE: std::ops::RangeInclusive<u32> = 8..=20;

// ── Scalar benchmark ─────────────────────────────────────────────────────────

fn bench_scalar_butterfly(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let p: u32 = 0x7800_0001;

    let mut group = c.benchmark_group("scalar_radix2_butterfly");

    for log_n in LOG_N_RANGE {
        let len: usize = 1 << log_n;
        let twid_len = len / 2;

        // Build canonical inputs once — setup work, never timed.
        let src: Vec<BabyBear> = (0..len)
            .map(|_| BabyBear::from_canonical_u32(rng.gen::<u32>() % p))
            .collect();
        let twiddles: Vec<BabyBear> = (0..twid_len)
            .map(|_| BabyBear::from_canonical_u32(rng.gen::<u32>() % p))
            .collect();

        group.bench_with_input(BenchmarkId::from_parameter(log_n), &log_n, |b, _| {
            // iter_batched_ref: setup closure runs outside timing; timed closure
            // receives fresh &mut data each iteration. No allocation, RNG, or
            // twiddle generation is inside the timed path.
            b.iter_batched(
                || src.clone(),           // setup: clone only
                |mut data| {
                    // hot path: load → butterfly → store
                    scalar_radix2_butterfly(black_box(&mut data), black_box(&twiddles));
                },
                BatchSize::LargeInput,    // appropriate for 2^8..2^20 field vectors
            );
        });
    }

    group.finish();
}

// ── AVX-512 benchmark ─────────────────────────────────────────────────────────
//
// Compile-time gate: this entire block is only compiled when the target CPU
// exposes avx512f + avx512dq. Without the gate, cargo bench on a non-AVX-512
// host would silently fall back to scalar, producing meaningless data under
// the AVX-512 label.
//
// Runtime check (is_x86_feature_detected!): guards against the case where the
// binary was cross-compiled with AVX-512 features but runs on hardware that
// does not actually support them.

#[cfg(target_feature = "avx512f")]
fn bench_avx512_butterfly(c: &mut Criterion) {
    assert!(
        is_x86_feature_detected!("avx512f"),
        "AVX-512F not available at runtime — aborting benchmark to prevent scalar masquerade"
    );
    assert!(
        is_x86_feature_detected!("avx512dq"),
        "AVX-512DQ not available at runtime — aborting benchmark to prevent scalar masquerade"
    );

    let mut rng = rand::thread_rng();
    let p: u32 = 0x7800_0001;

    let mut group = c.benchmark_group("avx512_radix2_butterfly");

    for log_n in LOG_N_RANGE {
        let len: usize = 1 << log_n;
        let twid_len = len / 2;

        let src: Vec<BabyBear> = (0..len)
            .map(|_| BabyBear::from_canonical_u32(rng.gen::<u32>() % p))
            .collect();
        let twiddles: Vec<BabyBear> = (0..twid_len)
            .map(|_| BabyBear::from_canonical_u32(rng.gen::<u32>() % p))
            .collect();

        group.bench_with_input(BenchmarkId::from_parameter(log_n), &log_n, |b, _| {
            b.iter_batched(
                || src.clone(),
                |mut data| unsafe {
                    avx512_butterfly::avx512_impl::avx512_radix2_butterfly(
                        black_box(&mut data),
                        black_box(&twiddles),
                    );
                },
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

// ── Criterion registration ────────────────────────────────────────────────────
//
// AVX-512 group is only registered when the compile-time gate is satisfied.
// A non-AVX-512 build produces a scalar-only binary — no phantom AVX-512
// benchmark entry, no false comparison possible.

#[cfg(not(target_feature = "avx512f"))]
criterion_group!(benches, bench_scalar_butterfly);

#[cfg(target_feature = "avx512f")]
criterion_group!(benches, bench_scalar_butterfly, bench_avx512_butterfly);

criterion_main!(benches);
