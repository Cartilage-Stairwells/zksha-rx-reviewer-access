# zkSHA-Rx Fly — External Facts

## Identity

Project:
zkSHA-Rx Fly

Description:
AVX-512 vectorized BabyBear NTT acceleration for Plonky3-derived proving systems.

Repository:
Cartilage-Stairwells/avx512-butterfly (private development repository)

Frozen release:
v0.1.0-zksha-rx

Frozen commit:
eb5533d

## Access Model

Repository status:
Private development repository with invited reviewer access.

Approved access language:
> Source code and benchmark artifacts are available for technical review by qualified reviewers.

Reviewer access:
> Reviewer access to the validation repository will be provided upon request or with an invitation.

Review target:
> Reviewers should evaluate tag v0.1.0-zksha-rx.

Do not use:
- "Open source (MIT/Apache 2.0)" — the repository is not publicly open
- git clone commands with public URLs — use the reviewer access language instead
- "public" or "publicly available" when referring to the repository

License note:
The repository contains a license file, but the source is not publicly distributed at this stage. Reviewer access does not constitute public release.

## Measured Performance

Benchmark framework:
Criterion 0.5.1

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
- Repository: Cartilage-Stairwells/avx512-butterfly
- Release: v0.1.0-zksha-rx
- Commit: eb5533d

Do not use:
- vep-0.1.4-sealed
- Triune-Oracle
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
> Source code and benchmark artifacts are available for technical review by qualified reviewers. Reviewer access to the validation repository will be provided upon request or with an invitation.

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
- "Open source (MIT/Apache 2.0)" (repository is private with reviewer access)
- git clone commands with public URLs (use reviewer access language instead)
- "public" or "publicly available" when referring to repository access
