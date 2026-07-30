/// Reference (oracle) implementation for BabyBear Montgomery multiplication.
///
/// This is the specification layer. Written for correctness, not performance:
/// all arithmetic uses u128 to avoid overflow. No optimized code in this
/// crate is trusted until it agrees with this oracle on all inputs.
///
/// Do not modify this file without updating the corpus golden vectors.
use super::constants::{BABYBEAR_P, BABYBEAR_R_INV_MOD_P, BABYBEAR_R_MOD_P};

/// Montgomery multiplication oracle.
///
/// Computes  a * b * R^{-1}  (mod p)  where R = 2^32.
///
/// Preconditions: a, b in [0, p)
/// Postcondition: result in [0, p), result ≡ a * b * R^{-1} (mod p)
#[must_use]
pub fn montgomery_mul_reference(a: u32, b: u32, modulus: u32, r_inv: u32) -> u32 {
    debug_assert!(a < modulus, "a={a} must be < modulus={modulus}");
    debug_assert!(b < modulus, "b={b} must be < modulus={modulus}");
    let p = modulus as u128;
    let result = ((a as u128) * (b as u128) * (r_inv as u128)) % p;
    result as u32
}

/// Convenience wrapper using BabyBear constants.
#[must_use]
pub fn babybear_mul_reference(a: u32, b: u32) -> u32 {
    montgomery_mul_reference(a, b, BABYBEAR_P, BABYBEAR_R_INV_MOD_P)
}

/// Convert canonical x in [0, p) → Montgomery domain: x * R mod p
#[must_use]
pub fn to_montgomery_reference(x: u32) -> u32 {
    debug_assert!(x < BABYBEAR_P);
    ((x as u64 * BABYBEAR_R_MOD_P as u64) % BABYBEAR_P as u64) as u32
}

/// Convert Montgomery-domain value → canonical: x * R^{-1} mod p
#[must_use]
pub fn from_montgomery_reference(x: u32) -> u32 {
    montgomery_mul_reference(x, 1, BABYBEAR_P, BABYBEAR_R_INV_MOD_P)
}

/// Reference addition mod p.
#[must_use]
pub fn babybear_add_reference(a: u32, b: u32) -> u32 {
    debug_assert!(a < BABYBEAR_P);
    debug_assert!(b < BABYBEAR_P);
    let s = a + b;
    if s >= BABYBEAR_P { s - BABYBEAR_P } else { s }
}

/// Reference subtraction mod p.
#[must_use]
pub fn babybear_sub_reference(a: u32, b: u32) -> u32 {
    debug_assert!(a < BABYBEAR_P);
    debug_assert!(b < BABYBEAR_P);
    if a >= b { a - b } else { a + BABYBEAR_P - b }
}

/// Reference radix-2 butterfly oracle.
///
/// Computes the DIF (decimation in frequency) Cooley-Tukey butterfly:
///
/// ```text
/// x = a + b       mod p
/// y = (a - b) * w mod p
/// ```
///
/// All values are Montgomery-encoded (`xR mod p`, R = 2³²).
/// Internally uses only `babybear_mul_reference`, `babybear_add_reference`,
/// and `babybear_sub_reference` — no shared code with any production path.
///
/// **Preconditions:** `a`, `b`, `w ∈ [0, p)` (Montgomery-encoded BabyBear values).
/// **Postconditions:** both outputs `∈ [0, p)`, Montgomery-encoded.
///
/// This is the acceptance oracle for Commit 4: any butterfly implementation
/// must produce bit-identical output to this function on every input.
#[must_use]
pub fn butterfly_reference(a: u32, b: u32, w: u32) -> (u32, u32) {
    debug_assert!(a < BABYBEAR_P, "a={a} must be < p");
    debug_assert!(b < BABYBEAR_P, "b={b} must be < p");
    debug_assert!(w < BABYBEAR_P, "w={w} must be < p");
    // DIF butterfly: x = a + b, y = (a - b) * w
    let x  = babybear_add_reference(a, b);
    let d  = babybear_sub_reference(a, b);
    let y  = babybear_mul_reference(d, w);
    (x, y)
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::constants::*;

    #[test]
    fn r_times_r_inv_is_one() {
        let p   = BABYBEAR_P as u128;
        let r   = (1u128 << 32) % p;
        let inv = BABYBEAR_R_INV_MOD_P as u128;
        assert_eq!((r * inv) % p, 1, "R * R_INV must be 1 mod p");
    }

    #[test]
    fn neg_inv_invariant() {
        let check = BABYBEAR_P.wrapping_mul(BABYBEAR_NEG_INV).wrapping_add(1);
        assert_eq!(check, 0, "p * neg_inv + 1 must be 0 mod 2^32");
    }

    #[test]
    fn roundtrip() {
        for x in [0u32, 1, 2, BABYBEAR_P - 1, 42, 0x1234_5678 % BABYBEAR_P] {
            let enc = to_montgomery_reference(x);
            let dec = from_montgomery_reference(enc);
            assert_eq!(dec, x, "round-trip failed for x={x}");
        }
    }

    #[test]
    fn commutativity() {
        for (a, b) in [(3u32, 7u32), (0, 1), (1, 0), (BABYBEAR_P - 1, 2)] {
            assert_eq!(
                babybear_mul_reference(a, b),
                babybear_mul_reference(b, a),
                "oracle must commute for ({a},{b})"
            );
        }
    }
}
