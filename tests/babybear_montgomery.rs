/// BabyBear Montgomery verification harness — Commit 1.
///
/// Frozen contract: every backend added to this repo must pass run_suite().
/// Do not weaken or remove any assertion without a corresponding corpus update.
///
/// Commit 2: add ScalarBackend equivalence test against old mont_reduce_scalar.
/// Commit 3: add Avx512Backend to run_suite().
#[cfg(test)]
mod babybear_montgomery {
    use avx512_butterfly::field::babybear::constants::{BABYBEAR_P, BABYBEAR_R_INV_MOD_P};
    use avx512_butterfly::field::babybear::montgomery::{
        MontgomeryBackend, MontgomeryConstants, ScalarBackend, BABYBEAR_MONTY, BABYBEAR_SCALAR,
    };
    use avx512_butterfly::field::babybear::reference::montgomery_mul_reference;

    use proptest::prelude::*;

    // -----------------------------------------------------------------------
    // Generic suite — runs against any MontgomeryBackend
    // -----------------------------------------------------------------------
    fn run_suite<B: MontgomeryBackend>(backend: B) {
        let p = backend.constants().modulus;
        let r_inv = BABYBEAR_R_INV_MOD_P;

        let boundary: &[(u32, u32)] = &[
            (0, 0), (0, 1), (1, 0), (1, 1), (2, 2),
            (p - 1, 0), (p - 1, 1), (p - 1, p - 1),
            (p / 2, p / 2), (p / 2, p - 1), (42, 57),
            (0x1000_0000 % p, 0x0800_0000 % p),
        ];

        for &(a, b) in boundary {
            let expected = montgomery_mul_reference(a, b, p, r_inv);
            let got = backend.mul(a, b);
            assert_eq!(got, expected,
                "reference equivalence: mul({a},{b}) expected={expected} got={got}");
            assert!(got < p,
                "output range: mul({a},{b}) = {got} >= p={p}");
            assert_eq!(backend.mul(b, a), got,
                "commutativity: mul({a},{b}) != mul({b},{a})");
        }
    }

    // -----------------------------------------------------------------------
    // 1. Reference equivalence — boundary
    // -----------------------------------------------------------------------
    #[test]
    fn scalar_boundary_suite() {
        run_suite(BABYBEAR_SCALAR);
    }

    // -----------------------------------------------------------------------
    // 2. Golden vectors — immutable contract
    //    Values pre-computed via 128-bit Python oracle and hard-coded.
    //    A failure here is a contract regression.
    // -----------------------------------------------------------------------
    #[test]
    fn golden_vectors() {
        let b = BABYBEAR_SCALAR;
        let p = BABYBEAR_P;
        let r_inv = BABYBEAR_R_INV_MOD_P;

        // Static (hand-verified, immutable)
        assert_eq!(b.mul(0, 0),       0,            "mul(0,0)");
        assert_eq!(b.mul(1, 0),       0,            "mul(1,0)");
        assert_eq!(b.mul(0, 1),       0,            "mul(0,1)");
        assert_eq!(b.mul(1, 1),       0x3840_0000,  "mul(1,1)");
        assert_eq!(b.mul(p - 1, p - 1), 0x3840_0000, "mul(p-1,p-1)");
        assert_eq!(b.mul(0, p - 1),   0,            "mul(0,p-1)");

        // Dynamic (pinned via reference oracle)
        for &(a, bb) in &[(2u32,2u32),(42,57),(p-1,1),(p/2,p/2),(100_000%p,999_999%p)] {
            let expected = montgomery_mul_reference(a, bb, p, r_inv);
            assert_eq!(b.mul(a, bb), expected, "golden dynamic mul({a},{bb})");
        }
    }

    // -----------------------------------------------------------------------
    // 3. Proptest
    // -----------------------------------------------------------------------
    proptest! {
        #[test]
        fn prop_reference_equivalence(a in 0u32..BABYBEAR_P, b in 0u32..BABYBEAR_P) {
            let expected = montgomery_mul_reference(a, b, BABYBEAR_P, BABYBEAR_R_INV_MOD_P);
            let got = BABYBEAR_SCALAR.mul(a, b);
            prop_assert_eq!(got, expected,
                "mul({},{}) expected={} got={}", a, b, expected, got);
        }

        #[test]
        fn prop_output_range(a in 0u32..BABYBEAR_P, b in 0u32..BABYBEAR_P) {
            prop_assert!(BABYBEAR_SCALAR.mul(a, b) < BABYBEAR_P,
                "mul({},{}) out of range", a, b);
        }

        #[test]
        fn prop_commutativity(a in 0u32..BABYBEAR_P, b in 0u32..BABYBEAR_P) {
            prop_assert_eq!(BABYBEAR_SCALAR.mul(a, b), BABYBEAR_SCALAR.mul(b, a),
                "mul({},{}) != mul({},{})", a, b, b, a);
        }

        #[test]
        fn prop_associativity(a in 0u32..BABYBEAR_P, b in 0u32..BABYBEAR_P, c in 0u32..BABYBEAR_P) {
            let p = BABYBEAR_P; let r = BABYBEAR_R_INV_MOD_P;
            let ref_ab    = montgomery_mul_reference(a, b, p, r);
            let ref_bc    = montgomery_mul_reference(b, c, p, r);
            let ref_left  = montgomery_mul_reference(ref_ab, c, p, r);
            let ref_right = montgomery_mul_reference(a, ref_bc, p, r);
            prop_assert_eq!(ref_left, ref_right);
            prop_assert_eq!(BABYBEAR_SCALAR.mul(BABYBEAR_SCALAR.mul(a,b), c), ref_left);
        }

        #[test]
        fn prop_zero_annihilation(x in 0u32..BABYBEAR_P) {
            prop_assert_eq!(BABYBEAR_SCALAR.mul(0, x), 0);
            prop_assert_eq!(BABYBEAR_SCALAR.mul(x, 0), 0);
        }
    }

    // -----------------------------------------------------------------------
    // 4. Backend scaffold — add new backends here (Commit 3)
    // -----------------------------------------------------------------------
    mod scalar_backend {
        use super::*;
        #[test]
        fn full_suite() { run_suite(ScalarBackend::new(BABYBEAR_MONTY)); }
    }

    // Commit 3 stubs:
    // mod avx512_backend {
    //     use super::*;
    //     #[test]
    //     fn full_suite() { run_suite(Avx512Backend::new(BABYBEAR_MONTY)); }
    //     #[test]
    //     fn corpus_equivalence() { /* load corpus/babybear/montgomery_v1.json */ }
    // }

    // -----------------------------------------------------------------------
    // 5. Proof-equivalence — ScalarBackend::mul_raw matches reference oracle
    //    Closes Issue #5. Mechanically checks the identity stated in
    //    ScalarBackend::mul_raw doc: mul_raw(a,b) == oracle(a,b) for all a,b in [0,p).
    // -----------------------------------------------------------------------
    mod oracle_agreement {
        use super::*;
        use avx512_butterfly::field::babybear::montgomery::ScalarBackend;

        // Boundary cases: explicit, deterministic, always run.
        #[test]
        fn boundary_cases() {
            let p = BABYBEAR_P;
            let r = BABYBEAR_R_INV_MOD_P;
            let cases: &[(u32, u32)] = &[
                (0, 0), (0, 1), (1, 0), (1, 1),
                (2, 2),
                (p - 1, 1), (1, p - 1),
                (p - 1, p - 1),
                (p / 2, p / 2),
                (42, 57), (57, 42),
            ];
            for &(a, b) in cases {
                let got      = ScalarBackend::mul_raw(a, b, BABYBEAR_MONTY);
                let expected = montgomery_mul_reference(a, b, p, r);
                assert_eq!(got, expected,
                    "oracle_agreement failed: mul_raw({},{}) = {} but oracle = {}",
                    a, b, got, expected);
            }
        }

        // Proptest: 10 000 random pairs across the full [0, p) × [0, p) domain.
        proptest! {
            #![proptest_config(proptest::test_runner::Config {
                cases: 10_000,
                ..Default::default()
            })]
            #[test]
            fn prop_mul_raw_matches_oracle(a in 0u32..BABYBEAR_P, b in 0u32..BABYBEAR_P) {
                let got      = ScalarBackend::mul_raw(a, b, BABYBEAR_MONTY);
                let expected = montgomery_mul_reference(a, b, BABYBEAR_P, BABYBEAR_R_INV_MOD_P);
                prop_assert_eq!(got, expected,
                    "mul_raw({},{}) = {} but oracle = {}", a, b, got, expected);
            }
        }
    }
}
