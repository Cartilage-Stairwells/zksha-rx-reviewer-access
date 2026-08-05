//! Three-lane DIF butterfly benchmark: Scalar vs AVX2 vs AVX-512.
//!
//! Correctness gate verifies all lanes produce identical output before timing.
//! Uses raw u32 Montgomery-encoded BabyBear values.

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use std::hint::black_box;
use avx512_butterfly::field::babybear::constants::BABYBEAR_P;
use avx512_butterfly::field::babybear::reference::babybear_mul_reference;
use avx512_butterfly::avx512_butterfly_32bit::{
    is_avx512_supported, avx512_butterfly_pass_32,
};
use rand::Rng;

const LOG_N_RANGE: std::ops::RangeInclusive<u32> = 8..=20;

// ---------------------------------------------------------------------------
// Lane 1: Scalar baseline (no SIMD, pure u32 arithmetic)
// ---------------------------------------------------------------------------

fn scalar_butterfly_pass(data: &mut [u32], twiddles: &[u32]) {
    let n = data.len();
    let n2 = n / 2;
    let p = BABYBEAR_P;
    for i in 0..n2 {
        let a = data[i];
        let b = data[i + n2];
        let w = twiddles[i];
        // DIF butterfly: (a + b, (a - b) * w)
        let sum = a.wrapping_add(b);
        let sum = if sum >= p { sum - p } else { sum };
        let diff = if a >= b { a - b } else { a + p - b };
        let y = babybear_mul_reference(diff, w);
        data[i] = sum;
        data[i + n2] = y;
    }
}

// ---------------------------------------------------------------------------
// Lane 2: Compiler-vectorized AVX2 (uses target_feature to hint the compiler)
// ---------------------------------------------------------------------------

#[target_feature(enable = "avx2")]
unsafe fn avx2_butterfly_pass(data: *mut u32, twiddles: *const u32, n: usize) {
    let n2 = n / 2;
    let p = BABYBEAR_P;
    for i in 0..n2 {
        let a = *data.add(i);
        let b = *data.add(i + n2);
        let w = *twiddles.add(i);
        // DIF butterfly: (a + b, (a - b) * w)
        let sum = a.wrapping_add(b);
        let sum = if sum >= p { sum - p } else { sum };
        let diff = if a >= b { a - b } else { a + p - b };
        let y = babybear_mul_reference(diff, w);
        *data.add(i) = sum;
        *data.add(i + n2) = y;
    }
}

// ---------------------------------------------------------------------------
// Lane 3: Hand-written AVX-512 (direct intrinsic kernel)
// ---------------------------------------------------------------------------

unsafe fn avx512_butterfly_pass(data: *mut u32, twiddles: *const u32, n: usize) {
    avx512_butterfly_pass_32(data, twiddles, n);
}

// ---------------------------------------------------------------------------
// Correctness gate: verify all three lanes produce identical output
// ---------------------------------------------------------------------------

fn correctness_gate() -> bool {
    let mut rng = rand::thread_rng();
    for &log_n in &[8u32, 10, 12, 16, 20] {
        let n = 1usize << log_n;
        let n2 = n / 2;
        let data_init: Vec<u32> = (0..n).map(|_| rng.gen::<u32>() % BABYBEAR_P).collect();
        let twiddles: Vec<u32> = (0..n2).map(|_| rng.gen::<u32>() % BABYBEAR_P).collect();

        // Lane 1: scalar
        let mut d1 = data_init.clone();
        scalar_butterfly_pass(&mut d1, &twiddles);

        // Lane 2: AVX2
        let mut d2 = data_init.clone();
        unsafe { avx2_butterfly_pass(d2.as_mut_ptr(), twiddles.as_ptr(), n); }

        // Lane 3: AVX-512
        let mut d3 = data_init.clone();
        unsafe { avx512_butterfly_pass(d3.as_mut_ptr(), twiddles.as_ptr(), n); }

        if d1 != d2 || d1 != d3 {
            eprintln!("CORRECTNESS GATE FAILED at log_n={}", log_n);
            eprintln!("  scalar[0..4]: {:?}", &d1[..4.min(n)]);
            eprintln!("  avx2[0..4]:   {:?}", &d2[..4.min(n)]);
            eprintln!("  avx512[0..4]: {:?}", &d3[..4.min(n)]);
            return false;
        }
    }
    eprintln!("Correctness gate: PASS (all three lanes agree on sizes 2^8 through 2^20)");
    true
}

// ---------------------------------------------------------------------------
// Benchmark
// ---------------------------------------------------------------------------

fn bench_three_lane(c: &mut Criterion) {
    if !is_avx512_supported() {
        eprintln!("AVX-512 not supported - skipping three-lane benchmark");
        return;
    }

    if !correctness_gate() {
        eprintln!("Correctness gate failed - aborting benchmark");
        return;
    }

    let mut rng = rand::thread_rng();
    let mut group = c.benchmark_group("dif_butterfly_three_lane");

    for log_n in LOG_N_RANGE {
        let n = 1usize << log_n;
        let n2 = n / 2;

        // Shared setup
        let data_init: Vec<u32> = (0..n).map(|_| rng.gen::<u32>() % BABYBEAR_P).collect();
        let twiddles: Vec<u32> = (0..n2).map(|_| rng.gen::<u32>() % BABYBEAR_P).collect();
        let label = format!("2^{}", log_n);

        // Lane 1: Scalar
        group.bench_with_input(BenchmarkId::new("scalar", &label), &n, |b, _| {
            b.iter_batched(
                || data_init.clone(),
                |mut d| {
                    black_box(&mut d);
                    scalar_butterfly_pass(&mut d, &twiddles);
                    black_box(d);
                },
                BatchSize::SmallInput,
            );
        });

        // Lane 2: AVX2 (compiler-vectorized)
        group.bench_with_input(BenchmarkId::new("avx2", &label), &n, |b, _| {
            b.iter_batched(
                || data_init.clone(),
                |mut d| {
                    black_box(&mut d);
                    unsafe { avx2_butterfly_pass(d.as_mut_ptr(), twiddles.as_ptr(), n); }
                    black_box(d);
                },
                BatchSize::SmallInput,
            );
        });

        // Lane 3: AVX-512 (hand-written SIMD)
        group.bench_with_input(BenchmarkId::new("avx512", &label), &n, |b, _| {
            b.iter_batched(
                || data_init.clone(),
                |mut d| {
                    black_box(&mut d);
                    unsafe { avx512_butterfly_pass(d.as_mut_ptr(), twiddles.as_ptr(), n); }
                    black_box(d);
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group! {
    name = three_lane;
    config = Criterion::default().sample_size(50).warm_up_time(std::time::Duration::from_millis(500)).measurement_time(std::time::Duration::from_secs(2));
    targets = bench_three_lane
}
criterion_main!(three_lane);
