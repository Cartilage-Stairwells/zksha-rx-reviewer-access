# review-v0.1.15 Release Note

review-v0.1.15 aligns the formal-verification counts to a stated, reproducible census and repairs a phantom tag reference. The three prior counts (83 / 33 / 33) were mutually inconsistent; all now read from the census at tscp-anchor tag `formal-corpus-v2026-09-11`: **88 theorem declarations across 12 Lean files** (method: theorem-declaration grep). Compile verification continues to be asserted only for Montgomery.lean (12 theorems, 0 sorries, 0 axioms) — no new verification claims are made by this release.

Clone with `--branch review-v0.1.15` and run `./validate_release.sh`, then `make test` (AVX-512 hardware for the SIMD lanes). To reproduce the census: clone tscp-anchor at tag `formal-corpus-v2026-09-11` and count `theorem` declarations in its Lean files.
