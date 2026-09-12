//! B2 — Execution-Path Evidence: real STARK proof through the AVX-512 NTT adapter.
//!
//! This is the canonical Plonky3 0.1.0 Fibonacci STARK (same structure as
//! p3-uni-stark's own `fib_air.rs` test), with exactly one substitution:
//! the DFT injected into `TwoAdicFriPcs` is our `ZkshaDifAdapter`
//! (AVX-512 lane) instead of Plonky3's `Radix2DitParallel`.
//!
//! Proven here:
//! 1. The full prove() path executes with the adapter as the DFT backend
//!    of the polynomial commitment scheme.
//! 2. The shared adapter tracker records AVX-512 NTT calls made *inside*
//!    proof generation (trace LDE + quotient LDE commit paths).
//! 3. The resulting proof verifies.
//! 4. Proof-output invariance: the identical fixture driven through
//!    Plonky3's `Radix2Dit` produces a byte-identical serialized proof
//!    (same seeded permutation, same trace, same transcript — the DFT is
//!    the only variable, and B1 proved bit-identical DFT output).
//!
//! NOT claimed here: performance, end-to-end prover speedup, or anything
//! about Experiment A. Correctness and execution-path linkage only.

use p3_air::{Air, AirBuilder, BaseAir};
use p3_baby_bear::{BabyBear, DiffusionMatrixBabyBear};
use p3_challenger::DuplexChallenger;
use p3_commit::ExtensionMmcs;
use p3_dft::{Radix2Dit, TwoAdicSubgroupDft};
use p3_field::extension::BinomialExtensionField;
use p3_field::{AbstractField, Field};
use p3_fri::{FriConfig, TwoAdicFriPcs};
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;
use p3_merkle_tree::FieldMerkleTreeMmcs;
use p3_poseidon2::{Poseidon2, Poseidon2ExternalMatrixGeneral};
use p3_symmetric::{PaddingFreeSponge, TruncatedPermutation};
use p3_uni_stark::{prove, verify, StarkConfig};
use p3_util::log2_ceil_usize;
use rand::rngs::StdRng;
use rand::SeedableRng;

use avx512_butterfly::plonky3_adapter::{is_avx512_supported, ZkshaDifAdapter};

/// The AIR: Fibonacci, two columns (left, right).
/// Row i: left = fib(i), right = fib(i+1).
pub struct FibonacciAir;

impl<F> BaseAir<F> for FibonacciAir {
    fn width(&self) -> usize {
        2
    }
}

impl<AB: AirBuilder> Air<AB> for FibonacciAir {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local = main.row_slice(0);
        let next = main.row_slice(1);

        let (a, b) = (local[0], local[1]);
        let (a_next, b_next) = (next[0], next[1]);

        // First row: a = 0, b = 1.
        builder.when_first_row().assert_eq(a, AB::F::zero());
        builder.when_first_row().assert_eq(b, AB::F::one());

        // Transition: a' = b, b' = a + b.
        builder.when_transition().assert_eq(b, a_next);
        builder.when_transition().assert_eq(a + b, b_next);
    }
}

/// Deterministic Fibonacci trace, two columns, `n` rows (n = power of two).
fn generate_trace(n: usize) -> RowMajorMatrix<BabyBear> {
    assert!(n.is_power_of_two());
    let mut values = vec![BabyBear::zero(); n * 2];
    let mut a = BabyBear::zero();
    let mut b = BabyBear::one();
    for i in 0..n {
        values[2 * i] = a;
        values[2 * i + 1] = b;
        let (a2, b2) = (b, a + b);
        a = a2;
        b = b2;
    }
    RowMajorMatrix::new(values, 2)
}

// ---------------------------------------------------------------------------
// Type configuration — identical to p3-uni-stark's fib_air.rs, except Dft.
// ---------------------------------------------------------------------------

type Val = BabyBear;
type Perm = Poseidon2<Val, Poseidon2ExternalMatrixGeneral, DiffusionMatrixBabyBear, 16, 7>;
type MyHash = PaddingFreeSponge<Perm, 16, 8, 8>;
type MyCompress = TruncatedPermutation<Perm, 2, 8, 16>;
type ValMmcs =
    FieldMerkleTreeMmcs<<Val as Field>::Packing, <Val as Field>::Packing, MyHash, MyCompress, 8>;
type Challenge = BinomialExtensionField<Val, 4>;
type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
type Challenger = DuplexChallenger<Val, Perm, 16, 8>;

/// Fully deterministic Poseidon2 permutation (fixed seed — no thread_rng).
fn deterministic_perm() -> Perm {
    let mut rng = StdRng::seed_from_u64(0x5eed_b2_e2);
    Perm::new_from_rng_128(Poseidon2ExternalMatrixGeneral, DiffusionMatrixBabyBear, &mut rng)
}

/// Build a STARK config around a generic DFT backend.
/// The ONLY variable between the two comparison runs is this DFT.
fn build_config<Dft: TwoAdicSubgroupDft<Val>>(
    dft: Dft,
    trace_height: usize,
    perm: &Perm,
) -> StarkConfig<TwoAdicFriPcs<Val, Dft, ValMmcs, ChallengeMmcs>, Challenge, Challenger> {
    let hash = MyHash::new(perm.clone());
    let compress = MyCompress::new(perm.clone());
    let val_mmcs = ValMmcs::new(hash, compress);
    let challenge_mmcs = ChallengeMmcs::new(val_mmcs.clone());
    let fri_config = FriConfig {
        log_blowup: 2,
        num_queries: 28,
        proof_of_work_bits: 8,
        mmcs: challenge_mmcs,
    };
    let pcs = TwoAdicFriPcs::new(
        log2_ceil_usize(trace_height),
        dft,
        val_mmcs,
        fri_config,
    );
    StarkConfig::new(pcs)
}

const TRACE_ROWS: usize = 1 << 6;

// ---------------------------------------------------------------------------
// Evidence tests
// ---------------------------------------------------------------------------

#[test]
fn test_prove_via_zksha_adapter_reaches_avx512_and_verifies() {
    let perm = deterministic_perm();
    let trace = generate_trace(TRACE_ROWS);

    // Construct the adapter and retain a shared tracker handle BEFORE
    // ownership moves into the PCS. The PCS may clone the adapter freely —
    // every clone increments the same shared counters.
    let adapter = ZkshaDifAdapter::avx512();
    let tracker_handle = adapter.tracker.clone();
    assert!(
        is_avx512_supported(),
        "host must support AVX-512 for this evidence test"
    );

    let config = build_config(adapter, trace.height(), &perm);

    // Reset counters, then prove.
    {
        let mut t = tracker_handle.borrow_mut();
        *t = Default::default();
        t.avx512_available = is_avx512_supported();
    }

    let mut challenger = Challenger::new(perm.clone());
    let proof = prove(&config, &FibonacciAir {}, &mut challenger, trace, &vec![]);

    // Read the tracker AFTER proof generation: these calls happened inside
    // the prove() execution (PCS commit → coset_lde_batch → dft_batch).
    let t = tracker_handle.borrow();
    println!(
        "tracker during prove(): reference_calls={}, scalar_calls={}, avx512_calls={}",
        t.reference_calls, t.scalar_calls, t.avx512_calls
    );
    assert!(
        t.avx512_calls > 0,
        "AVX-512 NTT lane must be exercised inside proof generation \
         (got reference={}, scalar={}, avx512={})",
        t.reference_calls,
        t.scalar_calls,
        t.avx512_calls
    );
    assert_eq!(
        t.reference_calls, 0,
        "adapter is configured for AVX-512; reference lane must be idle"
    );
    assert_eq!(
        t.scalar_calls, 0,
        "AVX-512 is available on this host; scalar fallback must not engage"
    );

    // Verify the proof produced by the adapter-backed pipeline.
    let mut verifier_challenger = Challenger::new(perm.clone());
    verify(
        &config,
        &FibonacciAir {},
        &mut verifier_challenger,
        &proof,
        &vec![],
    )
    .expect("verification of adapter-backed STARK proof failed");
    println!("adapter-backed STARK proof: VERIFIED");
}

#[test]
fn test_proof_output_invariance_vs_radix2dit() {
    let perm = deterministic_perm();
    let trace = generate_trace(TRACE_ROWS);

    // Proof A: through our adapter (AVX-512 lane).
    let adapter = ZkshaDifAdapter::avx512();
    let tracker_handle = adapter.tracker.clone();
    let config_a = build_config(adapter, trace.height(), &perm);
    let mut challenger_a = Challenger::new(perm.clone());
    let proof_a = prove(
        &config_a,
        &FibonacciAir {},
        &mut challenger_a,
        trace.clone(),
        &vec![],
    );

    // Verify A independently of the invariance claim.
    let mut vch_a = Challenger::new(perm.clone());
    verify(&config_a, &FibonacciAir {}, &mut vch_a, &proof_a, &vec![])
        .expect("verification of adapter proof failed");

    // Proof B: identical fixture, Plonky3's Radix2Dit as the DFT.
    let config_b = build_config(Radix2Dit::<Val>::default(), trace.height(), &perm);
    let mut challenger_b = Challenger::new(perm.clone());
    let proof_b = prove(
        &config_b,
        &FibonacciAir {},
        &mut challenger_b,
        trace.clone(),
        &vec![],
    );

    // Verify B independently.
    let mut vch_b = Challenger::new(perm);
    verify(&config_b, &FibonacciAir {}, &mut vch_b, &proof_b, &vec![])
        .expect("verification of Radix2Dit proof failed");

    // Confirm the adapter was actually exercised in run A (not idle).
    assert!(
        tracker_handle.borrow().avx512_calls > 0,
        "adapter tracker must show AVX-512 activity in the invariance run"
    );

    // Strongest valid invariance comparison: full serialized proof bytes.
    let bytes_a = serde_json::to_vec(&proof_a).expect("serializing adapter proof");
    let bytes_b = serde_json::to_vec(&proof_b).expect("serializing Radix2Dit proof");
    println!(
        "proof sizes: adapter={} bytes, Radix2Dit={} bytes",
        bytes_a.len(),
        bytes_b.len()
    );

    if bytes_a == bytes_b {
        println!(
            "PROOF-OUTPUT INVARIANCE: BYTE-IDENTICAL ({} bytes). \
             Transcript, commitments, openings, and FRI proof all identical.",
            bytes_a.len()
        );
    } else {
        let mut first_diff = None;
        for (i, (x, y)) in bytes_a.iter().zip(bytes_b.iter()).enumerate() {
            if x != y {
                first_diff = Some(i);
                break;
            }
        }
        eprintln!(
            "NOT byte-identical. lengths: {} vs {}, first divergence at byte {:?}",
            bytes_a.len(),
            bytes_b.len(),
            first_diff
        );
    }
    assert_eq!(
        bytes_a, bytes_b,
        "proof outputs must be byte-identical: B1 proved bit-identical DFT output, \
         so the full Fiat-Shamir transcript must agree"
    );
}

#[test]
fn test_b2_evidence_summary() {
    println!("\n=== B2 EXECUTION-PATH EVIDENCE SUMMARY ===");
    println!("Fixture: Plonky3 0.1.0 Fibonacci STARK (p3-uni-stark fib_air structure)");
    println!(
        "Trace: {} rows x 2 cols, FRI log_blowup=2, 28 queries, PoW 8 bits",
        TRACE_ROWS
    );
    println!("Hash/compress: Poseidon2-16 (deterministic seed 0x5eed_b2e2)");
    println!("Challenger: DuplexChallenger (Fiat-Shamir)");
    println!("PCS: TwoAdicFriPcs with DFT = ZkshaDifAdapter (AVX-512 lane)");
    println!("Host AVX-512: {}", is_avx512_supported());
    println!("No performance claims. Experiment A untouched. main untouched.");
    println!("=== END B2 EVIDENCE ===\n");
}
