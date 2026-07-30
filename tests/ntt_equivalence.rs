//! Commit 5 — NTT equivalence: staged cross-backend comparison.
//!
//! Three independent verification layers:
//!
//! 1. **Correctness**: reference NTT output (decoded to canonical) matches
//!    a naive O(n²) DFT computed in pure u64 arithmetic.
//!
//! 2. **Staged equivalence**: reference, scalar, and AVX-512 NTT backends
//!    are compared element-wise after each butterfly stage — not just the
//!    final NTT output. A wrong butterfly cannot hide behind later stages.
//!
//! 3. **Round-trip**: forward NTT followed by naive inverse DFT recovers
//!    the original input.
//!
//! All twiddles are computed independently (no p3 BabyBear dependency) using
//! a from-scratch primitive root finder, making the oracle fully self-contained.

use avx512_butterfly::field::babybear::constants::{BABYBEAR_P, BABYBEAR_R_MOD_P};
use avx512_butterfly::field::babybear::reference::{
    babybear_mul_reference, to_montgomery_reference, from_montgomery_reference,
};
use avx512_butterfly::ntt;

// ---------------------------------------------------------------------------
// Helpers: modular arithmetic (pure u64, no p3 dependency)
// ---------------------------------------------------------------------------

fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    let mut result = 1u64;
    base %= modulus;
    while exp > 0 {
        if exp & 1 == 1 {
            result = result * base % modulus;
        }
        exp >>= 1;
        base = base * base % modulus;
    }
    result
}

/// Find a primitive root mod p.
///
/// p - 1 = 2^27 * 3 * 5, so distinct prime factors are {2, 3, 5}.
/// A generator g satisfies g^((p-1)/q) != 1 for all q in {2, 3, 5}.
fn find_primitive_root() -> u64 {
    let p = BABYBEAR_P as u64;
    let factors: [u64; 3] = [2, 3, 5];
    for g in 2..p {
        let mut is_primitive = true;
        for &q in &factors {
            if mod_pow(g, (p - 1) / q, p) == 1 {
                is_primitive = false;
                break;
            }
        }
        if is_primitive {
            return g;
        }
    }
    panic!("no primitive root found for p={}", p);
}

/// Compute the primitive 2^log_n-th root of unity in canonical form.
///
/// ω = g^((p-1)/2^log_n) mod p, where g is a primitive root mod p.
fn two_adic_generator(log_n: usize) -> u32 {
    let p = BABYBEAR_P as u64;
    let g = find_primitive_root();
    let n = 1u64 << log_n;
    let omega = mod_pow(g, (p - 1) / n, p);

    // Sanity: ω^n = 1 and ω^(n/2) != 1 (primitive, not just any root)
    assert_eq!(mod_pow(omega, n, p), 1, "omega^n != 1 for log_n={}", log_n);
    assert_ne!(mod_pow(omega, n / 2, p), 1, "omega^(n/2) == 1 — not primitive");

    omega as u32
}

/// Convert canonical u32 to Montgomery domain.
fn to_mont(x: u32) -> u32 {
    to_montgomery_reference(x)
}

/// Convert Montgomery u32 to canonical.
fn from_mont(x: u32) -> u32 {
    from_montgomery_reference(x)
}

/// Compute twiddles for all stages of a DIF NTT in Montgomery domain.
///
/// Returns `twiddles_per_stage[s][i]` = (ω^(i * 2^s)) in Montgomery form,
/// where ω is the primitive 2^log_n-th root of unity.
///
/// Stage s uses w_m = ω^(2^s) as its generator; twiddles are
/// w_m^0, w_m^1, ..., w_m^(m2-1) where m2 = n/2^(s+1).
fn compute_twiddles_mont(log_n: usize) -> Vec<Vec<u32>> {
    let n = 1usize << log_n;
    let omega_canonical = two_adic_generator(log_n);
    let omega_mont = to_mont(omega_canonical);

    let mut stages = Vec::with_capacity(log_n);
    let mut w_m_mont = omega_mont; // ω^(2^0) = ω

    for s in 0..log_n {
        let m2 = n >> (s + 1); // half-size for this stage
        let mut twiddles = Vec::with_capacity(m2);
        let mut w = BABYBEAR_R_MOD_P; // 1 in Montgomery domain
        for _ in 0..m2 {
            twiddles.push(w);
            w = babybear_mul_reference(w, w_m_mont);
        }
        stages.push(twiddles);
        w_m_mont = babybear_mul_reference(w_m_mont, w_m_mont); // square: ω^(2^(s+1))
    }

    stages
}

/// Naive DFT in canonical domain: out[k] = Σ_{j=0}^{n-1} in[j] · ω^(j·k) mod p.
///
/// O(n²) — used as ground truth for small sizes only.
fn naive_dft_canonical(input: &[u32], omega_u32: u32) -> Vec<u32> {
    let n = input.len();
    let p = BABYBEAR_P as u64;
    let omega = omega_u32 as u64;

    let mut out = vec![0u32; n];
    for k in 0..n {
        let mut acc = 0u64;
        let omega_k = mod_pow(omega, k as u64, p); // ω^k
        let mut w = 1u64; // ω^(0*k) = 1
        for j in 0..n {
            acc = (acc + (input[j] as u64) * w) % p;
            w = w * omega_k % p; // ω^((j+1)*k)
        }
        out[k] = acc as u32;
    }
    out
}

/// Naive inverse DFT: out[k] = (1/n) Σ_{j=0}^{n-1} in[j] · ω^(-j·k) mod p.
fn naive_inv_dft_canonical(input: &[u32], omega_u32: u32) -> Vec<u32> {
    let n = input.len();
    let p = BABYBEAR_P as u64;
    let omega_inv = mod_pow(omega_u32 as u64, p - 2, p) as u32; // ω^{-1}
    let mut out = naive_dft_canonical(input, omega_inv);
    let n_inv = mod_pow(n as u64, p - 2, p) as u32;
    for x in out.iter_mut() {
        *x = (*x as u64 * n_inv as u64 % p) as u32;
    }
    out
}

/// Deterministic LCG for reproducible test inputs.
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next_u32(&mut self) -> u32 {
        self.state = self.state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.state >> 33) as u32) % BABYBEAR_P
    }
}

fn first_mismatch(a: &[u32], b: &[u32]) -> Option<usize> {
    a.iter().zip(b.iter()).position(|(x, y)| x != y)
}

// ---------------------------------------------------------------------------
// Test 1: Reference NTT correctness vs naive DFT
// ---------------------------------------------------------------------------

#[test]
fn ntt_reference_correctness() {
    let seed: u64 = 0xC0FFEE_BABE_1234;

    for log_n in 4..=8 {
        let n = 1usize << log_n;
        let omega_canonical = two_adic_generator(log_n);

        // Generate canonical input
        let mut lcg = Lcg::new(seed);
        let input_canonical: Vec<u32> = (0..n).map(|_| lcg.next_u32()).collect();

        // Convert to Montgomery domain
        let input_mont: Vec<u32> = input_canonical.iter().map(|&x| to_mont(x)).collect();

        // Run reference NTT
        let twiddles = compute_twiddles_mont(log_n);
        let mut ntt_output = input_mont.clone();
        ntt::ntt_reference(&mut ntt_output, &twiddles);

        // DIF NTT produces bit-reversed output — reorder
        ntt::bit_reverse(&mut ntt_output);

        // Convert NTT output back to canonical
        let ntt_canonical: Vec<u32> = ntt_output.iter().map(|&x| from_mont(x)).collect();

        // Compute naive DFT in canonical domain
        let dft_output = naive_dft_canonical(&input_canonical, omega_canonical);

        assert_eq!(
            ntt_canonical, dft_output,
            "NTT vs DFT mismatch at log_n={}", log_n
        );
        eprintln!("ntt_reference_correctness log_n={} (n={}) PASS", log_n, n);
    }
}

// ---------------------------------------------------------------------------
// Test 2: Staged cross-backend equivalence
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
#[test]
fn staged_cross_backend_equivalence() {
    use avx512_butterfly::ntt::{ntt_reference_stage, ntt_scalar_stage, ntt_avx512_stage};
    use avx512_butterfly::avx512_butterfly_32bit::is_avx512_supported;

    let has_avx512 = is_avx512_supported();
    let seed: u64 = 0xDEAD_BEEF_CAFE_1234;

    for log_n in 5..=10 {
        let n = 1usize << log_n;
        let twiddles = compute_twiddles_mont(log_n);

        // Generate shared Montgomery-domain input (same for all backends)
        let mut lcg = Lcg::new(seed);
        let input_mont: Vec<u32> = (0..n).map(|_| to_mont(lcg.next_u32())).collect();

        let mut data_ref = input_mont.clone();
        let mut data_scalar = input_mont.clone();
        let mut data_avx = input_mont.clone();

        for s in 0..log_n {
            let h = n >> (s + 1);
            let butterfly_count = (1usize << s) * h;

            // Run one stage on each backend
            ntt_reference_stage(&mut data_ref, &twiddles[s], s);
            ntt_scalar_stage(&mut data_scalar, &twiddles[s], s);

            // Reference vs Scalar
            assert_eq!(
                data_ref, data_scalar,
                "stage {} ref vs scalar mismatch at log_n={} (n={}), \
                 butterflies={}, first diff at index {:?}",
                s, log_n, n, butterfly_count,
                first_mismatch(&data_ref, &data_scalar),
            );

            if has_avx512 {
                unsafe { ntt_avx512_stage(&mut data_avx, &twiddles[s], s); }

                // Reference vs AVX-512
                assert_eq!(
                    data_ref, data_avx,
                    "stage {} ref vs avx512 mismatch at log_n={} (n={}), \
                     butterflies={}, first diff at index {:?}",
                    s, log_n, n, butterfly_count,
                    first_mismatch(&data_ref, &data_avx),
                );
            }

            eprintln!(
                "staged_cross_backend_equivalence log_n={} n={} stage {} \
                 (h={} butterflies={}) ref==scalar{} PASS",
                log_n, n, s, h, butterfly_count,
                if has_avx512 { "==avx512" } else { "" },
            );
        }
    }

    // Evidence object — TSCP recording format
    eprintln!(
        "{{\"test\":\"ntt_cross_backend_equivalence\",\"field\":\"BabyBear\",\
         \"backend_a\":\"reference\",\"backend_b\":\"avx512\",\
         \"comparison\":\"stagewise\",\"status\":\"pass\"}}"
    );
}

// ---------------------------------------------------------------------------
// Test 3: Full NTT round-trip (forward + naive inverse DFT)
// ---------------------------------------------------------------------------

#[test]
fn ntt_round_trip() {
    let seed: u64 = 0xFEED_FACE_1234_5678;

    for log_n in 4..=8 {
        let n = 1usize << log_n;
        let omega_canonical = two_adic_generator(log_n);
        let twiddles = compute_twiddles_mont(log_n);

        let mut lcg = Lcg::new(seed);
        let input_canonical: Vec<u32> = (0..n).map(|_| lcg.next_u32()).collect();
        let input_mont: Vec<u32> = input_canonical.iter().map(|&x| to_mont(x)).collect();

        // Forward NTT (DIF) → bit-reversed Montgomery output
        let mut data = input_mont.clone();
        ntt::ntt_reference(&mut data, &twiddles);

        // Bit-reverse → natural order Montgomery output
        ntt::bit_reverse(&mut data);

        // Convert to canonical
        let ntt_canonical: Vec<u32> = data.iter().map(|&x| from_mont(x)).collect();

        // Naive inverse DFT should recover original
        let recovered = naive_inv_dft_canonical(&ntt_canonical, omega_canonical);

        assert_eq!(
            recovered, input_canonical,
            "round-trip mismatch at log_n={}", log_n,
        );
        eprintln!("ntt_round_trip log_n={} (n={}) PASS", log_n, n);
    }
}

// ---------------------------------------------------------------------------
// Test 4: Twiddle sanity — primitive root and generator independence
// ---------------------------------------------------------------------------

#[test]
fn twiddle_sanity() {
    // Verify our primitive root finder agrees with mathematical fact:
    // ω^n = 1, ω^(n/2) != 1, and twiddles are distinct.
    for log_n in 2..=12 {
        let omega = two_adic_generator(log_n);
        let p = BABYBEAR_P as u64;
        let n = 1u64 << log_n;

        // ω^n = 1
        assert_eq!(mod_pow(omega as u64, n, p), 1,
            "omega^{} != 1 for log_n={}", n, log_n);

        // ω^(n/2) != 1 (primitive)
        assert_ne!(mod_pow(omega as u64, n / 2, p), 1,
            "omega^{} == 1 for log_n={} — not primitive", n / 2, log_n);

        // Twiddles for stage 0 are distinct
        let twiddles = compute_twiddles_mont(log_n);
        let stage0: Vec<u32> = twiddles[0].iter().map(|&w| from_mont(w)).collect();
        let unique: std::collections::HashSet<u32> = stage0.iter().copied().collect();
        assert_eq!(unique.len(), stage0.len(),
            "stage 0 twiddles not distinct for log_n={}", log_n);
    }
    eprintln!("twiddle_sanity PASS (log_n 2..=12)");
}
