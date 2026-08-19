//! avx512-butterfly: AVX-512 vectorized NTT for the BabyBear field.
//!
//! This crate has two implementation tiers and a Plonky3 adapter:
//!
//! 1. **Scalar reference** (`scalar_radix2_butterfly`): Pure Rust, no SIMD.
//! 2. **AVX-512 SIMD kernel** (`avx512_butterfly_32bit::avx512_radix2_butterfly_32`):
//!    True 16-lane SIMD using `__m512i` intrinsics.
//! 3. **Plonky3 adapter** (`plonky3_adapter::ZkshaDifAdapter`):
//!    Implements `TwoAdicSubgroupDft<BabyBear>` routing through our NTT.

use p3_baby_bear::BabyBear;


/// The BabyBear prime: 2^32 - 2^28 + 1 = 0x78000001
pub const P: u32 = 0x7800_0001;

/// Montgomery R constant: R = 2^32 mod p
pub const R: u32 = 1 << 31;

/// -p^{-1} mod 2^32 (Montgomery magic constant)
pub const P_INV_NEG: u32 = 0x0000_0001;

// Module declarations
pub mod avx512_butterfly_32bit;
pub mod plonky3_adapter;
// pub mod core; // Not needed for integration test
pub mod field;
pub mod ntt;

/// Scalar radix-2 DIF butterfly (reference implementation).
pub fn scalar_radix2_butterfly(src: &mut [BabyBear], twiddles: &[BabyBear]) {
    let n = src.len();
    let n2 = n / 2;
    debug_assert_eq!(twiddles.len(), n2);
    for i in 0..n2 {
        let a = src[i];
        let b = src[i + n2];
        let w = twiddles[i];
        src[i] = a + b;
        src[i + n2] = (a - b) * w;
    }
}

/// AVX-512 compatibility module (placeholder delegating to scalar).
pub mod avx512_impl {
    use super::*;
    use std::arch::x86_64::*;

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

    #[target_feature(enable = "avx512f,avx512dq")]
    pub unsafe fn scalar_compat_radix2_butterfly(src: &mut [BabyBear], twiddles: &[BabyBear]) {
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
