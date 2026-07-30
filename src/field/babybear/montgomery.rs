/// Optimized Montgomery multiplication for BabyBear — scalar backend.
///
/// Architecture
/// ------------
/// MontgomeryBackend trait  — the parameterization seam.
///   Every implementation (scalar, portable SIMD, NEON, AVX-512) satisfies
///   this trait. The shared test harness in tests/babybear_montgomery.rs
///   runs the full property suite against any backend via run_suite<B>().
///
/// ScalarBackend             — verified CIOS reduction, u32 lanes.
///   This is the canonical, reference-verified scalar path. lib.rs and
///   avx512_butterfly_32bit route through this type — Commit 2 completed.
///
/// Future backends (Commit 3 adds equivalence tests for each):
///   PortableSimdBackend   — std::simd / packed_simd
///   NeonBackend           — ARM intrinsics
///   Avx512Backend         — wraps avx512_butterfly_32bit, verified by corpus
use super::constants::{BABYBEAR_NEG_INV, BABYBEAR_P};

// ---------------------------------------------------------------------------
// Constants struct
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MontgomeryConstants {
    pub modulus: u32,
    /// -p^{-1} mod 2^32. Invariant: modulus.wrapping_mul(neg_inv).wrapping_add(1) == 0
    pub neg_inv: u32,
}

impl MontgomeryConstants {
    #[must_use]
    pub const fn new(modulus: u32, neg_inv: u32) -> Self {
        debug_assert!(
            modulus.wrapping_mul(neg_inv).wrapping_add(1) == 0,
            "neg_inv invariant violated"
        );
        Self { modulus, neg_inv }
    }
}

pub const BABYBEAR_MONTY: MontgomeryConstants =
    MontgomeryConstants::new(BABYBEAR_P, BABYBEAR_NEG_INV);

// ---------------------------------------------------------------------------
// Backend trait
// ---------------------------------------------------------------------------

/// Shared contract for all Montgomery multiplication backends.
///
/// To add a backend:
///   1. Implement this trait.
///   2. Add `YourBackend::new(BABYBEAR_MONTY)` to the `run_suite` call
///      in tests/babybear_montgomery.rs.
///   3. No other changes needed.
pub trait MontgomeryBackend: Copy {
    /// The field modulus p. Compile-time constant; used by generic code
    /// that needs p without instantiating a backend.
    const MODULUS: u32;

    fn constants(&self) -> MontgomeryConstants;

    /// Compute `a * b * R⁻¹ mod p` (CIOS Montgomery reduction, R = 2³²).
    ///
    /// **Preconditions:**  `a, b ∈ [0, p)`
    /// **Postconditions:** result `∈ [0, p)`,
    ///   `result == montgomery_mul_reference(a, b, p, R⁻¹ mod p)`
    ///   where `montgomery_mul_reference` is the pure-Rust oracle in
    ///   `field::babybear::reference`. Any backend that disagrees with
    ///   that oracle on any input in [0, p) × [0, p) is incorrect.
    fn mul(&self, a: u32, b: u32) -> u32;

    /// Reduce a pre-computed `u64` product via Montgomery reduction.
    ///
    /// Equivalent to `mul(a, b)` when `prod = (a as u64) * (b as u64)`.
    /// Useful for SIMD backends where the hardware multiply produces a
    /// full-width product and only the reduction step is Montgomery-specific.
    ///
    /// **Preconditions:**  `prod < p²`  (i.e. factors were in `[0, p)`)
    /// **Postconditions:** result `∈ [0, p)`, result == `mul` of the factors
    fn reduce(&self, prod: u64) -> u32;
}

// ---------------------------------------------------------------------------
// Scalar backend — CIOS reduction
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct ScalarBackend {
    pub consts: MontgomeryConstants,
}

impl ScalarBackend {
    #[must_use]
    pub const fn new(consts: MontgomeryConstants) -> Self {
        Self { consts }
    }

    /// Raw CIOS Montgomery reduction — the single implementation of R=2³² mul.
    ///
    /// **Proof-equivalence contract:**
    /// For all `a, b ∈ [0, p)`:
    /// ```text
    /// ScalarBackend::mul_raw(a, b, BABYBEAR_MONTY)
    ///   == montgomery_mul_reference(a, b, BABYBEAR_P, BABYBEAR_R_INV_MOD_P)
    /// ```
    /// This identity is the acceptance criterion for any refactor of this
    /// function. It is verified by `tests/babybear_montgomery.rs::oracle_agreement`
    /// (proptest, 10 000 random pairs + all boundary cases).
    ///
    /// **Also pub for:** benchmarks, `canonical.rs` domain conversions.
    ///
    /// Replaced (Commit 2):
    ///   `lib.rs::mont_reduce_scalar`
    ///   `avx512_butterfly_32bit::scalar_montgomery_mul_32`
    #[inline(always)]
    pub fn mul_raw(a: u32, b: u32, c: MontgomeryConstants) -> u32 {
        let t: u64 = (a as u64) * (b as u64);
        let m: u32 = (t as u32).wrapping_mul(c.neg_inv);
        let u: u32 = ((t + (m as u64) * (c.modulus as u64)) >> 32) as u32;
        let (sub, borrow) = u.overflowing_sub(c.modulus);
        if borrow { u } else { sub }
    }

    /// Raw Montgomery reduction of a pre-computed product.
    /// Same arithmetic as `mul_raw` but skips the multiply step.
    #[inline(always)]
    pub fn reduce_raw(prod: u64, c: MontgomeryConstants) -> u32 {
        let m: u32 = (prod as u32).wrapping_mul(c.neg_inv);
        let u: u32 = ((prod + (m as u64) * (c.modulus as u64)) >> 32) as u32;
        let (sub, borrow) = u.overflowing_sub(c.modulus);
        if borrow { u } else { sub }
    }
}

impl MontgomeryBackend for ScalarBackend {
    const MODULUS: u32 = super::constants::BABYBEAR_P;

    #[inline(always)]
    fn constants(&self) -> MontgomeryConstants { self.consts }

    #[inline(always)]
    fn mul(&self, a: u32, b: u32) -> u32 {
        Self::mul_raw(a, b, self.consts)
    }

    #[inline(always)]
    fn reduce(&self, prod: u64) -> u32 {
        Self::reduce_raw(prod, self.consts)
    }
}

pub const BABYBEAR_SCALAR: ScalarBackend = ScalarBackend::new(BABYBEAR_MONTY);

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_valid() {
        let c = BABYBEAR_MONTY;
        assert_eq!(c.modulus.wrapping_mul(c.neg_inv).wrapping_add(1), 0);
    }

    #[test]
    fn zero_annihilation() {
        assert_eq!(BABYBEAR_SCALAR.mul(0, 12345), 0);
        assert_eq!(BABYBEAR_SCALAR.mul(12345, 0), 0);
    }

    #[test]
    fn reduce_equals_mul() {
        for a in [0u32, 1, 2, BABYBEAR_P - 1, 42, BABYBEAR_P / 2] {
            for b in [0u32, 1, 2, BABYBEAR_P - 1, 42, BABYBEAR_P / 2] {
                let via_mul = BABYBEAR_SCALAR.mul(a, b);
                let via_reduce = BABYBEAR_SCALAR.reduce((a as u64) * (b as u64));
                assert_eq!(via_mul, via_reduce,
                    "reduce != mul for ({}, {}): mul={} reduce={}", a, b, via_mul, via_reduce);
            }
        }
    }

    #[test]
    fn output_in_range() {
        for a in [0u32, 1, 2, BABYBEAR_P - 1, 42, BABYBEAR_P / 2] {
            for b in [0u32, 1, 2, BABYBEAR_P - 1, 42, BABYBEAR_P / 2] {
                assert!(BABYBEAR_SCALAR.mul(a, b) < BABYBEAR_P);
            }
        }
    }
}
