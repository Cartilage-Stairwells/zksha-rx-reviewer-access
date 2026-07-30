//! IEP Promotion Event — the immutable output of a successful transition.
//!
//! A PromotionEvent is itself an artifact: it records which artifact was
//! promoted from, which artifact was promoted to, under which policy, and
//! with which evidence. The event can be hashed and committed alongside
//! the candidate artifact to make the transition permanently auditable.

/// The immutable record of a successful promotion transition.
///
/// This is not a log entry. It is a trust-chain link. The `from` → `to`
/// pair, combined with `policy_hash` and `evidence_hash`, is sufficient to
/// verify the transition without re-running the evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct PromotionEvent {
    /// ID of the prior trusted artifact being replaced.
    pub from:          String,
    /// ID of the newly promoted artifact.
    pub to:            String,
    /// Policy that governed this transition.
    pub policy_id:     String,
    /// Hash of the committed policy file. Drift from the declared hash is
    /// detectable by comparing against the committed policy document.
    pub policy_hash:   String,
    /// Hash of the candidate's evidence graph at evaluation time.
    pub evidence_hash: String,
    /// Candidate median_ns (if available).
    pub candidate_ns:  Option<u64>,
    /// Parent median_ns (if available). Together with candidate_ns, this
    /// documents the performance delta at the moment of promotion.
    pub parent_ns:     Option<u64>,
    /// Always "promoted" for events emitted by evaluate_transition.
    pub result:        String,
}

impl PromotionEvent {
    /// Serialize the event to a stable JSON-like string for hashing or logging.
    pub fn to_record(&self) -> String {
        let c_ns = self.candidate_ns.map(|n| n.to_string()).unwrap_or("null".into());
        let p_ns = self.parent_ns.map(|n| n.to_string()).unwrap_or("null".into());
        format!(
            r#"{{"event":"promotion","from":"{from}","to":"{to}","policy_id":"{pid}","policy_hash":"{ph}","evidence_hash":"{eh}","candidate_ns":{c_ns},"parent_ns":{p_ns},"result":"{result}"}}"#,
            from   = self.from,
            to     = self.to,
            pid    = self.policy_id,
            ph     = self.policy_hash,
            eh     = self.evidence_hash,
            result = self.result,
        )
    }

    /// SHA256 of the canonical record. Use this as the promotion_event_hash
    /// in downstream artifacts or decisions/ records.
    pub fn commitment_hash(&self) -> String {
        use sha2::{Digest, Sha256};
        let digest = Sha256::digest(self.to_record().as_bytes());
        format!("sha256:{digest:x}")
    }
}
