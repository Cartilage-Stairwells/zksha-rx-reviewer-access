//! Evidence Contract v1 — the versioned JSON artifact interface.
//!
//! The evidence contract is the interface between tests (which produce
//! evidence) and the verification gate (which consumes it). The contract
//! is versioned so that the schema can evolve without silent mutation:
//!
//!   schema_version 1 → schema_version 2 (explicit migration, not drift)
//!
//! Logs explain what happened. Evidence artifacts prove what happened.
//! The JSON artifact is therefore treated as an interface — its schema
//! is the contract.
//!
//! Minimal v1 contract:
//! {
//!   "schema_version": 1,
//!   "run_id": "...",
//!   "timestamp": "...",
//!   "backend_identity": {
//!     "arch": "...",
//!     "os": "...",
//!     "rustc": "...",
//!     "cpu_features": []
//!   },
//!   "coverage": {
//!     "avx512": { "mul": 10012, "butterfly": 5328, "ntt_stage": 1024 },
//!     "scalar": { "mul": 10012, "butterfly": 5328, "ntt_stage": 1024 }
//!   },
//!   "parity": {
//!     "passed": true,
//!     "mismatches": 0
//!   }
//! }

use crate::instrument::coverage::CoverageMap;

/// Backend identity — environment context that prevents a parity result
/// from losing environmental context (which CPU, which compiler, which
/// features were enabled).
#[derive(Debug, Clone)]
pub struct BackendIdentity {
    pub arch: String,
    pub os: String,
    pub rustc: String,
    pub cpu_features: Vec<String>,
}

impl BackendIdentity {
    pub fn capture() -> Self {
        let mut features = Vec::new();

        if cfg!(target_arch = "x86_64") {
            features.push("x86_64".to_string());
        }
        if cfg!(target_feature = "avx512f") {
            features.push("avx512f".to_string());
        }
        if cfg!(target_feature = "avx512dq") {
            features.push("avx512dq".to_string());
        }
        if cfg!(target_feature = "avx2") {
            features.push("avx2".to_string());
        }
        if cfg!(target_feature = "neon") {
            features.push("neon".to_string());
        }

        Self {
            arch: std::env::consts::ARCH.to_string(),
            os: std::env::consts::OS.to_string(),
            rustc: option_env!("RUSTC_VERSION").unwrap_or("unknown").to_string(),
            cpu_features: features,
        }
    }

    pub fn to_json(&self) -> String {
        let features: Vec<String> = self.cpu_features.iter()
            .map(|f| format!("\"{}\"", f))
            .collect();
        format!(
            "\"arch\":\"{}\",\"os\":\"{}\",\"rustc\":\"{}\",\"cpu_features\":[{}]",
            self.arch, self.os, self.rustc, features.join(",")
        )
    }
}

/// Parity result — the pass/fail summary of the backend parity comparison.
#[derive(Debug, Clone)]
pub struct ParityResult {
    pub passed: bool,
    pub mismatches: u64,
}

impl ParityResult {
    pub fn pass() -> Self {
        Self { passed: true, mismatches: 0 }
    }

    pub fn fail(mismatches: u64) -> Self {
        Self { passed: false, mismatches }
    }

    pub fn to_json(&self) -> String {
        format!(
            "\"passed\":{},\"mismatches\":{}",
            self.passed, self.mismatches
        )
    }
}

/// The Evidence Contract v1 artifact.
///
/// This is the frozen interface between test producers and the gate.
/// Producers emit this JSON. The gate (via predicates) references it.
/// The schema_version field allows future evolution without silent mutation.
#[derive(Debug, Clone)]
pub struct EvidenceContractV1 {
    pub schema_version: u32,
    pub run_id: String,
    pub timestamp: String,
    pub backend_identity: BackendIdentity,
    pub coverage: CoverageMap,
    pub parity: ParityResult,
}

impl EvidenceContractV1 {
    pub fn new(run_id: &str, coverage: CoverageMap, parity: ParityResult) -> Self {
        Self {
            schema_version: 1,
            run_id: run_id.to_string(),
            timestamp: current_iso8601(),
            backend_identity: BackendIdentity::capture(),
            coverage,
            parity,
        }
    }

    /// Serialize to a JSON string.
    ///
    /// The output is a single-line JSON object suitable for piping to
    /// `jq`, writing to a file, or embedding in a promotion event.
    pub fn to_json(&self) -> String {
        format!(
            "{{\
            \"schema_version\":{},\
            \"run_id\":\"{}\",\
            \"timestamp\":\"{}\",\
            \"backend_identity\":{{{}}},\
            \"coverage\":{},\
            \"parity\":{{{}}}}}",
            self.schema_version,
            self.run_id,
            self.timestamp,
            self.backend_identity.to_json(),
            self.coverage.to_json(),
            self.parity.to_json(),
        )
    }
}

/// Generate an ISO 8601 timestamp.
///
/// Uses system time. For deterministic builds, the run_id should be
/// the primary correlation key; the timestamp is informational.
fn current_iso8601() -> String {
    // We don't have chrono as a dependency, so we produce a simple
    // Unix epoch seconds timestamp. This is sufficient for the v1
    // contract — the run_id is the primary identifier.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("epoch:{}", secs)
}
