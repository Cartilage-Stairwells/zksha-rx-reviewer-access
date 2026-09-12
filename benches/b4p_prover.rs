//! B4-P — end-to-end performance of this deterministic Fibonacci STARK
//! fixture (NOT a production-prover benchmark; no extrapolation permitted).
//!
//! Arms:
//! - zksha_avx512   : prove() with ZkshaDifAdapter (AVX-512 lane) as PCS DFT
//! - plonky3_radix2dit : prove() with Plonky3 Radix2Dit as PCS DFT
//!
//! Sizes: 2^10, 2^12, 2^14 trace rows (new fixtures; the frozen B2 64-row
//! fixture is an integration/equivalence gate only and is NOT timed).
//!
//! Correctness gate BEFORE timing: each arm proves and verifies once per
//! size; verify failure aborts the run. Both arms' proofs must also be
//! byte-identical (continuity with the frozen B2 invariance finding).
//!
//! Release/bench profile: p3-uni-stark's debug-only check_constraints is
//! compiled out — measured wall-time is the real proving path.

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
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
use std::time::Duration;

use avx512_butterfly::plonky3_adapter::ZkshaDifAdapter;

type Val = BabyBear;
type Perm = Poseidon2<Val, Poseidon2ExternalMatrixGeneral, DiffusionMatrixBabyBear, 16, 7>;
type MyHash = PaddingFreeSponge<Perm, 16, 8, 8>;
type MyCompress = TruncatedPermutation<Perm, 2, 8, 16>;
type ValMmcs =
    FieldMerkleTreeMmcs<<Val as Field>::Packing, <Val as Field>::Packing, MyHash, MyCompress, 8>;
type Challenge = BinomialExtensionField<Val, 4>;
type ChallengeMmcs = ExtensionMmcs<Val, Challenge, ValMmcs>;
type Challenger = DuplexChallenger<Val, Perm, 16, 8>;

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
        builder.when_first_row().assert_eq(a, AB::F::zero());
        builder.when_first_row().assert_eq(b, AB::F::one());
        builder.when_transition().assert_eq(b, a_next);
        builder.when_transition().assert_eq(a + b, b_next);
    }
}

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

fn deterministic_perm() -> Perm {
    let mut rng = StdRng::seed_from_u64(0x5eed_b2_e2);
    Perm::new_from_rng_128(Poseidon2ExternalMatrixGeneral, DiffusionMatrixBabyBear, &mut rng)
}

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
    let pcs = TwoAdicFriPcs::new(log2_ceil_usize(trace_height), dft, val_mmcs, fri_config);
    StarkConfig::new(pcs)
}

fn bench_b4p(c: &mut Criterion) {
    let mut group = c.benchmark_group("b4p_prove");
    group
        .sample_size(15)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(5));

    for log_n in [10usize, 12, 14] {
        let n = 1usize << log_n;
        let perm = deterministic_perm();
        let trace = generate_trace(n);

        // ---------------- Correctness gates before timing ----------------
        // Arm A: adapter-backed
        let adapter = ZkshaDifAdapter::avx512();
        let tracker_handle = adapter.tracker.clone();
        let config_a = build_config(adapter, trace.height(), &perm);
        let proof_a = {
            let mut ch = Challenger::new(perm.clone());
            prove(&config_a, &FibonacciAir {}, &mut ch, trace.clone(), &vec![])
        };
        {
            let mut ch = Challenger::new(perm.clone());
            verify(&config_a, &FibonacciAir {}, &mut ch, &proof_a, &vec![])
                .unwrap_or_else(|e| panic!("gate: adapter proof must verify (2^{log_n}): {e:?}"));
        }
        let avx_calls = tracker_handle.borrow().avx512_calls;
        assert!(avx_calls > 0, "gate: AVX-512 backend must engage (got {avx_calls})");

        // Arm B: Radix2Dit-backed
        let config_b = build_config(Radix2Dit::<Val>::default(), trace.height(), &perm);
        let proof_b = {
            let mut ch = Challenger::new(perm.clone());
            prove(&config_b, &FibonacciAir {}, &mut ch, trace.clone(), &vec![])
        };
        {
            let mut ch = Challenger::new(perm.clone());
            verify(&config_b, &FibonacciAir {}, &mut ch, &proof_b, &vec![])
                .unwrap_or_else(|e| panic!("gate: Radix2Dit proof must verify (2^{log_n}): {e:?}"));
        }

        // Byte-identity continuity check (B2 finding must hold at every size)
        let bytes_a = serde_json::to_vec(&proof_a).expect("serialize A");
        let bytes_b = serde_json::to_vec(&proof_b).expect("serialize B");
        assert_eq!(
            bytes_a, bytes_b,
            "gate: proof-output byte identity must hold at 2^{log_n}"
        );
        println!(
            "gates PASS @ 2^{log_n}: both proofs verify, byte-identical ({} bytes), avx512_calls={}",
            bytes_a.len(),
            avx_calls
        );
        // ------------------------------------------------------------------

        let size_label = format!("n2^{log_n}");

        group.bench_with_input(
            BenchmarkId::new("zksha_avx512", &size_label),
            &trace,
            |b, t| {
                b.iter_batched(
                    || (t.clone(), Challenger::new(perm.clone())),
                    |(t, mut ch)| {
                        let proof = prove(&config_a, &FibonacciAir {}, &mut ch, t, &vec![]);
                        criterion::black_box(proof);
                    },
                    BatchSize::PerIteration,
                )
            },
        );
        group.bench_with_input(
            BenchmarkId::new("plonky3_radix2dit", &size_label),
            &trace,
            |b, t| {
                b.iter_batched(
                    || (t.clone(), Challenger::new(perm.clone())),
                    |(t, mut ch)| {
                        let proof = prove(&config_b, &FibonacciAir {}, &mut ch, t, &vec![]);
                        criterion::black_box(proof);
                    },
                    BatchSize::PerIteration,
                )
            },
        );
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench_b4p
}
criterion_main!(benches);
