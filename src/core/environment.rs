//! EnvironmentContract — the execution boundary.
//!
//! "The same code was run" is NOT the same as "the same computation
//! was performed." The environment contract captures what MUST be true
//! for an evaluation to be reproducible.
//!
//! Identity derivation:
//!   EnvironmentContract → canonical encoding → hash → identity
//!
//! The object is not self-authenticating. You cannot claim a hash;
//! you must compute it from the canonical encoding.
//!
//! For simple mathematical validation:
//!   architecture, compiler, dependency graph, serialization format
//!
//! For AVX-512 NTT:
//!   CPU features, SIMD width, compiler flags, field arithmetic mode,
//!   backend selection, numerical representation
//!
//! The core does not know which case applies. It stores the fields
//! and computes identity. Domain knowledge lives in the adapter.

use std::collections::BTreeMap;
use super::hash::Hash;
use super::canonical::Canonical;

/// The environment contract. Declares what MUST be true for an
/// evaluation to be reproducible.
///
/// Required fields: architecture, compiler, evaluator_version,
/// serialization_format.
///
/// Optional fields are stored in the extra map. The core does not
/// interpret them — it includes them in the canonical encoding so
/// they affect identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentContract {
    pub contract_id: String,
    pub architecture: String,
    pub compiler: String,
    pub evaluator_version: String,
    pub serialization_format: String,
    pub extra: BTreeMap<String, String>,
}

impl EnvironmentContract {
    /// Create a new environment contract with the required fields.
    pub fn new(
        contract_id: &str,
        architecture: &str,
        compiler: &str,
        evaluator_version: &str,
        serialization_format: &str,
    ) -> Self {
        EnvironmentContract {
            contract_id: contract_id.to_string(),
            architecture: architecture.to_string(),
            compiler: compiler.to_string(),
            evaluator_version: evaluator_version.to_string(),
            serialization_format: serialization_format.to_string(),
            extra: BTreeMap::new(),
        }
    }

    /// Add an optional field to the contract.
    /// The field is included in the canonical encoding and affects identity.
    pub fn with_extra(mut self, key: &str, value: &str) -> Self {
        self.extra.insert(key.to_string(), value.to_string());
        self
    }

    /// Compute the identity hash from the canonical encoding.
    /// This is the immutable reference — two contracts are comparable
    /// only if their identity hashes are equal.
    pub fn identity_hash(&self) -> Hash {
        self.canonical_hash()
    }
}

impl Canonical for EnvironmentContract {
    fn canonical_bytes(&self) -> Vec<u8> {
        // Canonical JSON with sorted keys.
        // The encoding includes all fields that affect identity.
        use std::collections::BTreeMap;
        let mut fields: BTreeMap<String, String> = BTreeMap::new();
        fields.insert("contract_id".to_string(), format!("{:?}", self.contract_id));
        fields.insert("architecture".to_string(), format!("{:?}", self.architecture));
        fields.insert("compiler".to_string(), format!("{:?}", self.compiler));
        fields.insert("evaluator_version".to_string(), format!("{:?}", self.evaluator_version));
        fields.insert("serialization_format".to_string(), format!("{:?}", self.serialization_format));

        // Include extra fields
        for (k, v) in &self.extra {
            fields.insert(k.clone(), format!("{:?}", v));
        }

        super::canonical::encode_map(&fields)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_identity_is_deterministic() {
        let c1 = EnvironmentContract::new("host-v1", "x86_64", "rustc 1.97", "runner-v1", "json-canonical");
        let c2 = EnvironmentContract::new("host-v1", "x86_64", "rustc 1.97", "runner-v1", "json-canonical");
        assert_eq!(c1.identity_hash(), c2.identity_hash());
    }

    #[test]
    fn contract_identity_changes_with_fields() {
        let c1 = EnvironmentContract::new("host-v1", "x86_64", "rustc 1.97", "runner-v1", "json-canonical");
        let c2 = EnvironmentContract::new("host-v1", "x86_64", "rustc 1.98", "runner-v1", "json-canonical");
        assert_ne!(c1.identity_hash(), c2.identity_hash());
    }

    #[test]
    fn contract_extra_affects_identity() {
        let c1 = EnvironmentContract::new("host-v1", "x86_64", "rustc 1.97", "runner-v1", "json-canonical");
        let c2 = EnvironmentContract::new("host-v1", "x86_64", "rustc 1.97", "runner-v1", "json-canonical")
            .with_extra("cpu_features", "avx512f");
        assert_ne!(c1.identity_hash(), c2.identity_hash());
    }

    #[test]
    fn contract_is_domain_blind() {
        // The contract stores "cpu_features" as an opaque string.
        // It does not interpret it. A π contract and an NTT contract
        // with the same fields have the same type.
        let math = EnvironmentContract::new("math-v1", "x86_64", "python 3.11", "runner-v1", "json-canonical");
        let ntt = EnvironmentContract::new("ntt-v1", "x86_64", "python 3.11", "runner-v1", "json-canonical");
        // Same type, different contract_id → different identity
        assert_ne!(math.identity_hash(), ntt.identity_hash());
        // But the type is identical — no domain-specific fields
        assert_eq!(std::mem::size_of_val(&math), std::mem::size_of_val(&ntt));
    }
}
