# Architecture wins — avx512-butterfly

This document records the reasoning behind each structural decision made
during the verification sequence. It is a historical record, not a spec.
When something looks intentionally ugly or surprisingly constrained,
the answer is probably here.

---

## Win 1 — Single arithmetic authority (Commit 2)

**Problem:** `scalar_montgomery_mul_32` in `avx512_butterfly_32bit.rs` and
`mont_reduce_scalar` in `lib.rs` were independent inline copies of the same
R=2³² Montgomery reduction. Each had its own local `P` and `P_INV_NEG`
constants. Any future constant correction would require finding and fixing
all copies.

**Decision:** Delete both. Route every R=2³² scalar multiply through
`ScalarBackend::mul(a, b)`.

**Result:** One function owns the arithmetic. Future backends
(AVX-512 vectorized, NEON, GPU) implement `MontgomeryBackend` and are
verified against the same reference oracle. The oracle itself
(`field/babybear/reference.rs`) is never modified — it is the ground truth.

**Intentional non-change:** `mont_reduce_r64` (the R=2⁶⁴ two-step reduction
in `lib.rs`) was left untouched. It is a different reduction domain and
requires its own decision — see Issue #1.

---

## Win 2 — Compile-time domain separation (Commit 3A)

**Problem:** Every function on the NTT critical path accepted raw `u32` values.
`butterfly(a: u32, b: u32, omega: u32)` is ambiguous — are the inputs
canonical field elements or Montgomery-encoded? Getting it wrong produces
wrong outputs with no error. This is the category of bug that only appears
at the NTT output comparison stage, after the arithmetic has already
propagated the mistake through every stage.

**Decision:** Introduce two newtypes:

```
CanonicalBabyBear(u32)   — stores x,     invariant: x < p
MontgomeryBabyBear(u32)  — stores xR mod p
```

`CanonicalBabyBear * MontgomeryBabyBear` does not compile. Mixing
domains is a type error, not a logic error discovered at runtime.

**Result:** The compiler catches the entire class of canonical/Montgomery
confusion bugs before the binary is built. Commit 3B (butterfly migration)
becomes a mechanical wiring job: replace `u32` with the appropriate wrapper,
confirm tests still pass, done.

**Intentional omission:** `Add`, `Sub`, `Neg` for `CanonicalBabyBear` are
not implemented yet. Canonical arithmetic is needed for the butterfly's
modular add/sub steps, but those are addressed in Commit 3B where the
butterfly signature is being reworked anyway. Adding them now without a
consumer would be speculative.

---

## Win 3 — Deferred decisions are documented, not silenced (Issue #1)

**Problem:** After Commit 2, `mont_reduce_r64` and `P_INV_NEG` remained in
`lib.rs`. A future reader would see code that looks like it survived the
cleanup by accident and might remove it without understanding the distinction
between R=2³² and R=2⁶⁴ reduction.

**Decision:** Open Issue #1 before moving forward. The issue records:
- Why it was left (different reduction domain, out of Commit 2 scope)
- The two acceptable outcomes (explicit `R64Backend` or proven dead code)
- The mechanical acceptance criterion (`rg` zero-result check)
- The constraint (do not touch before Commit 4)

**Result:** The ugly code has a ticket. Anyone looking at `lib.rs` can
find the reasoning in thirty seconds. The decision is deferred on purpose,
not forgotten.

---

## Win 4 — Raw u32 boundary rule (documentation commit)

**Problem:** The wrapper types cannot reach into every layer. AVX-512
intrinsics, packed SIMD buffers, and FFI surfaces will always use raw
integers. Without a written rule, the domain annotation discipline that the
wrappers enforce in Rust code quietly disappears at those boundaries.

**Decision:** Any function accepting raw `u32` field values must document
the domain in its signature or `SAFETY` comment. Three tiers:

1. **Typed signature** — preferred whenever possible.
2. **`SAFETY` comment with explicit domain** — required at SIMD/FFI boundaries.
3. **Bare `u32` with no annotation** — never acceptable.

**Result:** The rule extends the wrapper discipline to layers the type
system cannot reach. It is enforced by code review, not the compiler.

---

## Invariant table

| # | Invariant | Where enforced |
|---|---|---|
| I1 | `ScalarBackend::mul` is the only R=2³² arithmetic authority | By deletion — no other copy exists |
| I2 | The reference oracle is never modified or removed | By convention + code review |
| I3 | Corpus and golden vectors are immutable | `tests/babybear_montgomery.rs` — failure = contract regression |
| I4 | `mont_reduce_r64` is a separate domain, not a duplicate | Issue #1 + inline comment in `lib.rs` |
| I5 | Raw `u32` boundaries carry domain annotations | By convention + code review |
| I6 | `CanonicalBabyBear * MontgomeryBabyBear` does not compile | By type system |

---

## What comes next

**Commit 3B** — migrate the butterfly signature from raw `u32` to
`MontgomeryBabyBear`. This is a mechanical substitution. The arithmetic
does not change. The test for success is:

> `cargo test --test babybear_domain` and `cargo test --test babybear_montgomery`
> both pass with zero changes to assertion values.

The AVX-512 layer stays as raw `u32` internally — it gets a `SAFETY` comment
documenting the domain. The typed wrappers stop at the SIMD boundary for now.

**Commit 4** — explicit butterfly contract. Pre- and postconditions written
down. Scalar reference NTT aligned with the AVX-512 implementation.

**Commits 5–6** — NTT equivalence proof, then SIMD acceleration.
