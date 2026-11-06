//! Bit-exact admission gate: optimized kernel vs original kernel.
//!
//! The optimized pass (avx512_butterfly_opt::avx512_butterfly_pass_32_opt)
//! may only replace the original (avx512_butterfly_32bit::avx512_butterfly_pass_32)
//! if it produces byte-for-byte identical output in every configuration:
//! all power-of-two sizes 2^8..2^20, non-multiple-of-16 tails, boundary values,
//! and randomized inputs. This test is that gate.

#![cfg(target_arch = "x86_64")]

use avx512_butterfly::avx512_butterfly_32bit::{is_avx512_supported, avx512_butterfly_pass_32};
use avx512_butterfly::avx512_butterfly_opt::avx512_butterfly_pass_32_opt;
use avx512_butterfly::avx512_butterfly_opt::avx512_butterfly_pass_32_unroll;
use avx512_butterfly::field::babybear::constants::BABYBEAR_P;
use rand::Rng;

fn run_pass_over(data: &mut [u32], twiddles: &[u32], use_opt: u8) {
    let n = data.len();
    debug_assert_eq!(twiddles.len(), n / 2);
    unsafe {
        match use_opt {
            0 => avx512_butterfly_pass_32(data.as_mut_ptr(), twiddles.as_ptr(), n),
            1 => avx512_butterfly_pass_32_opt(data.as_mut_ptr(), twiddles.as_ptr(), n),
            _ => avx512_butterfly_pass_32_unroll(data.as_mut_ptr(), twiddles.as_ptr(), n),
        }
    }
}

#[test]
fn optimized_kernel_bit_exact_vs_original() {
    if !is_avx512_supported() {
        eprintln!("AVX-512 not supported on this runner — skipping kernel A/B gate");
        return;
    }

    let mut rng = rand::thread_rng();

    // 1. All power-of-two sizes used by the canonical benchmark (full SIMD path)
    for log_n in 8..=20u32 {
        let n = 1usize << log_n;
        let data_init: Vec<u32> = (0..n).map(|_| rng.gen::<u32>() % BABYBEAR_P).collect();
        let twiddles: Vec<u32> = (0..n / 2).map(|_| rng.gen::<u32>() % BABYBEAR_P).collect();

        let mut d_orig = data_init.clone();
        let mut d_opt = data_init.clone();
        run_pass_over(&mut d_orig, &twiddles, 0);
        run_pass_over(&mut d_opt, &twiddles, 1);
        let mut d_unr = data_init.clone();
        run_pass_over(&mut d_unr, &twiddles, 2);
        assert_eq!(d_orig, d_opt, "bit-exact failure at n=2^{}", log_n);
        assert_eq!(d_orig, d_unr, "unroll bit-exact failure at n=2^{}", log_n);
    }

    // 2. Non-multiple-of-16 tails (mixed SIMD + scalar tail path)
    for &n in &[34usize, 40, 100, 1000, 2050, 4096 + 22] {
        let data_init: Vec<u32> = (0..n).map(|_| rng.gen::<u32>() % BABYBEAR_P).collect();
        let twiddles: Vec<u32> = (0..n / 2).map(|_| rng.gen::<u32>() % BABYBEAR_P).collect();

        let mut d_orig = data_init.clone();
        let mut d_opt = data_init.clone();
        run_pass_over(&mut d_orig, &twiddles, 0);
        run_pass_over(&mut d_opt, &twiddles, 1);
        assert_eq!(d_orig, d_opt, "bit-exact failure at n={}", n);
        let mut d_unr = data_init.clone();
        run_pass_over(&mut d_unr, &twiddles, 2);
        assert_eq!(d_orig, d_unr, "unroll bit-exact failure at n={}", n);
    }

    // 3. Boundary values: 0, 1, p-1 in every lane combination class
    for &n in &[32usize, 64, 96] {
        let mut data_init: Vec<u32> = vec![0; n];
        for (i, v) in data_init.iter_mut().enumerate() {
            *v = match i % 4 {
                0 => 0,
                1 => 1,
                2 => BABYBEAR_P - 1,
                _ => BABYBEAR_P - 2,
            };
        }
        let mut twiddles: Vec<u32> = vec![0; n / 2];
        for (i, w) in twiddles.iter_mut().enumerate() {
            *w = match i % 3 {
                0 => 0,
                1 => 1,
                _ => BABYBEAR_P - 1,
            };
        }
        let mut d_orig = data_init.clone();
        let mut d_opt = data_init.clone();
        run_pass_over(&mut d_orig, &twiddles, 0);
        run_pass_over(&mut d_opt, &twiddles, 1);
        assert_eq!(d_orig, d_opt, "bit-exact failure at boundary n={}", n);
        let mut d_unr = data_init.clone();
        run_pass_over(&mut d_unr, &twiddles, 2);
        assert_eq!(d_orig, d_unr, "unroll bit-exact failure at boundary n={}", n);
    }

    // 4. Repeated randomized rounds (stateless distributions)
    for round in 0..20 {
        let n = 256 + rng.gen::<usize>() % 2048;
        let data_init: Vec<u32> = (0..n).map(|_| rng.gen::<u32>() % BABYBEAR_P).collect();
        let twiddles: Vec<u32> = (0..n / 2).map(|_| rng.gen::<u32>() % BABYBEAR_P).collect();
        let mut d_orig = data_init.clone();
        let mut d_opt = data_init.clone();
        run_pass_over(&mut d_orig, &twiddles, 0);
        run_pass_over(&mut d_opt, &twiddles, 1);
        assert_eq!(d_orig, d_opt, "bit-exact failure at random round {}", round);
        let mut d_unr = data_init.clone();
        run_pass_over(&mut d_unr, &twiddles, 2);
        assert_eq!(d_orig, d_unr, "unroll bit-exact failure at random round {}", round);
    }

    eprintln!("kernel opt admission gate: PASS (bit-exact on all sizes, tails, boundaries, random rounds)");
}
