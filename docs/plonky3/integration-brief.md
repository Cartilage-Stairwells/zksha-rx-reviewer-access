# Integration Brief

**Goal:** Integrate zkSHA-Rx's verified NTT backend into Plonky3 as an alternative DFT implementation.

**Approach:** Adapter pattern via Plonky3's `TwoAdicSubgroupDft` trait.

**Status:** Prototype adapter compiles and produces bit-identical output. AVX-512 benchmark pending.

**Risk:** Low. Types are binary-compatible. Adapter is additive — does not modify Plonky3's existing NTT path.

**Timeline:** Depends on hardware availability for AVX-512 benchmarking. Adapter itself is ~200 lines.
