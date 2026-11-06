//! Kernel A/B benchmark: original AVX-512 kernel vs optimized interleave kernel,
//! with the optimized scalar CIOS lane as the fair baseline.
//!
//! Correctness gate: original kernel and optimized kernel must produce
//! bit-identical output before any timing happens.
//!
//! Configuration matches CANONICAL_BENCHMARK_METHODOLOGY.md (50 samples, 2s).
//! Run WITHOUT RUSTFLAGS for the true-scalar comparison; with
//! RUSTFLAGS="-C target-cpu=native" for the compiler-vectorized comparison.

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use std::hint::black_box;
use avx512_butterfly::field::babybear::constants::BABYBEAR_P;
use avx512_butterfly::field::babybear::montgomery::{BABYBEAR_MONTY, ScalarBackend};
use avx512_butterfly::avx512_butterfly_32bit::{
    is_avx512_supported, avx512_butterfly_pass_32,
};
use avx512_butterfly::avx512_butterfly_opt::{avx512_butterfly_pass_32_opt, avx512_butterfly_pass_32_unroll};
use rand::Rng;

const LOG_N_RANGE: std::ops::RangeInclusive<u32> = 8..=20;

fn scalar_cios_butterfly_pass(data: &mut [u32], twiddles: &[u32]) {
    let n = data.len();
    let n2 = n / 2;
    let p = BABYBEAR_P;
    let c = BABYBEAR_MONTY;
    for i in 0..n2 {
        let a = data[i];
        let b = data[i + n2];
        let w = twiddles[i];
        let sum = a.wrapping_add(b);
        let sum = if sum >= p { sum - p } else { sum };
        let diff = if a >= b { a - b } else { a + p - b };
        let y = ScalarBackend::mul_raw(diff, w, c);
        data[i] = sum;
        data[i + n2] = y;
    }
}

fn correctness_gate() -> bool {
    let mut rng = rand::thread_rng();
    for &log_n in &[8u32, 10, 12, 16, 20] {
        let n = 1usize << log_n;
        let n2 = n / 2;
        let data_init: Vec<u32> = (0..n).map(|_| rng.gen::<u32>() % BABYBEAR_P).collect();
        let twiddles: Vec<u32> = (0..n2).map(|_| rng.gen::<u32>() % BABYBEAR_P).collect();

        let mut d_scalar = data_init.clone();
        scalar_cios_butterfly_pass(&mut d_scalar, &twiddles);

        let mut d_orig = data_init.clone();
        unsafe { avx512_butterfly_pass_32(d_orig.as_mut_ptr(), twiddles.as_ptr(), n); }

        let mut d_opt = data_init.clone();
        unsafe { avx512_butterfly_pass_32_opt(d_opt.as_mut_ptr(), twiddles.as_ptr(), n); }

        let mut d_unr = data_init.clone();
        unsafe { avx512_butterfly_pass_32_unroll(d_unr.as_mut_ptr(), twiddles.as_ptr(), n); }

        if d_scalar != d_orig || d_orig != d_opt || d_orig != d_unr {
            eprintln!("CORRECTNESS GATE FAILED at log_n={}", log_n);
            eprintln!("  scalar[0..4]: {:?}", &d_scalar[..4.min(n)]);
            eprintln!("  orig[0..4]:   {:?}", &d_orig[..4.min(n)]);
            eprintln!("  opt[0..4]:    {:?}", &d_opt[..4.min(n)]);
            return false;
        }
    }
    eprintln!("Correctness gate: PASS (scalar, orig, opt, unroll all agree on sizes 2^8 through 2^20)");
    true
}

fn bench_kernel_ab(c: &mut Criterion) {
    if !is_avx512_supported() {
        eprintln!("AVX-512 not supported - skipping kernel A/B benchmark");
        return;
    }

    if !correctness_gate() {
        eprintln!("Correctness gate failed - aborting benchmark");
        return;
    }

    let mut rng = rand::thread_rng();
    let mut group = c.benchmark_group("kernel_ab");

    for log_n in LOG_N_RANGE {
        let n = 1usize << log_n;
        let n2 = n / 2;

        let data_init: Vec<u32> = (0..n).map(|_| rng.gen::<u32>() % BABYBEAR_P).collect();
        let twiddles: Vec<u32> = (0..n2).map(|_| rng.gen::<u32>() % BABYBEAR_P).collect();
        let label = format!("2^{}", log_n);

        group.bench_with_input(BenchmarkId::new("scalar_cios", &label), &n, |b, _| {
            b.iter_batched(
                || data_init.clone(),
                |mut d| {
                    black_box(&mut d);
                    scalar_cios_butterfly_pass(&mut d, &twiddles);
                    black_box(d);
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("avx512_orig", &label), &n, |b, _| {
            b.iter_batched(
                || data_init.clone(),
                |mut d| {
                    black_box(&mut d);
                    unsafe { avx512_butterfly_pass_32(d.as_mut_ptr(), twiddles.as_ptr(), n); }
                    black_box(d);
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("avx512_opt", &label), &n, |b, _| {
            b.iter_batched(
                || data_init.clone(),
                |mut d| {
                    black_box(&mut d);
                    unsafe { avx512_butterfly_pass_32_opt(d.as_mut_ptr(), twiddles.as_ptr(), n); }
                    black_box(d);
                },
                BatchSize::SmallInput,
            );
        });

        group.bench_with_input(BenchmarkId::new("avx512_unroll", &label), &n, |b, _| {
            b.iter_batched(
                || data_init.clone(),
                |mut d| {
                    black_box(&mut d);
                    unsafe { avx512_butterfly_pass_32_unroll(d.as_mut_ptr(), twiddles.as_ptr(), n); }
                    black_box(d);
                },
                BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group! {
    name = kernel_ab;
    config = Criterion::default().sample_size(50).warm_up_time(std::time::Duration::from_millis(500)).measurement_time(std::time::Duration::from_secs(2));
    targets = bench_kernel_ab
}
criterion_main!(kernel_ab);
