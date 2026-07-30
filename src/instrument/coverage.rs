//! Coverage Map — nested backend → operation → count tracking.
//!
//! Uses a nested BTreeMap rather than a fixed struct so that adding a new
//! backend (neon, gpu, cuda) or operation is adding evidence, not
//! redesigning the gate schema.
//!
//! Example JSON:
//! {
//!   "coverage": {
//!     "avx512": { "mul": 10012, "butterfly": 5328, "ntt_stage": 1024 },
//!     "scalar": { "mul": 10012, "butterfly": 5328, "ntt_stage": 1024 }
//!   }
//! }

use std::collections::BTreeMap;

/// Nested coverage map: backend → operation → count.
///
/// The map is intentionally schema-free — backends and operations are
/// string keys. This means:
///   - Adding "neon" or "cuda" is just inserting a new key.
///   - Adding "ntt_full" is just inserting a new operation.
///   - No struct changes, no schema migration, no gate redesign.
#[derive(Debug, Clone, Default)]
pub struct CoverageMap {
    pub coverage: BTreeMap<String, BTreeMap<String, u64>>,
}

impl CoverageMap {
    pub fn new() -> Self {
        Self { coverage: BTreeMap::new() }
    }

    /// Record a count for a specific backend and operation.
    pub fn record(&mut self, backend: &str, operation: &str, count: u64) {
        self.coverage
            .entry(backend.to_string())
            .or_default()
            .insert(operation.to_string(), count);
    }

    /// Increment a count by 1.
    pub fn increment(&mut self, backend: &str, operation: &str) {
        let entry = self.coverage
            .entry(backend.to_string())
            .or_default()
            .entry(operation.to_string())
            .or_insert(0);
        *entry += 1;
    }

    /// Add a count to an existing entry.
    pub fn add(&mut self, backend: &str, operation: &str, delta: u64) {
        let entry = self.coverage
            .entry(backend.to_string())
            .or_default()
            .entry(operation.to_string())
            .or_insert(0);
        *entry += delta;
    }

    /// Merge another coverage map into this one (sums counts).
    pub fn merge(&mut self, other: &CoverageMap) {
        for (backend, ops) in &other.coverage {
            for (op, &count) in ops {
                self.add(backend, op, count);
            }
        }
    }

    /// Get the count for a specific backend and operation.
    pub fn get(&self, backend: &str, operation: &str) -> u64 {
        self.coverage
            .get(backend)
            .and_then(|ops| ops.get(operation))
            .copied()
            .unwrap_or(0)
    }

    /// List all backends that have coverage data.
    pub fn backends(&self) -> Vec<&str> {
        self.coverage.keys().map(|s| s.as_str()).collect()
    }

    /// List all operations for a given backend.
    pub fn operations(&self, backend: &str) -> Vec<&str> {
        self.coverage
            .get(backend)
            .map(|ops| ops.keys().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Total operations across all backends.
    pub fn total_operations(&self) -> usize {
        self.coverage.values().map(|ops| ops.len()).sum()
    }

    /// Serialize to a JSON string.
    pub fn to_json(&self) -> String {
        let mut backends = Vec::new();
        for (backend, ops) in &self.coverage {
            let mut op_entries = Vec::new();
            for (op, count) in ops {
                op_entries.push(format!("\"{}\":{}", op, count));
            }
            backends.push(format!("\"{}\":{{{}}}", backend, op_entries.join(",")));
        }
        format!("{{{}}}", backends.join(","))
    }
}
