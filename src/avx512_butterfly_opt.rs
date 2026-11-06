// avx512_butterfly_opt.rs
// Optimized AVX-512 radix-2 butterfly pass — candidate successor to
// avx512_butterfly_pass_32 in avx512_butterfly_32bit.rs.
//
// What changed (and nothing else):
//   1. mont_mul_16 even/odd interleave: 7 vector ops → 2 ops.
//      Original: unpacklo + unpackhi + permute2x128 x2 + 2 casts + shuffle.
//      Optimized: _mm512_inserti32x8 (merge two __m256i halves) + a single
//      _mm512_permutexvar_epi32 (vpermd) with a compile-time-fixed lane map.
//      Same lane ordering as the original path, verified bit-exact by
//      tests/kernel_opt_equivalence.rs against avx512_butterfly_pass_32.
//   2. Montgomery constants (P, neg_inv, masks, lane map) are materialized
//      once per pass call and passed explicitly, instead of relying on the
//      compiler to hoist set1 intrinsics out of the loop.
//
// What did NOT change: the butterfly arithmetic, the Montgomery reduction
// sequence, the even/odd product split, the load/store pattern, the scalar
// tail loop, and the half/half data layout contract.
//
// Admission gate: bit-exact equality with avx512_butterfly_pass_32 across
// all benchmark sizes (2^8..2^20), non-multiple-of-16 tails, and randomized
// inputs. No SIMD optimization may enter unless it preserves the sealed
// reference receipt (tscp-ntt-equivalence-v1).

#![cfg(target_arch = "x86_64")]

use std::arch::x86_64::*;

use crate::field::babybear::constants::BABYBEAR_P;
use crate::field::babybear::montgomery::BABYBEAR_SCALAR;
use crate::field::babybear::montgomery::MontgomeryBackend;
use BABYBEAR_P as P;
use crate::field::babybear::constants::BABYBEAR_NEG_INV;

/// Hoisted vector constants — created once per pass, used in the hot loop.
#[derive(Clone, Copy)]
pub struct AvxConsts {
    vp32: __m512i,      // P broadcast to 16 x u32
    vp64: __m512i,      // P broadcast to 8 x u64
    vinv64: __m512i,    // neg_inv broadcast to 8 x u64
    mask32_64: __m512i, // 0xFFFF_FFFF broadcast to 8 x u64
    interleave: __m512i, // lane map: even-group/odd-group -> natural order
}

/// Lane map for combining the two 8-lane reduction results.
///
/// After the even/odd product split, `red_even` holds result lanes
/// [r0, r2, r4, ...r14] and `red_odd` holds [r1, r3, ... r15].
/// Merging red_odd into the upper 256 bits of a zmm whose lower 256 bits are
/// red_even gives source lanes:
///   0..7  = [r0, r2,  r4,  r6,  r8,  r10, r12, r14]
///   8..15 = [r1, r3,  r5,  r7,  r9,  r11, r13, r15]
/// so output lane i must gather source lane i/2 (even) or 8 + i/2 (odd):
///   [0, 8, 1, 9, 2, 10, 3, 11, 4, 12, 5, 13, 6, 14, 7, 15]
#[target_feature(enable = "avx512f,avx512dq")]
unsafe fn make_consts() -> AvxConsts {
    AvxConsts {
        vp32: _mm512_set1_epi32(P as i32),
        vp64: _mm512_set1_epi64(P as u64 as i64),
        vinv64: _mm512_set1_epi64(BABYBEAR_NEG_INV as u64 as i64),
        mask32_64: _mm512_set1_epi64(0xFFFF_FFFFu64 as i64),
        interleave: _mm512_setr_epi32(
            0, 8, 1, 9, 2, 10, 3, 11, 4, 12, 5, 13, 6, 14, 7, 15,
        ),
    }
}

/// SIMD Montgomery reduction: 8 lanes of u64 products → 8 lanes of u32.
/// Identical arithmetic to avx512_butterfly_32bit::mont_reduce_epu64;
/// constants are passed instead of re-materialized.
///
/// # Safety
/// - Requires AVX-512F and AVX-512DQ.
/// - `prod` lanes must be products of two values in `[0, p)`.
#[target_feature(enable = "avx512f,avx512dq")]
#[inline]
unsafe fn mont_reduce_epu64_h(prod: __m512i, c: &AvxConsts) -> __m256i {
    // m = (prod_lo * neg_inv) mod 2^32 (low 32 bits only, wrapping)
    let lo = _mm512_and_si512(prod, c.mask32_64);
    let m = _mm512_and_si512(_mm512_mul_epu32(lo, c.vinv64), c.mask32_64);

    // t = (prod + m * p) >> 32 (the Montgomery quotient)
    let mp = _mm512_mul_epu32(m, c.vp64);
    let t = _mm512_srli_epi64::<32>(_mm512_add_epi64(prod, mp));

    // Conditional subtract p if t >= p
    let ge = _mm512_cmpge_epu64_mask(t, c.vp64);
    let t = _mm512_mask_sub_epi64(t, ge, t, c.vp64);

    // Narrow 8 x u64 -> 8 x u32 (lower 32 bits of each)
    _mm512_cvtepi64_epi32(t)
}

/// 16-lane Montgomery multiplication, optimized interleave.
///
/// Same even/odd product split as avx512_butterfly_32bit::mont_mul_16; the
/// two 8-lane reduction results are combined with inserti32x8 + one vpermd
/// instead of unpack/permute/shuffle chains.
///
/// # Safety
/// - Requires AVX-512F and AVX-512DQ.
/// - All 16 lanes of `vb` and `vw` must be Montgomery-encoded values in `[0, p)`.
#[target_feature(enable = "avx512f,avx512dq")]
#[inline]
unsafe fn mont_mul_16_h(vb: __m512i, vw: __m512i, c: &AvxConsts) -> __m512i {
    // Even-indexed lanes: multiply low 32 bits of each 64-bit lane
    let prod_even = _mm512_mul_epu32(vb, vw);
    // Odd-indexed lanes: shift right 32 to get high 32 bits, then multiply
    let prod_odd = _mm512_mul_epu32(
        _mm512_srli_epi64::<32>(vb),
        _mm512_srli_epi64::<32>(vw),
    );

    // Reduce each group of 8 -> __m256i (8 x u32)
    let red_even = mont_reduce_epu64_h(prod_even, c); // [r0,r2,...,r14]
    let red_odd = mont_reduce_epu64_h(prod_odd, c);   // [r1,r3,...,r15]

    // Combine: 2 ops (insert + lane permutation) instead of 7.
    let combined = _mm512_inserti32x8::<1>(_mm512_castsi256_si512(red_even), red_odd);
    _mm512_permutexvar_epi32(c.interleave, combined)
}

/// AVX-512 radix-2 butterfly, 16 lanes — same arithmetic as the original.
///
/// # Safety
/// - Requires AVX-512F and AVX-512DQ.
/// - `va`, `vb`, `vw`: 16 lanes of Montgomery-encoded BabyBear values in `[0, p)`.
#[target_feature(enable = "avx512f,avx512dq")]
#[inline]
unsafe fn butterfly_16_h(
    va: __m512i,
    vb: __m512i,
    vw: __m512i,
    c: &AvxConsts,
) -> (__m512i, __m512i) {
    // x = a + b mod p
    let sum = _mm512_add_epi32(va, vb);
    let ge_sum = _mm512_cmpge_epu32_mask(sum, c.vp32);
    let a_new = _mm512_mask_sub_epi32(sum, ge_sum, sum, c.vp32);

    // diff = a - b mod p
    let raw_diff = _mm512_sub_epi32(va, vb);
    let adjusted = _mm512_add_epi32(raw_diff, c.vp32);
    let ge_a = _mm512_cmpge_epu32_mask(va, vb);
    let diff = _mm512_mask_blend_epi32(ge_a, adjusted, raw_diff);

    // y = diff * w (Montgomery multiplication, 16 lanes)
    let b_new = mont_mul_16_h(diff, vw, c);

    (a_new, b_new)
}

/// Optimized full butterfly pass over `n` elements in half/half layout.
///
/// Bit-exact with `avx512_butterfly_32bit::avx512_butterfly_pass_32`
/// (see tests/kernel_opt_equivalence.rs for the admission gate).
///
/// Layout: `data[0..n/2]` = a-values, `data[n/2..n]` = b-values.
///
/// # Safety
/// - Caller must ensure AVX-512F and AVX-512DQ are available.
/// - `data`: pointer to `n` Montgomery-encoded BabyBear values; `n >= 1`.
/// - `twiddles`: pointer to `n/2` Montgomery-encoded twiddle factors.
/// - Canonical inputs produce arithmetically wrong outputs with no panic or error.
#[target_feature(enable = "avx512f,avx512dq")]
pub unsafe fn avx512_butterfly_pass_32_opt(
    data: *mut u32,
    twiddles: *const u32,
    n: usize,
) {
    let n2 = n / 2;
    let c = make_consts();

    let mut i = 0;
    while i + 16 <= n2 {
        let va = _mm512_loadu_si512(data.add(i) as *const __m512i);
        let vb = _mm512_loadu_si512(data.add(i + n2) as *const __m512i);
        let vw = _mm512_loadu_si512(twiddles.add(i) as *const __m512i);
        let (va_new, vb_new) = butterfly_16_h(va, vb, vw, &c);
        _mm512_storeu_si512(data.add(i) as *mut __m512i, va_new);
        _mm512_storeu_si512(data.add(i + n2) as *mut __m512i, vb_new);
        i += 16;
    }

    // Scalar tail (non-multiple-of-16 n2) — same fallback as the original pass.
    while i < n2 {
        let a = *data.add(i);
        let b = *data.add(i + n2);
        let w = *twiddles.add(i);
        let sum = a.wrapping_add(b);
        let a_new = if sum >= P { sum - P } else { sum };
        let diff = if a >= b { a - b } else { a + P - b };
        let b_new = BABYBEAR_SCALAR.mul(diff, w);
        *data.add(i) = a_new;
        *data.add(i + n2) = b_new;
        i += 1;
    }
}

// ---------------------------------------------------------------------------
// Experiment 2: unroll-by-2 of the ORIGINAL arithmetic (7-op interleave kept).
// Not for admission — measures whether loop front-end overhead matters.
// Identical arithmetic and lane ordering to avx512_butterfly_32bit.
// ---------------------------------------------------------------------------

#[target_feature(enable = "avx512f,avx512dq")]
unsafe fn mont_reduce_epu64_orig(prod: __m512i, c: &AvxConsts) -> __m256i {
    let lo = _mm512_and_si512(prod, c.mask32_64);
    let m = _mm512_and_si512(_mm512_mul_epu32(lo, c.vinv64), c.mask32_64);
    let mp = _mm512_mul_epu32(m, c.vp64);
    let t = _mm512_srli_epi64::<32>(_mm512_add_epi64(prod, mp));
    let ge = _mm512_cmpge_epu64_mask(t, c.vp64);
    let t = _mm512_mask_sub_epi64(t, ge, t, c.vp64);
    _mm512_cvtepi64_epi32(t)
}

#[target_feature(enable = "avx512f,avx512dq")]
#[inline]
unsafe fn mont_mul_16_orig(vb: __m512i, vw: __m512i, c: &AvxConsts) -> __m512i {
    let prod_even = _mm512_mul_epu32(vb, vw);
    let prod_odd = _mm512_mul_epu32(
        _mm512_srli_epi64::<32>(vb),
        _mm512_srli_epi64::<32>(vw),
    );
    let red_even = mont_reduce_epu64_orig(prod_even, c);
    let red_odd = mont_reduce_epu64_orig(prod_odd, c);
    let lo = _mm256_unpacklo_epi32(red_even, red_odd);
    let hi = _mm256_unpackhi_epi32(red_even, red_odd);
    let lower = _mm256_permute2x128_si256::<0x20>(lo, hi);
    let upper = _mm256_permute2x128_si256::<0x31>(lo, hi);
    let lower_512 = _mm512_castsi256_si512(lower);
    let upper_512 = _mm512_castsi256_si512(upper);
    _mm512_shuffle_i32x4::<0x44>(lower_512, upper_512)
}

#[target_feature(enable = "avx512f,avx512dq")]
#[inline]
unsafe fn butterfly_16_orig(va: __m512i, vb: __m512i, vw: __m512i, c: &AvxConsts) -> (__m512i, __m512i) {
    let sum = _mm512_add_epi32(va, vb);
    let ge_sum = _mm512_cmpge_epu32_mask(sum, c.vp32);
    let a_new = _mm512_mask_sub_epi32(sum, ge_sum, sum, c.vp32);
    let raw_diff = _mm512_sub_epi32(va, vb);
    let adjusted = _mm512_add_epi32(raw_diff, c.vp32);
    let ge_a = _mm512_cmpge_epu32_mask(va, vb);
    let diff = _mm512_mask_blend_epi32(ge_a, adjusted, raw_diff);
    let b_new = mont_mul_16_orig(diff, vw, c);
    (a_new, b_new)
}

#[target_feature(enable = "avx512f,avx512dq")]
pub unsafe fn avx512_butterfly_pass_32_unroll(
    data: *mut u32,
    twiddles: *const u32,
    n: usize,
) {
    let n2 = n / 2;
    let c = make_consts();
    let mut i = 0;
    while i + 32 <= n2 {
        // Two independent groups in flight
        let va0 = _mm512_loadu_si512(data.add(i) as *const __m512i);
        let vb0 = _mm512_loadu_si512(data.add(i + n2) as *const __m512i);
        let vw0 = _mm512_loadu_si512(twiddles.add(i) as *const __m512i);
        let va1 = _mm512_loadu_si512(data.add(i + 16) as *const __m512i);
        let vb1 = _mm512_loadu_si512(data.add(i + n2 + 16) as *const __m512i);
        let vw1 = _mm512_loadu_si512(twiddles.add(i + 16) as *const __m512i);
        let (a0, b0) = butterfly_16_orig(va0, vb0, vw0, &c);
        let (a1, b1) = butterfly_16_orig(va1, vb1, vw1, &c);
        _mm512_storeu_si512(data.add(i) as *mut __m512i, a0);
        _mm512_storeu_si512(data.add(i + n2) as *mut __m512i, b0);
        _mm512_storeu_si512(data.add(i + 16) as *mut __m512i, a1);
        _mm512_storeu_si512(data.add(i + n2 + 16) as *mut __m512i, b1);
        i += 32;
    }
    if i + 16 <= n2 {
        let va = _mm512_loadu_si512(data.add(i) as *const __m512i);
        let vb = _mm512_loadu_si512(data.add(i + n2) as *const __m512i);
        let vw = _mm512_loadu_si512(twiddles.add(i) as *const __m512i);
        let (va_new, vb_new) = butterfly_16_orig(va, vb, vw, &c);
        _mm512_storeu_si512(data.add(i) as *mut __m512i, va_new);
        _mm512_storeu_si512(data.add(i + n2) as *mut __m512i, vb_new);
        i += 16;
    }
    while i < n2 {
        let a = *data.add(i);
        let b = *data.add(i + n2);
        let w = *twiddles.add(i);
        let sum = a.wrapping_add(b);
        let a_new = if sum >= P { sum - P } else { sum };
        let diff = if a >= b { a - b } else { a + P - b };
        let b_new = BABYBEAR_SCALAR.mul(diff, w);
        *data.add(i) = a_new;
        *data.add(i + n2) = b_new;
        i += 1;
    }
}
