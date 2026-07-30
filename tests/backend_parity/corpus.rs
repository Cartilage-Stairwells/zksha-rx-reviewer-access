//! Backend Parity Corpus — shared test data for Issue #4.
//!
//! All values are Montgomery-encoded BabyBear residues (`xR mod p`, R = 2³²).
//! The corpus is deterministic: same seed → same inputs → reproducible failures.
//!
//! Categories:
//!   1. Boundary values — where representation assumptions break
//!   2. Reduction stress — near Montgomery reduction thresholds
//!   3. Adversarial near-modulus — where vectorized implementations fail
//!   4. Deterministic random — fixed seed, reproducible corpus hash

use avx512_butterfly::field::babybear::constants::{BABYBEAR_P, BABYBEAR_R_MOD_P, BABYBEAR_NEG_INV};

pub const P: u32 = BABYBEAR_P;


// ---------------------------------------------------------------------------
// 6. Backend identity — environment context for evidence artifacts
// ---------------------------------------------------------------------------

/// Capture the execution environment for evidence records.
///
/// Prevents a parity result from losing environmental context:
/// which backends were actually compiled in, which CPU features
/// were available, and which Rust toolchain produced the binary.
pub struct EnvInfo {
    pub arch: &'static str,
    pub os: &'static str,
    pub rustc: String,
    pub has_avx512_compile: bool,
    pub has_avx512_runtime: bool,
}

impl EnvInfo {
    pub fn capture() -> Self {
        Self {
            arch: std::env::consts::ARCH,
            os: std::env::consts::OS,
            rustc: option_env!("RUSTC_VERSION").unwrap_or("unknown").to_string(),
            has_avx512_compile: cfg!(all(target_arch = "x86_64", target_feature = "avx512f")),
            has_avx512_runtime: {
                #[cfg(target_arch = "x86_64")]
                {
                    avx512_butterfly::avx512_butterfly_32bit::is_avx512_supported()
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    false
                }
            },
        }
    }

    /// Emit as a JSON fragment for evidence artifacts.
    pub fn to_json(&self) -> String {
        format!(
            "\"arch\":\"{}\",\"os\":\"{}\",\"rustc\":\"{}\",\
             \"avx512_compiled\":{},\"avx512_runtime\":{}",
            self.arch, self.os, self.rustc,
            self.has_avx512_compile, self.has_avx512_runtime,
        )
    }
}

/// Track whether AVX-512 was actually exercised during the test run.
/// This is a process-global flag — any AVX-512 test that executes
/// at least one SIMD operation sets it to true.
use std::sync::atomic::{AtomicBool, Ordering};

static AVX512_EXECUTED: AtomicBool = AtomicBool::new(false);

/// Mark that the AVX-512 path was actually exercised (not skipped).
pub fn mark_avx512_executed() {
    AVX512_EXECUTED.store(true, Ordering::Relaxed);
}

/// Check whether any AVX-512 test actually ran.
pub fn avx512_was_executed() -> bool {
    AVX512_EXECUTED.load(Ordering::Relaxed)
}

// ---------------------------------------------------------------------------
// 1. Boundary values
// ---------------------------------------------------------------------------

/// Values at the edges of the valid residue range [0, p).
///
/// These are the inputs where lazy reduction, conditional subtract,
/// and signed/unsigned interpretation are most likely to differ
/// between scalar and vectorized implementations.
pub const BOUNDARY_VALUES: &[u32] = &[
    0,                // additive identity
    1,                // multiplicative identity (canonical 1, not Mont 1)
    2,                // smallest non-trivial
    P / 2,            // midpoint — tests carry into the subtract branch
    P / 3,            // asymmetric split
    P / 4,            // quarter point
    P - 3,            // just below the max
    P - 2,            // second-highest residue
    P - 1,            // max residue — (P-1)² is the largest possible product
    BABYBEAR_R_MOD_P, // R mod p = 1 in Montgomery domain (Montgomery identity)
];

/// Butterfly test triplets: (a, b, w) — all Montgomery-encoded.
///
/// These cover the interesting interaction spaces:
/// - zero inputs (tests annihilation)
/// - max residue inputs (tests lazy reduction bounds)
/// - identity twiddle (w=R mod p → Montgomery 1 → simple add/sub)
/// - max twiddle (w=P-1 → near-modulus multiplication)
/// - asymmetric combinations
pub const BUTTERFLY_CASES: &[(u32, u32, u32)] = &[
    // Trivial
    (0, 0, BABYBEAR_R_MOD_P),       // all zeros, identity twiddle
    (0, 0, P - 1),                  // all zeros, max twiddle
    (1, 0, BABYBEAR_R_MOD_P),       // identity a, zero b
    (0, 1, BABYBEAR_R_MOD_P),       // zero a, identity b

    // Identity
    (1, 1, BABYBEAR_R_MOD_P),       // all ones, identity twiddle
    (1, 1, P - 1),                  // all ones, max twiddle

    // Max residue — where lazy reduction is stressed
    (P - 1, P - 1, P - 1),          // everything max
    (P - 1, 1, BABYBEAR_R_MOD_P),   // max a, unit b
    (1, P - 1, BABYBEAR_R_MOD_P),   // unit a, max b
    (P - 1, P - 1, BABYBEAR_R_MOD_P), // max a, max b, identity twiddle
    (P - 2, P - 1, P - 1),         // near-max

    // Midpoint and asymmetric
    (P / 2, P / 2, BABYBEAR_R_MOD_P), // midpoint, identity
    (P / 2, P / 3, P / 5),          // asymmetric
    (P / 2, P - 1, P - 1),         // midpoint × max × max

    // Near reduction thresholds
    (P - 1, 2, P - 1),             // (P-1)*2 near 2P boundary
    (P / 2, 2, BABYBEAR_R_MOD_P),   // P/2 * 2 = P - 1 (since P is odd)
    (P - 1, P / 2, P - 1),         // max × midpoint × max
];

/// Montgomery multiplication test pairs: (a, b).
///
/// These target the conditional subtraction in Montgomery reduction:
/// u = (t + m*p) >> 32; if u >= p then u -= p.
/// The boundary is u == p (subtracts to 0) vs u == p-1 (no subtract).
pub const MUL_PAIRS: &[(u32, u32)] = &[
    (0, 0),                       // zero annihilation
    (0, P - 1),                   // zero × max
    (1, 1),                       // identity
    (1, P - 1),                   // near modulus
    (2, P - 1),                   // 2(P-1) = 2P-2, near 2P
    (P - 1, 2),                   // same, commutative
    (P - 1, P - 1),               // max residue squared = P²-2P+1
    (P - 2, P - 1),               // (P-2)(P-1) = P²-3P+2
    (P / 2, P / 2),               // P²/4 — midpoint product
    (P / 2, P - 1),               // P/2 × (P-1) — asymmetric near max
    (P / 3, P - 1),               // P/3 × (P-1)
    (BABYBEAR_R_MOD_P, BABYBEAR_R_MOD_P), // R² mod p (Montgomery identity squared)
];

// ---------------------------------------------------------------------------
// 2. Reduction stress — near-boundary scan
// ---------------------------------------------------------------------------

/// Compute the Montgomery reduction intermediate `u` for a product a*b.
///
/// This is the value before the conditional subtract:
///   t = a*b (u64)
///   m = (t as u32).wrapping_mul(neg_inv)
///   u = (t + m*p) >> 32
///
/// The conditional subtract fires when u >= p. The boundary is u == p
/// (subtracts to 0) vs u == p-1 (no subtract, returns p-1).
#[inline]
pub fn reduction_intermediate(a: u32, b: u32) -> u32 {
    let t: u64 = (a as u64) * (b as u64);
    let m: u32 = (t as u32).wrapping_mul(BABYBEAR_NEG_INV);
    let u: u32 = ((t + (m as u64) * (P as u64)) >> 32) as u32;
    u
}

/// Scan for pairs (a, b) where the Montgomery reduction intermediate `u`
/// is within `threshold` of `p`. These are the cases where the conditional
/// subtract is at the boundary — the most likely place for vectorized
/// implementations to diverge from scalar.
///
/// Returns (a, b, u) tuples sorted by proximity to p.
/// Scans a in [0, max_val) and b in [a, max_val) — limited range for CI speed.
pub fn near_boundary_reduction_cases(max_val: u32, threshold: u32, max_results: usize) -> Vec<(u32, u32, u32)> {
    let scan_limit = max_val.min(P);
    let mut results: Vec<(u32, u32, u32)> = Vec::new();

    for a in 0..scan_limit {
        for b in a..scan_limit {
            let u = reduction_intermediate(a, b);
            if u >= P.saturating_sub(threshold) && u <= P.saturating_add(threshold) {
                results.push((a, b, u));
                if results.len() >= max_results {
                    results.sort_by_key(|&(_, _, u)| {
                        let dist = if u >= P { u - P } else { P - u };
                        dist
                    });
                    return results;
                }
            }
        }
    }

    results.sort_by_key(|&(_, _, u)| {
        let dist = if u >= P { u - P } else { P - u };
        dist
    });
    results
}

// ---------------------------------------------------------------------------
// 3. Deterministic random generator (LCG — same params as ntt_equivalence.rs)
// ---------------------------------------------------------------------------

/// Deterministic LCG for reproducible test inputs.
///
/// Same parameters as `ntt_equivalence.rs::Lcg` — ensures the corpus
/// is consistent across test files.
pub struct Lcg {
    state: u64,
}

impl Lcg {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Generate a canonical BabyBear value in [0, p).
    pub fn next_canonical(&mut self) -> u32 {
        self.state = self.state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.state >> 33) as u32) % P
    }

    /// Generate a Montgomery-encoded BabyBear value.
    pub fn next_montgomery(&mut self) -> u32 {
        use avx512_butterfly::field::babybear::reference::to_montgomery_reference;
        to_montgomery_reference(self.next_canonical())
    }
}

/// Canonical seed for the backend parity corpus.
/// Any test using deterministic random data should derive from this seed.
pub const CORPUS_SEED: u64 = 0x1544_BEEF_CAFE_4242;

// ---------------------------------------------------------------------------
// 4. Failure record — structured output for evidence chain
// ---------------------------------------------------------------------------

/// Structured failure record for backend parity mismatches.
///
/// When a backend produces a different result than the reference oracle,
/// this record captures the full context needed to reproduce and diagnose
/// the failure. The format is designed to be machine-parseable.
#[derive(Debug, Clone)]
pub struct FailureRecord {
    pub seed: u64,
    pub input: String,
    pub backend: &'static str,
    pub operation: &'static str,
    pub expected: String,
    pub actual: String,
    pub representation_domain: &'static str,
}

impl FailureRecord {
    pub fn mul(seed: u64, a: u32, b: u32, backend: &'static str, expected: u32, actual: u32) -> Self {
        Self {
            seed,
            input: format!("({}, {})", a, b),
            backend,
            operation: "montgomery_mul",
            expected: format!("{}", expected),
            actual: format!("{}", actual),
            representation_domain: "Montgomery",
        }
    }

    pub fn butterfly(seed: u64, a: u32, b: u32, w: u32, backend: &'static str,
                     expected: (u32, u32), actual: (u32, u32)) -> Self {
        Self {
            seed,
            input: format!("({}, {}, {})", a, b, w),
            backend,
            operation: "butterfly",
            expected: format!("({}, {})", expected.0, expected.1),
            actual: format!("({}, {})", actual.0, actual.1),
            representation_domain: "Montgomery",
        }
    }

    pub fn ntt_stage(seed: u64, stage: usize, index: usize, backend: &'static str,
                     expected: u32, actual: u32) -> Self {
        Self {
            seed,
            input: format!("stage={}, index={}", stage, index),
            backend,
            operation: "ntt_stage",
            expected: format!("{}", expected),
            actual: format!("{}", actual),
            representation_domain: "Montgomery",
        }
    }

    /// Emit the failure record as a structured string to stderr.
    pub fn emit(&self) {
        eprintln!(
            "BACKEND_PARITY_FAILURE {{\
            \"seed\":\"0x{:016X}\",\
            \"input\":\"{}\",\
            \"backend\":\"{}\",\
            \"operation\":\"{}\",\
            \"expected\":\"{}\",\
            \"actual\":\"{}\",\
            \"representation_domain\":\"{}\"}}",
            self.seed,
            self.input,
            self.backend,
            self.operation,
            self.expected,
            self.actual,
            self.representation_domain,
        );
    }
}



// ---------------------------------------------------------------------------
// 7. Coverage tracking — real evidence, not estimates
// ---------------------------------------------------------------------------

use std::sync::atomic::AtomicU64;

/// Global coverage counters. Each test increments these as it executes.
/// The evidence test reads them to produce the EvidenceContractV1 artifact.
static SCALAR_MUL_COUNT: AtomicU64 = AtomicU64::new(0);
static SCALAR_BUTTERFLY_COUNT: AtomicU64 = AtomicU64::new(0);
static AVX512_BUTTERFLY_COUNT: AtomicU64 = AtomicU64::new(0);
static SCALAR_NTT_STAGE_COUNT: AtomicU64 = AtomicU64::new(0);
static AVX512_NTT_STAGE_COUNT: AtomicU64 = AtomicU64::new(0);
static ORACLE_MUL_COUNT: AtomicU64 = AtomicU64::new(0);
static ORACLE_BUTTERFLY_COUNT: AtomicU64 = AtomicU64::new(0);
static ORACLE_NTT_STAGE_COUNT: AtomicU64 = AtomicU64::new(0);

pub fn record_scalar_mul(count: u64) {
    SCALAR_MUL_COUNT.fetch_add(count, Ordering::Relaxed);
}
pub fn record_scalar_butterfly(count: u64) {
    SCALAR_BUTTERFLY_COUNT.fetch_add(count, Ordering::Relaxed);
}
pub fn record_avx512_butterfly(count: u64) {
    AVX512_BUTTERFLY_COUNT.fetch_add(count, Ordering::Relaxed);
}
pub fn record_scalar_ntt_stage(count: u64) {
    SCALAR_NTT_STAGE_COUNT.fetch_add(count, Ordering::Relaxed);
}
pub fn record_avx512_ntt_stage(count: u64) {
    AVX512_NTT_STAGE_COUNT.fetch_add(count, Ordering::Relaxed);
}
pub fn record_oracle_mul(count: u64) {
    ORACLE_MUL_COUNT.fetch_add(count, Ordering::Relaxed);
}
pub fn record_oracle_butterfly(count: u64) {
    ORACLE_BUTTERFLY_COUNT.fetch_add(count, Ordering::Relaxed);
}
pub fn record_oracle_ntt_stage(count: u64) {
    ORACLE_NTT_STAGE_COUNT.fetch_add(count, Ordering::Relaxed);
}

/// Collect coverage into a CoverageMap for the evidence contract.
pub fn collect_coverage() -> avx512_butterfly::instrument::coverage::CoverageMap {
    use avx512_butterfly::instrument::coverage::CoverageMap;
    let mut map = CoverageMap::new();

    let s_mul = SCALAR_MUL_COUNT.load(Ordering::Relaxed);
    let s_bf = SCALAR_BUTTERFLY_COUNT.load(Ordering::Relaxed);
    let a_bf = AVX512_BUTTERFLY_COUNT.load(Ordering::Relaxed);
    let s_ntt = SCALAR_NTT_STAGE_COUNT.load(Ordering::Relaxed);
    let a_ntt = AVX512_NTT_STAGE_COUNT.load(Ordering::Relaxed);
    let o_mul = ORACLE_MUL_COUNT.load(Ordering::Relaxed);
    let o_bf = ORACLE_BUTTERFLY_COUNT.load(Ordering::Relaxed);
    let o_ntt = ORACLE_NTT_STAGE_COUNT.load(Ordering::Relaxed);

    map.record("oracle", "mul", o_mul);
    map.record("oracle", "butterfly", o_bf);
    map.record("oracle", "ntt_stage", o_ntt);
    map.record("scalar", "mul", s_mul);
    map.record("scalar", "butterfly", s_bf);
    map.record("scalar", "ntt_stage", s_ntt);
    map.record("avx512", "butterfly", a_bf);
    map.record("avx512", "ntt_stage", a_ntt);

    map
}

// ---------------------------------------------------------------------------
// 5. Shared helpers
// ---------------------------------------------------------------------------

/// Compare a backend result against the oracle and emit a failure record on mismatch.
/// Returns true if the results match.
pub fn assert_mul_parity(seed: u64, a: u32, b: u32, backend: &'static str,
                         oracle: u32, actual: u32) -> bool {
    if oracle != actual {
        FailureRecord::mul(seed, a, b, backend, oracle, actual).emit();
        false
    } else {
        true
    }
}

/// Compare a butterfly result against the oracle and emit a failure record on mismatch.
/// Returns true if both outputs match.
pub fn assert_butterfly_parity(seed: u64, a: u32, b: u32, w: u32, backend: &'static str,
                                oracle: (u32, u32), actual: (u32, u32)) -> bool {
    if oracle != actual {
        FailureRecord::butterfly(seed, a, b, w, backend, oracle, actual).emit();
        false
    } else {
        true
    }
}
