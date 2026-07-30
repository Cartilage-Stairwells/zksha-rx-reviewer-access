//! Predicate model — machine-verifiable claims.
//!
//! Predicates are protocol objects, not anonymous functions.
//!
//! The evidence artifact stores only the descriptor (id, version, order) —
//! not the runtime state. The evidence graph contains predicate identity,
//! not predicate execution.
//!
//! Predicate execution order (protocol behavior, not configurable):
//!   1. SemanticCorrectness  — does the computation mean what it claims?
//!   2. CanonicalRepresentation — is the output in canonical form?
//!   3. IdentityStability     — does the same input produce the same hash?
//!   4. EvidenceAdmission    — does the evidence satisfy the contract?
//!
//! A failure at stage N terminates the path. This prevents:
//!   hash equality → assumed correctness
//!
//! The required chain is:
//!   meaning → representation → identity → admission

/// The status of a predicate evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredicateStatus {
    Pass,
    Fail(String),
    Error(String),
}

impl PredicateStatus {
    pub fn is_pass(&self) -> bool {
        matches!(self, PredicateStatus::Pass)
    }

    pub fn is_fail(&self) -> bool {
        matches!(self, PredicateStatus::Fail(_))
    }
}

/// The result of evaluating a single predicate.
/// Stored in the evaluation result. Contains identity, not runtime state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PredicateResult {
    pub predicate_id: String,
    pub status: PredicateStatus,
}

impl PredicateResult {
    pub fn new(predicate_id: &str, status: PredicateStatus) -> Self {
        PredicateResult {
            predicate_id: predicate_id.to_string(),
            status,
        }
    }

    pub fn passed(predicate_id: &str) -> Self {
        Self::new(predicate_id, PredicateStatus::Pass)
    }

    pub fn failed(predicate_id: &str, reason: &str) -> Self {
        Self::new(predicate_id, PredicateStatus::Fail(reason.to_string()))
    }
}

/// A descriptor for a predicate in an evidence graph.
/// This is what gets stored — identity, not runtime state.
///
/// The `order` field encodes the execution stage:
///   0 = SemanticCorrectness
///   1 = CanonicalRepresentation
///   2 = IdentityStability
///   3 = EvidenceAdmission
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PredicateDescriptor {
    pub id: String,
    pub version: String,
    pub order: u32,
}

impl PredicateDescriptor {
    pub fn new(id: &str, version: &str, order: u32) -> Self {
        PredicateDescriptor {
            id: id.to_string(),
            version: version.to_string(),
            order,
        }
    }
}

/// The four execution stages. Ordering is protocol behavior.
/// A failure at stage N terminates the evaluation path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PredicateStage {
    /// Stage 0: Does the computation mean what it claims?
    /// This is the semantic gate — it prevents hash equality
    /// from being mistaken for correctness.
    SemanticCorrectness = 0,

    /// Stage 1: Is the output in canonical form?
    /// Ensures there is exactly one valid representation per state.
    CanonicalRepresentation = 1,

    /// Stage 2: Does the same input produce the same hash?
    /// Tests hash stability through serialization cycles.
    IdentityStability = 2,

    /// Stage 3: Does the evidence satisfy the contract?
    /// The admission gate — the policy layer's question.
    EvidenceAdmission = 3,
}

impl PredicateStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SemanticCorrectness => "semantic_correctness",
            Self::CanonicalRepresentation => "canonical_representation",
            Self::IdentityStability => "identity_stability",
            Self::EvidenceAdmission => "evidence_admission",
        }
    }

    pub fn order(&self) -> u32 {
        *self as u32
    }
}

/// A predicate set — an ordered collection of predicate descriptors.
/// The set hash covers all member descriptors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PredicateSet {
    pub set_id: String,
    pub descriptors: Vec<PredicateDescriptor>,
}

impl PredicateSet {
    pub fn new(set_id: &str, descriptors: Vec<PredicateDescriptor>) -> Self {
        PredicateSet {
            set_id: set_id.to_string(),
            descriptors,
        }
    }

    /// Compute the set hash from the canonical encoding of descriptors.
    pub fn set_hash(&self) -> super::hash::Hash {
        let mut data = Vec::new();
        for d in &self.descriptors {
            data.extend_from_slice(d.id.as_bytes());
            data.push(b'\0');
            data.extend_from_slice(d.version.as_bytes());
            data.push(b'\0');
            data.extend_from_slice(&d.order.to_le_bytes());
            data.push(b'\0');
        }
        super::hash::Hash::compute(&data)
    }
}

/// The runtime predicate trait. Adapters implement this to provide
/// the actual evaluation logic. The core stores only descriptors;
/// the runtime state lives in the adapter.
///
/// The evaluate() method receives an EvaluationContext and returns
/// a PredicateResult. The context provides access to the artifact
/// output, environment contract, and other evaluation inputs.
pub trait Predicate {
    /// The stable identifier for this predicate.
    fn id(&self) -> &'static str;

    /// The version of this predicate's definition.
    fn version(&self) -> &'static str;

    /// The execution stage this predicate belongs to.
    fn stage(&self) -> PredicateStage;

    /// Evaluate the predicate against the given context.
    fn evaluate(&self, context: &EvaluationContext) -> PredicateResult;

    /// Produce the descriptor for this predicate (identity, not state).
    fn descriptor(&self) -> PredicateDescriptor {
        PredicateDescriptor::new(self.id(), self.version(), self.stage().order())
    }
}

/// The context passed to a predicate during evaluation.
/// Contains the inputs needed to evaluate the claim.
///
/// This is deliberately minimal — the core does not assume what
/// predicates need. Adapters extend this through their own context types.
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// The output digest of the computation being evaluated.
    pub output_digest: super::hash::Hash,
    /// Optional reference output for comparison predicates.
    pub reference_digest: Option<super::hash::Hash>,
}

impl EvaluationContext {
    pub fn new(output_digest: super::hash::Hash) -> Self {
        EvaluationContext {
            output_digest,
            reference_digest: None,
        }
    }

    pub fn with_reference(mut self, reference_digest: super::hash::Hash) -> Self {
        self.reference_digest = Some(reference_digest);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_ordering_is_protocol() {
        assert!(PredicateStage::SemanticCorrectness < PredicateStage::CanonicalRepresentation);
        assert!(PredicateStage::CanonicalRepresentation < PredicateStage::IdentityStability);
        assert!(PredicateStage::IdentityStability < PredicateStage::EvidenceAdmission);
    }

    #[test]
    fn descriptor_is_identity_not_state() {
        let d = PredicateDescriptor::new("pi-match-v1", "1", 0);
        assert_eq!(d.id, "pi-match-v1");
        assert_eq!(d.version, "1");
        assert_eq!(d.order, 0);
        // No field stores evaluation result, input, or runtime state
    }

    #[test]
    fn predicate_set_hash_is_deterministic() {
        let d1 = PredicateDescriptor::new("p1", "1", 0);
        let d2 = PredicateDescriptor::new("p2", "1", 1);
        let s1 = PredicateSet::new("set-v1", vec![d1.clone(), d2.clone()]);
        let s2 = PredicateSet::new("set-v1", vec![d1, d2]);
        assert_eq!(s1.set_hash(), s2.set_hash());
    }

    #[test]
    fn predicate_set_hash_changes_with_order() {
        let d1 = PredicateDescriptor::new("p1", "1", 0);
        let d2 = PredicateDescriptor::new("p2", "1", 1);
        let s1 = PredicateSet::new("set-v1", vec![d1.clone(), d2.clone()]);
        let s2 = PredicateSet::new("set-v1", vec![d2, d1]);
        assert_ne!(s1.set_hash(), s2.set_hash());
    }

    #[test]
    fn fail_status_carries_reason() {
        let s = PredicateStatus::Fail("mismatch at position 42".to_string());
        assert!(s.is_fail());
        assert!(!s.is_pass());
    }
}
