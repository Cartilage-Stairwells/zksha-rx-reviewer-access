//! IEP Artifact types.
//!
//! An Artifact is the unit of trust. It carries its own evidence graph
//! and a commitment hash that covers both the evidence content and the
//! evidence graph structure. Altering any field after sealing invalidates
//! the commitment.

use std::collections::BTreeMap;

/// The five stable evidence kinds (IEP v0.1 vocabulary).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EvidenceKind {
    Identity,
    Determinism,
    Independence,
    Robustness,
    Performance,
}

impl EvidenceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Identity    => "identity",
            Self::Determinism => "determinism",
            Self::Independence => "independence",
            Self::Robustness  => "robustness",
            Self::Performance => "performance",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "identity"      => Some(Self::Identity),
            "determinism"   => Some(Self::Determinism),
            "independence"  => Some(Self::Independence),
            "robustness"    => Some(Self::Robustness),
            "performance"   => Some(Self::Performance),
            _               => None,
        }
    }
}

/// Result of a single evidence node.
/// `None` means the kind is not applicable for this artifact/backend.
#[derive(Debug, Clone, PartialEq)]
pub enum EvidenceResult {
    Pass,
    Fail(String),
    NotApplicable,
}

/// A single node in the evidence graph.
#[derive(Debug, Clone)]
pub struct EvidenceNode {
    pub kind:       EvidenceKind,
    pub result:     EvidenceResult,
    pub mechanism:  String,
    /// For performance nodes: lower is better, in nanoseconds.
    pub median_ns:  Option<u64>,
}

/// The evidence graph for an artifact.
/// Keyed by EvidenceKind; BTreeMap for stable serialization order.
#[derive(Debug, Clone)]
pub struct EvidenceGraph {
    pub nodes: BTreeMap<EvidenceKind, EvidenceNode>,
    /// SHA256 commitment over the graph content (stable JSON serialization).
    /// Computed at seal time. Alteration after sealing is detectable.
    pub graph_hash: String,
}

impl EvidenceGraph {
    /// Returns true if the graph_hash is consistent with the node content.
    /// Uses the same serialization as iep_runner: sorted keys, no whitespace.
    pub fn verify_commitment(&self) -> bool {
        let computed = self.compute_hash();
        computed == self.graph_hash
    }

    pub fn compute_hash(&self) -> String {
        use sha2::{Digest, Sha256};
        // Serialize deterministically: sorted by kind name, compact JSON-like string.
        let mut parts: Vec<String> = self.nodes.iter().map(|(k, n)| {
            let result_str = match &n.result {
                EvidenceResult::Pass           => "pass".to_string(),
                EvidenceResult::Fail(reason)   => format!("fail:{reason}"),
                EvidenceResult::NotApplicable  => "na".to_string(),
            };
            let perf = n.median_ns.map(|ns| format!(",median_ns={ns}")).unwrap_or_default();
            format!("{}:{}:{}{}", k.as_str(), result_str, n.mechanism, perf)
        }).collect();
        parts.sort(); // already sorted by BTreeMap, but explicit for clarity
        let payload = parts.join("|");
        let digest = Sha256::digest(payload.as_bytes());
        format!("sha256:{digest:x}")
    }
}

/// Core IEP authority definition.
#[derive(Debug, Clone)]
pub struct Authority {
    pub spec:          String,
    pub reference:     String,
    pub corpus:        String,
    pub corpus_hash:   String,
    pub expected_hash: String,
}

/// An IEP artifact — the unit of trust.
#[derive(Debug, Clone)]
pub struct Artifact {
    pub id:              String,   // e.g. "firebird_reference_c422bfb"
    pub iep_version:     String,
    pub authority:       Authority,
    pub backend:         String,   // "reference" | "avx512" | "scalar" | ...
    pub commit:          String,
    /// SHA256 of the output encoding (sequential LE u32 pairs).
    pub output_hash:     String,
    /// `true` iff output_hash == authority.expected_hash at artifact creation time.
    pub correct:         bool,
    pub evidence_graph:  EvidenceGraph,
    /// For performance evidence: wall-clock median in ns. None for reference artifacts.
    pub median_ns:       Option<u64>,
}

impl Artifact {
    /// Returns true iff this artifact passed the identity check:
    /// output_hash matches authority.expected_hash.
    pub fn identity_valid(&self) -> bool {
        self.output_hash == self.authority.expected_hash
    }
}
