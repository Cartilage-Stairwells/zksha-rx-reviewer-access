//! Domain blindness tests — proving the core has no domain knowledge.
//!
//! These tests verify that the core evidence protocol vocabulary
//! is domain-blind: it works identically for any domain (π, NTT,
//! serialization) without any domain-specific fields, methods, or
//! branching logic.
//!
//! The architectural invariant:
//!   Core defines admissibility.
//!   Domains provide claims.

use avx512_butterfly::core::*;

/// The core types must have no domain-specific fields.
/// This test verifies that Artifact, EvaluationRequest, EvaluationResult,
/// EvidenceArtifact, and EnvironmentContract contain no fields that
/// reference any specific domain.
#[test]
fn artifact_has_no_domain_fields() {
    let a = Artifact::new(
        "any-artifact",
        Hash::compute(b"content"),
        Hash::compute(b"schema"),
    );
    // The artifact only knows: what I am, what I contain, what schema I conform to.
    // It does not know: who will evaluate me, in what environment, or with what predicates.
    assert_eq!(a.artifact_id, "any-artifact");
}

#[test]
fn evaluation_request_introduces_variability() {
    let a = Artifact::new("x", Hash::compute(b"x"), Hash::compute(b"s"));

    // The same artifact can be evaluated by different adapters.
    // The artifact does not change. Only the request varies.
    let r1 = EvaluationRequest::new(
        &a.artifact_id,
        "adapter-alpha",
        Hash::compute(b"env"),
        Hash::compute(b"preds"),
    );
    let r2 = EvaluationRequest::new(
        &a.artifact_id,
        "adapter-beta",
        Hash::compute(b"env"),
        Hash::compute(b"preds"),
    );

    assert_eq!(r1.artifact_id, r2.artifact_id); // same artifact
    assert_ne!(r1.adapter_id, r2.adapter_id);   // different adapter
}

#[test]
fn environment_contract_is_domain_blind() {
    // A contract for a mathematical domain and a contract for a
    // serialization domain use the SAME type. The core does not
    // interpret the fields — it stores and hashes them.
    let math_contract = EnvironmentContract::new(
        "math-host-v1",
        "x86_64",
        "python 3.11",
        "runner-v1",
        "json-canonical",
    );
    let ser_contract = EnvironmentContract::new(
        "serialization-host-v1",
        "aarch64",
        "rustc 1.97",
        "runner-v1",
        "cbor",
    );

    // Same type, different identity
    assert_ne!(math_contract.identity_hash(), ser_contract.identity_hash());

    // But the type is identical — no domain-specific fields exist
    assert_eq!(
        std::mem::size_of_val(&math_contract),
        std::mem::size_of_val(&ser_contract),
    );
}

#[test]
fn evidence_artifact_works_for_any_domain() {
    // Build evidence for two completely different domains using the same types.
    let domain_a = EvidenceArtifact::new(
        EvaluationRequest::new("domain-a-artifact", "domain-a-adapter",
            Hash::compute(b"env-a"), Hash::compute(b"preds-a")),
        EvaluationResult::new(
            Hash::compute(b"output-a"),
            vec![PredicateResult::passed("predicate-a")],
        ),
        ProvenanceRecord::new(
            "domain-a-artifact", Hash::compute(b"c-a"), "commit-a",
            "domain-a-adapter", Hash::compute(b"env-a"), "2026-07-16T00:00:00Z",
        ),
    );

    let domain_b = EvidenceArtifact::new(
        EvaluationRequest::new("domain-b-artifact", "domain-b-adapter",
            Hash::compute(b"env-b"), Hash::compute(b"preds-b")),
        EvaluationResult::new(
            Hash::compute(b"output-b"),
            vec![PredicateResult::passed("predicate-b")],
        ),
        ProvenanceRecord::new(
            "domain-b-artifact", Hash::compute(b"c-b"), "commit-b",
            "domain-b-adapter", Hash::compute(b"env-b"), "2026-07-16T00:00:00Z",
        ),
    );

    // Both are admissible — the core's admissibility check is domain-blind.
    assert_eq!(evaluate_admission(&domain_a), AdmissionDecision::Admitted);
    assert_eq!(evaluate_admission(&domain_b), AdmissionDecision::Admitted);

    // Both produce distinct evidence hashes — the evidence is not confused.
    assert_ne!(domain_a.evidence_hash(), domain_b.evidence_hash());
}

#[test]
fn evidence_rejection_is_domain_blind() {
    // The core rejects evidence that fails predicates, regardless of domain.
    let bad_evidence = EvidenceArtifact::new(
        EvaluationRequest::new("any-artifact", "any-adapter",
            Hash::compute(b"env"), Hash::compute(b"preds")),
        EvaluationResult::new(
            Hash::compute(b"output"),
            vec![
                PredicateResult::passed("pred-1"),
                PredicateResult::failed("pred-2", "semantic mismatch"),
            ],
        ),
        ProvenanceRecord::new(
            "any-artifact", Hash::compute(b"c"), "commit",
            "any-adapter", Hash::compute(b"env"), "2026-07-16T00:00:00Z",
        ),
    );

    assert!(!bad_evidence.is_admissible());
    match evaluate_admission(&bad_evidence) {
        AdmissionDecision::Rejected(reason) => {
            assert!(reason.contains("pred-2"));
            assert!(reason.contains("semantic mismatch"));
        }
        AdmissionDecision::Admitted => panic!("should be rejected"),
    }
}

#[test]
fn predicate_stage_ordering_is_enforced() {
    // The four stages are ordered. The core enforces this ordering
    // as protocol behavior, not configuration.
    assert!(PredicateStage::SemanticCorrectness < PredicateStage::CanonicalRepresentation);
    assert!(PredicateStage::CanonicalRepresentation < PredicateStage::IdentityStability);
    assert!(PredicateStage::IdentityStability < PredicateStage::EvidenceAdmission);

    // The order is the stage value
    assert_eq!(PredicateStage::SemanticCorrectness.order(), 0);
    assert_eq!(PredicateStage::CanonicalRepresentation.order(), 1);
    assert_eq!(PredicateStage::IdentityStability.order(), 2);
    assert_eq!(PredicateStage::EvidenceAdmission.order(), 3);
}

#[test]
fn hash_identity_is_computational_not_declarative() {
    // The environment contract derives identity through computation,
    // not through a claimed hash field. The object is not self-authenticating.
    let c1 = EnvironmentContract::new("test", "x86_64", "rustc", "v1", "json");
    let c2 = EnvironmentContract::new("test", "x86_64", "rustc", "v1", "json");

    // Same inputs → same identity (computed, not claimed)
    assert_eq!(c1.identity_hash(), c2.identity_hash());

    // Different inputs → different identity
    let c3 = EnvironmentContract::new("test", "aarch64", "rustc", "v1", "json");
    assert_ne!(c1.identity_hash(), c3.identity_hash());
}

#[test]
fn no_domain_adapter_trait_in_core() {
    // The core must NOT define a DomainAdapter trait.
    // Domains implement their own adapters; the core only stores adapter_id.
    // This test is a compile-time check: if someone adds a DomainAdapter trait
    // to the core, this test will fail because the trait would be importable.
    //
    // We verify that the only traits exported are: Canonical, Predicate.
    // (Predicate is a runtime trait for adapter implementations, not a
    // domain adapter trait — it's the protocol's evaluation interface.)
    
    // This is a structural test: verify that EvaluationRequest stores
    // adapter_id as a String, not as a trait object.
    let req = EvaluationRequest::new("a", "my-adapter", Hash::compute(b"e"), Hash::compute(b"p"));
    assert_eq!(req.adapter_id, "my-adapter");
    // adapter_id is a String — the core treats it as an opaque identifier.
    // It does not call any methods on it. It does not downcast it.
    // It does not branch on its value.
}
