//! Verification Predicates — the unit of verification for Issue #5.
//!
//! A predicate is the contract between a subsystem and the verification gate.
//! The gate does not understand what the predicate proves — it only checks
//! that the predicate's status is PASS.
//!
//! This separation means:
//!   - Adding a new verification dimension (e.g., "constant_time") is adding
//!     a predicate, not redesigning the gate.
//!   - The evidence reference is opaque to the gate — it points to whatever
//!     artifact the subsystem produced (JSON, log, hash, etc.).
//!   - Predicate versions allow the proof contract to evolve without breaking
//!     older evidence artifacts.

use std::fmt;

/// A predicate identifier. Stable across versions.
///
/// Convention: snake_case, subsystem-prefixed.
/// Examples: "representation_custody", "execution_custody", "execution_coverage"
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PredicateId(pub String);

impl PredicateId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PredicateId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The status of a predicate evaluation.
///
/// Only `Pass` satisfies the promotion rule. All other variants reject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PredicateStatus {
    /// The predicate's evidence contract is satisfied.
    Pass,
    /// The predicate's evidence contract is not satisfied.
    /// The reason string is human-readable context for the rejection.
    Fail(String),
    /// The predicate was not evaluated (e.g., backend not available).
    /// Treated as a rejection — "not tested" is not "passing".
    Skip(String),
    /// The predicate has not yet been evaluated.
    /// Treated as a rejection — incomplete evidence cannot promote.
    Pending,
}

impl PredicateStatus {
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass)
    }

    pub fn is_reject(&self) -> bool {
        !self.is_pass()
    }
}

impl fmt::Display for PredicateStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pass => write!(f, "PASS"),
            Self::Fail(reason) => write!(f, "FAIL({})", reason),
            Self::Skip(reason) => write!(f, "SKIP({})", reason),
            Self::Pending => write!(f, "PENDING"),
        }
    }
}

/// A single verification predicate.
///
/// This is the unit the gate evaluates. The gate checks `status == Pass`
/// for every predicate in the collection. The `evidence_reference` is
/// opaque to the gate — it typically points to a JSON artifact produced
/// by the subsystem's test suite.
#[derive(Debug, Clone)]
pub struct Predicate {
    /// Stable identifier (e.g., "representation_custody").
    pub id: PredicateId,
    /// Contract version — allows the proof standard to evolve.
    pub version: u32,
    /// Current evaluation status.
    pub status: PredicateStatus,
    /// Reference to the evidence artifact (file path, URI, or hash).
    pub evidence_reference: String,
}

impl Predicate {
    pub fn new(id: &str, version: u32, status: PredicateStatus, evidence: &str) -> Self {
        Self {
            id: PredicateId::new(id),
            version,
            status,
            evidence_reference: evidence.to_string(),
        }
    }

    /// Convenience constructor for a passing predicate.
    pub fn pass(id: &str, version: u32, evidence: &str) -> Self {
        Self::new(id, version, PredicateStatus::Pass, evidence)
    }

    /// Convenience constructor for a failing predicate.
    pub fn fail(id: &str, version: u32, evidence: &str, reason: &str) -> Self {
        Self::new(id, version, PredicateStatus::Fail(reason.to_string()), evidence)
    }

    /// Convenience constructor for a skipped predicate.
    pub fn skip(id: &str, version: u32, evidence: &str, reason: &str) -> Self {
        Self::new(id, version, PredicateStatus::Skip(reason.to_string()), evidence)
    }

    /// Convenience constructor for a pending predicate.
    pub fn pending(id: &str, version: u32, evidence: &str) -> Self {
        Self::new(id, version, PredicateStatus::Pending, evidence)
    }
}

/// A collection of predicates for a single promotion evaluation.
///
/// The collection is the input to the verification gate. The gate
/// evaluates all predicates and derives a promotion decision.
#[derive(Debug, Clone, Default)]
pub struct PredicateCollection {
    pub predicates: Vec<Predicate>,
}

impl PredicateCollection {
    pub fn new() -> Self {
        Self { predicates: Vec::new() }
    }

    pub fn add(&mut self, predicate: Predicate) {
        self.predicates.push(predicate);
    }

    pub fn get(&self, id: &PredicateId) -> Option<&Predicate> {
        self.predicates.iter().find(|p| &p.id == id)
    }

    pub fn all_pass(&self) -> bool {
        self.predicates.iter().all(|p| p.status.is_pass())
    }

    /// Iterate over predicates that are not passing.
    pub fn failures(&self) -> impl Iterator<Item = &Predicate> {
        self.predicates.iter().filter(|p| p.status.is_reject())
    }

    /// The canonical Issue #5 predicate set for the avx512-butterfly crate.
    ///
    /// These are the predicates that must all pass for a promotion:
    ///   1. representation_custody — Issue #3 (compiler-enforced construction)
    ///   2. execution_custody     — Issue #4 (backend semantic parity)
    ///   3. execution_coverage    — Issue #4 (corpus coverage evidence)
    pub fn avx512_butterfly_v1() -> Self {
        let mut collection = Self::new();
        collection.add(Predicate::pending("representation_custody", 1, "tests/representation_audit.json"));
        collection.add(Predicate::pending("execution_custody", 1, "tests/backend_parity.json"));
        collection.add(Predicate::pending("execution_coverage", 1, "tests/backend_parity.json"));
        collection
    }
}
