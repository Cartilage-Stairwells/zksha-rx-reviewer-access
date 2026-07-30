#![cfg_attr(not(feature = "std"), no_std)]

// Core evidence protocol — immutable vocabulary (Commit 1).
// Domain-blind: no pi, no AVX-512, no serialization knowledge.
// Core defines admissibility. Domains provide claims.
pub mod core;
// Field arithmetic module — verified Montgomery core (Commit 1).
// Commit 2: mont_reduce_scalar / scalar_montgomery_mul_32 routed through
// field::babybear::montgomery::ScalarBackend. Duplicates removed.
pub mod field;
pub mod instrument;
/// DIF NTT — three backends with staged equivalence (Commit 5).
/// Reference backend is platform-independent; scalar/AVX-512 are x86_64-only.
pub mod ntt;
use field::babybear::montgomery::{BABYBEAR_SCALAR, MontgomeryBackend};
use field::babybear::constants::BABYBEAR_P as P;

use p3_baby_bear::BabyBear;

// P imported from canonical source — Commit 2.
// P_INV_NEG: retained for AVX-512 vectorized reduction in lib.rs.
// R64 cleanup (Issue #1): mont_reduce_r64 removed in Commit 2. R=2^32 is the sole
// Montgomery radix. Representation invariants enforced via debug_assert in
// canonical.rs and butterfly(). See tests/representation_audit.rs.
use field::babybear::constants::BABYBEAR_NEG_INV as P_INV_NEG;

/// Scalar radix-2 butterfly (legacy path — uses p3 `BabyBear` type).
///
/// # Domain
/// `src` and `twiddles` use the p3 `BabyBear` type, which has its own internal
/// representation (potentially Montgomery). This is the original code path from
/// before domain types were introduced. Future migration to `MontgomeryBabyBear`
/// domain types is tracked separately.
///
/// For the domain-typed path, use `avx512_butterfly_32bit::butterfly()`.
pub fn scalar_radix2_butterfly(src: &mut [BabyBear], twiddles: &[BabyBear]) {
    assert_eq!(src.len(), 2 * twiddles.len());
    for i in (0..src.len()).step_by(2) {
        let a = src[i];
        let b = src[i + 1];
        let w = twiddles[i / 2];
        src[i]     = a + b;
        src[i + 1] = (a - b) * w;
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod avx512_impl {
    use super::*;
    use std::arch::x86_64::*;

    /// SIMD Montgomery reduction: reduces 8 lanes of u64 products to 8 lanes of u32.
    ///
    /// # Safety
    /// - Requires AVX-512F and AVX-512DQ.
    /// - `prod` lanes must be products of two values in `[0, p)` — the reduction
    ///   computes `prod * R^{-1} mod p` where R = 2³².
    /// - Output lanes are in `[0, p)` (Montgomery domain).
    #[target_feature(enable = "avx512f,avx512dq")]
    #[inline]
    unsafe fn mont_reduce_epu64(prod: __m512i) -> __m256i {
        let mask32 = _mm512_set1_epi64(0xFFFF_FFFFu64 as i64);
        let vp     = _mm512_set1_epi64(P as u64 as i64);
        let vinv   = _mm512_set1_epi64(P_INV_NEG as u64 as i64);

        let lo = _mm512_and_si512(prod, mask32);
        let m  = _mm512_and_si512(_mm512_mul_epu32(lo, vinv), mask32);
        let mp = _mm512_mul_epu32(m, vp);
        let t  = _mm512_srli_epi64::<32>(_mm512_add_epi64(prod, mp));

        let ge = _mm512_cmpge_epu64_mask(t, vp);
        let t  = _mm512_mask_sub_epi64(t, ge, t, vp);

        _mm512_cvtepi64_epi32(t)
    }

    /// AVX-512 radix-2 butterfly (placeholder — currently delegates to scalar).
    ///
    /// # Safety
    /// - Requires AVX-512F and AVX-512DQ.
    /// - `src` and `twiddles` use the p3 `BabyBear` type (legacy path).
    ///   Future migration to `MontgomeryBabyBear` domain types tracked separately.
    /// - `src.len()` must equal `2 * twiddles.len()`.
    #[target_feature(enable = "avx512f,avx512dq")]
    pub unsafe fn avx512_radix2_butterfly(src: &mut [BabyBear], twiddles: &[BabyBear]) {
        // placeholder: fill in full SIMD body here once scalar path is verified
        super::scalar_radix2_butterfly(src, twiddles);
        let _ = mont_reduce_epu64(_mm512_setzero_si512());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use p3_field::AbstractField;

    #[test]
    fn test_scalar_runs() {
        let mut rng = rand::thread_rng();
        for len in [8, 16, 32, 64, 128, 1024] {
            let twid_len = len / 2;
            let mut src: Vec<BabyBear> = (0..len)
                .map(|_| BabyBear::from_canonical_u32(rng.gen::<u32>() % P))
                .collect();
            let twiddles: Vec<BabyBear> = (0..twid_len)
                .map(|_| BabyBear::from_canonical_u32(rng.gen::<u32>() % P))
                .collect();
            scalar_radix2_butterfly(&mut src, &twiddles);
        }
    }
}

// mont_reduce_scalar removed — route through BABYBEAR_SCALAR.mul(a, b) [Commit 2]

#[cfg(test)]
mod mont_tests_v2 {
    use super::*;

    #[test]
    fn test_mont_roundtrip_correct() {
        // Migrated to BABYBEAR_SCALAR.mul — Commit 2.
        // roundtrip: (a*R mod p) * (b*R mod p) * R^{-1} mod p = ab*R mod p,
        // then one more reduce = ab mod p.
        let r2 = field::babybear::constants::BABYBEAR_R2_MOD_P;
        let mut failures = 0;
        for a in 1u64..1000 {
            for b in 1u64..1000 {
                let a_mont = BABYBEAR_SCALAR.mul(a as u32, r2 as u32);  // aR mod P
                let b_mont = BABYBEAR_SCALAR.mul(b as u32, r2 as u32);  // bR mod P
                let prod_mont = BABYBEAR_SCALAR.mul(a_mont, b_mont);    // abR mod P
                let result = BABYBEAR_SCALAR.mul(prod_mont, 1);          // ab mod P (exit domain)
                let expected = (a * b) % (P as u64);
                if result as u64 != expected {
                    failures += 1;
                    if failures <= 5 {
                        eprintln!("a={} b={} expected={} got={}", a, b, expected, result);
                    }
                }
            }
        }
        assert_eq!(failures, 0, "{} mismatches out of 998001 pairs", failures);
    }
}

#[cfg(target_arch = "x86_64")]
pub mod avx512_butterfly_32bit;

