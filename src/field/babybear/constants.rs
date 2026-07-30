/// BabyBear prime: p = 2^31 - 2^27 + 1 = 0x7800_0001
pub const BABYBEAR_P: u32 = 0x7800_0001;

/// Montgomery parameter: R = 2^32 (implicit, one machine word)
/// R mod p = 2^32 mod 0x7800_0001 = 0x0FFF_FFFE
pub const BABYBEAR_R_MOD_P: u32 = 0x0FFF_FFFE;

/// R^2 mod p — used to convert a canonical value into Montgomery domain
/// via montgomery_mul(x, R2_MOD_P).
/// R^2 mod 0x7800_0001 = 0x45DD_DDE3
pub const BABYBEAR_R2_MOD_P: u32 = 0x45DD_DDE3;

/// R^{-1} mod p — used by the reference oracle and from_montgomery.
/// R * R^{-1} ≡ 1 (mod p)
/// R^{-1} mod 0x7800_0001 = 0x3840_0000
pub const BABYBEAR_R_INV_MOD_P: u32 = 0x3840_0000;

/// neg_inv: -p^{-1} mod 2^32
///
/// Required by the Montgomery reduction step: m = (t mod R) * neg_inv (mod R)
/// Invariant: p.wrapping_mul(neg_inv).wrapping_add(1) == 0
///
/// Verified: 0x7800_0001 * 0x77FF_FFFF + 1 == 0 (mod 2^32)
pub const BABYBEAR_NEG_INV: u32 = 0x77FF_FFFF;

/// Compile-time invariant: catches any future edit that breaks the reduction.
const _: () = {
    let check = BABYBEAR_P.wrapping_mul(BABYBEAR_NEG_INV).wrapping_add(1);
    assert!(check == 0, "neg_inv invariant violated: p * neg_inv + 1 != 0 mod 2^32");
};

// Note on existing repo constants
// --------------------------------
// src/lib.rs and src/avx512_butterfly_32bit.rs both define:
//   const P: u32 = 0x78000001;          -- same as BABYBEAR_P ✓
//   const P_INV_NEG: u32 = 0x77FFFFFF;  -- same as BABYBEAR_NEG_INV ✓
//
// The numeric values agree. Migration (Commit 2) will route those sites
// through this module and delete the inline copies.
