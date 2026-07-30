//! Artifact and Evaluation types — the immutability boundary.
//!
//! The critical separation:
//!
//!   Artifact → immutable → stable across environments, compilers, adapters
//!   Evaluation → variable → changes with each execution
//!   Evidence → variable → the record of what happened
//!
//! New machine → same artifact
//! New compiler → same artifact
//! New adapter → same artifact
//! New evaluation time → same artifact
//!
//! Only the evidence record changes.

use super::hash::Hash;

/// Minimal immutable artifact identity.
///
/// No evaluator information belongs here. The artifact does not know
/// who will evaluate it, in what environment, or with which predicates.
/// It only knows: what I am (artifact_id), what I contain (content_hash),
/// and what schema I conform to (schema_hash).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub artifact_id: String,
    pub content_hash: Hash,
    pub schema_hash: Hash,
}

impl Artifact {
    pub fn new(artifact_id: &str, content_hash: Hash, schema_hash: Hash) -> Self {
        Artifact {
            artifact_id: artifact_id.to_string(),
            content_hash,
            schema_hash,
        }
    }
}

/// The evaluation request. This is where variability enters.
///
/// An EvaluationRequest authorizes a computation: "evaluate this artifact
/// using this adapter under this environment contract against this
/// predicate set."
///
/// R = F(A, P, V, E)
///   A = artifact identity + content hash
///   P = predicate definitions + predicate-set hash
///   V = evaluator/adapter identity + version
///   E = environment contract
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationRequest {
    pub artifact_id: String,
    pub adapter_id: String,
    pub environment_contract_hash: Hash,
    pub predicate_set_hash: Hash,
}

impl EvaluationRequest {
    pub fn new(
        artifact_id: &str,
        adapter_id: &str,
        environment_contract_hash: Hash,
        predicate_set_hash: Hash,
    ) -> Self {
        EvaluationRequest {
            artifact_id: artifact_id.to_string(),
            adapter_id: adapter_id.to_string(),
            environment_contract_hash,
            predicate_set_hash,
        }
    }
}

/// The evaluation result. The output of the computation.
///
/// Contains: output digest, predicate results, optional measurements.
/// The result hash proves the existence of a result. The trace (in
/// evidence.rs) proves the relationship between inputs and result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationResult {
    pub output_digest: Hash,
    pub predicate_results: Vec<super::predicate::PredicateResult>,
    pub all_passed: bool,
}

impl EvaluationResult {
    pub fn new(
        output_digest: Hash,
        predicate_results: Vec<super::predicate::PredicateResult>,
    ) -> Self {
        let all_passed = predicate_results.iter().all(|p| p.status.is_pass());
        EvaluationResult {
            output_digest,
            predicate_results,
            all_passed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::predicate::{PredicateResult, PredicateStatus};

    #[test]
    fn artifact_has_no_evaluator_info() {
        let a = Artifact::new(
            "test-artifact",
            Hash::compute(b"content"),
            Hash::compute(b"schema"),
        );
        // Artifact should only have: artifact_id, content_hash, schema_hash
        // No adapter_id, no environment, no evaluator
        assert_eq!(a.artifact_id, "test-artifact");
        assert!(*a.content_hash.as_bytes() != [0u8; 32]);
        assert!(*a.schema_hash.as_bytes() != [0u8; 32]);
    }

    #[test]
    fn same_artifact_different_evaluations() {
        // The same artifact can be evaluated by different adapters.
        // The artifact does not change.
        let artifact = Artifact::new(
            "pi-1488",
            Hash::compute(b"pi digits"),
            Hash::compute(b"pi schema"),
        );

        let req1 = EvaluationRequest::new(
            &artifact.artifact_id,
            "agm_adapter",
            Hash::compute(b"env1"),
            Hash::compute(b"preds"),
        );

        let req2 = EvaluationRequest::new(
            &artifact.artifact_id,
            "machin_adapter",
            Hash::compute(b"env1"),
            Hash::compute(b"preds"),
        );

        // Same artifact_id, different adapter_id
        assert_eq!(req1.artifact_id, req2.artifact_id);
        assert_ne!(req1.adapter_id, req2.adapter_id);
    }

    #[test]
    fn evaluation_result_all_passed() {
        let results = vec![
            PredicateResult::new("pred-1", PredicateStatus::Pass),
            PredicateResult::new("pred-2", PredicateStatus::Pass),
        ];
        let er = EvaluationResult::new(Hash::compute(b"output"), results);
        assert!(er.all_passed);
    }

    #[test]
    fn evaluation_result_not_all_passed() {
        let results = vec![
            PredicateResult::new("pred-1", PredicateStatus::Pass),
            PredicateResult::new("pred-2", PredicateStatus::Fail("mismatch at position 42".to_string())),
        ];
        let er = EvaluationResult::new(Hash::compute(b"output"), results);
        assert!(!er.all_passed);
    }
}
