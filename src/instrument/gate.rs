//! Verification Gate v1 — the promotion decision function.
//!
//! The gate is deliberately simple. It evaluates a collection of predicates
//! and derives a deterministic promotion decision:
//!
//!   ∀ predicate: status == PASS  →  PROMOTION = PASS
//!   otherwise                    →  PROMOTION = REJECT
//!
//! The gate does not understand what each predicate proves. It only checks
//! the predicate contract (status == PASS). This means:
//!   - Adding a new verification dimension is adding a predicate, not
//!     redesigning the gate.
//!   - The gate logic never changes — only the predicate set changes.
//!   - The decision is deterministic and free of human interpretation.

use crate::instrument::predicate::{Predicate, PredicateCollection, PredicateStatus};

/// The result of a verification gate evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromotionResult {
    /// All predicates passed. The candidate is eligible for promotion.
    Pass {
        /// Number of predicates evaluated.
        predicate_count: usize,
        /// List of predicate IDs that passed.
        passed: Vec<String>,
    },
    /// At least one predicate did not pass. Promotion is rejected.
    Reject {
        /// The predicate that caused the rejection.
        rejecting_predicate: String,
        /// Why the predicate was rejected (Fail reason, Skip reason, or "pending").
        reason: String,
        /// Full list of all predicate statuses at rejection time.
        all_statuses: Vec<(String, String)>,
    },
}

impl PromotionResult {
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass { .. })
    }

    pub fn is_reject(&self) -> bool {
        matches!(self, Self::Reject { .. })
    }
}

/// Evaluate a predicate collection and return a promotion decision.
///
/// The evaluation is ordered: predicates are checked in the order they were
/// added to the collection. The first non-passing predicate determines the
/// rejection reason. This makes the decision deterministic and the rejection
/// reason unambiguous.
///
/// The rule is intentionally total — there is no "partial pass" state.
/// Either all predicates pass, or the gate rejects.
pub fn evaluate_gate(predicates: &PredicateCollection) -> PromotionResult {
    let mut passed = Vec::new();
    let mut all_statuses: Vec<(String, String)> = Vec::new();

    for predicate in &predicates.predicates {
        let id_str = predicate.id.as_str().to_string();
        let status_str = predicate.status.to_string();

        match &predicate.status {
            PredicateStatus::Pass => {
                passed.push(id_str.clone());
                all_statuses.push((id_str, status_str));
            }
            PredicateStatus::Fail(reason) => {
                // Collect remaining predicate statuses for the full picture,
                // then reject.
                all_statuses.push((id_str.clone(), status_str));
                for p in predicates.predicates.iter().skip(all_statuses.len()) {
                    all_statuses.push((p.id.as_str().to_string(), p.status.to_string()));
                }
                return PromotionResult::Reject {
                    rejecting_predicate: id_str,
                    reason: reason.clone(),
                    all_statuses,
                };
            }
            PredicateStatus::Skip(reason) => {
                all_statuses.push((id_str.clone(), status_str));
                for p in predicates.predicates.iter().skip(all_statuses.len()) {
                    all_statuses.push((p.id.as_str().to_string(), p.status.to_string()));
                }
                return PromotionResult::Reject {
                    rejecting_predicate: id_str,
                    reason: format!("skipped: {}", reason),
                    all_statuses,
                };
            }
            PredicateStatus::Pending => {
                all_statuses.push((id_str.clone(), status_str));
                for p in predicates.predicates.iter().skip(all_statuses.len()) {
                    all_statuses.push((p.id.as_str().to_string(), p.status.to_string()));
                }
                return PromotionResult::Reject {
                    rejecting_predicate: id_str,
                    reason: "evaluation pending — evidence not yet collected".to_string(),
                    all_statuses,
                };
            }
        }
    }

    PromotionResult::Pass {
        predicate_count: passed.len(),
        passed,
    }
}

/// A builder for constructing predicate collections and evaluating them.
///
/// Typical usage:
///   let result = GateBuilder::new()
///       .pass("representation_custody", 1, "representation_audit.json")
///       .pass("execution_custody", 1, "backend_parity.json")
///       .pass("execution_coverage", 1, "backend_parity.json")
///       .evaluate();
pub struct GateBuilder {
    collection: PredicateCollection,
}

impl GateBuilder {
    pub fn new() -> Self {
        Self { collection: PredicateCollection::new() }
    }

    pub fn pass(mut self, id: &str, version: u32, evidence: &str) -> Self {
        self.collection.add(Predicate::pass(id, version, evidence));
        self
    }

    pub fn fail(mut self, id: &str, version: u32, evidence: &str, reason: &str) -> Self {
        self.collection.add(Predicate::fail(id, version, evidence, reason));
        self
    }

    pub fn skip(mut self, id: &str, version: u32, evidence: &str, reason: &str) -> Self {
        self.collection.add(Predicate::skip(id, version, evidence, reason));
        self
    }

    pub fn pending(mut self, id: &str, version: u32, evidence: &str) -> Self {
        self.collection.add(Predicate::pending(id, version, evidence));
        self
    }

    pub fn add(mut self, predicate: Predicate) -> Self {
        self.collection.add(predicate);
        self
    }

    pub fn evaluate(self) -> PromotionResult {
        evaluate_gate(&self.collection)
    }

    pub fn collection(self) -> PredicateCollection {
        self.collection
    }
}

impl Default for GateBuilder {
    fn default() -> Self {
        Self::new()
    }
}
