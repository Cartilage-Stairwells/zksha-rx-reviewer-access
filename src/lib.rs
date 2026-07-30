//! avx512-butterfly: AVX-512 vectorized NTT for the BabyBear field.
//!
//! ## Architecture
//!
//! This crate has two implementation tiers:
//!
//! 1. **Scalar reference** (`scalar_radix2_butterfly`): Pure Rust, no SIMD.
//!    Used as the correctness oracle for differential testing.
//!
//! 2. **AVX-512 SIMD kernel** (`avx512_butterfly_32bit::avx512_radix2_butterfly_32`):
//!    True 16-lane SIMD using `__m512i` intrinsics. This is the performance path.
//!    Located in `src/avx512_butterfly_32bit.rs`.
//!
//! The function `scalar_compat_radix2_butterfly` in the `avx512_impl` module
//
//! is a compatibility wrapper that delegates to scalar — it is NOT the SIMD
//! kernel and should not be used for benchmarking AVX-512 performance.

use p3_baby_bear::BabyBear;
use p3_field::AbstractField;

/// The BabyBear prime: 2^32 - 2^28 + 1 = 0x78000001
pub const P: u32 = 0x7800_0001;

/// Montgomery R constant: R = 2^32 mod p
pub const R: u32 = 1 << 31;

/// -p^{-1} mod 2^32 (Montgomery magic constant)
pub const P_INV_NEG: u32 = 0x0000_0001;

/// Scalar radix-2 DIF butterfly.
///
/// Computes the DIF butterfly: x = a + b mod p, y = (a - b) * w mod p
/// for each pair (a, b) with twiddle factor w.
///
/// This is the reference implementation — pure scalar, no SIMD.
/// Used as the correctness oracle for backend equivalence testing.
pub fn scalar_radix2_butterfly(src: &mut [BabyBear], twiddles: &[BabyBear]) {
    let n = src.len();
    let n2 = n / 2;
    debug_assert_eq!(twiddles.len(), n2);
    for i in 0..n2 {
        let a = src[i];
        let b = src[i + n2];
        let w = twiddles[i];
        // DIF butterfly: x = a + b, y = (a - b) * w
        src[i] = a + b;
        src[i + n2] = (a - b) * w;
    }
}

/// AVX-512 compatibility module.
///
/// WARNING: `scalar_compat_radix2_butterfly` is a PLACEHOLDER that delegates
/// to the scalar reference. It is NOT the AVX-512 SIMD kernel.
///
/// The real AVX-512 SIMD implementation is:
///   `avx512_butterfly_32bit::avx512_radix2_butterfly_32`
/// which operates on `__m512i` vectors (16 lanes of u32).
///
/// This compatibility wrapper exists only to maintain the public API
/// for the `BabyBear` type. Do NOT use it for AVX-512 benchmarking.
pub mod avx512_impl {
    use super::*;
    use std::arch::x86_64::*;

    /// SIMD Montgomery reduction: reduces 8 lanes of u64 products to 8 lanes of u32.
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

    /// PLACEHOLDER: delegates to scalar. NOT the AVX-512 SIMD kernel.
    ///
    /// The real SIMD kernel is `avx512_butterfly_32bit::avx512_radix2_butterfly_32`.
    /// This function exists only for API compatibility with the `BabyBear` type.
    #[target_feature(enable = "avx512f,avx512dq")]
    pub unsafe fn scalar_compat_radix2_butterfly(src: &mut [BabyBear], twiddles: &[BabyBear]) {
        // Delegates to scalar reference — do NOT benchmark as AVX-512.
        super::scalar_radix2_butterfly(src, twiddles);
        let _ = mont_reduce_epu64(_mm512_setzero_si512());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_scalar_runs() {
        let mut rng = rand::thread_rng();
        let p = P;
        let len = 256;
        let mut src: Vec<BabyBear> = (0..len)
            .map(|_| BabyBear::from_canonical_u32(rng.gen::<u32>() % p))
            .collect();
        let twiddles: Vec<BabyBear> = (0..len / 2)
            .map(|_| BabyBear::from_canonical_u32(rng.gen::<u32>() % p))
            .collect();
        scalar_radix2_butterfly(&mut src, &twiddles);
    }
}
