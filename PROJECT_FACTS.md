# zkSHA-Rx Fly — External Facts

## Identity

Project:
zkSHA-Rx Fly

Description:
AVX-512 vectorized BabyBear NTT acceleration for Plonky3-derived proving systems.

Repository:
Cartilage-Stairwells/zksha-rx-reviewer-access (frozen reviewer snapshot)

Frozen release:
review-v0.1.4

Frozen commit:
review-v0.1.4 (current snapshot)

## Access Model

Repository status:
This repository is the frozen public reviewer snapshot. Historical development artifacts may exist outside this tree.

Approved access language:
> Source code and benchmark artifacts are available for technical review by qualified reviewers.

Reviewer access:
> This repository is publicly accessible. No access request needed.

Review target:
> Reviewers should evaluate tag review-v0.1.4.

Do not use:
- "Open source project" — this is a frozen review snapshot, not a maintained open-source project
- References to unavailable historical tooling or artifacts
- "production-ready" or "drop-in" — no integration claims are made

License note:
This repository is public for technical review. It contains a license file. The snapshot is frozen — no ongoing development occurs here. Historical development artifacts may exist outside this tree.

## Measured Performance

Benchmark framework:
Criterion (historical 0.5.1; snapshot 0.8.2)

Method:
- 100 samples
- 3s warmup
- outlier rejection

Measured kernel results:
- BabyBear DIT butterfly: 9.15× speedup
- Raw i32 XOR butterfly: 4.58× speedup

## Verification Boundary

Lean 4 formalization:
- Montgomery arithmetic foundation
- Related arithmetic invariants
- 33 theorems across 7 modules

Implementation validation:
- Independent mathematical oracle
- Differential testing
- Backend equivalence testing
- Reproducible benchmark artifacts

## Claim Boundary

Measured:
- Kernel-level AVX-512 acceleration

Not yet measured:
- End-to-end proving-system acceleration

Next milestone:
- Hardware validation
- Plonky3/SP1 integration evaluation

## Canonical Positioning

> zkSHA-Rx Fly is a reproducible AVX-512 BabyBear NTT accelerator supported by independent correctness validation and formal arithmetic foundations, with a roadmap toward proving-system integration.

## Repository Identity

All external references must use:
- Project: zkSHA-Rx Fly
- Repository: Cartilage-Stairwells/zksha-rx-reviewer-access
- Release: v0.1.0-zksha-rx
- Current snapshot: review-v0.1.4 (this tag)
- Historical development: v0.1.0-zksha-rx / eb5533d (pre-review-v0.1.1, outside this tree)

Do not use:
- vep-0.1.4-sealed
- (historical: Triune-Oracle, superseded by Cartilage-Stairwells)
- SHARx (pre-rebrand)
- BabyBearVerified.lean (wrong filename)

## Approved Language

Opening sentence:
> zkSHA-Rx Fly is an AVX-512 vectorized BabyBear NTT acceleration engine for Plonky3-derived proving systems.

Trust paragraph:
> Correctness is supported by an independent mathematical oracle, differential backend validation, formal arithmetic foundations, and reproducible benchmark artifacts.

Funding paragraph:
> Funding enables independent hardware validation, ecosystem integration evaluation, and expanded reproducibility infrastructure.

TSCP framing:
> TSCP provides provenance and reproducibility infrastructure used to preserve benchmark evidence and artifact history.

Provenance statement:
> The repository contains reproducible provenance records documenting implementation history, verification artifacts, benchmark methodology, and release state.

Access statement:
> Source code and benchmark artifacts are available for technical review by qualified reviewers. This repository is publicly accessible. No access request needed.

## Prohibited in External Materials

- AI authorship arguments
- "AI cannot fake this"
- "impossible to replicate"
- "only one who solved this"
- Exclusivity language
- Negotiation strategy
- Target ranking language
- custody plane / authority plane / receipt algebra / external verifier architecture (move to appendix)
- 3.94× without explicit benchmark context (historical — archive, do not use in external materials)
- "Formally verified AVX-512 engine" (the SIMD path is not formally verified — only the arithmetic foundation)
- "drop-in acceleration" (not yet integration-validated)
- "Open source" (this is a frozen review snapshot, not an open-source project)
- References to unavailable historical tooling or artifacts
- "public" or "publicly available" when referring to repository access
