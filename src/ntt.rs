//! DIF radix-2 NTT for BabyBear Montgomery arithmetic.
//!
//! Three backends with identical algorithm structure:
//! - [`ntt_reference_stage`]: uses `butterfly_reference` (independent oracle)
//! - `ntt_scalar_stage`: uses typed `butterfly()` wrapper (x86_64 only)
//! - `ntt_avx512_stage`: uses `avx512_butterfly_pass_32` (x86_64 only)
//!
//! All three use DIF (decimation in frequency) with half/half butterfly layout.
//! Stage s: h = n/2^(s+1), groups = 2^s. Within each group, pair element i
//! with element i+h using twiddle[i].
//!
//! After all log_n stages, output is in bit-reversed order.
//! Call [`bit_reverse`] to restore natural order.
//!
//! # Verification model (Commit 5)
//!
//! Stage-by-stage comparison prevents false positives where two wrong
//! transforms accidentally converge after multiple stages. The comparison
//! operates on the butterfly-pass level, not the final NTT output:
//!
//! ```text
//! Input vector (shared, deterministic seed)
//!     |
//!     +---> reference NTT (butterfly_reference)
//!     |         stage 0 -- compare -- stage 1 -- compare -- ... -- final
//!     |
//!     +---> scalar NTT (butterfly typed)
//!     |         stage 0 -- compare -- stage 1 -- compare -- ... -- final
//!     |
//!     +---> AVX-512 NTT (avx512_butterfly_pass_32)
//!               stage 0 -- compare -- stage 1 -- compare -- ... -- final
//! ```

use crate::field::babybear::reference::butterfly_reference;

// ---------------------------------------------------------------------------
// Platform-independent: reference NTT (the specification)
// ---------------------------------------------------------------------------

/// Run one stage of DIF NTT using the independent reference oracle.
///
/// # Domain
/// `data` and `twiddles` are **Montgomery-encoded** BabyBear values
/// (`xR mod p`, R = 2³²). All values must be in `[0, p)`. Outputs remain
/// Montgomery-encoded. Passing canonical values produces wrong results silently.
///
/// Stage `s`: `h = n / 2^(s+1)`, `groups = 2^s`. Within each group,
/// pair element `i` with element `i+h` using `twiddles[i]`.
///
/// No platform restrictions — this is the specification.
pub fn ntt_reference_stage(data: &mut [u32], twiddles: &[u32], s: usize) {
    let n = data.len();
    let h = n >> (s + 1);
    let groups = 1usize << s;

    // Representation invariant: Montgomery-domain values must be < p at stage entry.
    debug_assert!(data.iter().all(|&x| x < crate::field::babybear::constants::BABYBEAR_P),
        "ntt_reference_stage entry: value >= p at stage {}", s);
    debug_assert!(twiddles.iter().all(|&x| x < crate::field::babybear::constants::BABYBEAR_P),
        "ntt_reference_stage entry: twiddle >= p at stage {}", s);

    for g in 0..groups {
        let offset = g * 2 * h;
        for i in 0..h {
            let (x, y) = butterfly_reference(
                data[offset + i],
                data[offset + i + h],
                twiddles[i],
            );
            data[offset + i] = x;
            data[offset + i + h] = y;
        }
    }

    // Postcondition: outputs remain in Montgomery domain (< p).
    debug_assert!(data.iter().all(|&x| x < crate::field::babybear::constants::BABYBEAR_P),
        "ntt_reference_stage exit: value >= p at stage {}", s);
}

/// Full DIF NTT using the reference oracle.
///
/// # Domain
/// `data` and all twiddles are **Montgomery-encoded** BabyBear values.
/// Output is Montgomery-encoded, in bit-reversed order.
///
/// Runs stages `0..log_n` in order.
pub fn ntt_reference(data: &mut [u32], twiddles_per_stage: &[Vec<u32>]) {
    for s in 0..twiddles_per_stage.len() {
        ntt_reference_stage(data, &twiddles_per_stage[s], s);
    }
}

/// Bit-reverse permutation in-place.
///
/// # Domain
/// `data` may be in any representation — this is a pure index permutation
/// that does not touch values. The representation is preserved.
pub fn bit_reverse(data: &mut [u32]) {
    let n = data.len();
    if n <= 1 {
        return;
    }
    let bits = n.trailing_zeros() as usize;
    for i in 0..n {
        let j = (i as u32).reverse_bits() >> (32 - bits) as u32;
        let j = j as usize;
        if j > i {
            data.swap(i, j);
        }
    }
}

// ---------------------------------------------------------------------------
// x86_64 only: scalar and AVX-512 NTT backends
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
mod x86 {
    use crate::field::babybear::canonical::MontgomeryBabyBear;

    /// Run one stage of DIF NTT using the typed `butterfly()` wrapper.
    ///
    /// # Domain
    /// `data` and `twiddles` are **Montgomery-encoded** BabyBear values
    /// (`xR mod p`, R = 2³²). All values must be in `[0, p)`.
    /// Internally wraps each raw u32 in `MontgomeryBabyBear` before calling
    /// `butterfly()`, then unwraps the result back to raw u32.
    ///
    /// Same algorithm structure as [`super::ntt_reference_stage`], but
    /// routes through the typed public API to verify boundary correctness.
    pub fn ntt_scalar_stage(data: &mut [u32], twiddles: &[u32], s: usize) {
        let n = data.len();
        let h = n >> (s + 1);
        let groups = 1usize << s;
        for g in 0..groups {
            let offset = g * 2 * h;
            for i in 0..h {
                let (ra, rb) = crate::avx512_butterfly_32bit::butterfly(
                    MontgomeryBabyBear(data[offset + i]),
                    MontgomeryBabyBear(data[offset + i + h]),
                    MontgomeryBabyBear(twiddles[i]),
                );
                data[offset + i] = ra.inner();
                data[offset + i + h] = rb.inner();
            }
        }
    }

    /// Full DIF NTT using typed `butterfly()`.
    ///
    /// # Domain
    /// `data` and all twiddles are **Montgomery-encoded** BabyBear values.
    /// Output is Montgomery-encoded, in bit-reversed order.
    pub fn ntt_scalar(data: &mut [u32], twiddles_per_stage: &[Vec<u32>]) {
        for s in 0..twiddles_per_stage.len() {
            ntt_scalar_stage(data, &twiddles_per_stage[s], s);
        }
    }

    /// Run one stage of DIF NTT using AVX-512 butterfly pass.
    ///
    /// # Domain
    /// `data` and `twiddles` are **Montgomery-encoded** BabyBear values
    /// (`xR mod p`, R = 2³²). All values must be in `[0, p)`.
    ///
    /// # Safety
    /// - Caller must ensure AVX-512F and AVX-512DQ are available
    ///   (use `is_avx512_supported`).
    /// - `data.len()` must be a power of 2 and `>= 2`.
    /// - `twiddles.len()` must be `>= data.len() / 2^(s+1)` (half-size for stage s).
    /// - All values in `data` and `twiddles` must be `< BABYBEAR_P`.
    /// - Canonical values produce wrong results silently.
    pub unsafe fn ntt_avx512_stage(data: &mut [u32], twiddles: &[u32], s: usize) {
        let n = data.len();
        let h = n >> (s + 1);
        let groups = 1usize << s;
        for g in 0..groups {
            let offset = g * 2 * h;
            crate::avx512_butterfly_32bit::avx512_butterfly_pass_32(
                data.as_mut_ptr().add(offset),
                twiddles.as_ptr(),
                2 * h,
            );
        }
    }

    /// Full DIF NTT using AVX-512.
    ///
    /// # Domain
    /// `data` and all twiddles are **Montgomery-encoded** BabyBear values.
    /// Output is Montgomery-encoded, in bit-reversed order.
    ///
    /// # Safety
    /// - Caller must ensure AVX-512F and AVX-512DQ are available.
    /// - `data.len()` must be a power of 2 and `>= 2`.
    /// - `twiddles_per_stage` must have `log_n = data.len().trailing_zeros()` entries.
    /// - Stage `s` twiddles must have `>= data.len() / 2^(s+1)` elements.
    /// - All values must be `< BABYBEAR_P` (Montgomery domain).
    pub unsafe fn ntt_avx512(data: &mut [u32], twiddles_per_stage: &[Vec<u32>]) {
        for s in 0..twiddles_per_stage.len() {
            ntt_avx512_stage(data, &twiddles_per_stage[s], s);
        }
    }
}

#[cfg(target_arch = "x86_64")]
pub use x86::{ntt_scalar_stage, ntt_scalar, ntt_avx512_stage, ntt_avx512};
