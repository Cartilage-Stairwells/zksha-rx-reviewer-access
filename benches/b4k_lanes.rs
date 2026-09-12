//! B4-K — kernel/DFT-level comparative performance.
//!
//! Every lane is benchmarked under the CURRENT build configuration; the
//! configuration (toolchain + target features) is disclosed per run in the
//! B4 record — differing toolchains do NOT constitute a controlled
//! single-variable experiment across runs.
//!
//! Lanes:
//! - zksha_avx512      : ZkshaDifAdapter, AVX-512 backend (subject)
//! - zksha_scalar      : ZkshaDifAdapter, scalar backend (attribution baseline)
//! - zksha_reference   : ZkshaDifAdapter, reference backend (correctness oracle)
//! - plonky3_radix2dit : Plonky3 Radix2Dit (oracle; packing selected by build cfg)
//!
//! Correctness gate BEFORE timing: each lane's output must be bit-identical
//! to the reference output at every size; any mismatch aborts the run.

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use p3_baby_bear::BabyBear;
use p3_dft::{Radix2Dit, TwoAdicSubgroupDft};
use p3_field::AbstractField;
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;
use std::time::Duration;

use avx512_butterfly::plonky3_adapter::{is_avx512_supported, ZkshaDifAdapter};

type F = BabyBear;

/// Deterministic pseudorandom field elements (same inputs for every lane).
fn make_values(count: usize, seed: u64) -> Vec<F> {
    let mut s = seed;
    (0..count)
        .map(|_| {
            s = s
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let v = ((s >> 33) % 0x78000001) as u32;
            F::from_canonical_u32(v)
        })
        .collect()
}

fn extract(m: RowMajorMatrix<F>) -> Vec<F> {
    m.to_row_major_matrix().values
}

fn assert_identical(a: &[F], b: &[F], label: &str) {
    assert_eq!(a.len(), b.len(), "{label}: length mismatch");
    for i in 0..a.len() {
        if a[i] != b[i] {
            panic!(
                "{label}: mismatch at index {i}: lane={} reference={}",
                a[i], b[i]
            );
        }
    }
}

fn bench_shape(c: &mut Criterion, shape_name: &str, width: usize, log_sizes: &[usize]) {
    let mut group = c.benchmark_group(format!("b4k_dft_batch_w{width}"));
    group
        .sample_size(30)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(2));

    println!("\n=== B4-K shape {shape_name} (width {width}) — host AVX-512: {} ===", is_avx512_supported());

    for &log_n in log_sizes {
        let n = 1usize << log_n;
        let mat = RowMajorMatrix::new(make_values(n * width, 0xB4C0FFEE), width);

        // ---- Correctness gate before timing (every lane, every size) ----
        let reference_out = {
            let lane = ZkshaDifAdapter::reference();
            extract(lane.dft_batch(mat.clone()))
        };

        let zksha_avx = ZkshaDifAdapter::avx512();
        assert_identical(
            &extract(zksha_avx.dft_batch(mat.clone())),
            &reference_out,
            &format!("gate zksha_avx512 w{width} 2^{log_n}"),
        );

        let zksha_sca = ZkshaDifAdapter::scalar();
        assert_identical(
            &extract(zksha_sca.dft_batch(mat.clone())),
            &reference_out,
            &format!("gate zksha_scalar w{width} 2^{log_n}"),
        );

        let p3 = Radix2Dit::<F>::default();
        assert_identical(
            &extract(p3.dft_batch(mat.clone())),
            &reference_out,
            &format!("gate plonky3_radix2dit w{width} 2^{log_n}"),
        );

        group.throughput(Throughput::Bytes((n * width * 4) as u64));

        let size_label = format!("n2^{log_n}");
        group.bench_with_input(
            BenchmarkId::new("zksha_avx512", &size_label),
            &mat,
            |b, m| {
                b.iter_batched(
                    || m.clone(),
                    |m| zksha_avx.dft_batch(m),
                    BatchSize::SmallInput,
                )
            },
        );
        group.bench_with_input(
            BenchmarkId::new("zksha_scalar", &size_label),
            &mat,
            |b, m| {
                b.iter_batched(
                    || m.clone(),
                    |m| zksha_sca.dft_batch(m),
                    BatchSize::SmallInput,
                )
            },
        );
        group.bench_with_input(
            BenchmarkId::new("plonky3_radix2dit", &size_label),
            &mat,
            |b, m| {
                b.iter_batched(|| m.clone(), |m| p3.dft_batch(m), BatchSize::SmallInput)
            },
        );
    }
    group.finish();
}

fn bench_b4k(c: &mut Criterion) {
    // 2^8 through 2^20 (authorized size range)
    let log_sizes: Vec<usize> = (8..=20).collect();
    bench_shape(c, "single_column", 1, &log_sizes);
    bench_shape(c, "prover_batch", 4, &log_sizes);
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench_b4k
}
criterion_main!(benches);
