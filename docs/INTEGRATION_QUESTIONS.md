# Integration Questions

Tracks technical questions related to Plonky3/SP1 integration that arise from external review.

## Open Questions

| # | Question | Source | Date | Status | Resolution |
|---|----------|--------|------|--------|------------|
| 1 | What is the performance overhead of the adapter vs direct Plonky3 NTT? | Internal | 2026-08-03 | Measured | Scalar: 1.015× (within noise). AVX-512: pending hardware. |
| 2 | Does the adapter support all Plonky3 NTT call patterns? | Internal | 2026-08-03 | Open | Adapter implements Butterfly trait; full DFT trait coverage pending. |
| 3 | What happens when AVX-512 is unavailable? | Internal | 2026-08-03 | Resolved | Scalar fallback via `cfg` attribute. No AVX-512 → scalar path. |
| 4 | Are twiddle factors compatible? | Internal | 2026-08-03 | Resolved | Yes — same DIF convention, same twiddle indexing. Bit-identical at sizes 17, 64, 256. |

## Anticipated External Questions

| Question | Prepared Answer |
|----------|----------------|
| Can I drop in zkSHA-Rx as a Plonky3 backend? | Not yet — the adapter is a prototype. Integration requires wrapping the full `TwoAdicSubgroupDft` trait, not just `Butterfly`. |
| Does this work with SP1? | Not yet tested. SP1 uses Plonky3's field types, so binary compatibility should hold, but the adapter has not been tested against SP1. |
| What about Goldilocks field? | The formal verification is BabyBear-specific. Goldilocks would require a new Montgomery parameter set and re-verification. |
| Can I use the Lean proofs with my own NTT? | The proofs cover the algorithm, not a specific implementation. The proof-to-code map shows how the Lean theorems correspond to Rust functions. Adapting to a different implementation requires building a new correspondence. |
