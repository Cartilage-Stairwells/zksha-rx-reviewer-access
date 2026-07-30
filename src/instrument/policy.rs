//! IEP Promotion Policy.
//!
//! The policy is the executable decision rule. It declares which evidence
//! kinds are required for a transition, which are informational, and what
//! happens when requirements are unmet. The policy itself is hashed — a
//! promotion event records the policy_hash alongside the evidence_hash,
//! making the evaluation rule part of the custody chain.

use crate::instrument::artifact::EvidenceKind;

/// The gate classification for an evidence kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gate {
    /// Must be present and passing. Absence or failure → reject transition.
    Required,
    /// Absence generates a warning. Failure → reject.
    Recommended,
    /// Collected after promotion is allowed. Cannot block or force promotion.
    Informational,
}

/// Promotion policy for a single instrument instance.
#[derive(Debug, Clone)]
pub struct PromotionPolicy {
    pub policy_id:   String,
    pub spec:        String,
    /// SHA256 of the canonical policy serialization.
    /// Recorded in every promotion event. Drift is detectable.
    pub policy_hash: String,

    /// Evidence kinds required for promotion. Ordered: evaluation stops at
    /// first failure; subsequent kinds are not evaluated. This makes the
    /// decision tree deterministic and its rejection reason unambiguous.
    pub required:      Vec<EvidenceKind>,
    /// Evidence kinds whose absence is a warning, not a rejection.
    pub recommended:   Vec<EvidenceKind>,
    /// Evidence collected informational-only. Never blocks promotion.
    pub informational: Vec<EvidenceKind>,
}

impl PromotionPolicy {
    pub fn gate_for(&self, kind: &EvidenceKind) -> Gate {
        if self.required.contains(kind)      { return Gate::Required; }
        if self.recommended.contains(kind)   { return Gate::Recommended; }
        Gate::Informational
    }

    /// The canonical Firebird policy instance.
    /// Matches instrument/policy/promotion_policy.json exactly.
    pub fn firebird_v1() -> Self {
        Self {
            policy_id:     "firebird-promotion-v1".to_string(),
            spec:          "ntt-contract-v1".to_string(),
            // This hash must match the committed promotion_policy.json.
            // If policy_hash mismatches at evaluation time, the transition
            // is rejected before any evidence is examined.
            policy_hash:   "sha256:403af6631e0c10cfe324cc43b0021ec09f7e15edad683bc5bed9d9a7e6fbe42d".to_string(),
            required:      vec![
                EvidenceKind::Identity,
                EvidenceKind::Determinism,
                EvidenceKind::Robustness,
            ],
            recommended:   vec![EvidenceKind::Independence],
            informational: vec![EvidenceKind::Performance],
        }
    }
}
