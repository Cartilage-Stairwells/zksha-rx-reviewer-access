# Compatibility Matrix

| Property | zkSHA-Rx | Plonky3 | Compatible? |
|----------|----------|---------|-------------|
| Field prime | 0x78000001 | 0x78000001 | ✅ Identical |
| Montgomery R | 2^32 mod P | 2^32 mod P | ✅ Identical |
| Butterfly type | DIF | DIF | ✅ Same |
| Butterfly formula | (a+b, (a-b)*w) | (a+b, (a-b)*w) | ✅ Identical |
| Element type | u32 (Montgomery) | BabyBear (Montgomery) | ✅ Binary-compatible |
| NTT structure | Radix-2 DIF | Radix-2 DIF | ✅ Same |
| Twiddle convention | Standard DIF | Standard DIF | ✅ Same |
| Output order | Bit-reversed | Bit-reversed | ✅ Same |
| Formal verification | 83 Lean theorems | None | N/A (zkSHA-Rx only) |
| AVX-512 kernel | ✅ Hand-written | ✅ Hand-written | Comparable |
| Scalar fallback | ✅ | ✅ | ✅ |
