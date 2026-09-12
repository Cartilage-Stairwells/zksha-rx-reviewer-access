//! Plonky3 `TwoAdicSubgroupDft` adapter for the zkSHA-Rx AVX-512 NTT backend.
//!
//! This adapter implements Plonky3's `TwoAdicSubgroupDft<BabyBear>` trait,
//! routing the DFT computation through zkSHA-Rx's DIF radix-2 NTT with
//! three execution lanes (reference, scalar, AVX-512).
//!
//! ## Binary compatibility
//!
//! `BabyBear` is `#[repr(transparent)]` over a single `u32` field storing
//! the Montgomery-encoded value (xR mod p, R = 2^32). Our NTT operates on
//! raw `u32` Montgomery-domain values. Since the representations are
//!
//! ## B2: shared tracker
//!
//! The tracker is `Arc<RefCell<..>>` so that when the adapter is moved into
//! a `TwoAdicFriPcs` (which owns its `Dft`) and cloned internally, every
//! clone shares the same counters. A handle retained by the caller records
//! calls made inside the proving pipeline.
//! raw `u32` Montgomery-domain values. Since the representations are
//! identical, we can safely reinterpret between `&mut [BabyBear]` and
//! `&mut [u32]` without any conversion overhead.

use std::vec::Vec;
use p3_baby_bear::BabyBear;
use p3_dft::TwoAdicSubgroupDft;
use p3_field::{TwoAdicField, AbstractField};
use p3_matrix::dense::RowMajorMatrix;
use p3_matrix::Matrix;

use crate::ntt;

/// Which NTT backend to use.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NttBackend {
    #[default]
    Reference,
    Scalar,
    Avx512,
}

/// Instrumentation tracker: records which backend was actually called.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BackendCallTracker {
    pub reference_calls: u64,
    pub scalar_calls: u64,
    pub avx512_calls: u64,
    pub avx512_available: bool,
}

/// The adapter implementing Plonky3's `TwoAdicSubgroupDft` for BabyBear
/// using zkSHA-Rx's DIF radix-2 NTT.
#[derive(Clone, Debug, Default)]
pub struct ZkshaDifAdapter {
    pub backend: NttBackend,
    pub tracker: std::sync::Arc<std::cell::RefCell<BackendCallTracker>>,
}

impl ZkshaDifAdapter {
    pub fn reference() -> Self {
        Self {
            backend: NttBackend::Reference,
            tracker: std::sync::Arc::new(std::cell::RefCell::new(BackendCallTracker::default())),
        }
    }

    pub fn scalar() -> Self {
        Self {
            backend: NttBackend::Scalar,
            tracker: std::sync::Arc::new(std::cell::RefCell::new(BackendCallTracker::default())),
        }
    }

    pub fn avx512() -> Self {
        let avail = is_avx512_supported();
        Self {
            backend: NttBackend::Avx512,
            tracker: std::sync::Arc::new(std::cell::RefCell::new(BackendCallTracker {
                avx512_available: avail,
                ..Default::default()
            })),
        }
    }

    #[cfg(target_arch = "x86_64")]
    fn check_avx512(&self) -> bool {
        is_avx512_supported()
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn check_avx512(&self) -> bool {
        false
    }

    /// Compute twiddle factors for each stage of the DIF NTT.
    ///
    /// Uses Plonky3's `two_adic_generator` to get the root, ensuring
    /// our output matches Plonky3's own DFT implementations.
    ///
    /// Twiddles are computed using BabyBear's native arithmetic (which
    /// uses Plonky3's Montgomery convention) and then transmuted to raw
    /// u32 values for our NTT (safe due to #[repr(transparent)]).
    fn compute_twiddles(log_n: usize) -> Vec<Vec<u32>> {
        let n = 1usize << log_n;
        let generator = BabyBear::two_adic_generator(log_n);

        let mut twiddles_per_stage = Vec::with_capacity(log_n);
        for s in 0..log_n {
            let half_len = n >> (s + 1);

            // g_s = generator^(2^s) — the stage-specific root of unity
            let mut g_s = generator;
            for _ in 0..s {
                g_s = g_s * g_s; // BabyBear Montgomery multiplication
            }

            // twiddles[i] = g_s^i in Montgomery domain
            let mut stage_twiddles = Vec::with_capacity(half_len);
            let mut tw = BabyBear::one(); // 1 in Montgomery = R mod p
            for _ in 0..half_len {
                // Safe transmute: BabyBear is #[repr(transparent)] over u32
                stage_twiddles.push(unsafe { std::mem::transmute(tw) });
                tw = tw * g_s;
            }
            twiddles_per_stage.push(stage_twiddles);
        }
        twiddles_per_stage
    }

    /// Run the NTT on a u32 slice using the selected backend.
    fn run_ntt(&self, data: &mut [u32], twiddles: &[Vec<u32>]) {
        match self.backend {
            NttBackend::Reference => {
                ntt::ntt_reference(data, twiddles);
                self.tracker.borrow_mut().reference_calls += 1;
            }
            #[cfg(target_arch = "x86_64")]
            NttBackend::Scalar => {
                ntt::ntt_scalar(data, twiddles);
                self.tracker.borrow_mut().scalar_calls += 1;
            }
            #[cfg(target_arch = "x86_64")]
            NttBackend::Avx512 => {
                if self.check_avx512() {
                    unsafe {
                        ntt::ntt_avx512(data, twiddles);
                    }
                    self.tracker.borrow_mut().avx512_calls += 1;
                } else {
                    ntt::ntt_scalar(data, twiddles);
                    self.tracker.borrow_mut().scalar_calls += 1;
                }
            }
            #[cfg(not(target_arch = "x86_64"))]
            _ => {
                ntt::ntt_reference(data, twiddles);
                self.tracker.borrow_mut().reference_calls += 1;
            }
        }
    }
}

/// Check AVX-512 availability at runtime.
#[cfg(target_arch = "x86_64")]
pub fn is_avx512_supported() -> bool {
    if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
        cpuinfo.contains("avx512f") && cpuinfo.contains("avx512dq")
    } else {
        false
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn is_avx512_supported() -> bool {
    false
}

impl TwoAdicSubgroupDft<BabyBear> for ZkshaDifAdapter {
    type Evaluations = RowMajorMatrix<BabyBear>;

    fn dft_batch(&self, mat: RowMajorMatrix<BabyBear>) -> RowMajorMatrix<BabyBear> {
        let h = mat.height();
        let w = mat.width();
        let log_h = h.trailing_zeros() as usize;

        // Compute twiddles using Plonky3's generator
        let twiddles_per_stage = Self::compute_twiddles(log_h);

        // Single-column case (most common): transmute in place
        if w == 1 {
            let mut values = mat.values;
            let u32_slice: &mut [u32] = unsafe {
                std::slice::from_raw_parts_mut(values.as_mut_ptr() as *mut u32, values.len())
            };

            self.run_ntt(u32_slice, &twiddles_per_stage);
            ntt::bit_reverse(u32_slice);

            return RowMajorMatrix::new(values, 1);
        }

        // Multi-column case: process each column
        let mut mat = mat;
        for col in 0..w {
            let mut col_vals: Vec<u32> = (0..h)
                .map(|row| unsafe { std::mem::transmute(mat.values[row * w + col]) })
                .collect();

            self.run_ntt(&mut col_vals, &twiddles_per_stage);
            ntt::bit_reverse(&mut col_vals);

            for row in 0..h {
                mat.values[row * w + col] = unsafe { std::mem::transmute(col_vals[row]) };
            }
        }
        mat
    }
}
