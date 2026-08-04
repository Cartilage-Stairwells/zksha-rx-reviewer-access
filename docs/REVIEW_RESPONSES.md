# Review Responses

This document tracks all responses from external reviewers, including questions raised, evidence requested, and changes made in response.

## Response Log

| Date | Reviewer | Channel | Summary | Status | Action Taken |
|------|----------|---------|---------|--------|-------------|
| _pending_ | _—_ | _—_ | _—_ | _—_ | _—_ |

## Response Handling Protocol

1. **Acknowledge within 24 hours** — even if the response is "I need time to review"
2. **Record the response** in this document and the ExternalizationLog entity
3. **Classify the response**: technical question, evidence request, integration interest, skeptical challenge, or rejection
4. **Prepare a precise answer** — never improvise on technical claims; defer to PROJECT_FACTS.md
5. **Update artifacts** if the response reveals a gap or error
6. **Track resolution** — record what changed as a result of the feedback

## Prepared Responses to Likely Questions

### Q: "Are the Lean proofs connected to the optimized SIMD implementation?"

**A:** The verified specification covers the algorithmic level — Montgomery arithmetic, butterfly algebra, and NTT stage composition. SIMD backend equivalence (reference ↔ scalar ↔ AVX-512) is validated through 102 differential tests, not formal proof. The formal model proves the algorithm correct; the tests prove the implementations agree. The conformance gap between Lean and Rust is bridged by the proof-to-code correspondence map and differential testing. See `docs/SECURITY_MODEL.md` for the full boundary.

### Q: "Why integrate another NTT instead of improving Plonky3's?"

**A:** The integration is intentionally backend-level. The adapter implements Plonky3's `Butterfly<BabyBear>` trait without modifying Plonky3's existing NTT path. This allows evaluation of the verified backend without architectural disruption. The value proposition is not kernel speed (Plonky3 already has a performant AVX-512 NTT) — it is the formal verification layer and custody framework.

### Q: "What is the actual end-to-end proving speedup?"

**A:** We do not claim end-to-end proving speedup. The 2.65× is a kernel-level measurement (AVX-512 vs scalar). End-to-end impact depends on NTT's share of total proving time (30–60% in STARK systems) and integration overhead, which we have not yet measured.

### Q: "Has anyone independently reproduced the benchmarks?"

**A:** Not yet. Independent reproduction is the highest-priority next step. The three-lane benchmark is reproducible via `make bench` on any AVX-512 capable machine. Evidence artifacts (benchmark output, CPU info, correctness receipt) are in the `evidence/` directory.
