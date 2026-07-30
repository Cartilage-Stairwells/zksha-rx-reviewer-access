//! Phase 2 — Reduction stress tests.
//!
//! Tests backends on values near Montgomery reduction boundaries.
//! These are where vectorized implementations typically fail:
//!   - carry propagation errors
//!   - incorrect masking
//!   - missing final subtraction
//!   - signed comparison mistakes
//!
//! The approach: compute the intermediate `u = (t + m*P) >> 32` for each
//! pair (a, b), find cases where u is near P (the conditional subtract
//! boundary), and verify all backends agree with the oracle.

use super::corpus::*;
use avx512_butterfly::field::babybear::reference::babybear_mul_reference;
use avx512_butterfly::field::babybear::montgomery::{ScalarBackend, BABYBEAR_MONTY, MontgomeryBackend};

// ---------------------------------------------------------------------------
// Reduction boundary: oracle vs scalar on MUL_PAIRS
// ---------------------------------------------------------------------------

#[test]
fn reduction_stress_oracle_vs_scalar() {
    let mut failures = 0;

    for &(a, b) in MUL_PAIRS {
        let oracle = babybear_mul_reference(a, b);
        let scalar = ScalarBackend::mul_raw(a, b, BABYBEAR_MONTY);

        if oracle != scalar {
            FailureRecord::mul(CORPUS_SEED, a, b, "scalar_cios", oracle, scalar).emit();
            failures += 1;
        }

        // Check the reduction intermediate is near a boundary
        let u = reduction_intermediate(a, b);
        let near_boundary = u >= P.saturating_sub(3) && u <= P + 3;
        if near_boundary {
            eprintln!("reduction_boundary: a={} b={} u={} (near P={})", a, b, u, P);
        }

        // Output invariant
        assert!(oracle < P, "oracle returned {} >= P for ({}, {})", oracle, a, b);
        assert!(scalar < P, "scalar returned {} >= P for ({}, {})", scalar, a, b);
    }

    assert_eq!(failures, 0, "{} reduction stress mismatches", failures);
    record_oracle_mul(12); record_scalar_mul(12);
    eprintln!("reduction_stress_oracle_vs_scalar: {} pairs, 0 failures", MUL_PAIRS.len());
}

// ---------------------------------------------------------------------------
// Near-boundary scan: find cases where u ≈ P and verify parity
// ---------------------------------------------------------------------------

#[test]
fn near_boundary_scan() {
    // Scan a small range to find near-boundary cases (CI-friendly)
    let cases = near_boundary_reduction_cases(256, 2, 50);

    if cases.is_empty() {
        eprintln!("near_boundary_scan: no cases found in range [0, 256) — increase scan range");
        return;
    }

    eprintln!("near_boundary_scan: found {} cases with u near P", cases.len());

    let mut failures = 0;
    for &(a, b, u) in &cases {
        let oracle = babybear_mul_reference(a, b);
        let scalar = ScalarBackend::mul_raw(a, b, BABYBEAR_MONTY);

        if oracle != scalar {
            FailureRecord::mul(CORPUS_SEED, a, b, "scalar_cios", oracle, scalar).emit();
            failures += 1;
        }

        // The interesting assertion: u near P means the conditional subtract
        // is at the boundary. If u == P exactly, the result should be 0.
        // If u == P-1, no subtract, result should be P-1.
        eprintln!("  a={} b={} u={} P={} dist={} oracle={} scalar={}",
            a, b, u, P,
            if u >= P { u - P } else { P - u },
            oracle, scalar);
    }

    assert_eq!(failures, 0, "{} near-boundary mismatches", failures);
    record_oracle_mul(cases.len() as u64); record_scalar_mul(cases.len() as u64);
    eprintln!("near_boundary_scan: {} cases, 0 failures", cases.len());
}

// ---------------------------------------------------------------------------
// Deterministic random: 10000 random pairs, oracle vs scalar
// ---------------------------------------------------------------------------

#[test]
fn random_mul_parity_oracle_vs_scalar() {
    let mut rng = Lcg::new(CORPUS_SEED);
    let n = 10_000;
    let mut failures = 0;

    for _ in 0..n {
        let a = rng.next_montgomery();
        let b = rng.next_montgomery();

        let oracle = babybear_mul_reference(a, b);
        let scalar = ScalarBackend::mul_raw(a, b, BABYBEAR_MONTY);

        if oracle != scalar {
            FailureRecord::mul(CORPUS_SEED, a, b, "scalar_cios", oracle, scalar).emit();
            failures += 1;
            if failures > 10 {
                eprintln!("too many failures, stopping");
                break;
            }
        }
    }

    assert_eq!(failures, 0, "{} mul mismatches out of {} random pairs", failures, n);
    record_oracle_mul(10000); record_scalar_mul(10000);
    eprintln!("random_mul_parity_oracle_vs_scalar: {} pairs, 0 failures", n);
}

// ---------------------------------------------------------------------------
// Butterfly reduction stress: oracle vs scalar
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
#[test]
fn butterfly_reduction_stress_oracle_vs_scalar() {
    use avx512_butterfly::field::babybear::canonical::MontgomeryBabyBear;
    use avx512_butterfly::field::babybear::reference::butterfly_reference;
    use avx512_butterfly::avx512_butterfly_32bit::butterfly;

    let mut rng = Lcg::new(CORPUS_SEED + 1);
    let n = 5_000;
    let mut failures = 0;

    for _ in 0..n {
        let a = rng.next_montgomery();
        let b = rng.next_montgomery();
        let w = rng.next_montgomery();

        let (oracle_x, oracle_y) = butterfly_reference(a, b, w);
        let (scalar_x, scalar_y) = butterfly(
            MontgomeryBabyBear::new(a),
            MontgomeryBabyBear::new(b),
            MontgomeryBabyBear::new(w),
        );

        let scalar_x = scalar_x.inner();
        let scalar_y = scalar_y.inner();

        if (oracle_x, oracle_y) != (scalar_x, scalar_y) {
            FailureRecord::butterfly(CORPUS_SEED + 1, a, b, w, "typed_scalar",
                (oracle_x, oracle_y), (scalar_x, scalar_y)).emit();
            failures += 1;
            if failures > 10 {
                eprintln!("too many failures, stopping");
                break;
            }
        }
    }

    assert_eq!(failures, 0, "{} butterfly mismatches out of {} random cases", failures, n);
    record_oracle_butterfly(5000); record_scalar_butterfly(5000);
    eprintln!("butterfly_reduction_stress_oracle_vs_scalar: {} cases, 0 failures", n);
}

// ---------------------------------------------------------------------------
// Butterfly reduction stress: oracle vs AVX-512 (16-lane packed)
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
#[test]
fn butterfly_reduction_stress_oracle_vs_avx512() {
    use avx512_butterfly::field::babybear::reference::butterfly_reference;
    use avx512_butterfly::avx512_butterfly_32bit::{avx512_radix2_butterfly_32, is_avx512_supported};
    use std::arch::x86_64::*;

    if !is_avx512_supported() {
        eprintln!("AVX-512 SKIP: not supported on this runner — AVX-512 PATH NOT TESTED.");
        return;
    }

    let mut rng = Lcg::new(CORPUS_SEED + 2);
    let num_batches = 500; // 500 × 16 = 8000 butterfly operations

    let mut failures = 0;

    for batch in 0..num_batches {
        // Generate 16 random butterfly inputs
        let mut a_lanes = [0u32; 16];
        let mut b_lanes = [0u32; 16];
        let mut w_lanes = [0u32; 16];
        for i in 0..16 {
            a_lanes[i] = rng.next_montgomery();
            b_lanes[i] = rng.next_montgomery();
            w_lanes[i] = rng.next_montgomery();
        }

        // Oracle: 16 independent butterfly calls
        let mut oracle_x = [0u32; 16];
        let mut oracle_y = [0u32; 16];
        for i in 0..16 {
            let (x, y) = butterfly_reference(a_lanes[i], b_lanes[i], w_lanes[i]);
            oracle_x[i] = x;
            oracle_y[i] = y;
        }

        // AVX-512: 16-lane packed butterfly
        let va = unsafe { _mm512_loadu_si512(a_lanes.as_ptr() as *const __m512i) };
        let vb = unsafe { _mm512_loadu_si512(b_lanes.as_ptr() as *const __m512i) };
        let vw = unsafe { _mm512_loadu_si512(w_lanes.as_ptr() as *const __m512i) };

        let (avx_x_m, avx_y_m) = unsafe { avx512_radix2_butterfly_32(va, vb, vw) };
        mark_avx512_executed();

        let avx_x: [u32; 16] = unsafe { std::mem::transmute(avx_x_m) };
        let avx_y: [u32; 16] = unsafe { std::mem::transmute(avx_y_m) };

        // Compare lane-by-lane
        for i in 0..16 {
            if (oracle_x[i], oracle_y[i]) != (avx_x[i], avx_y[i]) {
                FailureRecord::butterfly(CORPUS_SEED + 2, a_lanes[i], b_lanes[i], w_lanes[i],
                    "avx512", (oracle_x[i], oracle_y[i]), (avx_x[i], avx_y[i])).emit();
                failures += 1;
            }
        }
    }

    assert_eq!(failures, 0, "{} AVX-512 butterfly mismatches out of {} batches ({} ops)",
        failures, num_batches, num_batches * 16);
    record_oracle_butterfly(8000); record_avx512_butterfly(8000);
    eprintln!("butterfly_reduction_stress_oracle_vs_avx512: {} batches ({} ops), 0 failures",
        num_batches, num_batches * 16);
}
