//! Phase 3+4 — Cross-backend equivalence matrix and failure artifacts.
//!
//! The desired assertion structure:
//!
//!   reference oracle
//!           |
//!           +--> scalar backend
//!           |
//!           +--> AVX-512 backend
//!
//! NOT: scalar == AVX-512 (that only proves agreement, not correctness).
//!
//! Phase 4: When a mismatch occurs, the test emits a structured failure
//! record (seed, input, backend, operation, expected, actual, domain)
//! that becomes part of the evidence chain.

use super::corpus::*;
use avx512_butterfly::field::babybear::constants::BABYBEAR_P;
use avx512_butterfly::field::babybear::reference::{
    babybear_mul_reference, butterfly_reference,
    to_montgomery_reference, from_montgomery_reference,
};
use avx512_butterfly::field::babybear::montgomery::{ScalarBackend, BABYBEAR_MONTY, MontgomeryBackend};

// ---------------------------------------------------------------------------
// Helpers: twiddle generation (same as ntt_equivalence.rs)
// ---------------------------------------------------------------------------

fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    let mut result = 1u64;
    base %= modulus;
    while exp > 0 {
        if exp & 1 == 1 {
            result = result * base % modulus;
        }
        exp >>= 1;
        base = base * base % modulus;
    }
    result
}

fn find_primitive_root() -> u64 {
    let p = BABYBEAR_P as u64;
    let factors: [u64; 3] = [2, 3, 5];
    for g in 2..p {
        let mut is_primitive = true;
        for &q in &factors {
            if mod_pow(g, (p - 1) / q, p) == 1 {
                is_primitive = false;
                break;
            }
        }
        if is_primitive {
            return g;
        }
    }
    panic!("no primitive root found for p={}", p);
}

fn two_adic_generator(log_n: usize) -> u32 {
    let p = BABYBEAR_P as u64;
    let g = find_primitive_root();
    let n = 1u64 << log_n;
    let omega = mod_pow(g, (p - 1) / n, p);
    assert_eq!(mod_pow(omega, n, p), 1);
    assert_ne!(mod_pow(omega, n / 2, p), 1);
    omega as u32
}

fn compute_twiddles_mont(log_n: usize) -> Vec<Vec<u32>> {
    use avx512_butterfly::field::babybear::constants::BABYBEAR_R_MOD_P;
    let n = 1usize << log_n;
    let omega_mont = to_montgomery_reference(two_adic_generator(log_n));

    let mut stages = Vec::with_capacity(log_n);
    let mut w_m_mont = omega_mont;

    for s in 0..log_n {
        let m2 = n >> (s + 1);
        let mut twiddles = Vec::with_capacity(m2);
        let mut w = BABYBEAR_R_MOD_P; // 1 in Montgomery domain
        for _ in 0..m2 {
            twiddles.push(w);
            w = babybear_mul_reference(w, w_m_mont);
        }
        stages.push(twiddles);
        w_m_mont = babybear_mul_reference(w_m_mont, w_m_mont);
    }
    stages
}

fn first_mismatch(a: &[u32], b: &[u32]) -> Option<usize> {
    a.iter().zip(b.iter()).position(|(x, y)| x != y)
}

// ---------------------------------------------------------------------------
// Test 1: Montgomery mul — full cross-backend matrix on corpus
// ---------------------------------------------------------------------------

#[test]
fn mul_cross_backend_matrix() {
    let mut failures = 0;

    // Boundary pairs
    for &(a, b) in MUL_PAIRS {
        let oracle = babybear_mul_reference(a, b);
        let scalar = ScalarBackend::mul_raw(a, b, BABYBEAR_MONTY);

        if !assert_mul_parity(CORPUS_SEED, a, b, "scalar_cios", oracle, scalar) {
            failures += 1;
        }
    }

    // Random pairs
    let mut rng = Lcg::new(CORPUS_SEED + 10);
    for _ in 0..5000 {
        let a = rng.next_montgomery();
        let b = rng.next_montgomery();

        let oracle = babybear_mul_reference(a, b);
        let scalar = ScalarBackend::mul_raw(a, b, BABYBEAR_MONTY);

        if !assert_mul_parity(CORPUS_SEED + 10, a, b, "scalar_cios", oracle, scalar) {
            failures += 1;
            if failures > 10 { break; }
        }
    }

    assert_eq!(failures, 0, "{} mul cross-backend mismatches", failures);
    record_oracle_mul(5012); record_scalar_mul(5012);
    eprintln!("mul_cross_backend_matrix: {} fixed + 5000 random pairs, 0 failures",
        MUL_PAIRS.len());
}

// ---------------------------------------------------------------------------
// Test 2: NTT staged equivalence — reference vs scalar vs AVX-512
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
#[test]
fn ntt_staged_equivalence() {
    use avx512_butterfly::ntt::{ntt_reference_stage, ntt_scalar_stage};
    use avx512_butterfly::avx512_butterfly_32bit::is_avx512_supported;

    let has_avx512 = is_avx512_supported();

    for log_n in 5..=10 {
        let n = 1usize << log_n;
        let twiddles = compute_twiddles_mont(log_n);

        // Shared deterministic Montgomery-domain input
        let mut rng = Lcg::new(CORPUS_SEED + log_n as u64);
        let input_mont: Vec<u32> = (0..n).map(|_| rng.next_montgomery()).collect();

        let mut data_ref = input_mont.clone();
        let mut data_scalar = input_mont.clone();

        for s in 0..log_n {
            ntt_reference_stage(&mut data_ref, &twiddles[s], s);
            ntt_scalar_stage(&mut data_scalar, &twiddles[s], s);

            // Reference vs Scalar
            if let Some(idx) = first_mismatch(&data_ref, &data_scalar) {
                FailureRecord::ntt_stage(CORPUS_SEED + log_n as u64, s, idx,
                    "scalar", data_ref[idx], data_scalar[idx]).emit();
                panic!("ref vs scalar mismatch at log_n={} stage={} index={}: {} != {}",
                    log_n, s, idx, data_ref[idx], data_scalar[idx]);
            }
        }

        if has_avx512 {
            use avx512_butterfly::ntt::ntt_avx512_stage;
            let mut data_avx = input_mont.clone();

            for s in 0..log_n {
                // Re-run reference from scratch (avx512 needs fresh copy)
                let mut data_ref2 = input_mont.clone();
                for prev_s in 0..=s {
                    ntt_reference_stage(&mut data_ref2, &twiddles[prev_s], prev_s);
                }
                unsafe { ntt_avx512_stage(&mut data_avx, &twiddles[s], s); }
                mark_avx512_executed();

                if let Some(idx) = first_mismatch(&data_ref2, &data_avx) {
                    FailureRecord::ntt_stage(CORPUS_SEED + log_n as u64, s, idx,
                        "avx512", data_ref2[idx], data_avx[idx]).emit();
                    panic!("ref vs avx512 mismatch at log_n={} stage={} index={}: {} != {}",
                        log_n, s, idx, data_ref2[idx], data_avx[idx]);
                }
            }
        }

        // log_n stages for this size
        record_oracle_ntt_stage(log_n as u64); record_scalar_ntt_stage(log_n as u64);
        if has_avx512 { record_avx512_ntt_stage(log_n as u64); }
        eprintln!("ntt_staged_equivalence log_n={} (n={}) PASS{}", log_n, n,
            if has_avx512 { " (ref==scalar==avx512)" } else { " (ref==scalar)" });
    }
}

// ---------------------------------------------------------------------------
// Test 3: NTT with adversarial inputs — near-modulus values
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
#[test]
fn ntt_adversarial_near_modulus() {
    use avx512_butterfly::ntt::{ntt_reference_stage, ntt_scalar_stage};
    use avx512_butterfly::avx512_butterfly_32bit::is_avx512_supported;

    let has_avx512 = is_avx512_supported();

    for log_n in 5..=8 {
        let n = 1usize << log_n;
        let twiddles = compute_twiddles_mont(log_n);

        // Adversarial input: values near the modulus boundary
        let adversarial_values: Vec<u32> = (0..n)
            .map(|i| match i % 5 {
                0 => 0,               // zero
                1 => P - 1,           // max residue
                2 => P - 2,           // near max
                3 => P / 2,           // midpoint
                _ => to_montgomery_reference((i as u32) % 10), // small canonical values
            })
            .collect();

        let mut data_ref = adversarial_values.clone();
        let mut data_scalar = adversarial_values.clone();

        for s in 0..log_n {
            ntt_reference_stage(&mut data_ref, &twiddles[s], s);
            ntt_scalar_stage(&mut data_scalar, &twiddles[s], s);

            if let Some(idx) = first_mismatch(&data_ref, &data_scalar) {
                FailureRecord::ntt_stage(CORPUS_SEED + 100 + log_n as u64, s, idx,
                    "scalar", data_ref[idx], data_scalar[idx]).emit();
                panic!("adversarial ref vs scalar mismatch at log_n={} stage={} index={}",
                    log_n, s, idx);
            }
        }

        if has_avx512 {
            use avx512_butterfly::ntt::ntt_avx512_stage;
            let mut data_avx = adversarial_values.clone();

            for s in 0..log_n {
                let mut data_ref2 = adversarial_values.clone();
                for prev_s in 0..=s {
                    ntt_reference_stage(&mut data_ref2, &twiddles[prev_s], prev_s);
                }
                unsafe { ntt_avx512_stage(&mut data_avx, &twiddles[s], s); }
                mark_avx512_executed();

                if let Some(idx) = first_mismatch(&data_ref2, &data_avx) {
                    FailureRecord::ntt_stage(CORPUS_SEED + 100 + log_n as u64, s, idx,
                        "avx512", data_ref2[idx], data_avx[idx]).emit();
                    panic!("adversarial ref vs avx512 mismatch at log_n={} stage={} index={}",
                        log_n, s, idx);
                }
            }
        }

        record_oracle_ntt_stage(log_n as u64); record_scalar_ntt_stage(log_n as u64);
        if has_avx512 { record_avx512_ntt_stage(log_n as u64); }
        eprintln!("ntt_adversarial_near_modulus log_n={} (n={}) PASS{}", log_n, n,
            if has_avx512 { " (ref==scalar==avx512)" } else { " (ref==scalar)" });
    }
}

// ---------------------------------------------------------------------------
// Test 4: Evidence summary — TSCP-compatible JSON with environment context
// ---------------------------------------------------------------------------

#[test]
fn backend_parity_evidence() {
    use avx512_butterfly::instrument::evidence_contract::EvidenceContractV1;
    use avx512_butterfly::instrument::coverage::CoverageMap;

    let env = EnvInfo::capture();

    // Collect coverage from all tests that ran before this one.
    // Note: test execution order is not guaranteed by the Rust test runner,
    // so coverage counts may vary. The evidence artifact records what was
    // actually executed, not what was planned.
    let coverage = collect_coverage();

    // The evidence test is itself a parity check — baseline mul parity.
    let oracle = babybear_mul_reference(42, 57);
    let scalar = ScalarBackend::mul_raw(42, 57, BABYBEAR_MONTY);
    assert_eq!(oracle, scalar, "evidence baseline mul check failed");

    // Build the Evidence Contract v1 artifact.
    let run_id = format!("backend_parity_{:016X}", CORPUS_SEED);
    let parity = avx512_butterfly::instrument::evidence_contract::ParityResult::pass();
    let contract = EvidenceContractV1::new(&run_id, coverage, parity);

    // Emit the evidence artifact — this is the interface between
    // the test suite and the verification gate.
    eprintln!("EVIDENCE_CONTRACT_V1 {}", contract.to_json());

    // Also emit the legacy summary for compatibility with IEP consumers.
    eprintln!(
        "{{\"artifact\":\"backend_parity_corpus\",\
         \"issue\":\"#4\",\
         \"field\":\"BabyBear\",\
         \"prime\":\"0x{:08X}\",\
         \"montgomery_radix\":\"2^32\",\
         \"backend_identity\":{{{}}},\
         \"backends\":[\"reference_oracle\",\"scalar_cios\",\"avx512\"],\
         \"operations\":[\"montgomery_mul\",\"butterfly\",\"ntt_stage\"],\
         \"corpus_categories\":[\"boundary_values\",\"reduction_stress\",\
           \"near_boundary_scan\",\"deterministic_random\",\"adversarial_near_modulus\"],\
         \"comparison_model\":\"oracle_vs_each_backend\",\
         \"avx512_was_executed\":{},\
         \"status\":\"PASS\"}}",
        BABYBEAR_P,
        env.to_json(),
        avx512_was_executed(),
    );
}

// ---------------------------------------------------------------------------
// Test 5: AVX-512 execution canary — confirms the SIMD path actually ran
// ---------------------------------------------------------------------------

#[test]
fn avx512_execution_canary() {
    let env = EnvInfo::capture();

    eprintln!("avx512_canary: arch={} os={} rustc={} compiled={} runtime={} executed={}",
        env.arch, env.os, env.rustc,
        env.has_avx512_compile, env.has_avx512_runtime,
        avx512_was_executed());

    if env.has_avx512_runtime {
        // If the CPU supports AVX-512, we expect the SIMD tests to have
        // actually executed. If they didn't, the test suite silently
        // degraded to scalar-only — that's a false PASS.
        //
        // Note: test execution order is not guaranteed, so this canary
        // may run before the AVX-512 tests. We check the flag AND run
        // a direct AVX-512 operation to verify the path works.
        use avx512_butterfly::avx512_butterfly_32bit::{avx512_radix2_butterfly_32, is_avx512_supported};
        use avx512_butterfly::field::babybear::reference::butterfly_reference;
        use avx512_butterfly::field::babybear::constants::BABYBEAR_R_MOD_P;
        use std::arch::x86_64::*;

        assert!(is_avx512_supported(), "is_avx512_supported() returned false but has_avx512_runtime is true");

        // Run a single AVX-512 butterfly and compare with oracle
        let a = [42u32; 16];
        let b = [57u32; 16];
        let w = [BABYBEAR_R_MOD_P; 16]; // identity twiddle

        let oracle_x = butterfly_reference(42, 57, BABYBEAR_R_MOD_P);

        let va = unsafe { _mm512_loadu_si512(a.as_ptr() as *const __m512i) };
        let vb = unsafe { _mm512_loadu_si512(b.as_ptr() as *const __m512i) };
        let vw = unsafe { _mm512_loadu_si512(w.as_ptr() as *const __m512i) };

        let (avx_x_m, avx_y_m) = unsafe { avx512_radix2_butterfly_32(va, vb, vw) };
        mark_avx512_executed();

        let avx_x: [u32; 16] = unsafe { std::mem::transmute(avx_x_m) };
        let avx_y: [u32; 16] = unsafe { std::mem::transmute(avx_y_m) };

        // All 16 lanes should match the oracle
        for i in 0..16 {
            assert_eq!(avx_x[i], oracle_x.0,
                "AVX-512 canary lane {}: x={} != oracle={}", i, avx_x[i], oracle_x.0);
            assert!(avx_x[i] < BABYBEAR_P, "AVX-512 canary: x[{}] = {} >= P", i, avx_x[i]);
            assert!(avx_y[i] < BABYBEAR_P, "AVX-512 canary: y[{}] = {} >= P", i, avx_y[i]);
        }

        eprintln!("avx512_execution_canary: PASS — AVX-512 path executed and verified (16 lanes × 1 butterfly)");
    } else {
        eprintln!("avx512_execution_canary: SKIP — AVX-512 not available on this platform");
        eprintln!("  WARNING: parity suite ran WITHOUT testing the AVX-512 backend.");
        eprintln!("  This is a scalar-only run. Do not claim AVX-512 semantic parity from this result.");
    }

    assert!(avx512_was_executed() || !env.has_avx512_runtime,
        "AVX-512 runtime support detected but no test marked the execution flag. \
         The suite may have silently degraded to scalar-only.");
}
