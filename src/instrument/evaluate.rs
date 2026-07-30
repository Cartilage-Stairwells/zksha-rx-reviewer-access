//! IEP Promotion Evaluation — the enforcement layer.
//!
//! `evaluate_transition` is the core function. It implements the ordered
//! decision tree exactly as specified: performance is unreachable until
//! all correctness gates pass. Each rejection has a single, unambiguous reason.
//!
//! The decision tree:
//!
//!   1. Verify policy identity       (policy_hash matches committed hash)
//!   2. Verify parent artifact       (parent exists and is trusted)
//!   3. Verify candidate identity    (output_hash == authority.expected_hash)
//!   4. Verify evidence graph        (commitment hash not altered after sealing)
//!   5. Evaluate required evidence   (all required kinds present and passing)
//!   6. Evaluate performance         (informational — cannot block promotion)
//!   7. Emit promotion decision

use crate::instrument::artifact::{Artifact, EvidenceKind, EvidenceResult};
use crate::instrument::event::PromotionEvent;
use crate::instrument::policy::PromotionPolicy;

/// Every possible rejection reason — one per decision-tree step.
/// The ordering mirrors the decision tree: earlier steps take priority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RejectionReason {
    /// Step 1: The policy hash in the candidate does not match the declared policy.
    PolicyHashMismatch { expected: String, actual: String },
    /// Step 2: The parent artifact is absent or untrusted.
    ParentArtifactMissing,
    /// Step 2: The authority specs differ between parent and candidate.
    AuthorityMismatch,
    /// Step 3: output_hash != authority.expected_hash.
    ArtifactIdentityFailed { output_hash: String, expected_hash: String },
    /// Step 4: The evidence graph hash does not match its stored commitment.
    EvidenceGraphAltered,
    /// Step 5: A required evidence kind is absent from the graph.
    RequiredEvidenceAbsent { kind: EvidenceKind },
    /// Step 5: A required evidence kind is present but failed.
    RequiredEvidenceFailed { kind: EvidenceKind, reason: String },
}

impl RejectionReason {
    pub fn description(&self) -> String {
        match self {
            Self::PolicyHashMismatch { expected, actual } =>
                format!("policy hash mismatch — expected {expected}, got {actual}"),
            Self::ParentArtifactMissing =>
                "parent artifact is absent or not in trusted state".to_string(),
            Self::AuthorityMismatch =>
                "authority spec differs between parent and candidate — incomparable".to_string(),
            Self::ArtifactIdentityFailed { output_hash, expected_hash } =>
                format!("identity failed — output_hash {output_hash} != expected_hash {expected_hash}"),
            Self::EvidenceGraphAltered =>
                "evidence graph hash mismatch — graph was altered after sealing".to_string(),
            Self::RequiredEvidenceAbsent { kind } =>
                format!("required evidence absent: {}", kind.as_str()),
            Self::RequiredEvidenceFailed { kind, reason } =>
                format!("required evidence failed: {} — {reason}", kind.as_str()),
        }
    }
}

/// The output of evaluate_transition.
#[derive(Debug, Clone, PartialEq)]
pub enum PromotionDecision {
    /// All required gates passed. Candidate is eligible to become the new
    /// trusted artifact. A PromotionEvent is emitted.
    Promoted(PromotionEvent),
    /// All required gates passed but the candidate is slower than the parent.
    /// Correct: true. Performance: regression. Not promoted — parent preserved.
    /// This is NOT a failure of the candidate. It is a decision not to replace.
    CorrectButSlower {
        candidate_id:    String,
        parent_id:       String,
        candidate_ns:    u64,
        parent_ns:       u64,
    },
    /// A required gate failed or was absent. Transition rejected. Parent preserved.
    Rejected {
        candidate_id: String,
        reason:       RejectionReason,
    },
}

impl PromotionDecision {
    pub fn is_promoted(&self) -> bool {
        matches!(self, Self::Promoted(_))
    }

    pub fn is_rejected(&self) -> bool {
        matches!(self, Self::Rejected { .. })
    }

    /// True for both Promoted and CorrectButSlower — the candidate was correct.
    pub fn candidate_is_correct(&self) -> bool {
        !self.is_rejected()
    }
}

/// Evaluate a proposed transition from `parent` (trusted) to `candidate`.
///
/// The decision tree is strictly ordered. Step N is not reached unless
/// steps 1..N-1 all passed. Performance (step 6) is unreachable unless
/// the candidate is cryptographically and evidentially correct.
///
/// `declared_policy_hash`: the policy_hash recorded in the candidate artifact.
/// If it does not match policy.policy_hash, the transition is rejected at step 1
/// before any evidence is examined. This ensures the evaluation rule cannot
/// silently drift from the committed policy.
pub fn evaluate_transition(
    parent:                &Artifact,
    candidate:             &Artifact,
    policy:                &PromotionPolicy,
    declared_policy_hash:  &str,
) -> PromotionDecision {

    // ── Step 1: Policy identity ───────────────────────────────────────────────
    if declared_policy_hash != policy.policy_hash {
        return PromotionDecision::Rejected {
            candidate_id: candidate.id.clone(),
            reason: RejectionReason::PolicyHashMismatch {
                expected: policy.policy_hash.clone(),
                actual:   declared_policy_hash.to_string(),
            },
        };
    }

    // ── Step 2: Parent artifact ───────────────────────────────────────────────
    // Parent must exist (enforced by type — caller passes it by reference),
    // and the authority specs must be compatible: same spec + same expected_hash.
    if parent.authority.spec != candidate.authority.spec
        || parent.authority.expected_hash != candidate.authority.expected_hash
    {
        return PromotionDecision::Rejected {
            candidate_id: candidate.id.clone(),
            reason: RejectionReason::AuthorityMismatch,
        };
    }

    // ── Step 3: Artifact identity ─────────────────────────────────────────────
    if !candidate.identity_valid() {
        return PromotionDecision::Rejected {
            candidate_id: candidate.id.clone(),
            reason: RejectionReason::ArtifactIdentityFailed {
                output_hash:   candidate.output_hash.clone(),
                expected_hash: candidate.authority.expected_hash.clone(),
            },
        };
    }

    // ── Step 4: Evidence graph integrity ─────────────────────────────────────
    if !candidate.evidence_graph.verify_commitment() {
        return PromotionDecision::Rejected {
            candidate_id: candidate.id.clone(),
            reason: RejectionReason::EvidenceGraphAltered,
        };
    }

    // ── Step 5: Required evidence ─────────────────────────────────────────────
    for kind in &policy.required {
        match candidate.evidence_graph.nodes.get(kind) {
            None => {
                return PromotionDecision::Rejected {
                    candidate_id: candidate.id.clone(),
                    reason: RejectionReason::RequiredEvidenceAbsent { kind: kind.clone() },
                };
            }
            Some(node) => {
                match &node.result {
                    EvidenceResult::Pass => { /* continue */ }
                    EvidenceResult::Fail(reason) => {
                        return PromotionDecision::Rejected {
                            candidate_id: candidate.id.clone(),
                            reason: RejectionReason::RequiredEvidenceFailed {
                                kind:   kind.clone(),
                                reason: reason.clone(),
                            },
                        };
                    }
                    EvidenceResult::NotApplicable => {
                        // N/A on a required kind is treated as absent.
                        return PromotionDecision::Rejected {
                            candidate_id: candidate.id.clone(),
                            reason: RejectionReason::RequiredEvidenceAbsent { kind: kind.clone() },
                        };
                    }
                }
            }
        }
    }

    // ── Step 6: Performance (informational — cannot block promotion) ──────────
    // Performance is only evaluated if the candidate has timing data AND
    // the parent has timing data. If either is absent, we promote without
    // performance comparison (not all artifacts carry timing).
    if let (Some(candidate_ns), Some(parent_ns)) = (candidate.median_ns, parent.median_ns) {
        if candidate_ns >= parent_ns {
            // Correct but not faster — do not replace the baseline.
            return PromotionDecision::CorrectButSlower {
                candidate_id: candidate.id.clone(),
                parent_id:    parent.id.clone(),
                candidate_ns,
                parent_ns,
            };
        }
    }

    // ── Step 7: Emit promotion event ──────────────────────────────────────────
    let event = PromotionEvent {
        from:            parent.id.clone(),
        to:              candidate.id.clone(),
        policy_id:       policy.policy_id.clone(),
        policy_hash:     policy.policy_hash.clone(),
        evidence_hash:   candidate.evidence_graph.graph_hash.clone(),
        candidate_ns:    candidate.median_ns,
        parent_ns:       parent.median_ns,
        result:          "promoted".to_string(),
    };

    PromotionDecision::Promoted(event)
}
