# Measurement Protocol — avx512-butterfly

This document defines the epistemological structure of performance claims
in this project. It exists so the difference between a trustworthy benchmark
and an unverifiable one is explicit and permanent.

---

## The core distinction

A benchmark output is not evidence by itself.
Evidence requires three independent anchors:

```
Environment Anchor          Code Anchor             Artifact Anchor
──────────────────          ───────────────         ───────────────
CPU / ISA / compiler  +     exact source      +     output / hashes
runtime conditions          revision                and manifests
```

Together they answer the three questions a reviewer actually needs:

| Question | Answered by |
|---|---|
| What ran? | Environment anchor: `provenance/provenance.json` |
| What code produced it? | Code anchor: `commit.txt` + git history |
| Can I verify the result was not altered? | Artifact anchor: `SHA256SUMS` + `sha256sum -c` |

A benchmark missing any anchor is a claim, not evidence.

---

## The Firebird baseline

The first sealed performance specimen for this project is `firebird_74c6e5f`.

**Source commit:** `74c6e5f841d1c509b4b7166258ee3bb712535aae`
**Artifact directory:** `benchmark_reports/firebird_74c6e5f/`
**Capture tooling frozen at:** `f44d309`

The infrastructure history and measurement history are kept separate
by design. The commit that introduces the benchmark data should contain
only generated files. The commit ordering is itself part of the evidence:

```
Infrastructure history          Measurement history
──────────────────────          ───────────────────
4fa41f1  CI provenance          <future commit>
3e901bf  capture tooling            benchmark_reports/firebird_74c6e5f/
acd08d6  bundle sealing                 ├── criterion/
f44d309  pre-seal validation            ├── provenance/
38e65a2  this document                  ├── manifest.json
                                        ├── bench_output.txt
                                        ├── results.md
                                        ├── SHA256SUMS
                                        └── commit.txt
```

Do not squash, amend, or rebase the measurement commit. SHA256SUMS is
only meaningful if the commit that introduced it is immutable.

---

## Capture sequence

On an AVX-512 host (avx512f + avx512dq required):

```bash
# Confirm source state
git rev-parse HEAD    # must be 74c6e5f841d1c509b4b7166258ee3bb712535aae
git status --short    # must be empty (clean tree)

# Phase 1: provenance + benchmark
./tools/run_benchmark.sh firebird_74c6e5f benchmark_reports/firebird_74c6e5f

# Phase 2: fold in Criterion data
cp -r target/criterion/firebird_74c6e5f benchmark_reports/firebird_74c6e5f/criterion/

# Phase 3: verify + seal
./tools/run_benchmark.sh --seal benchmark_reports/firebird_74c6e5f
```

The measurement event should be intentionally boring:

```
prepare → capture environment → run correctness checks
    → measure → generate report → hash artifacts → commit data
```

No harness changes. No interpretation changes. No optimization changes.
Those belong after the specimen exists.

---

## Seal gates

The `--seal` flag enforces three gates before writing `SHA256SUMS`:

| Gate | Failure class prevented |
|---|---|
| `avx512_radix2_butterfly` present in `bench_output.txt` | Silent scalar fallback — wrong binary path |
| `avx512f` + `avx512dq` in `provenance.json` | Wrong physical host |
| `criterion/` directory present | Incomplete measurement bundle |

If any gate fails, the script aborts without writing `SHA256SUMS`.
The bundle is not sealed. Do not override manually.

---

## The optimization loop

Once the Firebird specimen exists, the development cycle becomes:

```
Firebird specimen
        ↓
profile bottleneck
        ↓
change implementation
        ↓
revalidate contract
        ↓
produce next specimen
```

The important transition is this: the project stops arguing about whether
a number is trustworthy and starts using trustworthy numbers to make
engineering decisions. AVX-512 optimization proceeds from evidence,
not expectation.

Each future specimen follows the same three-anchor model. The baseline
name changes; the protocol does not.

---

## Verification (at any time)

```bash
cd benchmark_reports/firebird_74c6e5f
sha256sum -c SHA256SUMS
```

Every file should report `OK`. A single `FAILED` line means the bundle
was modified after sealing and the measurement claim is invalidated.
