# Provenance Record — Experiments B and C Evidence Package

## Evidence Identities
- **Experiment B (observational):** experiment_b_v1 — SHA-256 71112896057db7c1bc2e4c6b8e2b38869b8170f21a4bb0762a6af2faab00f569
- **Experiment C (observational profiling):** experiment_c_profile_v1 — SHA-256 d0bfdc44d7b73dbf123d40e551d27da924e8e50789f82b044b08443db319e7e9 (stated in-document; file hash in SHA256SUMS)
- **Experiment A (kernel benchmark):** kernelbench_v1 — preserved separately on branch `evidence/experiment-a`
- **Cross-experiment summary:** EVIDENCE_SUMMARY_v1.md (Experiments A, B, C)

## Source State
- **Capture date:** 2026-08-18
- **Source repository:** tscp-anchor, branch master, HEAD 58208dab19156424d0b6d9e532f805732ed7be47
- **Tag context:** benchmark/firebird_74c6e5f (at 653ce061)
- **Preservation date:** 2026-09-18
- **Preservation trigger:** evidence previously existed only as untracked working-tree files; preserved by owner decision to prevent data loss and establish a public, tamper-evident home.

## Verification Performed Before Preservation
- Pre-push boundary check: **10/10 PASS** (2026-09-18) — no patent-strategy language, no legal communications material, no internal decision logs, no credentials, no non-public email addresses.
- SHA256SUMS generated for all 9 files in this directory; Experiment B in-file hash cross-checked against the sealed value (match).
- Attestation workflow artifacts (github-attestation.json, workflow-run.json, artifact-digests.txt, ATTESTATION_WORKFLOW_README.md) are the formal-backbone evidence context captured with the experiments.

## Signing Status (recorded separately from evidence claims)
- **This is an unsigned preservation commit.** No GPG signature is applied or claimed.
- Historical signing identity: the GPG key of record for Sean Christopher Southwick, as documented in the existing public release records of this project (tscp-anchor release documentation). Its full fingerprint is intentionally not restated here.
- The corresponding private key is not available in the preservation environment. No replacement key was generated; no key recovery attempt was performed as part of this operation.
- Recent commits on related repositories are likewise unsigned; this record does not change that state.
- The evidence claims above are independent of, and make no reference to, commit signature status. Custody claims are not created by signing; signing status is not evidence of experimental validity.
- If the original private key becomes available later, signatures can be verified or applied as a separate, deliberate action. This commit is not amended retroactively.

## Scope
This operation preserves Experiments B and C evidence only. It changes no code, no CI configuration, and no other repository state.
