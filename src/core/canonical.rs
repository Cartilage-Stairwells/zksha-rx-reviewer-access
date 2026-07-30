//! Canonical encoding — deterministic serialization for hashing.
//!
//! The evidence protocol requires that any two structurally identical
//! objects produce the same hash. This module provides a canonical
//! serialization scheme that is:
//!   1. Deterministic: same input → same bytes, always.
//!   2. Ordered: map keys are sorted lexicographically.
//!   3. Minimal: no whitespace, no optional fields absent.
//!
//! The canonical encoding is NOT a general serialization format.
//! It is specifically for computing identity hashes. The JSON
//! representation uses sorted keys and compact separators, matching
//! the Python evidence producers' `json.dumps(..., sort_keys=True,
//! separators=(",", ":"))` convention.

use std::collections::BTreeMap;

/// A value that can be canonically encoded for hashing.
///
/// The canonical encoding is JSON with sorted keys and compact
/// separators. This matches the Python evidence producers and
/// the JSON Schema hash computations.
pub trait Canonical {
    /// Encode this object into canonical JSON bytes.
    fn canonical_bytes(&self) -> Vec<u8>;

    /// Compute the SHA-256 hash of the canonical encoding.
    fn canonical_hash(&self) -> super::hash::Hash {
        super::hash::Hash::compute(&self.canonical_bytes())
    }
}

/// Encode a BTreeMap as canonical JSON.
/// Keys are already sorted (BTreeMap guarantees this).
pub fn encode_map(map: &BTreeMap<String, String>) -> Vec<u8> {
    let mut result = Vec::with_capacity(64);
    result.push(b'{');
    for (i, (k, v)) in map.iter().enumerate() {
        if i > 0 {
            result.push(b',');
        }
        encode_string(&mut result, k);
        result.push(b':');
        encode_string(&mut result, v);
    }
    result.push(b'}');
    result
}

/// Encode a string as canonical JSON (quoted, escaped).
fn encode_string(out: &mut Vec<u8>, s: &str) {
    out.push(b'"');
    for c in s.chars() {
        match c {
            '"' => { out.extend_from_slice(b"\\\""); }
            '\\' => { out.extend_from_slice(b"\\\\"); }
            '\n' => { out.extend_from_slice(b"\\n"); }
            '\r' => { out.extend_from_slice(b"\\r"); }
            '\t' => { out.extend_from_slice(b"\\t"); }
            c if c.is_control() => {
                out.extend_from_slice(format!("\\u{:04x}", c as u32).as_bytes());
            }
            c => { out.extend_from_slice(c.to_string().as_bytes()); }
        }
    }
    out.push(b'"');
}

/// Encode a sequence of (key, canonical_value) pairs as canonical JSON.
/// The caller must ensure keys are sorted.
pub fn encode_sorted_pairs(pairs: &[(&str, String)]) -> Vec<u8> {
    let mut result = Vec::with_capacity(128);
    result.push(b'{');
    for (i, (k, v)) in pairs.iter().enumerate() {
        if i > 0 {
            result.push(b',');
        }
        encode_string(&mut result, k);
        result.push(b':');
        // Values are already canonical JSON strings or raw values
        result.extend_from_slice(v.as_bytes());
    }
    result.push(b'}');
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn canonical_map_is_deterministic() {
        let mut m1 = BTreeMap::new();
        m1.insert("b".to_string(), "2".to_string());
        m1.insert("a".to_string(), "1".to_string());

        let mut m2 = BTreeMap::new();
        m2.insert("a".to_string(), "1".to_string());
        m2.insert("b".to_string(), "2".to_string());

        assert_eq!(encode_map(&m1), encode_map(&m2));
        // Keys are sorted regardless of insertion order
        let s = String::from_utf8(encode_map(&m1)).unwrap();
        assert_eq!(s, r#"{"a":"1","b":"2"}"#);
    }

    #[test]
    fn canonical_map_different_values_different_bytes() {
        let mut m1 = BTreeMap::new();
        m1.insert("a".to_string(), "1".to_string());

        let mut m2 = BTreeMap::new();
        m2.insert("a".to_string(), "2".to_string());

        assert_ne!(encode_map(&m1), encode_map(&m2));
    }
}
