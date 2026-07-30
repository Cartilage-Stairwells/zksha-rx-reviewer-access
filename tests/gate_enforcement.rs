//! Issue #5 — Verification Gate v1 enforcement tests.
//!
//! Tests the predicate model, coverage map, and verification gate.
//! The gate is deliberately simple — it evaluates predicates and derives
//! a deterministic promotion decision. These tests verify that the gate
//! logic is correct and that the evidence contract can be serialized.

use avx512_butterfly::instrument::predicate::{Predicate, PredicateCollection, PredicateStatus, PredicateId};
use avx512_butterfly::instrument::gate::{evaluate_gate, GateBuilder, PromotionResult};
use avx512_butterfly::instrument::coverage::CoverageMap;
use avx512_butterfly::instrument::evidence_contract::{
    EvidenceContractV1, BackendIdentity, ParityResult,
};

// ---------------------------------------------------------------------------
// Predicate model tests
// ---------------------------------------------------------------------------

#[test]
fn predicate_status_pass() {
    let p = Predicate::pass("representation_custody", 1, "representation_audit.json");
    assert!(p.status.is_pass());
    assert!(!p.status.is_reject());
    assert_eq!(p.status.to_string(), "PASS");
}

#[test]
fn predicate_status_fail() {
    let p = Predicate::fail("execution_custody", 1, "backend_parity.json", "3 mismatches");
    assert!(!p.status.is_pass());
    assert!(p.status.is_reject());
    assert!(p.status.to_string().contains("FAIL"));
}

#[test]
fn predicate_status_skip() {
    let p = Predicate::skip("execution_custody", 1, "backend_parity.json", "AVX-512 not available");
    assert!(!p.status.is_pass());
    assert!(p.status.is_reject());
    assert!(p.status.to_string().contains("SKIP"));
}

#[test]
fn predicate_status_pending() {
    let p = Predicate::pending("execution_coverage", 1, "coverage.json");
    assert!(!p.status.is_pass());
    assert!(p.status.is_reject());
    assert_eq!(p.status.to_string(), "PENDING");
}

#[test]
fn predicate_id_ordering() {
    let a = PredicateId::new("a_predicate");
    let b = PredicateId::new("b_predicate");
    assert!(a < b);
    assert_eq!(a.as_str(), "a_predicate");
    assert_eq!(format!("{}", a), "a_predicate");
}

// ---------------------------------------------------------------------------
// Gate evaluation tests — the promotion rule
// ---------------------------------------------------------------------------

#[test]
fn gate_all_pass_yields_promotion() {
    let result = GateBuilder::new()
        .pass("representation_custody", 1, "representation_audit.json")
        .pass("execution_custody", 1, "backend_parity.json")
        .pass("execution_coverage", 1, "coverage.json")
        .evaluate();

    assert!(result.is_pass());
    match &result {
        PromotionResult::Pass { predicate_count, passed } => {
            assert_eq!(*predicate_count, 3);
            assert_eq!(passed.len(), 3);
            assert!(passed.contains(&"representation_custody".to_string()));
            assert!(passed.contains(&"execution_custody".to_string()));
            assert!(passed.contains(&"execution_coverage".to_string()));
        }
        _ => panic!("expected Pass"),
    }
}

#[test]
fn gate_one_fail_rejects() {
    let result = GateBuilder::new()
        .pass("representation_custody", 1, "representation_audit.json")
        .fail("execution_custody", 1, "backend_parity.json", "3 mismatches found")
        .pass("execution_coverage", 1, "coverage.json")
        .evaluate();

    assert!(result.is_reject());
    match &result {
        PromotionResult::Reject { rejecting_predicate, reason, .. } => {
            assert_eq!(rejecting_predicate, "execution_custody");
            assert!(reason.contains("3 mismatches"));
        }
        _ => panic!("expected Reject"),
    }
}

#[test]
fn gate_one_skip_rejects() {
    let result = GateBuilder::new()
        .pass("representation_custody", 1, "representation_audit.json")
        .pass("execution_custody", 1, "backend_parity.json")
        .skip("execution_coverage", 1, "coverage.json", "no coverage data")
        .evaluate();

    assert!(result.is_reject());
    match &result {
        PromotionResult::Reject { rejecting_predicate, reason, .. } => {
            assert_eq!(rejecting_predicate, "execution_coverage");
            assert!(reason.contains("skipped"));
        }
        _ => panic!("expected Reject"),
    }
}

#[test]
fn gate_one_pending_rejects() {
    let result = GateBuilder::new()
        .pass("representation_custody", 1, "representation_audit.json")
        .pending("execution_custody", 1, "backend_parity.json")
        .pass("execution_coverage", 1, "coverage.json")
        .evaluate();

    assert!(result.is_reject());
    match &result {
        PromotionResult::Reject { rejecting_predicate, reason, .. } => {
            assert_eq!(rejecting_predicate, "execution_custody");
            assert!(reason.contains("pending"));
        }
        _ => panic!("expected Reject"),
    }
}

#[test]
fn gate_empty_collection_passes() {
    // Vacuous truth: no predicates → nothing to reject.
    let collection = PredicateCollection::new();
    let result = evaluate_gate(&collection);
    assert!(result.is_pass());
}

#[test]
fn gate_first_failure_wins() {
    // When multiple predicates fail, the first one in insertion order
    // is the rejection reason. This makes the decision deterministic.
    let result = GateBuilder::new()
        .fail("representation_custody", 1, "audit.json", "construction bypassed")
        .fail("execution_custody", 1, "parity.json", "5 mismatches")
        .evaluate();

    match &result {
        PromotionResult::Reject { rejecting_predicate, reason, .. } => {
            assert_eq!(rejecting_predicate, "representation_custody");
            assert!(reason.contains("construction bypassed"));
        }
        _ => panic!("expected Reject"),
    }
}

// ---------------------------------------------------------------------------
// Canonical predicate set for avx512-butterfly
// ---------------------------------------------------------------------------

#[test]
fn canonical_predicate_set_starts_pending() {
    let collection = PredicateCollection::avx512_butterfly_v1();
    assert_eq!(collection.predicates.len(), 3);

    // All predicates start as Pending — evidence not yet collected.
    for p in &collection.predicates {
        assert_eq!(p.status, PredicateStatus::Pending);
    }

    // A gate evaluation of the pending set must reject.
    let result = evaluate_gate(&collection);
    assert!(result.is_reject());
}

#[test]
fn canonical_predicate_set_passes_when_all_pass() {
    let mut collection = PredicateCollection::avx512_butterfly_v1();
    for p in collection.predicates.iter_mut() {
        p.status = PredicateStatus::Pass;
    }
    let result = evaluate_gate(&collection);
    assert!(result.is_pass());
}

// ---------------------------------------------------------------------------
// Coverage map tests
// ---------------------------------------------------------------------------

#[test]
fn coverage_map_basic_operations() {
    let mut map = CoverageMap::new();
    map.record("scalar", "mul", 10012);
    map.record("avx512", "butterfly", 5328);

    assert_eq!(map.get("scalar", "mul"), 10012);
    assert_eq!(map.get("avx512", "butterfly"), 5328);
    assert_eq!(map.get("scalar", "butterfly"), 0); // not recorded
    assert_eq!(map.get("neon", "mul"), 0); // backend not recorded
}

#[test]
fn coverage_map_increment_and_add() {
    let mut map = CoverageMap::new();

    map.increment("scalar", "mul");
    map.increment("scalar", "mul");
    assert_eq!(map.get("scalar", "mul"), 2);

    map.add("scalar", "mul", 10);
    assert_eq!(map.get("scalar", "mul"), 12);
}

#[test]
fn coverage_map_merge() {
    let mut map1 = CoverageMap::new();
    map1.record("scalar", "mul", 100);
    map1.record("avx512", "butterfly", 200);

    let mut map2 = CoverageMap::new();
    map2.record("scalar", "mul", 50);
    map2.record("scalar", "ntt_stage", 45);

    map1.merge(&map2);

    assert_eq!(map1.get("scalar", "mul"), 150); // 100 + 50
    assert_eq!(map1.get("avx512", "butterfly"), 200);
    assert_eq!(map1.get("scalar", "ntt_stage"), 45); // new operation
}

#[test]
fn coverage_map_backends_and_operations() {
    let mut map = CoverageMap::new();
    map.record("scalar", "mul", 100);
    map.record("scalar", "butterfly", 200);
    map.record("avx512", "mul", 300);

    let backends = map.backends();
    assert!(backends.contains(&"scalar"));
    assert!(backends.contains(&"avx512"));

    let scalar_ops = map.operations("scalar");
    assert!(scalar_ops.contains(&"mul"));
    assert!(scalar_ops.contains(&"butterfly"));

    assert_eq!(map.total_operations(), 3);
}

#[test]
fn coverage_map_json_serialization() {
    let mut map = CoverageMap::new();
    map.record("scalar", "mul", 10012);
    map.record("avx512", "butterfly", 5328);

    let json = map.to_json();
    // JSON should contain both backends and their operations
    assert!(json.contains("\"scalar\""));
    assert!(json.contains("\"mul\":10012"));
    assert!(json.contains("\"avx512\""));
    assert!(json.contains("\"butterfly\":5328"));

    eprintln!("coverage_map_json: {}", json);
}

#[test]
fn coverage_map_adding_backend_is_just_data() {
    // The key design property: adding a new backend is just adding
    // evidence, not redesigning the schema.
    let mut map = CoverageMap::new();
    map.record("neon", "mul", 5000);
    map.record("cuda", "butterfly", 10000);

    assert_eq!(map.get("neon", "mul"), 5000);
    assert_eq!(map.get("cuda", "butterfly"), 10000);
    assert_eq!(map.backends().len(), 2);
}

// ---------------------------------------------------------------------------
// Evidence contract v1 tests
// ---------------------------------------------------------------------------

#[test]
fn evidence_contract_v1_serialization() {
    let mut coverage = CoverageMap::new();
    coverage.record("scalar", "mul", 10012);
    coverage.record("avx512", "butterfly", 5328);

    let contract = EvidenceContractV1::new(
        "test_run_001",
        coverage,
        ParityResult::pass(),
    );

    let json = contract.to_json();

    // Verify required fields are present
    assert!(json.contains("\"schema_version\":1"));
    assert!(json.contains("\"run_id\":\"test_run_001\""));
    assert!(json.contains("\"timestamp\""));
    assert!(json.contains("\"backend_identity\""));
    assert!(json.contains("\"coverage\""));
    assert!(json.contains("\"parity\""));
    assert!(json.contains("\"passed\":true"));
    assert!(json.contains("\"mismatches\":0"));

    eprintln!("evidence_contract_v1: {}", json);
}

#[test]
fn evidence_contract_v1_fail_result() {
    let coverage = CoverageMap::new();
    let contract = EvidenceContractV1::new(
        "test_run_fail",
        coverage,
        ParityResult::fail(7),
    );

    let json = contract.to_json();
    assert!(json.contains("\"passed\":false"));
    assert!(json.contains("\"mismatches\":7"));
}

#[test]
fn backend_identity_capture() {
    let identity = BackendIdentity::capture();
    assert!(!identity.arch.is_empty());
    assert!(!identity.os.is_empty());

    let json = identity.to_json();
    assert!(json.contains("\"arch\""));
    assert!(json.contains("\"os\""));
    assert!(json.contains("\"rustc\""));
    assert!(json.contains("\"cpu_features\""));

    eprintln!("backend_identity: {}", json);
}

// ---------------------------------------------------------------------------
// End-to-end: tests produce evidence → gate evaluates → promotion decision
// ---------------------------------------------------------------------------

#[test]
fn end_to_end_promotion_flow() {
    // Simulate the full flow:
    // 1. Tests produce an evidence contract
    // 2. Predicates reference the evidence contract
    // 3. Gate evaluates predicates → promotion decision

    // Step 1: produce evidence
    let mut coverage = CoverageMap::new();
    coverage.record("scalar", "mul", 10012);
    coverage.record("scalar", "butterfly", 5017);
    coverage.record("scalar", "ntt_stage", 71);
    coverage.record("avx512", "butterfly", 8016);
    coverage.record("avx512", "ntt_stage", 71);

    let contract = EvidenceContractV1::new(
        "backend_parity_1544BEEFCAFE4242",
        coverage,
        ParityResult::pass(),
    );
    let evidence_json = contract.to_json();

    // Step 2: construct predicates referencing the evidence
    let result = GateBuilder::new()
        .pass("representation_custody", 1, "tests/representation_audit.json")
        .pass("execution_custody", 1, "tests/backend_parity.json")
        .pass("execution_coverage", 1, "tests/backend_parity.json")
        .evaluate();

    // Step 3: gate derives promotion decision
    assert!(result.is_pass());
    eprintln!("end_to_end_promotion: PASS");
    eprintln!("evidence: {}", evidence_json);
}

#[test]
fn end_to_end_rejection_flow() {
    // Same flow, but execution_custody fails due to parity mismatches.
    let result = GateBuilder::new()
        .pass("representation_custody", 1, "representation_audit.json")
        .fail("execution_custody", 1, "backend_parity.json", "5 mismatches in AVX-512 butterfly")
        .pass("execution_coverage", 1, "backend_parity.json")
        .evaluate();

    assert!(result.is_reject());

    match &result {
        PromotionResult::Reject { rejecting_predicate, reason, all_statuses } => {
            assert_eq!(rejecting_predicate, "execution_custody");
            assert!(reason.contains("5 mismatches"));
            assert!(all_statuses.iter().any(|(id, _)| id == "representation_custody"));
            assert!(all_statuses.iter().any(|(id, _)| id == "execution_custody"));

            eprintln!("end_to_end_rejection: REJECT — predicate={}, reason={}", rejecting_predicate, reason);
        }
        _ => panic!("expected Reject"),
    }
}
