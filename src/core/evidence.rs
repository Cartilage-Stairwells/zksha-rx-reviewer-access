//! EvidenceArtifact — the variable object that binds the evaluation.
//!
//! The evidence record binds:
//!   request  → what was asked (EvaluationRequest)
//!   result   → what was produced (EvaluationResult)
//!   provenance → what happened (ProvenanceRecord)
//!
//! The evidence artifact is the variable object.
//! The artifact itself (core::artifact::Artifact) remains stable.
//!
//! Evidence accumulates. It is never rewritten. A later object can
//! reference, summarize, classify, or decide from earlier objects,
//! but cannot replace them.

use super::hash::Hash;
use super::artifact::{EvaluationRequest, EvaluationResult};
use super::canonical::Canonical;

/// Provenance record — what happened during execution.
///
/// This is the trace: sufficient immutable references to reconstruct
/// the relationship between inputs, execution, and result.
/// A result hash proves the existence of a result. The provenance
/// record proves the relationship.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvenanceRecord {
    /// The artifact that was evaluated.
    pub artifact_id: String,
    /// The content hash of the artifact at evaluation time.
    pub artifact_content_hash: Hash,
    /// The source commit of the artifact.
    pub source_commit: String,
    /// The adapter that performed the evaluation.
    pub adapter_id: String,
    /// The adapter binary hash (if available).
    pub adapter_hash: Option<Hash>,
    /// The environment contract hash under which the evaluation ran.
    pub environment_contract_hash: Hash,
    /// ISO 8601 timestamp of execution.
    pub executed_at: String,
}

impl ProvenanceRecord {
    pub fn new(
        artifact_id: &str,
        artifact_content_hash: Hash,
        source_commit: &str,
        adapter_id: &str,
        environment_contract_hash: Hash,
        executed_at: &str,
    ) -> Self {
        ProvenanceRecord {
            artifact_id: artifact_id.to_string(),
            artifact_content_hash,
            source_commit: source_commit.to_string(),
            adapter_id: adapter_id.to_string(),
            adapter_hash: None,
            environment_contract_hash,
            executed_at: executed_at.to_string(),
        }
    }

    pub fn with_adapter_hash(mut self, hash: Hash) -> Self {
        self.adapter_hash = Some(hash);
        self
    }
}

/// The evidence artifact — the frozen evaluation record.
///
/// Binds: what was asked (request), what was produced (result),
/// and what happened (provenance). The evidence artifact is immutable
/// once created. Evidence accumulates; it is never rewritten.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceArtifact {
    pub request: EvaluationRequest,
    pub result: EvaluationResult,
    pub provenance: ProvenanceRecord,
}

impl EvidenceArtifact {
    pub fn new(
        request: EvaluationRequest,
        result: EvaluationResult,
        provenance: ProvenanceRecord,
    ) -> Self {
        EvidenceArtifact {
            request,
            result,
            provenance,
        }
    }

    /// Is this evidence admissible? All predicates must have passed.
    /// This is the gate the policy layer checks.
    pub fn is_admissible(&self) -> bool {
        self.result.all_passed
    }

    /// Compute the evidence hash — the custody commitment.
    /// This hash covers the request, result, and provenance.
    pub fn evidence_hash(&self) -> Hash {
        self.canonical_hash()
    }
}

impl Canonical for EvidenceArtifact {
    fn canonical_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        // Request
        data.extend_from_slice(self.request.artifact_id.as_bytes());
        data.push(b'\0');
        data.extend_from_slice(self.request.adapter_id.as_bytes());
        data.push(b'\0');
        data.extend_from_slice(self.request.environment_contract_hash.as_bytes());
        data.push(b'\0');
        data.extend_from_slice(self.request.predicate_set_hash.as_bytes());
        data.push(b'\0');
        // Result
        data.extend_from_slice(self.result.output_digest.as_bytes());
        data.push(b'\0');
        for pr in &self.result.predicate_results {
            data.extend_from_slice(pr.predicate_id.as_bytes());
            data.push(b'\0');
            match &pr.status {
                super::predicate::PredicateStatus::Pass => {
                    data.push(b'P');
                }
                super::predicate::PredicateStatus::Fail(r) => {
                    data.push(b'F');
                    data.extend_from_slice(r.as_bytes());
                }
                super::predicate::PredicateStatus::Error(e) => {
                    data.push(b'E');
                    data.extend_from_slice(e.as_bytes());
                }
            }
            data.push(b'\0');
        }
        // Provenance
        data.extend_from_slice(self.provenance.artifact_id.as_bytes());
        data.push(b'\0');
        data.extend_from_slice(self.provenance.artifact_content_hash.as_bytes());
        data.push(b'\0');
        data.extend_from_slice(self.provenance.source_commit.as_bytes());
        data.push(b'\0');
        data.extend_from_slice(self.provenance.adapter_id.as_bytes());
        data.push(b'\0');
        data.extend_from_slice(self.provenance.environment_contract_hash.as_bytes());
        data.push(b'\0');
        data.extend_from_slice(self.provenance.executed_at.as_bytes());
        data.push(b'\0');
        data
    }
}

/// The admission decision for an evidence artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionDecision {
    /// All predicates passed. Evidence is admissible.
    Admitted,
    /// One or more predicates failed. Evidence is not admissible.
    /// Contains the first failing predicate's reason.
    Rejected(String),
}

/// Evaluate admissibility of an evidence artifact.
/// The core defines admissibility. Domains provide claims.
pub fn evaluate_admission(evidence: &EvidenceArtifact) -> AdmissionDecision {
    if evidence.result.all_passed {
        AdmissionDecision::Admitted
    } else {
        // Find the first failing predicate
        let first_fail = evidence.result.predicate_results
            .iter()
            .find(|p| !p.status.is_pass())
            .map(|p| match &p.status {
                super::predicate::PredicateStatus::Fail(r) => format!("{}: {}", p.predicate_id, r),
                super::predicate::PredicateStatus::Error(e) => format!("{}: ERROR: {}", p.predicate_id, e),
                super::predicate::PredicateStatus::Pass => unreachable!(),
            })
            .unwrap_or_default();
        AdmissionDecision::Rejected(first_fail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::artifact::{EvaluationRequest, EvaluationResult};
    use super::super::predicate::{PredicateResult, PredicateStatus};

    fn make_request() -> EvaluationRequest {
        EvaluationRequest::new(
            "test-artifact",
            "test-adapter",
            Hash::compute(b"env"),
            Hash::compute(b"preds"),
        )
    }

    fn make_provenance() -> ProvenanceRecord {
        ProvenanceRecord::new(
            "test-artifact",
            Hash::compute(b"content"),
            "abc123",
            "test-adapter",
            Hash::compute(b"env"),
            "2026-07-16T00:00:00Z",
        )
    }

    #[test]
    fn admissible_evidence_all_passed() {
        let request = make_request();
        let result = EvaluationResult::new(
            Hash::compute(b"output"),
            vec![
                PredicateResult::passed("p1"),
                PredicateResult::passed("p2"),
            ],
        );
        let provenance = make_provenance();
        let evidence = EvidenceArtifact::new(request, result, provenance);

        assert!(evidence.is_admissible());
        assert_eq!(evaluate_admission(&evidence), AdmissionDecision::Admitted);
    }

    #[test]
    fn rejected_evidence_failing_predicate() {
        let request = make_request();
        let result = EvaluationResult::new(
            Hash::compute(b"output"),
            vec![
                PredicateResult::passed("p1"),
                PredicateResult::failed("p2", "mismatch"),
            ],
        );
        let provenance = make_provenance();
        let evidence = EvidenceArtifact::new(request, result, provenance);

        assert!(!evidence.is_admissible());
        match evaluate_admission(&evidence) {
            AdmissionDecision::Rejected(reason) => {
                assert!(reason.contains("p2"));
                assert!(reason.contains("mismatch"));
            }
            AdmissionDecision::Admitted => panic!("should be rejected"),
        }
    }

    #[test]
    fn evidence_hash_is_deterministic() {
        let request = make_request();
        let result = EvaluationResult::new(
            Hash::compute(b"output"),
            vec![PredicateResult::passed("p1")],
        );
        let provenance = make_provenance();
        let evidence1 = EvidenceArtifact::new(request, result, provenance);

        let request2 = make_request();
        let result2 = EvaluationResult::new(
            Hash::compute(b"output"),
            vec![PredicateResult::passed("p1")],
        );
        let provenance2 = make_provenance();
        let evidence2 = EvidenceArtifact::new(request2, result2, provenance2);

        assert_eq!(evidence1.evidence_hash(), evidence2.evidence_hash());
    }

    #[test]
    fn evidence_hash_changes_with_result() {
        let request = make_request();

        let result_pass = EvaluationResult::new(
            Hash::compute(b"output"),
            vec![PredicateResult::passed("p1")],
        );
        let provenance = make_provenance();
        let evidence_pass = EvidenceArtifact::new(request.clone(), result_pass, provenance.clone());

        let result_fail = EvaluationResult::new(
            Hash::compute(b"output"),
            vec![PredicateResult::failed("p1", "bad")],
        );
        let evidence_fail = EvidenceArtifact::new(request, result_fail, provenance);

        assert_ne!(evidence_pass.evidence_hash(), evidence_fail.evidence_hash());
    }

    #[test]
    fn evidence_is_domain_blind() {
        // The EvidenceArtifact type does not contain any domain-specific fields.
        // It works identically for pi, NTT, or serialization.
        let pi_evidence = EvidenceArtifact::new(
            EvaluationRequest::new("pi-1488", "agm-adapter", Hash::compute(b"e1"), Hash::compute(b"p1")),
            EvaluationResult::new(Hash::compute(b"pi-output"), vec![PredicateResult::passed("pi-match")]),
            ProvenanceRecord::new("pi-1488", Hash::compute(b"c"), "abc", "agm", Hash::compute(b"e1"), "2026"),
        );

        let ntt_evidence = EvidenceArtifact::new(
            EvaluationRequest::new("ntt-butterfly", "avx512-adapter", Hash::compute(b"e2"), Hash::compute(b"p2")),
            EvaluationResult::new(Hash::compute(b"ntt-output"), vec![PredicateResult::passed("ntt-match")]),
            ProvenanceRecord::new("ntt-butterfly", Hash::compute(b"c2"), "def", "avx512", Hash::compute(b"e2"), "2026"),
        );

        // Same type, same methods, different domains — no domain knowledge needed
        assert!(pi_evidence.is_admissible());
        assert!(ntt_evidence.is_admissible());
        // The types are identical — domain blindness is structural
        assert_eq!(
            std::mem::size_of_val(&pi_evidence),
            std::mem::size_of_val(&ntt_evidence),
        );
    }
}
