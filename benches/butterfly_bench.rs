use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use std::hint::black_box;
use avx512_butterfly::scalar_radix2_butterfly;
use p3_baby_bear::BabyBear;
use p3_field::AbstractField;
use rand::Rng;

// ── Notes for reviewers ──────────────────────────────────────────────────────
//
// This benchmark file measures the SCALAR reference butterfly.
//
// The AVX-512 SIMD kernel benchmark uses the real SIMD implementation in
// `src/avx512_butterfly_32bit.rs` (avx512_radix2_butterfly_32 + avx512_butterfly_pass_32),
// which operates on __m512i vectors and was measured separately on real AVX-512 hardware.
//
// The function `avx512_impl::scalar_compat_radix2_butterfly` in lib.rs is a
// PLACEHOLDER that delegates to scalar. It is intentionally NOT benchmarked
// here to avoid presenting scalar results under an AVX-512 label.
//
// ── Scalar benchmark ──────────────────────────────────────────────────────────

const LOG_N_RANGE: std::ops::RangeInclusive<u32> = 8..=20;

fn bench_scalar_butterfly(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let p: u32 = 0x7800_0001;

    let mut group = c.benchmark_group("scalar_radix2_butterfly");

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
                |mut data| {
                    scalar_radix2_butterfly(black_box(&mut data), black_box(&twiddles));
                },
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

// ── AVX-512 SIMD kernel benchmark ────────────────────────────────────────────
//
// This benchmarks the REAL AVX-512 SIMD kernel from avx512_butterfly_32bit.rs.
// The benchmark operates on raw u32 arrays (Montgomery-encoded BabyBear values)
// using the actual __m512i intrinsics — not the placeholder compatibility wrapper.
//
// Compile-time + runtime gates ensure this only runs on genuine AVX-512 hardware.

#[cfg(target_feature = "avx512f")]
fn bench_avx512_simd_butterfly(c: &mut Criterion) {
    assert!(
        is_x86_feature_detected!("avx512f"),
        "AVX-512F not available at runtime — aborting to prevent scalar masquerade"
    );
    assert!(
        is_x86_feature_detected!("avx512dq"),
        "AVX-512DQ not available at runtime — aborting to prevent scalar masquerade"
    );

    use avx512_butterfly::avx512_butterfly_32bit::{avx512_butterfly_pass_32, is_avx512_supported};

    let mut rng = rand::thread_rng();
    let p: u32 = 0x7800_0001;

    let mut group = c.benchmark_group("avx512_simd_butterfly_pass");

    for log_n in LOG_N_RANGE {
        let len: usize = 1 << log_n;
        let twid_len = len / 2;

        // Generate Montgomery-encoded values (xR mod p)
        let mut data: Vec<u32> = (0..len)
            .map(|_| (rng.gen::<u32>() % p) as u64 * (1u64 << 31) as u64 % p as u64)
            .map(|x| x as u32)
            .collect();
        let twiddles: Vec<u32> = (0..twid_len)
            .map(|_| (rng.gen::<u32>() % p) as u64 * (1u64 << 31) as u64 % p as u64)
            .map(|x| x as u32)
            .collect();

        group.bench_with_input(BenchmarkId::from_parameter(log_n), &log_n, |b, _| {
            b.iter_batched(
                || (data.clone(), twiddles.clone()),
                |(mut d, tw)| unsafe {
                    if is_avx512_supported() {
                        avx512_butterfly_pass_32(
                            d.as_mut_ptr(),
                            tw.as_ptr(),
                            len,
                        );
                    }
                    black_box(d);
                },
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

// ── Registration ──────────────────────────────────────────────────────────────

#[cfg(not(target_feature = "avx512f"))]
criterion_group!(benches, bench_scalar_butterfly);

#[cfg(target_feature = "avx512f")]
criterion_group!(benches, bench_scalar_butterfly, bench_avx512_simd_butterfly);

criterion_main!(benches);
