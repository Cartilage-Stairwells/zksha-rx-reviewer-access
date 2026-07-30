//! Domain-wrapper verification — Commit 3A.
//!
//! Frozen contract: these tests must remain green forever.
//! - Roundtrip: canonical → Montgomery → canonical is identity.
//! - Multiplication semantics: decode(mont(a) * mont(b)) == a*b mod p.
//! - Domain separation: CanonicalBabyBear * MontgomeryBabyBear does not compile.
//!   (Enforced by the type system; no runtime test needed.)
//!
//! No corpus changes. No NTT behavior changes.

#[cfg(test)]
mod babybear_domain {
    use avx512_butterfly::field::babybear::canonical::{CanonicalBabyBear, MontgomeryBabyBear};
    use avx512_butterfly::field::babybear::constants::BABYBEAR_P;
    use avx512_butterfly::field::babybear::reference::montgomery_mul_reference;
    use avx512_butterfly::field::babybear::constants::BABYBEAR_R_INV_MOD_P;
    use avx512_butterfly::field::babybear::montgomery::{ScalarBackend, BABYBEAR_MONTY};

    use proptest::prelude::*;

    // -----------------------------------------------------------------------
    // 1. Roundtrip: canonical → Montgomery → canonical == identity
    // -----------------------------------------------------------------------

    #[test]
    fn roundtrip_boundary() {
        let cases = [
            0u32, 1, 2, 42,
            BABYBEAR_P / 2,
            BABYBEAR_P - 2,
            BABYBEAR_P - 1,
        ];
        for x in cases {
            let c = CanonicalBabyBear::new_unchecked(x);
            let recovered = c.to_montgomery().to_canonical();
            assert_eq!(recovered, c, "roundtrip failed for x={}", x);
        }
    }

    proptest! {
        #[test]
        fn prop_roundtrip(x in 0u32..BABYBEAR_P) {
            let c = CanonicalBabyBear::new_unchecked(x);
            prop_assert_eq!(c.to_montgomery().to_canonical(), c,
                "roundtrip failed for x={}", x);
        }
    }

    // -----------------------------------------------------------------------
    // 2. Multiplication semantics:
    //    decode(mont(a) * mont(b)) == a*b mod p
    // -----------------------------------------------------------------------

    #[test]
    fn mul_semantics_boundary() {
        let pairs: &[(u32, u32)] = &[
            (0, 0), (0, 1), (1, 0), (1, 1),
            (2, 2), (42, 57),
            (BABYBEAR_P - 1, 1),
            (BABYBEAR_P - 1, BABYBEAR_P - 1),
            (BABYBEAR_P / 2, BABYBEAR_P / 2),
            (100_000 % BABYBEAR_P, 999_999 % BABYBEAR_P),
        ];
        for &(a, b) in pairs {
            let result = (CanonicalBabyBear::new(a).to_montgomery()
                * CanonicalBabyBear::new(b).to_montgomery())
                .to_canonical()
                .inner();
            let expected = ((a as u64 * b as u64) % BABYBEAR_P as u64) as u32;
            assert_eq!(result, expected,
                "mul semantics failed for a={} b={}: got {} expected {}", a, b, result, expected);
        }
    }

    proptest! {
        #[test]
        fn prop_mul_semantics(a in 0u32..BABYBEAR_P, b in 0u32..BABYBEAR_P) {
            let result = (CanonicalBabyBear::new_unchecked(a).to_montgomery()
                * CanonicalBabyBear::new_unchecked(b).to_montgomery())
                .to_canonical()
                .inner();
            let expected = ((a as u64 * b as u64) % BABYBEAR_P as u64) as u32;
            prop_assert_eq!(result, expected,
                "mul({},{}) expected {} got {}", a, b, expected, result);
        }

        #[test]
        fn prop_mul_commutativity(a in 0u32..BABYBEAR_P, b in 0u32..BABYBEAR_P) {
            let ma = CanonicalBabyBear::new_unchecked(a).to_montgomery();
            let mb = CanonicalBabyBear::new_unchecked(b).to_montgomery();
            prop_assert_eq!((ma * mb).inner(), (mb * ma).inner(),
                "commutativity failed for a={} b={}", a, b);
        }

        #[test]
        fn prop_mul_reference_equivalence(a in 0u32..BABYBEAR_P, b in 0u32..BABYBEAR_P) {
            // Verify against the full reference oracle (includes implicit R factor)
            let ma = CanonicalBabyBear::new_unchecked(a).to_montgomery();
            let mb = CanonicalBabyBear::new_unchecked(b).to_montgomery();
            let product_raw = (ma * mb).inner();
            // oracle: (aR)(bR)R^{-1} = abR
            let oracle_raw = montgomery_mul_reference(ma.inner(), mb.inner(), BABYBEAR_P, BABYBEAR_R_INV_MOD_P);
            // exit domain: abR * R^{-1} = ab
            let oracle_canonical = ScalarBackend::mul_raw(oracle_raw, 1, BABYBEAR_MONTY);
            let product_canonical = (ma * mb).to_canonical().inner();
            prop_assert_eq!(product_canonical, oracle_canonical,
                "reference equivalence failed for a={} b={}", a, b);
            prop_assert_eq!(product_raw, oracle_raw,
                "raw Montgomery product mismatch for a={} b={}", a, b);
        }

        #[test]
        fn prop_output_in_range(a in 0u32..BABYBEAR_P, b in 0u32..BABYBEAR_P) {
            let product = CanonicalBabyBear::new_unchecked(a).to_montgomery()
                * CanonicalBabyBear::new_unchecked(b).to_montgomery();
            prop_assert!(product.inner() < BABYBEAR_P,
                "product.inner()={} >= p for a={} b={}", product.inner(), a, b);
            prop_assert!(product.to_canonical().inner() < BABYBEAR_P,
                "canonical product out of range for a={} b={}", a, b);
        }
    }

    // -----------------------------------------------------------------------
    // 3. Zero and one identity elements
    // -----------------------------------------------------------------------

    #[test]
    fn zero_annihilation() {
        let zero = CanonicalBabyBear::new(0).to_montgomery();
        for x in [0u32, 1, 42, BABYBEAR_P - 1, BABYBEAR_P / 2] {
            let mx = CanonicalBabyBear::new(x).to_montgomery();
            assert_eq!((zero * mx).to_canonical().inner(), 0, "0 * x != 0 for x={}", x);
            assert_eq!((mx * zero).to_canonical().inner(), 0, "x * 0 != 0 for x={}", x);
        }
    }

    // -----------------------------------------------------------------------
    // NOTE: Domain separation is a compile-time property.
    // The following does NOT compile — that is the point:
    //
    //   let c = CanonicalBabyBear::new(1);
    //   let m = MontgomeryBabyBear::from_canonical(c);
    //   let _ = c * m;  // error[E0308]: mismatched types
    //
    // No runtime test is needed or possible.
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Commit 3B — typed butterfly wrapper
    // Verifies: butterfly(mont(a), mont(b), mont(w)) is in-range after decode.
    // Success criterion: no assertion value changes vs Commit 3A.
    // -----------------------------------------------------------------------

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn typed_butterfly_matches_raw() {
        use avx512_butterfly::avx512_butterfly_32bit::butterfly;
        use avx512_butterfly::field::babybear::montgomery::{BABYBEAR_SCALAR, MontgomeryBackend};

        // Inline re-derivation of scalar_butterfly_32 (private fn) — identical arithmetic.
        // This is the raw reference we compare against.
        let raw_butterfly = |a: u32, b: u32, w: u32| -> (u32, u32) {
            // DIF butterfly: x = a + b mod p, y = (a - b) * w mod p
            let p = BABYBEAR_P;
            let sum = a.wrapping_add(b);
            let a_new = if sum >= p { sum - p } else { sum };
            let diff = if a >= b { a - b } else { a + p - b };
            let b_new = BABYBEAR_SCALAR.mul(diff, w);
            (a_new, b_new)
        };

        let cases: &[(u32, u32, u32)] = &[
            (0, 0, 1),
            (1, 0, 1),
            (0, 1, 1),
            (1, 1, 1),
            (42, 57, 13),
            (BABYBEAR_P - 1, 1, 1),
            (BABYBEAR_P - 1, BABYBEAR_P - 1, BABYBEAR_P - 1),
            (BABYBEAR_P / 2, BABYBEAR_P / 3, BABYBEAR_P / 5),
        ];

        for &(a_raw, b_raw, w_raw) in cases {
            let a = MontgomeryBabyBear(a_raw);
            let b = MontgomeryBabyBear(b_raw);
            let w = MontgomeryBabyBear(w_raw);

            let (ra, rb) = butterfly(a, b, w);

            // Equivalence: typed output matches raw derivation exactly.
            let (expected_a, expected_b) = raw_butterfly(a_raw, b_raw, w_raw);
            assert_eq!(ra.inner(), expected_a,
                "butterfly a mismatch for inputs ({},{},{}): got {} expected {}",
                a_raw, b_raw, w_raw, ra.inner(), expected_a);
            assert_eq!(rb.inner(), expected_b,
                "butterfly b mismatch for inputs ({},{},{}): got {} expected {}",
                a_raw, b_raw, w_raw, rb.inner(), expected_b);

            // Validity: canonical decode is in-range (belt-and-suspenders).
            assert!(ra.to_canonical().inner() < BABYBEAR_P,
                "butterfly output a canonical out of range for inputs ({},{},{})",
                a_raw, b_raw, w_raw);
            assert!(rb.to_canonical().inner() < BABYBEAR_P,
                "butterfly output b canonical out of range for inputs ({},{},{})",
                a_raw, b_raw, w_raw);
        }
    }

    // -----------------------------------------------------------------------
    // Commit 4 — Test 1: butterfly_reference_agreement
    // Verifies: butterfly() output == butterfly_reference() output on all inputs.
    // This is the oracle contract stated in the butterfly() doc block.
    // -----------------------------------------------------------------------

    mod butterfly_reference_agreement {
        use avx512_butterfly::field::babybear::canonical::MontgomeryBabyBear;
        use avx512_butterfly::field::babybear::constants::BABYBEAR_P;
        use avx512_butterfly::field::babybear::reference::butterfly_reference;
        use avx512_butterfly::avx512_butterfly_32bit::butterfly;
        use proptest::prelude::*;

        // Deterministic boundary cases — run always.
        #[test]
        fn boundary_cases() {
            let p = BABYBEAR_P;
            let cases: &[(u32, u32, u32)] = &[
                (0, 0, 1),
                (0, 1, 1),
                (1, 0, 1),
                (1, 1, 1),
                (42, 57, 13),
                (p - 1, 1, 1),
                (p - 1, p - 1, 1),
                (p - 1, p - 1, p - 1),
                (p / 2, p / 3, p / 5),
                (p / 7, p / 11, p / 13),
            ];
            for &(a_raw, b_raw, w_raw) in cases {
                let (exp_a, exp_b) = butterfly_reference(a_raw, b_raw, w_raw);
                let (got_a, got_b) = butterfly(
                    MontgomeryBabyBear(a_raw),
                    MontgomeryBabyBear(b_raw),
                    MontgomeryBabyBear(w_raw),
                );
                assert_eq!(got_a.inner(), exp_a,
                    "butterfly a mismatch for ({},{},{}): got={} exp={}",
                    a_raw, b_raw, w_raw, got_a.inner(), exp_a);
                assert_eq!(got_b.inner(), exp_b,
                    "butterfly b mismatch for ({},{},{}): got={} exp={}",
                    a_raw, b_raw, w_raw, got_b.inner(), exp_b);
            }
        }

        // Proptest: 10 000 random triples across [0, p) × [0, p) × [0, p).
        proptest! {
            #![proptest_config(proptest::test_runner::Config {
                cases: 10_000,
                ..Default::default()
            })]
            #[test]
            fn prop_matches_reference(
                a in 0u32..BABYBEAR_P,
                b in 0u32..BABYBEAR_P,
                w in 0u32..BABYBEAR_P,
            ) {
                let (exp_a, exp_b) = butterfly_reference(a, b, w);
                let (got_a, got_b) = butterfly(
                    MontgomeryBabyBear(a),
                    MontgomeryBabyBear(b),
                    MontgomeryBabyBear(w),
                );
                prop_assert_eq!(got_a.inner(), exp_a,
                    "butterfly a: ({},{},{}) got={} exp={}", a, b, w, got_a.inner(), exp_a);
                prop_assert_eq!(got_b.inner(), exp_b,
                    "butterfly b: ({},{},{}) got={} exp={}", a, b, w, got_b.inner(), exp_b);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Commit 4 — Test 2: cross_backend_equivalence
    // Verifies: avx512_butterfly_pass_32 == scalar_butterfly_32 lane-by-lane.
    // Uses a single shared input vector generated once. Both backends receive
    // identical data — failures are deterministic and reproducible.
    // Only runs on x86_64 (AVX-512 path requires it).
    // -----------------------------------------------------------------------

    #[cfg(target_arch = "x86_64")]
    mod cross_backend_equivalence {
        use avx512_butterfly::avx512_butterfly_32bit::{
            avx512_butterfly_pass_32, is_avx512_supported, butterfly,
        };
        use avx512_butterfly::field::babybear::canonical::MontgomeryBabyBear;
        use avx512_butterfly::field::babybear::constants::BABYBEAR_P;

        /// Scalar reference pass — mirrors scalar_butterfly_32 exactly,
        /// without depending on any production function being pub.
        fn scalar_pass_reference(data: &mut [u32], twiddles: &[u32]) {
            let n2 = data.len() / 2;
            let p  = BABYBEAR_P;
            for i in 0..n2 {
                let a = data[i];
                let b = data[i + n2];
                let w = twiddles[i];
                let (a_new, b_new) = butterfly(
                    MontgomeryBabyBear(a),
                    MontgomeryBabyBear(b),
                    MontgomeryBabyBear(w),
                );
                let _ = p;
                data[i]      = a_new.inner();
                data[i + n2] = b_new.inner();
            }
        }

        #[test]
        fn avx512_matches_scalar_shared_input() {
            if !is_avx512_supported() {
                eprintln!("AVX-512 not available — skipping cross_backend_equivalence");
                return;
            }

            // Fixed seed — deterministic, reproducible on any failure.
            // Values are valid Montgomery-encoded BabyBear (< p).
            let p = BABYBEAR_P;
            let seed: u64 = 0xDEAD_BEEF_CAFE_1234;
            let mut rng_state = seed;
            let lcg = |state: &mut u64| -> u32 {
                *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                ((*state >> 33) as u32) % p
            };

            for log_n in 5..=10usize {
                let n2 = 1usize << log_n;
                let n  = n2 * 2;

                // ONE shared input vector — both backends get identical data.
                let data_init: Vec<u32> = (0..n).map(|_| lcg(&mut rng_state)).collect();
                let twiddles:  Vec<u32> = (0..n2).map(|_| lcg(&mut rng_state)).collect();

                let mut data_scalar = data_init.clone();
                let mut data_avx    = data_init.clone();

                // Scalar path (typed butterfly, one lane at a time).
                scalar_pass_reference(&mut data_scalar, &twiddles);

                // AVX-512 path.
                unsafe {
                    avx512_butterfly_pass_32(data_avx.as_mut_ptr(), twiddles.as_ptr(), n);
                }

                assert_eq!(data_scalar, data_avx,
                    "cross-backend mismatch at log_n={} (n={})", log_n, n);
                eprintln!("cross_backend_equivalence log_n={} PASS", log_n);
            }
        }
    }

}
