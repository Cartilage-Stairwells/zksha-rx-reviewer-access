//! Fair four-lane DIF butterfly benchmark: u128 oracle vs optimized scalar CIOS
//! vs compiler AVX2 vs hand-written AVX-512.
//!
//! Purpose: establish the actual engineering improvement of the AVX-512 kernel
//! over the OPTIMIZED scalar implementation (ScalarBackend::mul_raw, CIOS),
//! not over the deliberately slow u128 correctness oracle.
//!
//! Correctness gate verifies all four lanes produce identical output before timing.
//! Configuration matches CANONICAL_BENCHMARK_METHODOLOGY.md (50 samples, 2s).

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use std::hint::black_box;
use avx512_butterfly::field::babybear::constants::BABYBEAR_P;
use avx512_butterfly::field::babybear::reference::babybear_mul_reference;
use avx512_butterfly::field::babybear::montgomery::{BABYBEAR_MONTY, ScalarBackend};
use avx512_butterfly::avx512_butterfly_32bit::{
    is_avx512_supported, avx512_butterfly_pass_32,
};
use rand::Rng;

const LOG_N_RANGE: std::ops::RangeInclusive<u32> = 8..=20;

// ---------------------------------------------------------------------------
// Lane 1: u128 reference oracle (correctness reference, deliberately slow)
// ---------------------------------------------------------------------------

fn oracle_butterfly_pass(data: &mut [u32], twiddles: &[u32]) {
    let n = data.len();
    let n2 = n / 2;
    let p = BABYBEAR_P;
    for i in 0..n2 {
        let a = data[i];
        let b = data[i + n2];
        let w = twiddles[i];
        let sum = a.wrapping_add(b);
        let sum = if sum >= p { sum - p } else { sum };
        let diff = if a >= b { a - b } else { a + p - b };
        let y = babybear_mul_reference(diff, w);
        data[i] = sum;
        data[i + n2] = y;
    }
}

// ---------------------------------------------------------------------------
// Lane 2: Optimized scalar CIOS (ScalarBackend::mul_raw) — the fair baseline.
// Identical loop structure and mod-p add/sub as every other lane; only the
// multiply differs: CIOS Montgomery reduction instead of the u128 oracle.
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Lane 3: Compiler-vectorized AVX2 (target_feature hint, oracle multiply)
// ---------------------------------------------------------------------------

#[target_feature(enable = "avx2")]
unsafe fn avx2_butterfly_pass(data: *mut u32, twiddles: *const u32, n: usize) {
    let n2 = n / 2;
    let p = BABYBEAR_P;
    for i in 0..n2 {
        let a = *data.add(i);
        let b = *data.add(i + n2);
        let w = *twiddles.add(i);
        let sum = a.wrapping_add(b);
        let sum = if sum >= p { sum - p } else { sum };
        let diff = if a >= b { a - b } else { a + p - b };
        let y = babybear_mul_reference(diff, w);
        *data.add(i) = sum;
        *data.add(i + n2) = y;
    }
}

// ---------------------------------------------------------------------------
// Lane 4: Hand-written AVX-512 (direct intrinsic kernel)
// ---------------------------------------------------------------------------

unsafe fn avx512_butterfly_pass(data: *mut u32, twiddles: *const u32, n: usize) {
    avx512_butterfly_pass_32(data, twiddles, n);
}

// ---------------------------------------------------------------------------
// Correctness gate: all four lanes must agree
// ---------------------------------------------------------------------------

fn correctness_gate() -> bool {
    let mut rng = rand::thread_rng();
    for &log_n in &[8u32, 10, 12, 16, 20] {
        let n = 1usize << log_n;
        let n2 = n / 2;
        let data_init: Vec<u32> = (0..n).map(|_| rng.gen::<u32>() % BABYBEAR_P).collect();
        let twiddles: Vec<u32> = (0..n2).map(|_| rng.gen::<u32>() % BABYBEAR_P).collect();

        let mut d1 = data_init.clone();
        oracle_butterfly_pass(&mut d1, &twiddles);

        let mut d2 = data_init.clone();
        scalar_cios_butterfly_pass(&mut d2, &twiddles);

        let mut d3 = data_init.clone();
        unsafe { avx2_butterfly_pass(d3.as_mut_ptr(), twiddles.as_ptr(), n); }

        let mut d4 = data_init.clone();
        unsafe { avx512_butterfly_pass(d4.as_mut_ptr(), twiddles.as_ptr(), n); }

        if d1 != d2 || d1 != d3 || d1 != d4 {
            eprintln!("CORRECTNESS GATE FAILED at log_n={}", log_n);
            eprintln!("  oracle[0..4]:  {:?}", &d1[..4.min(n)]);
            eprintln!("  cios[0..4]:    {:?}", &d2[..4.min(n)]);
            eprintln!("  avx2[0..4]:    {:?}", &d3[..4.min(n)]);
            eprintln!("  avx512[0..4]:  {:?}", &d4[..4.min(n)]);
            return false;
        }
    }
    eprintln!("Correctness gate: PASS (all four lanes agree on sizes 2^8 through 2^20)");
    true
}

// ---------------------------------------------------------------------------
// Benchmark
// ---------------------------------------------------------------------------

fn bench_fair_lane(c: &mut Criterion) {
    if !is_avx512_supported() {
        eprintln!("AVX-512 not supported - skipping fair-lane benchmark");
        return;
    }

    if !correctness_gate() {
        eprintln!("Correctness gate failed - aborting benchmark");
        return;
    }

    let mut rng = rand::thread_rng();
    let mut group = c.benchmark_group("dif_butterfly_fair_lane");

    for log_n in LOG_N_RANGE {
        let n = 1usize << log_n;
        let n2 = n / 2;

        let data_init: Vec<u32> = (0..n).map(|_| rng.gen::<u32>() % BABYBEAR_P).collect();
        let twiddles: Vec<u32> = (0..n2).map(|_| rng.gen::<u32>() % BABYBEAR_P).collect();
        let label = format!("2^{}", log_n);

        group.bench_with_input(BenchmarkId::new("oracle_u128", &label), &n, |b, _| {
            b.iter_batched(
                || data_init.clone(),
                |mut d| {
                    black_box(&mut d);
                    oracle_butterfly_pass(&mut d, &twiddles);
                    black_box(d);
                },
                BatchSize::SmallInput,
            );
        });

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
    name = fair_lane;
    config = Criterion::default().sample_size(50).warm_up_time(std::time::Duration::from_millis(500)).measurement_time(std::time::Duration::from_secs(2));
    targets = bench_fair_lane
}
criterion_main!(fair_lane);
