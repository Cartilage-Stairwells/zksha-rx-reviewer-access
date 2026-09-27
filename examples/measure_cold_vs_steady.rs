use std::time::Instant;
use p3_baby_bear::BabyBear;
use p3_dft::TwoAdicSubgroupDft;
use p3_field::AbstractField;
use p3_matrix::dense::RowMajorMatrix;
use avx512_butterfly::plonky3_adapter::ZkshaDifAdapter;

type F = BabyBear;

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

fn measure_cold_steady_lane(adapter: &ZkshaDifAdapter, log_n: usize, runs: usize) {
    let n = 1usize << log_n;

    let mat = RowMajorMatrix::new(make_values(n, 0xB4C0FFEE), 1);

    // Measure COLD: clear cache first
    ZkshaDifAdapter::clear_twiddle_cache();
    let start_cold = Instant::now();
    let res_cold = adapter.dft_batch(mat.clone());
    let cold_time_ms = start_cold.elapsed().as_nanos() as f64 / 1_000_000.0;
    std::hint::black_box(res_cold);

    // Measure STEADY: cache is now warm, repeat `runs` times
    let mut steady_times = Vec::with_capacity(runs);
    for _ in 0..runs {
        let start_steady = Instant::now();
        let res_steady = adapter.dft_batch(mat.clone());
        let elapsed_ms = start_steady.elapsed().as_nanos() as f64 / 1_000_000.0;
        std::hint::black_box(res_steady);
        steady_times.push(elapsed_ms);
    }
    steady_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let steady_median_ms = steady_times[runs / 2];

    println!("Lane {:?} (2^{}): COLD = {:.3} ms | STEADY median = {:.3} ms (over {} runs)",
             adapter.backend, log_n, cold_time_ms, steady_median_ms, runs);
}

fn main() {
    println!("=== COLD vs STEADY First-Call Cost (w1 2^20) ===");
    let avx = ZkshaDifAdapter::avx512();
    let sca = ZkshaDifAdapter::scalar();
    
    measure_cold_steady_lane(&avx, 20, 30);
    measure_cold_steady_lane(&sca, 20, 30);
}
