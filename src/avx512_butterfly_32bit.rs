// avx512_butterfly_32bit.rs
// AVX-512 radix-2 butterfly using verified 32-bit Montgomery constants.
// Uses 16 lanes of 32-bit values per __m512i.
//
// STATUS: SIMD vectorized (Commit 6). Uses AVX-512F + AVX-512DQ for true
// 16-lane parallel Montgomery multiplication and modular add/sub.
// The scalar_butterfly_32 fallback is retained for:
//   (a) the typed butterfly() public API
//   (b) the tail loop in avx512_butterfly_pass_32 (non-multiple-of-16 elements)
// Layout: data[0..n/2] = a-values, data[n/2..n] = b-values.
//
// Admission gate: test_avx512_vs_scalar_half_half_layout verifies SIMD output
// matches scalar output exactly. No SIMD optimization may enter unless it
// preserves the sealed reference receipt (tscp-ntt-equivalence-v1).

#![cfg(target_arch = "x86_64")]

use std::arch::x86_64::*;
use crate::field::babybear::montgomery::BABYBEAR_SCALAR;
use crate::field::babybear::montgomery::MontgomeryBackend;
use crate::field::babybear::canonical::MontgomeryBabyBear;

use crate::field::babybear::constants::BABYBEAR_P;
use crate::field::babybear::constants::BABYBEAR_NEG_INV;
use BABYBEAR_P as P;
use BABYBEAR_NEG_INV as P_INV_NEG;

/// Internal scalar fallback used lane-by-lane inside the AVX-512 path.
///
/// # Domain
/// `a`, `b`, and `w` (twiddle factor) are **Montgomery-encoded** BabyBear values
/// (`xR mod p`, R = 2³²). The return values are also Montgomery-encoded.
/// Passing canonical values produces arithmetically wrong outputs with no panic.
#[inline(always)]
fn scalar_butterfly_32(a: u32, b: u32, w: u32) -> (u32, u32) {
    // DIF butterfly: x = a + b mod p, y = (a - b) * w mod p
    let sum = a.wrapping_add(b);
    let a_new = if sum >= P { sum - P } else { sum };
    let diff = if a >= b { a - b } else { a + P - b };
    let b_new = BABYBEAR_SCALAR.mul(diff, w);
    (a_new, b_new)
}

/// Computes a radix-2 Cooley-Tukey Montgomery butterfly.
///
/// **Preconditions:**
/// - `a`, `b`, and `w` are valid Montgomery-encoded BabyBear elements (`xR mod p`, R = 2³²).
/// - All inputs satisfy `value.inner() < p` (= `BABYBEAR_P`).
///
/// **Postconditions:**
/// - Returns `(x, y)` where (in the Montgomery domain):
///   ```text
///   x = a + b       mod p
///   y = (a - b) * w mod p
///   ```
/// - Both outputs are Montgomery-encoded: `x.inner() < p`, `y.inner() < p`.
/// - No canonical conversion occurs internally; representation is preserved.
///
/// **Oracle contract (Commit 4):**
/// ```text
/// butterfly(a, b, w).0.inner() == butterfly_reference(a.inner(), b.inner(), w.inner()).0
/// butterfly(a, b, w).1.inner() == butterfly_reference(a.inner(), b.inner(), w.inner()).1
/// ```
/// where `butterfly_reference` is the independent oracle in
/// `field::babybear::reference`. This identity is verified by
/// `tests/babybear_domain::butterfly_reference_agreement`.
///
/// **Stability:** the mathematical contract above must be satisfied by all
/// future backends (NEON, AVX-512, GPU). Backends may not share implementation
/// with the reference oracle; independence is the point.
#[inline(always)]
pub fn butterfly(
    a: MontgomeryBabyBear,
    b: MontgomeryBabyBear,
    w: MontgomeryBabyBear,
) -> (MontgomeryBabyBear, MontgomeryBabyBear) {
    // Representation invariant: all Montgomery-domain values must be < p.
    debug_assert!(a.inner() < P, "butterfly input a={} >= P={}", a.inner(), P);
    debug_assert!(b.inner() < P, "butterfly input b={} >= P={}", b.inner(), P);
    debug_assert!(w.inner() < P, "butterfly input w={} >= P={}", w.inner(), P);

    let (ra, rb) = scalar_butterfly_32(a.inner(), b.inner(), w.inner());

    // Postcondition: outputs remain in Montgomery domain (< p).
    debug_assert!(ra < P, "butterfly output a={} >= P={}", ra, P);
    debug_assert!(rb < P, "butterfly output b={} >= P={}", rb, P);

    (MontgomeryBabyBear(ra), MontgomeryBabyBear(rb))
}

/// SIMD Montgomery reduction: 8 lanes of u64 products → 8 lanes of u32.
///
/// Computes `prod * R^{-1} mod p` where R = 2³² for each of 8 lanes.
/// This is the core Montgomery reduction step, vectorized across 8 u64 lanes.
///
/// # Safety
/// - Requires AVX-512F and AVX-512DQ.
/// - `prod` lanes must be products of two values in `[0, p)` — i.e., each lane
///   is `a * b` where `a, b < p`, so `prod < p² < 2⁶²`.
/// - Output lanes are in `[0, p)` (Montgomery domain).
#[target_feature(enable = "avx512f,avx512dq")]
#[inline]
unsafe fn mont_reduce_epu64(prod: __m512i) -> __m256i {
    let mask32 = _mm512_set1_epi64(0xFFFF_FFFFu64 as i64);
    let vp     = _mm512_set1_epi64(P as u64 as i64);
    let vinv   = _mm512_set1_epi64(P_INV_NEG as u64 as i64);

    // m = (prod_lo * neg_inv) mod 2^32  (low 32 bits only, wrapping)
    let lo = _mm512_and_si512(prod, mask32);
    let m  = _mm512_and_si512(_mm512_mul_epu32(lo, vinv), mask32);

    // t = (prod + m * p) >> 32  (the Montgomery quotient)
    let mp = _mm512_mul_epu32(m, vp);
    let t  = _mm512_srli_epi64::<32>(_mm512_add_epi64(prod, mp));

    // Conditional subtract p if t >= p
    let ge = _mm512_cmpge_epu64_mask(t, vp);
    let t  = _mm512_mask_sub_epi64(t, ge, t, vp);

    // Narrow 8 × u64 → 8 × u32 (lower 32 bits of each)
    _mm512_cvtepi64_epi32(t)
}

/// 16-lane Montgomery multiplication: `b * w → bw` (all in Montgomery domain).
///
/// Uses an even/odd split strategy:
/// 1. `_mm512_mul_epu32(vb, vw)` multiplies the low 32 bits of each 64-bit lane,
///    giving 8 u64 products for even-indexed lanes (0,2,4,...,14).
/// 2. Shift right by 32 and multiply again for odd-indexed lanes (1,3,5,...,15).
/// 3. Reduce each group of 8 with `mont_reduce_epu64`.
/// 4. Interleave the two groups back to 16 × u32 using AVX2 unpack + permute
///    and `_mm512_shuffle_i32x4` to combine.
///
/// # Safety
/// - Requires AVX-512F and AVX-512DQ.
/// - All 16 lanes of `vb` and `vw` must be Montgomery-encoded values in `[0, p)`.
/// - Output is 16 lanes of Montgomery-encoded values in `[0, p)`.
#[target_feature(enable = "avx512f,avx512dq")]
#[inline]
unsafe fn mont_mul_16(vb: __m512i, vw: __m512i) -> __m512i {
    // Even-indexed lanes: multiply low 32 bits of each 64-bit lane
    let prod_even = _mm512_mul_epu32(vb, vw);
    // Odd-indexed lanes: shift right 32 to get high 32 bits, then multiply
    let prod_odd = _mm512_mul_epu32(
        _mm512_srli_epi64::<32>(vb),
        _mm512_srli_epi64::<32>(vw),
    );

    // Reduce each group of 8 → __m256i (8 × u32)
    let red_even = mont_reduce_epu64(prod_even);  // [r0, r2, r4, r6, r8, r10, r12, r14]
    let red_odd  = mont_reduce_epu64(prod_odd);    // [r1, r3, r5, r7, r9, r11, r13, r15]

    // Interleave using AVX2 unpack (per 128-bit lane) + permute (across 128-bit lanes)
    let lo = _mm256_unpacklo_epi32(red_even, red_odd);  // [r0,r1,r2,r3, r8,r9,r10,r11]
    let hi = _mm256_unpackhi_epi32(red_even, red_odd);  // [r4,r5,r6,r7, r12,r13,r14,r15]
    let lower = _mm256_permute2x128_si256::<0x20>(lo, hi);  // [r0,r1,r2,r3, r4,r5,r6,r7]
    let upper = _mm256_permute2x128_si256::<0x31>(lo, hi);  // [r8,r9,r10,r11, r12,r13,r14,r15]

    // Combine two __m256i into one __m512i
    // Cast places each 256-bit result in the lower 256 bits (upper bits undefined,
    // but _mm512_shuffle_i32x4 selects only the lower two 128-bit blocks from each).
    let lower_512 = _mm512_castsi256_si512(lower);
    let upper_512 = _mm512_castsi256_si512(upper);
    _mm512_shuffle_i32x4::<0x44>(lower_512, upper_512)
}

/// AVX-512 radix-2 butterfly — 16 lanes of 32-bit BabyBear elements.
///
/// Computes the DIF butterfly `(a + b mod p, (a - b) * w mod p)` for all 16 lanes
/// in parallel using true SIMD arithmetic:
/// - Modular addition via `_mm512_cmpge_epu32_mask` + `_mm512_mask_sub_epi32`
/// - Modular subtraction via `_mm512_mask_blend_epi32` (conditional +p)
/// - Montgomery multiplication via `mont_mul_16` (even/odd split + reduction)
///
/// # Safety
/// - Caller must ensure AVX-512F and AVX-512DQ are available (use `is_avx512_supported()`).
/// - `va`, `vb`: 16 lanes of **Montgomery-encoded** BabyBear values (`xR mod p`, R = 2³²).
/// - `vw`: 16 lanes of Montgomery-encoded twiddle factors.
/// - Return values are Montgomery-encoded. Canonical inputs produce wrong results silently.
#[target_feature(enable = "avx512f,avx512dq")]
pub unsafe fn avx512_radix2_butterfly_32(
    va: __m512i,
    vb: __m512i,
    vw: __m512i,
) -> (__m512i, __m512i) {
    // DIF butterfly: x = a + b mod p, y = (a - b) * w mod p

    let vp = _mm512_set1_epi32(P as i32);

    // Step 1: a_new = a + b mod p
    // Sum can be at most (p-1) + (p-1) = 2p - 2 < 2^32, so wrapping_add is safe.
    let sum = _mm512_add_epi32(va, vb);
    let ge_sum = _mm512_cmpge_epu32_mask(sum, vp);
    let a_new = _mm512_mask_sub_epi32(sum, ge_sum, sum, vp);

    // Step 2: diff = a - b mod p
    // raw_diff = a - b (wrapping u32 subtraction)
    // adjusted = a - b + p (correct when a < b)
    // When a >= b: select raw_diff. When a < b: select adjusted.
    let raw_diff = _mm512_sub_epi32(va, vb);
    let adjusted = _mm512_add_epi32(raw_diff, vp);
    let ge_a = _mm512_cmpge_epu32_mask(va, vb);
    let diff = _mm512_mask_blend_epi32(ge_a, adjusted, raw_diff);

    // Step 3: b_new = diff * w (Montgomery multiplication, 16 lanes)
    let b_new = mont_mul_16(diff, vw);

    (a_new, b_new)
}

/// Full butterfly pass over `n` elements in half/half layout.
///
/// Layout: `data[0..n/2]` = a-values, `data[n/2..n]` = b-values.
/// This matches the scalar reference in `lib.rs::scalar_radix2_butterfly`.
///
/// # Safety
/// - Caller must ensure AVX-512F and AVX-512DQ are available.
/// - `data`: pointer to `n` **Montgomery-encoded** BabyBear values (`xR mod p`, R = 2³²).
///   `n` must be a multiple of 2; `n ≥ 32` for the vectorized path to engage.
/// - `twiddles`: pointer to `n/2` Montgomery-encoded twiddle factors.
/// - All pointers must be valid for reads/writes of the stated lengths.
/// - Canonical inputs produce arithmetically wrong outputs with no panic or error.
#[target_feature(enable = "avx512f,avx512dq")]
pub unsafe fn avx512_butterfly_pass_32(
    data: *mut u32,
    twiddles: *const u32,
    n: usize,
) {
    let n2 = n / 2;
    let mut i = 0;
    while i + 16 <= n2 {
        let va = _mm512_loadu_si512(data.add(i) as *const __m512i);
        let vb = _mm512_loadu_si512(data.add(i + n2) as *const __m512i);
        let vw = _mm512_loadu_si512(twiddles.add(i) as *const __m512i);
        let (va_new, vb_new) = avx512_radix2_butterfly_32(va, vb, vw);
        _mm512_storeu_si512(data.add(i) as *mut __m512i, va_new);
        _mm512_storeu_si512(data.add(i + n2) as *mut __m512i, vb_new);
        i += 16;
    }
    while i < n2 {
        let a = *data.add(i);
        let b = *data.add(i + n2);
        let w = *twiddles.add(i);
        let (a_new, b_new) = scalar_butterfly_32(a, b, w);
        *data.add(i) = a_new;
        *data.add(i + n2) = b_new;
        i += 1;
    }
}

pub fn is_avx512_supported() -> bool {
    is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512dq")
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_avx512_vs_scalar_half_half_layout() {
        if !is_avx512_supported() {
            eprintln!("AVX-512 not supported on this runner — skipping SIMD verification (requires avx512f+avx512dq).");
            return;
        }

        let mut rng = rand::thread_rng();
        for log_n in 5..=10 {
            let n2 = 1usize << log_n; // n2 must be multiple of 16 for full SIMD path
            let n = n2 * 2;

            let data_init: Vec<u32> = (0..n).map(|_| rng.gen::<u32>() % P).collect();
            let twiddles: Vec<u32> = (0..n2).map(|_| rng.gen::<u32>() % P).collect();

            let mut data_scalar = data_init.clone();
            let mut data_avx = data_init.clone();

            for i in 0..n2 {
                let a = data_scalar[i];
                let b = data_scalar[i + n2];
                let w = twiddles[i];
                let (a_new, b_new) = scalar_butterfly_32(a, b, w);
                data_scalar[i] = a_new;
                data_scalar[i + n2] = b_new;
            }

            unsafe {
                avx512_butterfly_pass_32(data_avx.as_mut_ptr(), twiddles.as_ptr(), n);
            }

            assert_eq!(data_scalar, data_avx, "mismatch for n2=2^{}", log_n);
            eprintln!("n2=2^{} (n={}) PASS", log_n, n);
        }
    }
}
