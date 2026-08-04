# Makefile for reviewer reproduction
# Usage: make reproduce

.PHONY: validate verify test bench reproduce inspect

validate:
	./validate_release.sh

verify:
	sha256sum -c SHA256SUMS

test:
	RUSTFLAGS="-Ctarget-cpu=native" cargo test

bench:
	RUSTFLAGS="-Ctarget-cpu=native" cargo bench --bench three_lane_bench

inspect:
	@echo "=== Evidence Artifacts ==="
	@ls -la evidence/
	@echo ""
	@echo "=== Correctness Receipt ==="
	@cat evidence/correctness_receipt_dif_fix.json
	@echo ""
	@echo "=== Benchmark Summary ==="
	@head -50 evidence/avx512_bench_dif_fix_20260804_023129.txt

reproduce: validate verify
	@echo ""
	@echo "=== Release validated and checksums verified ==="
	@echo "=== To run tests: make test ==="
	@echo "=== To run benchmarks: make bench ==="
	@echo "=== AVX-512 hardware required for full benchmark ==="
