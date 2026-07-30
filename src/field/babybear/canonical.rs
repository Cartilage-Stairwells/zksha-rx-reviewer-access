//! Domain-typed BabyBear field elements — Commit 3A.
//!
//! # Design
//!
//! Two newtype wrappers make Montgomery/canonical confusion a compile error
//! instead of a silent arithmetic bug:
//!
//! - [`CanonicalBabyBear`]: a value `x` in [0, p), stored as `x`.
//! - [`MontgomeryBabyBear`]: a value `x` stored as `xR mod p` (Montgomery form).
//!
//! Conversions between domains go through [`ScalarBackend::mul_raw`] with
//! the appropriate precomputed constant (R² for enter, 1 for exit).
//!
//! # What this commit intentionally omits
//!
//! - No `Add`, `Sub`, or `Neg` impls — those operate on canonical values and
//!   will be added when the butterfly signature is migrated (Commit 3B).
//! - No `From`/`Into` blanket impls — explicit `to_montgomery()` /
//!   `to_canonical()` calls are the intended API; implicit coercion would
//!   defeat the purpose of domain separation.
//! - `CanonicalBabyBear * MontgomeryBabyBear` is deliberately not implemented.
//!   The compiler enforces the domain boundary.
//!
//! # Verification
//!
//! All conversions are verified against the reference oracle in
//! `tests/babybear_domain.rs`.  The corpus and golden vectors are unchanged.

use super::constants::{BABYBEAR_P, BABYBEAR_R2_MOD_P};
use super::montgomery::{ScalarBackend, BABYBEAR_MONTY, BABYBEAR_SCALAR, MontgomeryBackend};


// ---------------------------------------------------------------------------
// CanonicalBabyBear
// ---------------------------------------------------------------------------

/// A BabyBear field element in canonical (standard) representation.
///
/// Invariant: `self.0 < BABYBEAR_P`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanonicalBabyBear(pub u32);

impl CanonicalBabyBear {
    /// Construct from an arbitrary `u32`, reducing mod p.
    #[inline]
    pub fn new(x: u32) -> Self {
        let reduced = Self(x % BABYBEAR_P);
        debug_assert!(reduced.0 < BABYBEAR_P, "CanonicalBabyBear::new invariant");
        reduced
    }

    /// Construct without reduction.
    ///
    /// # Safety
    /// Caller guarantees `x < BABYBEAR_P`.
    #[inline]
    pub const fn new_unchecked(x: u32) -> Self {
        // debug_assert omitted: const fn cannot call debug_assert! at const-eval time.
        // Callers must ensure x < BABYBEAR_P. Use `new(x)` for a checked constructor.
        Self(x)
    }

    /// Convert to Montgomery representation: stores `self.0 * R mod p`.
    ///
    /// Uses `mul_raw(x, R²) = x · R² · R⁻¹ = x · R mod p`.
    #[inline]
    pub fn to_montgomery(self) -> MontgomeryBabyBear {
        let mont = ScalarBackend::mul_raw(
            self.0,
            BABYBEAR_R2_MOD_P,
            BABYBEAR_MONTY,
        );
        debug_assert!(mont < BABYBEAR_P, "to_montgomery: result {} >= P", mont);
        MontgomeryBabyBear(mont)
    }

    /// Raw inner value.
    #[inline(always)]
    pub fn inner(self) -> u32 {
        self.0
    }
}

// ---------------------------------------------------------------------------
// MontgomeryBabyBear
// ---------------------------------------------------------------------------

/// A BabyBear field element in Montgomery representation.
///
/// Invariant: `self.0 == x * R mod p` for some canonical `x < BABYBEAR_P`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MontgomeryBabyBear(pub u32);

impl MontgomeryBabyBear {
    /// Checked constructor — verifies the value is in [0, p).
    ///
    /// Prefer this over direct `MontgomeryBabyBear(x)` construction.
    /// The debug_assert catches representation violations in debug builds;
    /// release builds are unaffected.
    #[inline]
    pub fn new(x: u32) -> Self {
        debug_assert!(x < BABYBEAR_P, "MontgomeryBabyBear::new: {} >= P={}", x, BABYBEAR_P);
        Self(x)
    }

    /// Enter Montgomery domain from a canonical value.
    #[inline]
    pub fn from_canonical(x: CanonicalBabyBear) -> Self {
        x.to_montgomery()
    }

    /// Exit Montgomery domain: recovers `x` from `xR`.
    ///
    /// Uses `mul_raw(xR, 1) = xR · 1 · R⁻¹ = x mod p`.
    #[inline]
    pub fn to_canonical(self) -> CanonicalBabyBear {
        let canonical = ScalarBackend::mul_raw(self.0, 1, BABYBEAR_MONTY);
        debug_assert!(canonical < BABYBEAR_P, "to_canonical: result {} >= P", canonical);
        CanonicalBabyBear(canonical)
    }

    /// Montgomery multiplication: `(aR) · (bR) → (abR)`.
    ///
    /// Both operands must be in Montgomery domain. The result is in Montgomery
    /// domain. This is the only multiplication defined across domain types —
    /// `CanonicalBabyBear * MontgomeryBabyBear` is intentionally absent.
    ///
    /// Prefer the `*` operator (via `Mul`) in expression context.
    #[inline]
    pub fn montgomery_mul(self, rhs: MontgomeryBabyBear) -> MontgomeryBabyBear {
        MontgomeryBabyBear(BABYBEAR_SCALAR.mul(self.0, rhs.0))
    }

    /// Raw inner value (the Montgomery-domain representation).
    #[inline(always)]
    pub fn inner(self) -> u32 {
        self.0
    }
}

impl core::ops::Mul for MontgomeryBabyBear {
    type Output = MontgomeryBabyBear;
    #[inline]
    fn mul(self, rhs: MontgomeryBabyBear) -> MontgomeryBabyBear {
        self.montgomery_mul(rhs)
    }
}

// ---------------------------------------------------------------------------
// Unit tests (inlined — fast compile-time sanity only)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::reference::montgomery_mul_reference;
    use super::super::constants::BABYBEAR_R_INV_MOD_P;

    #[test]
    fn canonical_new_reduces() {
        assert_eq!(CanonicalBabyBear::new(BABYBEAR_P).0, 0);
        assert_eq!(CanonicalBabyBear::new(BABYBEAR_P + 1).0, 1);
        assert_eq!(CanonicalBabyBear::new(0).0, 0);
    }

    #[test]
    fn roundtrip_boundary() {
        for x in [0u32, 1, 2, BABYBEAR_P - 1, BABYBEAR_P / 2, 42, 0x3FFF_FFFF] {
            let canonical = CanonicalBabyBear::new_unchecked(x % BABYBEAR_P);
            let recovered = canonical.to_montgomery().to_canonical();
            assert_eq!(
                recovered, canonical,
                "roundtrip failed for x={}",
                x
            );
        }
    }

    #[test]
    fn mul_equivalence_boundary() {
        let pairs: &[(u32, u32)] = &[
            (0, 0), (1, 1), (0, 1), (1, 0),
            (BABYBEAR_P - 1, BABYBEAR_P - 1),
            (BABYBEAR_P - 1, 1),
            (42, 57),
            (BABYBEAR_P / 2, BABYBEAR_P / 2),
        ];
        for &(a, b) in pairs {
            let ma = CanonicalBabyBear::new(a).to_montgomery();
            let mb = CanonicalBabyBear::new(b).to_montgomery();
            let product_canonical = (ma * mb).to_canonical();

            // Reference: a * b mod p (plain integer arithmetic)
            let expected = ((a as u64 * b as u64) % BABYBEAR_P as u64) as u32;
            // But our Montgomery mul computes a*b*R^{-1}*R = a*b, so check via oracle
            let oracle = montgomery_mul_reference(
                ma.0, mb.0, BABYBEAR_P, BABYBEAR_R_INV_MOD_P,
            );
            // oracle gives (aR)(bR)R^{-1} = abR; to_canonical gives abR * R^{-1} = ab
            let oracle_canonical = ScalarBackend::mul_raw(oracle, 1, BABYBEAR_MONTY);
            assert_eq!(
                product_canonical.0, oracle_canonical,
                "mul equivalence failed for a={} b={}",
                a, b
            );
            assert_eq!(
                product_canonical.0, expected,
                "plain product check failed for a={} b={}",
                a, b
            );
        }
    }
}

