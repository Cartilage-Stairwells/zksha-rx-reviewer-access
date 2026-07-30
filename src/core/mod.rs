//! Core evidence protocol — the immutable vocabulary.
//!
//! This module defines the protocol vocabulary that all domains share.
//! It contains NO domain knowledge:
//!   ❌ π
//!   ❌ AVX-512
//!   ❌ NEON
//!   ❌ SIMD concepts
//!   ❌ serialization adapters
//!   ❌ benchmark infrastructure
//!   ❌ DomainAdapter trait
//!   ❌ evaluator implementations
//!
//! The core defines admissibility. Domains provide claims.
//!
//! Dependency direction:
//!   EnvironmentContract → identity_hash()
//!   Artifact → EvaluationRequest → EvaluationResult → EvidenceArtifact
//!
//! The critical separation:
//!   Artifact → immutable → stable across environments, compilers, adapters
//!   Evaluation → variable → changes with each execution
//!   Evidence → variable → the record of what happened
//!
//! New machine → same artifact
//! New compiler → same artifact
//! New adapter → same artifact
//! New evaluation time → same artifact
//! Only the evidence record changes.

pub mod hash;
pub mod canonical;
pub mod environment;
pub mod artifact;
pub mod predicate;
pub mod evidence;

// Re-export the primary types for convenience.
pub use hash::Hash;
pub use canonical::Canonical;
pub use environment::EnvironmentContract;
pub use artifact::{Artifact, EvaluationRequest, EvaluationResult};
pub use predicate::{
    PredicateDescriptor, PredicateResult, PredicateStatus,
    PredicateSet, PredicateStage, Predicate, EvaluationContext,
};
pub use evidence::{EvidenceArtifact, ProvenanceRecord, AdmissionDecision, evaluate_admission};
