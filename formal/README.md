# Formal Verification — Lean 4 Montgomery Arithmetic

> The Lean 4 source is in a separate public repository. This file tells you where to find it and what it proves.

## Repository

- **URL:** https://github.com/Cartilage-Stairwells/tscp-anchor
- **Tag:** `v0.1.0-zksha-rx`
- **Key file:** `Montgomery.lean` (216 lines)

## What Is Proven

The Lean 4 formalization covers **Montgomery arithmetic foundations** for the BabyBear field (P = 0x78000001):

1. **Montgomery multiplication closure:** The Montgomery product of two residues is itself a valid residue.
2. **Residue laws:** Addition, subtraction, and multiplication preserve the residue invariant.
3. **Bézout identity for modular inverse:** `R * R_inv = P * NEG_INV + 1` (proven by `decide`).

## What Is NOT Proven

- The AVX-512 SIMD implementation is NOT formally verified.
- The butterfly operations are NOT formally verified.
- The NTT correctness is validated by differential testing, not formal proof.

## Verification Status

| Aspect | Value |
|--------|-------|
| Theorem count | 33 |
| Module count | 7 |
| `sorry` in core module | 0 |
| `sorry` in examples | 3 (isolated, documented) |
| New axioms | 0 |
| Proof method | `decide` (computational) |
| Classical logic | 0 uses |

## Build

```bash
git clone https://github.com/Cartilage-Stairwells/tscp-anchor.git
cd tscp-anchor
git checkout v0.1.0-zksha-rx
lake build
```

Requires: Lean 4.32.1, Lake 5.0.0

## Claim Boundary

The external claim is:

> "Supported by formally verified arithmetic foundations."

This means: the arithmetic primitives used by the implementation have formal proofs in Lean 4. It does NOT mean the implementation itself is formally verified. The SIMD implementation is validated by:

- Independent reference oracle comparison
- Backend equivalence testing (135 staged comparisons)
- Differential testing across three backends
