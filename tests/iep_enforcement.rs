//! IEP enforcement tests — the full decision matrix.
//!
//! These tests prove the enforcement layer behaves correctly on every
//! case in the specified matrix:
//!
//!   Case                              Expected
//!   ─────────────────────────────     ──────────────────
//!   Policy hash mismatch              Rejected
//!   Parent authority mismatch         Rejected
//!   Artifact identity failed          Rejected
//!   Evidence graph altered            Rejected
//!   Identity evidence absent          Rejected
//!   Determinism evidence absent       Rejected
//!   Robustness evidence absent        Rejected
//!   Required evidence failed          Rejected
//!   Correct artifact, slower          CorrectButSlower (parent preserved)
//!   Correct artifact, faster          Promoted
//!
//! The second-to-last case is the critical one: it proves the system
//! understands that `correctness preserved != improvement achieved`.

use avx512_butterfly::instrument::{
    artifact::{Artifact, Authority, EvidenceGraph, EvidenceKind, EvidenceNode, EvidenceResult},
    evaluate::{evaluate_transition, PromotionDecision, RejectionReason},
    policy::PromotionPolicy,
};
use std::collections::BTreeMap;

// ── Fixture builders ──────────────────────────────────────────────────────────

const POLICY_HASH: &str =
    "sha256:403af6631e0c10cfe324cc43b0021ec09f7e15edad683bc5bed9d9a7e6fbe42d";
const EXPECTED_HASH: &str =
    "sha256:d2a418be1dec267776a7f7392f521dee0d58651e37295656a4c9e82f4b35bddc";

fn test_authority() -> Authority {
    Authority {
        spec:          "ntt-contract-v1".to_string(),
        reference:     "scalar_butterfly".to_string(),
        corpus:        "babybear_vectors_001".to_string(),
        corpus_hash:   "sha256:48da63b99e1c7e0ce2490dd503e1d536850d286136b4b6e7d779a814f152e319".to_string(),
        expected_hash: EXPECTED_HASH.to_string(),
    }
}

/// Build a valid evidence graph with all five kinds populated correctly.
fn passing_graph(median_ns: Option<u64>) -> EvidenceGraph {
    let mut nodes = BTreeMap::new();

    nodes.insert(EvidenceKind::Identity, EvidenceNode {
        kind:      EvidenceKind::Identity,
        result:    EvidenceResult::Pass,
        mechanism: "output_hash matches authority.expected_hash".to_string(),
        median_ns: None,
    });
    nodes.insert(EvidenceKind::Determinism, EvidenceNode {
        kind:      EvidenceKind::Determinism,
        result:    EvidenceResult::Pass,
        mechanism: "pure function — no RNG, no time, no uninitialized memory".to_string(),
        median_ns: None,
    });
    nodes.insert(EvidenceKind::Independence, EvidenceNode {
        kind:      EvidenceKind::Independence,
        result:    EvidenceResult::Pass,
        mechanism: "butterfly_reference agrees on all corpus triples".to_string(),
        median_ns: None,
    });
    nodes.insert(EvidenceKind::Robustness, EvidenceNode {
        kind:      EvidenceKind::Robustness,
        result:    EvidenceResult::Pass,
        mechanism: "boundary corpus: zero, p-1, midpoint, cross-quadrant".to_string(),
        median_ns: None,
    });
    nodes.insert(EvidenceKind::Performance, EvidenceNode {
        kind:      EvidenceKind::Performance,
        result:    EvidenceResult::Pass,
        mechanism: "Criterion --baseline comparison".to_string(),
        median_ns,
    });

    // Compute and seal the commitment hash.
    let graph = EvidenceGraph { nodes, graph_hash: String::new() };
    let hash = graph.compute_hash();
    EvidenceGraph { nodes: graph.nodes, graph_hash: hash }
}

/// Build a valid parent artifact (reference backend, trusted baseline).
fn parent_artifact() -> Artifact {
    let graph = passing_graph(Some(800)); // 800ns reference (intentionally slow)
    Artifact {
        id:             "firebird_reference_c422bfb".to_string(),
        iep_version:    "0.1".to_string(),
        authority:      test_authority(),
        backend:        "reference".to_string(),
        commit:         "c422bfb".to_string(),
        output_hash:    EXPECTED_HASH.to_string(),
        correct:        true,
        evidence_graph: graph,
        median_ns:      Some(800),
    }
}

/// Build a valid candidate artifact (avx512 backend, faster than parent).
fn candidate_artifact_faster() -> Artifact {
    let graph = passing_graph(Some(400)); // 400ns — faster than parent 800ns
    Artifact {
        id:             "firebird_avx512_abc1234".to_string(),
        iep_version:    "0.1".to_string(),
        authority:      test_authority(),
        backend:        "avx512".to_string(),
        commit:         "abc1234".to_string(),
        output_hash:    EXPECTED_HASH.to_string(),
        correct:        true,
        evidence_graph: graph,
        median_ns:      Some(400),
    }
}

fn policy() -> PromotionPolicy {
    PromotionPolicy::firebird_v1()
}

// ── Negative tests ────────────────────────────────────────────────────────────

#[test]
fn reject_policy_hash_mismatch() {
    let parent    = parent_artifact();
    let candidate = candidate_artifact_faster();
    let policy    = policy();

    let result = evaluate_transition(
        &parent, &candidate, &policy,
        "sha256:000000wronghash",
    );

    assert!(result.is_rejected(), "Expected rejection on policy hash mismatch");
    if let PromotionDecision::Rejected { reason, .. } = result {
        assert!(
            matches!(reason, RejectionReason::PolicyHashMismatch { .. }),
            "Expected PolicyHashMismatch, got: {reason:?}"
        );
    }
}

#[test]
fn reject_authority_mismatch() {
    let parent    = parent_artifact();
    let policy    = policy();

    // Candidate claims a different spec — different authority.
    let mut wrong_authority = test_authority();
    wrong_authority.spec = "ntt-contract-v2".to_string();
    let graph = passing_graph(Some(400));
    let candidate = Artifact {
        id:             "firebird_avx512_badauth".to_string(),
        iep_version:    "0.1".to_string(),
        authority:      wrong_authority,
        backend:        "avx512".to_string(),
        commit:         "abc1234".to_string(),
        output_hash:    EXPECTED_HASH.to_string(),
        correct:        true,
        evidence_graph: graph,
        median_ns:      Some(400),
    };

    let result = evaluate_transition(&parent, &candidate, &policy, POLICY_HASH);

    assert!(result.is_rejected());
    assert!(
        matches!(result, PromotionDecision::Rejected {
            reason: RejectionReason::AuthorityMismatch, ..
        }),
        "Expected AuthorityMismatch"
    );
}

#[test]
fn reject_artifact_identity_failed() {
    let parent = parent_artifact();
    let policy = policy();

    // Candidate output_hash is wrong — implementation produced different output.
    let graph = passing_graph(Some(400));
    let candidate = Artifact {
        id:             "firebird_avx512_wrongout".to_string(),
        iep_version:    "0.1".to_string(),
        authority:      test_authority(),
        backend:        "avx512".to_string(),
        commit:         "abc1234".to_string(),
        output_hash:    "sha256:deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string(),
        correct:        false,
        evidence_graph: graph,
        median_ns:      Some(400),
    };

    let result = evaluate_transition(&parent, &candidate, &policy, POLICY_HASH);

    assert!(result.is_rejected());
    assert!(
        matches!(result, PromotionDecision::Rejected {
            reason: RejectionReason::ArtifactIdentityFailed { .. }, ..
        }),
        "Expected ArtifactIdentityFailed"
    );
}

#[test]
fn reject_evidence_graph_altered() {
    let parent = parent_artifact();
    let policy = policy();

    // Build a valid graph then tamper with its commitment hash.
    let mut tampered_graph = passing_graph(Some(400));
    tampered_graph.graph_hash = "sha256:tampered00000000000000000000000000000000000000000000000000000000".to_string();

    let candidate = Artifact {
        id:             "firebird_avx512_tampered".to_string(),
        iep_version:    "0.1".to_string(),
        authority:      test_authority(),
        backend:        "avx512".to_string(),
        commit:         "abc1234".to_string(),
        output_hash:    EXPECTED_HASH.to_string(),
        correct:        true,
        evidence_graph: tampered_graph,
        median_ns:      Some(400),
    };

    let result = evaluate_transition(&parent, &candidate, &policy, POLICY_HASH);

    assert!(result.is_rejected());
    assert!(
        matches!(result, PromotionDecision::Rejected {
            reason: RejectionReason::EvidenceGraphAltered, ..
        }),
        "Expected EvidenceGraphAltered"
    );
}

/// Helper: build a candidate missing a specific required evidence kind.
fn candidate_missing_kind(missing: EvidenceKind) -> Artifact {
    let mut nodes = BTreeMap::new();
    for kind in [
        EvidenceKind::Identity,
        EvidenceKind::Determinism,
        EvidenceKind::Independence,
        EvidenceKind::Robustness,
        EvidenceKind::Performance,
    ] {
        if kind == missing { continue; }
        nodes.insert(kind.clone(), EvidenceNode {
            kind:      kind.clone(),
            result:    EvidenceResult::Pass,
            mechanism: "ok".to_string(),
            median_ns: if kind == EvidenceKind::Performance { Some(400) } else { None },
        });
    }
    let graph = EvidenceGraph { nodes, graph_hash: String::new() };
    let hash  = graph.compute_hash();
    let graph = EvidenceGraph { nodes: graph.nodes, graph_hash: hash };

    Artifact {
        id:             format!("firebird_avx512_missing_{}", missing.as_str()),
        iep_version:    "0.1".to_string(),
        authority:      test_authority(),
        backend:        "avx512".to_string(),
        commit:         "abc1234".to_string(),
        output_hash:    EXPECTED_HASH.to_string(),
        correct:        true,
        evidence_graph: graph,
        median_ns:      Some(400),
    }
}

#[test]
fn reject_identity_evidence_absent() {
    let parent    = parent_artifact();
    let candidate = candidate_missing_kind(EvidenceKind::Identity);
    let result    = evaluate_transition(&parent, &candidate, &policy(), POLICY_HASH);
    assert!(result.is_rejected());
    assert!(matches!(result, PromotionDecision::Rejected {
        reason: RejectionReason::RequiredEvidenceAbsent {
            kind: EvidenceKind::Identity
        }, ..
    }), "Expected RequiredEvidenceAbsent(Identity)");
}

#[test]
fn reject_determinism_evidence_absent() {
    let parent    = parent_artifact();
    let candidate = candidate_missing_kind(EvidenceKind::Determinism);
    let result    = evaluate_transition(&parent, &candidate, &policy(), POLICY_HASH);
    assert!(result.is_rejected());
    assert!(matches!(result, PromotionDecision::Rejected {
        reason: RejectionReason::RequiredEvidenceAbsent {
            kind: EvidenceKind::Determinism
        }, ..
    }), "Expected RequiredEvidenceAbsent(Determinism)");
}

#[test]
fn reject_robustness_evidence_absent() {
    let parent    = parent_artifact();
    let candidate = candidate_missing_kind(EvidenceKind::Robustness);
    let result    = evaluate_transition(&parent, &candidate, &policy(), POLICY_HASH);
    assert!(result.is_rejected());
    assert!(matches!(result, PromotionDecision::Rejected {
        reason: RejectionReason::RequiredEvidenceAbsent {
            kind: EvidenceKind::Robustness
        }, ..
    }), "Expected RequiredEvidenceAbsent(Robustness)");
}

#[test]
fn reject_required_evidence_failed() {
    let parent = parent_artifact();
    let policy = policy();

    // Robustness present but failing — e.g. boundary case diverged.
    let mut nodes = BTreeMap::new();
    nodes.insert(EvidenceKind::Identity, EvidenceNode {
        kind: EvidenceKind::Identity, result: EvidenceResult::Pass,
        mechanism: "ok".to_string(), median_ns: None,
    });
    nodes.insert(EvidenceKind::Determinism, EvidenceNode {
        kind: EvidenceKind::Determinism, result: EvidenceResult::Pass,
        mechanism: "ok".to_string(), median_ns: None,
    });
    nodes.insert(EvidenceKind::Robustness, EvidenceNode {
        kind: EvidenceKind::Robustness,
        result: EvidenceResult::Fail("output mismatch at (p-1, p-1, p-1)".to_string()),
        mechanism: "boundary sweep".to_string(), median_ns: None,
    });

    let graph = EvidenceGraph { nodes, graph_hash: String::new() };
    let hash  = graph.compute_hash();
    let graph = EvidenceGraph { nodes: graph.nodes, graph_hash: hash };

    let candidate = Artifact {
        id: "firebird_avx512_robfail".to_string(),
        iep_version: "0.1".to_string(),
        authority: test_authority(),
        backend: "avx512".to_string(),
        commit: "abc1234".to_string(),
        output_hash: EXPECTED_HASH.to_string(),
        correct: true,
        evidence_graph: graph,
        median_ns: Some(400),
    };

    let result = evaluate_transition(&parent, &candidate, &policy, POLICY_HASH);
    assert!(result.is_rejected());
    assert!(matches!(result, PromotionDecision::Rejected {
        reason: RejectionReason::RequiredEvidenceFailed {
            kind: EvidenceKind::Robustness, ..
        }, ..
    }), "Expected RequiredEvidenceFailed(Robustness)");
}

// ── Critical case: correct but slower ────────────────────────────────────────
//
// This test proves the enforcement layer understands:
//   correctness preserved != improvement achieved
//
// A valid, correct candidate that is slower than the parent is NOT rejected.
// It is CorrectButSlower. The parent is preserved. The candidate is not invalid —
// it simply does not replace the baseline.

#[test]
fn correct_but_slower_preserves_parent() {
    let parent = parent_artifact(); // 800ns
    let policy = policy();

    let graph = passing_graph(Some(1200)); // 1200ns — slower than 800ns parent
    let slower_candidate = Artifact {
        id:             "firebird_avx512_slower".to_string(),
        iep_version:    "0.1".to_string(),
        authority:      test_authority(),
        backend:        "avx512".to_string(),
        commit:         "abc1234".to_string(),
        output_hash:    EXPECTED_HASH.to_string(),
        correct:        true,
        evidence_graph: graph,
        median_ns:      Some(1200),
    };

    let result = evaluate_transition(&parent, &slower_candidate, &policy, POLICY_HASH);

    // Not rejected — the candidate IS correct.
    assert!(!result.is_rejected(), "A correct candidate should never be Rejected");
    // Not promoted — it did not improve.
    assert!(!result.is_promoted(), "A slower candidate should not be Promoted");
    // Specifically: CorrectButSlower.
    assert!(
        matches!(result, PromotionDecision::CorrectButSlower {
            candidate_ns: 1200, parent_ns: 800, ..
        }),
        "Expected CorrectButSlower with correct ns values, got: {result:?}"
    );
    // candidate_is_correct() is true for CorrectButSlower.
    assert!(result.candidate_is_correct());
}

// ── Positive path: correct and faster → Promoted ─────────────────────────────

#[test]
fn correct_and_faster_is_promoted() {
    let parent    = parent_artifact(); // 800ns
    let candidate = candidate_artifact_faster(); // 400ns
    let policy    = policy();

    let result = evaluate_transition(&parent, &candidate, &policy, POLICY_HASH);

    assert!(result.is_promoted(), "Expected Promoted, got: {result:?}");
    assert!(result.candidate_is_correct());

    if let PromotionDecision::Promoted(event) = &result {
        assert_eq!(event.from, "firebird_reference_c422bfb");
        assert_eq!(event.to,   "firebird_avx512_abc1234");
        assert_eq!(event.policy_hash, POLICY_HASH);
        assert_eq!(event.result, "promoted");
        assert_eq!(event.candidate_ns, Some(400));
        assert_eq!(event.parent_ns,    Some(800));
        // Promotion event is hashable — commitment is stable.
        let h = event.commitment_hash();
        assert!(h.starts_with("sha256:"), "commitment_hash format wrong: {h}");
        // Idempotent: same event produces same hash.
        assert_eq!(h, event.commitment_hash());
    }
}

// ── Full chain: reference → candidate → promotion event ──────────────────────

#[test]
fn full_positive_chain_is_auditable() {
    let parent    = parent_artifact();
    let candidate = candidate_artifact_faster();
    let policy    = policy();

    let result = evaluate_transition(&parent, &candidate, &policy, POLICY_HASH);
    assert!(result.is_promoted());

    if let PromotionDecision::Promoted(event) = result {
        let record = event.to_record();
        // Record is parseable and contains the right fields.
        assert!(record.contains("\"event\":\"promotion\""));
        assert!(record.contains("\"result\":\"promoted\""));
        assert!(record.contains(&parent.id));
        assert!(record.contains(&candidate.id));
        // The evidence_hash in the event matches the candidate graph hash.
        assert!(record.contains(&candidate.evidence_graph.graph_hash));
    }
}
