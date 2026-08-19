//! Plonky3 Integration Test — B1 Backend Linkage
//!
//! This test proves that:
//! 1. The ZkshaDifAdapter implements TwoAdicSubgroupDft<BabyBear>
//! 2. Its output matches Plonky3's reference Radix2Dit implementation
//! 3. The AVX-512 backend is actually exercised when available
//! 4. The adapter compiles and runs against real Plonky3 v0.1.0 crates
//!
//! No performance claims are made — this is purely a linkage/correctness test.

use avx512_butterfly::plonky3_adapter::{ZkshaDifAdapter, is_avx512_supported};
use p3_field::PrimeField32;
use p3_baby_bear::BabyBear;
use p3_dft::{TwoAdicSubgroupDft, Radix2Dit};
use p3_field::AbstractField;
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;

/// Generate a deterministic input vector of BabyBear values.
fn make_input(log_n: usize, seed: u32) -> Vec<BabyBear> {
    let n = 1usize << log_n;
    (0..n)
        .map(|i| {
            let val = (i as u32).wrapping_mul(0x12345678).wrapping_add(seed).wrapping_mul(0x456789AB);
            BabyBear::from_canonical_u32(val % 0x78000001)
        })
        .collect()
}

/// Compare two BabyBear vectors element-by-element.
fn assert_eq_babybear(a: &[BabyBear], b: &[BabyBear], label: &str) {
    assert_eq!(a.len(), b.len(), "{}: length mismatch {} vs {}", label, a.len(), b.len());
    let mut mismatches = 0;
    for i in 0..a.len() {
        if a[i] != b[i] {
            if mismatches < 5 {
                eprintln!("  {} mismatch at {}: {} vs {}", label, i,
                    a[i].as_canonical_u32(), b[i].as_canonical_u32());
            }
            mismatches += 1;
        }
    }
    assert_eq!(mismatches, 0, "{}: {} mismatches out of {} elements", label, mismatches, a.len());
}

#[test]
fn test_adapter_compiles() {
    // If this test runs, the adapter compiled against real Plonky3 crates.
    let _adapter = ZkshaDifAdapter::reference();
    let _adapter_scalar = ZkshaDifAdapter::scalar();
    let _adapter_avx512 = ZkshaDifAdapter::avx512();
}

#[test]
fn test_adapter_matches_plonky3_reference_backend() {
    let log_n = 8; // n = 256
    let input = make_input(log_n, 42);

    // Plonky3's reference DFT
    let plonky3_dft = Radix2Dit::<BabyBear>::default();
    let plonky3_output = plonky3_dft.dft(input.clone());

    // Our adapter using the reference backend
    let adapter = ZkshaDifAdapter::reference();
    let adapter_output = adapter.dft(input.clone());

    assert_eq_babybear(&plonky3_output, &adapter_output, "reference vs Plonky3 Radix2Dit");
}

#[test]
fn test_adapter_matches_plonky3_scalar_backend() {
    let log_n = 8;
    let input = make_input(log_n, 99);

    let plonky3_dft = Radix2Dit::<BabyBear>::default();
    let plonky3_output = plonky3_dft.dft(input.clone());

    let adapter = ZkshaDifAdapter::scalar();
    let adapter_output = adapter.dft(input.clone());

    assert_eq_babybear(&plonky3_output, &adapter_output, "scalar vs Plonky3 Radix2Dit");
}

#[test]
fn test_adapter_matches_plonky3_avx512_backend() {
    let log_n = 8;
    let input = make_input(log_n, 137);

    let plonky3_dft = Radix2Dit::<BabyBear>::default();
    let plonky3_output = plonky3_dft.dft(input.clone());

    let adapter = ZkshaDifAdapter::avx512();
    let adapter_output = adapter.dft(input.clone());

    assert_eq_babybear(&plonky3_output, &adapter_output, "AVX-512 vs Plonky3 Radix2Dit");
}

#[test]
fn test_backend_tracker_shows_which_path_was_used() {
    let log_n = 8;
    let input = make_input(log_n, 7);

    // Reference backend
    let adapter_ref = ZkshaDifAdapter::reference();
    let _ = adapter_ref.dft(input.clone());
    let tracker = adapter_ref.tracker.borrow();
    assert_eq!(tracker.reference_calls, 1, "reference backend should have 1 call");
    assert_eq!(tracker.scalar_calls, 0);
    assert_eq!(tracker.avx512_calls, 0);

    // AVX-512 backend
    let adapter_avx = ZkshaDifAdapter::avx512();
    let _ = adapter_avx.dft(input.clone());
    let tracker = adapter_avx.tracker.borrow();
    if is_avx512_supported() {
        assert_eq!(tracker.avx512_calls, 1, "AVX-512 backend should have 1 call when available");
        assert_eq!(tracker.scalar_calls, 0, "should not fall back to scalar when AVX-512 available");
    } else {
        assert_eq!(tracker.scalar_calls, 1, "should fall back to scalar when AVX-512 unavailable");
        assert_eq!(tracker.avx512_calls, 0);
    }
}

#[test]
fn test_adapter_handles_multiple_sizes() {
    for log_n in [4, 5, 6, 7, 8, 9, 10] {
        let input = make_input(log_n, log_n as u32);

        let plonky3_dft = Radix2Dit::<BabyBear>::default();
        let plonky3_output = plonky3_dft.dft(input.clone());

        let adapter = ZkshaDifAdapter::reference();
        let adapter_output = adapter.dft(input.clone());

        assert_eq_babybear(&plonky3_output, &adapter_output,
            &format!("size 2^{}", log_n));
    }
}

#[test]
fn test_adapter_dft_batch_multi_column() {
    // Test multi-column matrix (width > 1)
    let h = 1 << 6; // 64 rows
    let w = 3; // 3 columns
    let values: Vec<BabyBear> = (0..h * w)
        .map(|i| BabyBear::from_canonical_u32((i as u32 * 0x1234) % 0x78000001))
        .collect();
    let mat = RowMajorMatrix::new(values, w);

    let plonky3_dft = Radix2Dit::<BabyBear>::default();
    let plonky3_output = plonky3_dft.dft_batch(mat.clone());

    let adapter = ZkshaDifAdapter::reference();
    let adapter_output = adapter.dft_batch(mat);

    let p3_vals = plonky3_output.to_row_major_matrix().values;
    let ad_vals = adapter_output.to_row_major_matrix().values;
    assert_eq_babybear(&p3_vals, &ad_vals, "multi-column dft_batch");
}

#[test]
fn test_avx512_backend_identity() {
    // Report which backend is available on this host
    let avx512 = is_avx512_supported();
    println!("AVX-512 available on host: {}", avx512);

    // Verify the tracker records availability
    let adapter = ZkshaDifAdapter::avx512();
    let tracker = adapter.tracker.borrow();
    assert_eq!(tracker.avx512_available, avx512,
        "tracker should match actual AVX-512 availability");
}
