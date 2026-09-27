use std::time::Instant;
use p3_baby_bear::BabyBear;
use p3_field::{TwoAdicField, AbstractField};

fn compute_twiddles_uncached(log_n: usize) -> Vec<Vec<u32>> {
    let n = 1usize << log_n;
    let generator = BabyBear::two_adic_generator(log_n);

    let mut twiddles_per_stage = Vec::with_capacity(log_n);
    for s in 0..log_n {
        let half_len = n >> (s + 1);

        let mut g_s = generator;
        for _ in 0..s {
            g_s = g_s * g_s;
        }

        let mut stage_twiddles = Vec::with_capacity(half_len);
        let mut tw = BabyBear::one();
        for _ in 0..half_len {
            stage_twiddles.push(unsafe { std::mem::transmute(tw) });
            tw = tw * g_s;
        }
        twiddles_per_stage.push(stage_twiddles);
    }
    twiddles_per_stage
}

fn measure(log_n: usize, runs: usize) {
    // Warmup
    let _ = compute_twiddles_uncached(log_n);

    let mut times_ns = Vec::with_capacity(runs);
    for _ in 0..runs {
        let start = Instant::now();
        let tw = compute_twiddles_uncached(log_n);
        let elapsed = start.elapsed().as_nanos() as f64;
        std::hint::black_box(tw);
        times_ns.push(elapsed);
    }
    times_ns.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_ns = times_ns[runs / 2];
    let median_ms = median_ns / 1_000_000.0;
    let min_ms = times_ns[0] / 1_000_000.0;
    let max_ms = times_ns[runs - 1] / 1_000_000.0;
    println!("log_n={:2} (2^{:<2}): median = {:.3} ms ({:.0} ns) [min={:.3}ms, max={:.3}ms] over {} runs", 
             log_n, log_n, median_ms, median_ns, min_ms, max_ms, runs);
}

fn main() {
    println!("=== Direct Twiddle Computation Measurement (Uncached) ===");
    for &log_n in &[12, 16, 20] {
        measure(log_n, 50);
    }
}
