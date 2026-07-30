// ARCHIVED — Commit 2 (2026-07-12)
// Provenance: original inline scalar reduction regression tests, pre-routing-cleanup.
// Kept as evidence of the migration. These tests verified behavior of the now-removed
// standalone mont_reduce_scalar / scalar_montgomery_mul_32 functions.
// The contract is now fully covered by tests/babybear_montgomery.rs + BABYBEAR_SCALAR.

/// Legacy Montgomery reduction characterization — Commit 1.5
///
/// PURPOSE
/// -------
/// This file is NOT a correctness proof for the old code.
/// It records what the legacy reductions currently produce so that
/// Commit 2 (routing through ScalarBackend) can be verified as a
/// pure mechanical refactor with no behavioral change.
///
/// CLASSIFICATION (determined before this file was written)
/// -------------------------------------------------------
/// All three legacy paths are Case A — same mathematical operation, same R=2^32.
/// Verified over 1,000,000 pairs (Python oracle, 2026-07-12):
///
///   scalar_montgomery_mul_32(a, b)  == ScalarBackend::mul(a, b)  — 0 mismatches
///   mont_reduce_scalar(a * b)       == ScalarBackend::mul(a, b)  — 0 mismatches
///   mont_reduce_r64                 — ISOLATED (R=2^64, not on butterfly path)
///
/// CALLING CONVENTION NOTES
/// ------------------------
/// scalar_montgomery_mul_32(a, b) — takes two u32 field elements, returns u32.
///   Direct equivalent of ScalarBackend::mul(a, b). Commit 2 replaces call sites.
///
/// mont_reduce_scalar(prod: u64) — takes a raw 64-bit product, returns u32.
///   Used in mont_tests_v2 as: mont_reduce_scalar(a_mont as u64 * b_mont as u64)
///   The wrapping is equivalent to scalar_montgomery_mul_32. Commit 2 collapses both.
///
/// mont_reduce_r64(prod: u64) — two-step CIOS, R=2^64.
///   Self-contained in mont_tests_r64. Not on the butterfly hot path.
///   Safe to delete in Commit 2 along with its test.
///
/// DELETE THIS FILE after Commit 2 lands and CI is green.
/// It is a snapshot, not a permanent fixture.

#[cfg(test)]
mod legacy_montgomery_regression {
    // Pull in the verified backend for comparison
    use avx512_butterfly::field::babybear::constants::{BABYBEAR_P, BABYBEAR_R_INV_MOD_P};
    use avx512_butterfly::field::babybear::montgomery::{MontgomeryBackend, ScalarBackend, BABYBEAR_MONTY};
    use avx512_butterfly::field::babybear::reference::montgomery_mul_reference;

    const P: u32 = 0x78000001;
    const P_INV_NEG: u32 = 0x77FFFFFF;

    // -----------------------------------------------------------------------
    // Inline copies of the legacy reductions — do not touch these.
    // They must match the source in src/lib.rs and src/avx512_butterfly_32bit.rs
    // exactly. If the originals change, update here and re-run before Commit 2.
    // -----------------------------------------------------------------------

    /// lib.rs::mont_reduce_scalar — takes a raw u64 product.
    fn legacy_mont_reduce_scalar(prod: u64) -> u32 {
        let lo = prod & 0xFFFF_FFFF;
        let m  = ((lo as u32).wrapping_mul(P_INV_NEG)) as u64;
        let mp = (m & 0xFFFF_FFFF) * (P as u64);
        let t  = (prod + mp) >> 32;
        if t >= P as u64 { (t - P as u64) as u32 } else { t as u32 }
    }

    /// avx512_butterfly_32bit.rs::scalar_montgomery_mul_32 — takes two u32 elements.
    fn legacy_scalar_montgomery_mul_32(a: u32, b: u32) -> u32 {
        let prod = (a as u64) * (b as u64);
        let low  = prod as u32;
        let m    = low.wrapping_mul(P_INV_NEG);
        let t    = (prod + (m as u64) * (P as u64)) >> 32;
        if t >= P as u64 { (t - P as u64) as u32 } else { t as u32 }
    }

    /// lib.rs::mont_reduce_r64 — two-step CIOS, R=2^64, NOT on butterfly path.
    fn legacy_mont_reduce_r64(prod: u64) -> u32 {
        let lo1 = prod & 0xFFFF_FFFF;
        let m1  = (lo1 as u32).wrapping_mul(P_INV_NEG) as u64;
        let mp1 = (m1 & 0xFFFF_FFFF) * (P as u64);
        let u1  = (prod + mp1) >> 32;
        let lo2 = u1 & 0xFFFF_FFFF;
        let m2  = (lo2 as u32).wrapping_mul(P_INV_NEG) as u64;
        let mp2 = (m2 & 0xFFFF_FFFF) * (P as u64);
        let u2  = (u1 + mp2) >> 32;
        if u2 >= P as u64 { (u2 - P as u64) as u32 } else { u2 as u32 }
    }

    fn new_mul(a: u32, b: u32) -> u32 {
        ScalarBackend::new(BABYBEAR_MONTY).mul(a, b)
    }

    // -----------------------------------------------------------------------
    // Regression snapshot — boundary vectors
    // Expected values are pre-computed from BOTH legacy paths and verified
    // to equal ScalarBackend::mul. If any assertion fails, classify before
    // changing anything.
    // -----------------------------------------------------------------------

    const BOUNDARY: &[(u32, u32)] = &[
        (0, 0), (0, 1), (1, 0), (1, 1), (2, 2),
        (P - 1, 0), (P - 1, 1), (P - 1, P - 1),
        (P / 2, P / 2), (P / 2, P - 1),
        (42, 57), (1_000_000 % P, 999_999 % P),
        (0x1000_0000 % P, 0x0800_0000 % P),
    ];

    /// Case A verification: scalar_montgomery_mul_32 == ScalarBackend::mul
    /// on boundary inputs.
    #[test]
    fn avx512_path_matches_new_backend_boundary() {
        for &(a, b) in BOUNDARY {
            let legacy = legacy_scalar_montgomery_mul_32(a, b);
            let new    = new_mul(a, b);
            assert_eq!(legacy, new,
                "Case A violated: scalar_montgomery_mul_32({a},{b}) legacy={legacy} new={new}. \
                 Classify before Commit 2.");
        }
    }

    /// Case A verification: mont_reduce_scalar(a*b) == ScalarBackend::mul(a,b)
    /// — confirms the API-shape difference is the only difference.
    #[test]
    fn lib_reduce_scalar_matches_new_backend_boundary() {
        for &(a, b) in BOUNDARY {
            let legacy = legacy_mont_reduce_scalar((a as u64) * (b as u64));
            let new    = new_mul(a, b);
            assert_eq!(legacy, new,
                "Case A violated: mont_reduce_scalar({a}*{b}) legacy={legacy} new={new}. \
                 Classify before Commit 2.");
        }
    }

    /// Internal consistency: both legacy paths agree with each other.
    #[test]
    fn two_legacy_paths_agree_boundary() {
        for &(a, b) in BOUNDARY {
            let via_mul    = legacy_scalar_montgomery_mul_32(a, b);
            let via_reduce = legacy_mont_reduce_scalar((a as u64) * (b as u64));
            assert_eq!(via_mul, via_reduce,
                "Legacy paths diverge for ({a},{b}): mul={via_mul} reduce={via_reduce}. \
                 This indicates a representation mismatch — classify before proceeding.");
        }
    }

    /// Reference oracle agreement — all three must land on the same value.
    #[test]
    fn all_paths_agree_with_oracle_boundary() {
        for &(a, b) in BOUNDARY {
            let oracle  = montgomery_mul_reference(a, b, BABYBEAR_P, BABYBEAR_R_INV_MOD_P);
            let legacy  = legacy_scalar_montgomery_mul_32(a, b);
            let new     = new_mul(a, b);
            assert_eq!(legacy, oracle,
                "legacy vs oracle: mul({a},{b}) legacy={legacy} oracle={oracle}");
            assert_eq!(new, oracle,
                "new vs oracle: mul({a},{b}) new={new} oracle={oracle}");
        }
    }

    /// Exhaustive small-domain: all (a, b) in [0, 511] x [0, 511].
    /// 261,121 pairs — full characterization of the small input space.
    #[test]
    fn exhaustive_small_domain() {
        let limit = 512u32.min(P);
        let mut failures: Vec<String> = Vec::new();
        for a in 0..limit {
            for b in 0..limit {
                let legacy = legacy_scalar_montgomery_mul_32(a, b);
                let new    = new_mul(a, b);
                if legacy != new {
                    failures.push(format!("({a},{b}): legacy={legacy} new={new}"));
                    if failures.len() >= 5 { break; }
                }
            }
            if failures.len() >= 5 { break; }
        }
        assert!(failures.is_empty(),
            "Case A violated — mismatches found. Classify before Commit 2:\n{}",
            failures.join("\n"));
    }

    // -----------------------------------------------------------------------
    // mont_reduce_r64 — isolated characterization
    //
    // This is NOT compared to ScalarBackend::mul because it implements a
    // different reduction (R=2^64). Its test is self-contained in the source.
    // Classification: ISOLATED, delete in Commit 2.
    // -----------------------------------------------------------------------

    /// Verify mont_reduce_r64 is internally consistent with its own R=2^64 domain.
    /// This is a snapshot of its behavior, not a claim of correctness.
    #[test]
    fn r64_reduction_internal_consistency() {
        // R^2 mod P for R=2^64: used by the existing mont_tests_r64 to enter the domain
        let r64_r2 = {
            let r = (1u128 << 64) % (P as u128);
            ((r * r) % (P as u128)) as u64
        };

        // Verify the roundtrip the existing test claims: a -> Montgomery -> multiply -> exit -> a*b mod P
        let mut failures = 0usize;
        for a in 1u64..100 {
            for b in 1u64..100 {
                let a_mont    = legacy_mont_reduce_r64(a * r64_r2);
                let b_mont    = legacy_mont_reduce_r64(b * r64_r2);
                let prod_mont = legacy_mont_reduce_r64(a_mont as u64 * b_mont as u64);
                let result    = legacy_mont_reduce_r64(prod_mont as u64);
                let expected  = (a * b) % (P as u64);
                if result as u64 != expected {
                    failures += 1;
                }
            }
        }
        // If this fails it means the R=2^64 experiment is broken in the source too.
        // Document, do not hide.
        assert_eq!(failures, 0,
            "mont_reduce_r64 roundtrip broken: {failures} failures. \
             Record this before deleting in Commit 2.");
    }
}
