//! IEP enforcement layer — Instrumented Evolution Protocol v0.1.
//!
//! This module converts the IEP specification into executable assertions.
//! The core entry point is `evaluate::evaluate_transition`.
//!
//! Module layout:
//!   artifact  — Artifact, EvidenceGraph, EvidenceKind, EvidenceResult (NTT-specific)
//!   evidence  — EvidenceArtifact, ReferenceCheck (general, language-agnostic)
//!   policy    — PromotionPolicy, Gate
//!   evaluate  — evaluate_transition(), PromotionDecision, RejectionReason
//!   event     — PromotionEvent (immutable transition record)

pub mod artifact;
pub mod evaluate;
pub mod event;
pub mod evidence;
pub mod policy;

// Issue #5: Verification Gate v1
pub mod predicate;
pub mod coverage;
pub mod gate;
pub mod evidence_contract;
