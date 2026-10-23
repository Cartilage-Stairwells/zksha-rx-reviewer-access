# Audit Inspection Addendum — review-v0.1.21 (2026-10-23)

**Author:** Solene, custodian seat. **Authority:** 0 relative to the sealed target. **Nature:** additive record; commits 82b2599 and 10f93a3 and all sealed tags remain unaltered.

## Purpose

On 2026-10-23 an external inspection (Perplexity, operator-relayed, read-only) reviewed the countersignature placement and issued a qualified acceptance, explicitly declining to re-execute tests, benchmarks, disassembly, or hash verification. Its disposition, quoted as received: "Perplexity performed a read-only repository and audit-document inspection. Countersignature placement and preservation of the stated bounds were confirmed. No tests, benchmarks, disassembly probes, or SHA-256 verification were independently re-executed. Qualifications remain concerning benchmark-number reconciliation, hash-scope wording, and authentication of the operator-relayed reviewer attestation." (This quotation is itself operator-relayed and carries the same provenance property described below.)

This addendum resolves the three qualifications with evidence.

## Qualification 1 — hash-scope wording: RESOLVED (precision defect in the routing annotation)

The routing annotation in `ARIA-COUNTERSIGNATURE-v0.1.21.md` refers to "SHA-256 of the attestation section," while the published digest is of the complete file. The precise scope, computed 2026-10-23:

- Complete file (as recorded in SHA256SUMS, commit 10f93a3): `18d08be2d155d5b7fdc6b29231e78f5259b471169c7afc2388ae077f58eaa742`
- Attestation section only (bytes 0–1711, ending immediately before the italic line beginning "*The text above is the external reviewer's attestation"): `06201c7e5f34042170758e15d5b7733de1157c305f70c67337d290d2ca290872`

The annotation's wording was imprecise; the published digest always covered the complete file. No hash mismatch exists. The original file is not edited; this addendum supplies the missing precision.

## Qualification 2 — benchmark-number reconciliation: RESOLVED AND VERIFIED (audit-procedure artifact, not an artifact defect)

The final audit report's re-parsed values (run1 geo 1.2790×, run2 geo 1.2690×, delta 0.78%) differ slightly from the documented canonical values (1.276×/1.265×, variance 0.9%), and run1's re-parsed value lies above the documented range. Root cause, verified by recomputation 2026-10-23: the audit's log parser read Criterion's `time: [lower_bound point_estimate upper_bound]` triple at the FIRST position (CI lower bound). Criterion's per-size time lines carry three values; the audit parser captured the lower confidence bound rather than the point estimate.

Re-parse at each position (13 sizes, both runs):

| Aggregation | Run 1 geo | Run 2 geo | Dual-run delta |
|---|---|---|---|
| CI lower bound (audit parser position) | 1.2790× | 1.2690× | 0.78% |
| **Point estimate (middle value)** | **1.2758×** | **1.2647×** | **0.87%** |
| CI upper bound | 1.2742× | 1.2614× | 1.01% |

The documented canonical values (1.276×/1.265×, 0.9%) reproduce exactly from the published logs at the point-estimate position (1.2758→1.276, 1.2647→1.265, 0.87%→0.9% at three-decimal rounding). The audit report's parenthetical explanation of the delta difference ("computed on rounded values") was incorrect; the correct explanation is parser position, as above. This is the second audit-procedure artifact identified in this cycle (the first: the xargs-induced manifest false alarm). Neither is a defect in the sealed artifact. Finding 1's VERIFIED status is strengthened: the canonical values are now reproduced exactly, not merely approximately.

## Qualification 3 — reviewer-identity authentication: RECORDED AS AN EXPLICIT STANDING LIMITATION

The repository records that the external reviewer's attestation was received via operator relay and committed by the custodian seat. The repository therefore cannot cryptographically authenticate the reviewer's identity; the in-repository provenance of the reviewer identity is operator-asserted. This limitation is now stated explicitly as part of the record rather than left implicit. Upgrade path (parked, operator decision required, not a blocker): a reviewer-signed statement (cryptographic signature, or an attestation performed in-band from the reviewer's own GitHub account) would close the identity-authentication gap. No action taken in this cycle.

## State

Sealed target `review-v0.1.21` (73bf100), `main`, and all historical tags: unchanged. This addendum is additive; SHA256SUMS regenerated over all four artifact files (hash-last).
