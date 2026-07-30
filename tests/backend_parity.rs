//! Issue #4 — Backend Parity Corpus
//!
//! Tests that every enabled execution backend preserves the same field
//! semantics as the reference oracle. The comparison model is:
//!
//!   reference oracle → scalar backend → AVX-512 backend
//!
//! NOT: scalar == AVX-512 (that only proves agreement, not correctness).
//!
//! Test structure:
//!   corpus.rs         — shared test data, failure records, helpers
//!   boundary_cases.rs — Phase 1: explicit boundary values
//!   reduction_cases.rs — Phase 2: reduction stress + near-boundary scan
//!   equivalence.rs    — Phase 3+4: cross-backend matrix + NTT staged equivalence
//!
//! All values are Montgomery-encoded BabyBear residues (xR mod p, R = 2³²).
//! Deterministic seeds ensure reproducible failures across runs and platforms.

#[path = "backend_parity/corpus.rs"]
mod corpus;
#[path = "backend_parity/boundary_cases.rs"]
mod boundary_cases;
#[path = "backend_parity/reduction_cases.rs"]
mod reduction_cases;
#[path = "backend_parity/equivalence.rs"]
mod equivalence;
