//! Phase 1 — Boundary value tests.
//!
//! Tests each backend against the reference oracle on explicit boundary values.
//! Boundary values are where representation assumptions break:
//!   - zero and identity (annihilation, identity propagation)
//!   - max residue P-1 (largest product, lazy reduction stress)
//!   - P/2 (carry into conditional subtract)
//!   - Montgomery identity R mod p

use super::corpus::*;
use avx512_butterfly::field::babybear::reference::babybear_mul_reference;
use avx512_butterfly::field::babybear::montgomery::{ScalarBackend, BABYBEAR_MONTY, MontgomeryBackend};

// ---------------------------------------------------------------------------
// Montgomery multiplication: oracle vs scalar on boundary values
// ---------------------------------------------------------------------------

#[test]
fn mul_boundary_oracle_vs_scalar() {
    let mut failures = 0;

    for &a in BOUNDARY_VALUES {
        for &b in BOUNDARY_VALUES {
            let oracle = babybear_mul_reference(a, b);
            let scalar = ScalarBackend::mul_raw(a, b, BABYBEAR_MONTY);

            if oracle != scalar {
                FailureRecord::mul(CORPUS_SEED, a, b, "scalar_cios", oracle, scalar).emit();
                failures += 1;
            }

            // Output invariant: result must be in [0, p)
            assert!(oracle < P, "oracle returned {} >= P for ({}, {})", oracle, a, b);
            assert!(scalar < P, "scalar returned {} >= P for ({}, {})", scalar, a, b);
        }
    }

    assert_eq!(failures, 0, "{} mul boundary mismatches (oracle vs scalar)", failures);
    record_oracle_mul(100); record_scalar_mul(100);
    eprintln!("mul_boundary_oracle_vs_scalar: {} pairs, 0 failures", BOUNDARY_VALUES.len() * BOUNDARY_VALUES.len());
}

// ---------------------------------------------------------------------------
// Montgomery reduction: reduce_raw vs mul_raw consistency
// ---------------------------------------------------------------------------

#[test]
fn reduce_equals_mul_on_boundary() {
    let mut failures = 0;

    for &a in BOUNDARY_VALUES {
        for &b in BOUNDARY_VALUES {
            let via_mul = ScalarBackend::mul_raw(a, b, BABYBEAR_MONTY);
            let via_reduce = ScalarBackend::reduce_raw((a as u64) * (b as u64), BABYBEAR_MONTY);

            if via_mul != via_reduce {
                eprintln!("reduce != mul for ({}, {}): mul={} reduce={}", a, b, via_mul, via_reduce);
                failures += 1;
            }
        }
    }

    assert_eq!(failures, 0, "{} reduce/mul mismatches on boundary values", failures);
    record_scalar_mul(100);
    eprintln!("reduce_equals_mul_on_boundary: {} pairs, 0 failures", BOUNDARY_VALUES.len() * BOUNDARY_VALUES.len());
}

// ---------------------------------------------------------------------------
// Butterfly: oracle vs typed scalar on boundary cases
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
#[test]
fn butterfly_boundary_oracle_vs_scalar() {
    use avx512_butterfly::field::babybear::canonical::MontgomeryBabyBear;
    use avx512_butterfly::field::babybear::reference::butterfly_reference;
    use avx512_butterfly::avx512_butterfly_32bit::butterfly;

    let mut failures = 0;

    for &(a, b, w) in BUTTERFLY_CASES {
        // Oracle: raw u32 butterfly
        let (oracle_x, oracle_y) = butterfly_reference(a, b, w);

        // Scalar: typed butterfly through MontgomeryBabyBear
        let (scalar_x, scalar_y) = butterfly(
            MontgomeryBabyBear::new(a),
            MontgomeryBabyBear::new(b),
            MontgomeryBabyBear::new(w),
        );

        let scalar_x = scalar_x.inner();
        let scalar_y = scalar_y.inner();

        if (oracle_x, oracle_y) != (scalar_x, scalar_y) {
            FailureRecord::butterfly(CORPUS_SEED, a, b, w, "typed_scalar",
                (oracle_x, oracle_y), (scalar_x, scalar_y)).emit();
            failures += 1;
        }

        // Output invariant: results must be in [0, p)
        assert!(oracle_x < P && oracle_y < P,
            "oracle butterfly out of range for ({}, {}, {})", a, b, w);
        assert!(scalar_x < P && scalar_y < P,
            "scalar butterfly out of range for ({}, {}, {})", a, b, w);
    }

    assert_eq!(failures, 0, "{} butterfly boundary mismatches (oracle vs scalar)", failures);
    record_oracle_butterfly(17); record_scalar_butterfly(17);
    eprintln!("butterfly_boundary_oracle_vs_scalar: {} cases, 0 failures", BUTTERFLY_CASES.len());
}

// ---------------------------------------------------------------------------
// Butterfly: oracle vs AVX-512 on boundary cases (16-lane packed)
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
#[test]
fn butterfly_boundary_oracle_vs_avx512() {
    use avx512_butterfly::field::babybear::reference::butterfly_reference;
    use avx512_butterfly::avx512_butterfly_32bit::{avx512_radix2_butterfly_32, is_avx512_supported};
    use std::arch::x86_64::*;

    if !is_avx512_supported() {
        eprintln!("AVX-512 SKIP: not supported on this runner — AVX-512 PATH NOT TESTED (requires avx512f+avx512dq).");
        return;
    }

    // Pad butterfly cases to 16 lanes by cycling through the corpus
    let mut a_lanes = [0u32; 16];
    let mut b_lanes = [0u32; 16];
    let mut w_lanes = [0u32; 16];

    for i in 0..16 {
        let (a, b, w) = BUTTERFLY_CASES[i % BUTTERFLY_CASES.len()];
        a_lanes[i] = a;
        b_lanes[i] = b;
        w_lanes[i] = w;
    }

    // Oracle: compute each lane independently
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
    let mut failures = 0;
    for i in 0..16 {
        if (oracle_x[i], oracle_y[i]) != (avx_x[i], avx_y[i]) {
            FailureRecord::butterfly(CORPUS_SEED, a_lanes[i], b_lanes[i], w_lanes[i],
                "avx512", (oracle_x[i], oracle_y[i]), (avx_x[i], avx_y[i])).emit();
            failures += 1;
        }

        // Output invariant
        assert!(avx_x[i] < P, "AVX-512 butterfly x[{}] = {} >= P", i, avx_x[i]);
        assert!(avx_y[i] < P, "AVX-512 butterfly y[{}] = {} >= P", i, avx_y[i]);
    }

    assert_eq!(failures, 0, "{} AVX-512 butterfly boundary mismatches", failures);
    record_oracle_butterfly(16); record_avx512_butterfly(16);
    eprintln!("butterfly_boundary_oracle_vs_avx512: 16 lanes, 0 failures");
}
