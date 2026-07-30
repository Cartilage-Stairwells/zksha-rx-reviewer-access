//! Hash primitive — the identity unit.
//!
//! A Hash is a SHA-256 digest. It is the only identity mechanism in the core.
//! Objects derive identity through canonical encoding → hash. The object is
//! not self-authenticating: you cannot claim a hash, you must compute it.

use std::fmt;

/// A SHA-256 hash. The identity primitive of the evidence protocol.
///
/// Two objects with the same Hash are considered to have the same identity
/// *for the purpose of evidence comparison*. This does NOT mean they are
/// mathematically equal — it means they share a canonical encoding.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Hash([u8; 32]);

impl Hash {
    /// Compute a Hash from arbitrary bytes.
    pub fn compute(data: &[u8]) -> Self {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        Hash(bytes)
    }

    /// Create a Hash from a raw 32-byte array.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Hash(bytes)
    }

    /// Return the raw 32 bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Return the hex representation with "sha256:" prefix.
    pub fn to_prefixed_hex(&self) -> String {
        format!("sha256:{}", self.to_hex())
    }

    /// Return the hex representation without prefix.
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Parse a "sha256:" prefixed hex string.
    pub fn from_prefixed_hex(s: &str) -> Option<Self> {
        let hex = s.strip_prefix("sha256:")?;
        Self::from_hex(hex)
    }

    /// Parse a raw hex string (64 chars).
    pub fn from_hex(hex: &str) -> Option<Self> {
        if hex.len() != 64 {
            return None;
        }
        let mut bytes = [0u8; 32];
        for i in 0..32 {
            let byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
            bytes[i] = byte;
        }
        Some(Hash(bytes))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_prefixed_hex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_compute_is_deterministic() {
        let h1 = Hash::compute(b"hello");
        let h2 = Hash::compute(b"hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_different_inputs_different_hashes() {
        let h1 = Hash::compute(b"hello");
        let h2 = Hash::compute(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn hash_roundtrip_hex() {
        let h = Hash::compute(b"test data");
        let hex = h.to_prefixed_hex();
        let parsed = Hash::from_prefixed_hex(&hex).unwrap();
        assert_eq!(h, parsed);
    }

    #[test]
    fn hash_display_is_prefixed() {
        let h = Hash::compute(b"x");
        let s = format!("{}", h);
        assert!(s.starts_with("sha256:"));
        assert_eq!(s.len(), 71); // "sha256:" (7) + 64 hex chars
    }
}
