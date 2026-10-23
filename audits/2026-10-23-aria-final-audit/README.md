# Audit artifacts for review-v0.1.21 — external countersignature stage (2026-10-23)

**Branch status:** this branch (`audit/aria-final-audit-v0.1.21`) is a custody placement only. It is NOT a reviewer release. The sealed reviewer snapshot remains tag `review-v0.1.21` (commit 73bf100); `main` is unchanged. These files carry **Authority: 0** relative to the sealed artifact — they document the audit session and the countersignature procedure, and do not modify, reinterpret, or re-seal anything.

## Contents

- `ARIA-FINAL-AUDIT-v0.1.21.md` — final audit report (read-only audit of review-v0.1.21; dispositions: benchmark provenance VERIFIED, lane-2 SIMD evidence SUPPORTED-PARTIAL, 139-test census VERIFIED, historical/tag integrity VERIFIED, seal reproductions VERIFIED; includes the documented-and-falsified 101/110 false alarm and the auditor-independence caveat)
- `ARIA-COUNTERSIGNATURE-REQUEST-v0.1.21.md` — PI ratification of the audit disposition, the countersignature protocol, and the handoff to the external Aria seat
- `ARIA-COUNTERSIGNATURE-v0.1.21.md` — the external reviewer's countersignature (2026-10-23), recorded verbatim with a custodian routing annotation; Final Review Status: COUNTERSIGNED WITH BOUNDS, lane-2 retained SUPPORTED-PARTIAL
- `AUDIT-INSPECTION-ADDENDUM-v0.1.21.md` — resolutions of the three qualifications raised by an external read-only inspection (2026-10-23): hash-scope precision (attestation-section digest supplied: `06201c7e…`), benchmark-number reconciliation VERIFIED (the audit parser read Criterion's CI lower bound; point-estimate re-parse reproduces the documented 1.276×/1.265×/0.9% exactly), and the operator-relay identity limitation recorded explicitly with a parked upgrade path
- `SHA256SUMS` — hashes of the four artifact files (seal is hash-last: this file was generated after the content files were placed)

## Countersignature boundary (per PI ratification)

- A countersignature is recorded as a separate artifact; the sealed target remains unchanged.
- A countersignature does not promote the lane-2 finding (SUPPORTED-PARTIAL stands until its missing reproduction is actually completed) and does not close the Zen 5 public-reproduction gap.
- Prior to placement, the external reviewer's status was REVIEW BLOCKED — ACCESS (artifacts were not repository-accessible). This placement resolves that access limitation without any change to the sealed artifact.

## Traceability

The audit's reproductions were executed from a fresh clone of tag review-v0.1.21 on 2026-10-23. The executing seat also sealed the tag (independence limitation disclosed in the report).


## State transition (2026-10-23)

The external reviewer's status transitioned from REVIEW BLOCKED — ACCESS to **COUNTERSIGNED WITH BOUNDS** after reviewing the artifact texts and repository-accessible copies. Per the countersignature: target tag SEALED / UNCHANGED; lane-2 SIMD finding retained SUPPORTED-PARTIAL (independent disasm probe re-execution still pending and remains the only path to promotion); overall disposition APPROVED / COUNTERSIGNED. The Zen 5 public-repo rerun remains parked as a separate cost/value decision.


## Inspection addendum (2026-10-23)

An external operator-relayed inspection (read-only) issued a qualified acceptance of the countersignature placement and raised three qualifications. Two of the three are resolved with corrective evidence in `AUDIT-INSPECTION-ADDENDUM-v0.1.21.md` (hash-scope wording; parser position in the benchmark re-parse — the documented canonical values now reproduce exactly); the third is not resolved but disclosed as a standing limitation (operator-relayed reviewer identity is operator-asserted until a signed attestation exists — documenting an authentication gap is not authentication). No findings contradict the sealed artifact; lane-2 remains SUPPORTED-PARTIAL and the Zen 5 rerun remains parked.


## Final inspection record (2026-10-23, follow-up)

The external inspector reviewed the additive correction at commit `d9c07f5` and issued an updated **qualified acceptance with narrower reservations**, distinguishing the two corrective resolutions from the disclosed limitation. Its closeout wording, recorded verbatim as received via operator relay:

> Perplexity inspected the additive correction at commit `d9c07f5`. The addendum addresses the hash-scope ambiguity and supplies a documented benchmark-number reconciliation, including an explicit correction of the audit's earlier explanation. Reviewer identity remains operator-asserted and is now disclosed as a standing limitation. Perplexity has not independently recomputed the digests or re-parsed the benchmark logs.

Scope of the follow-up inspection, as disclosed by the inspector: the corrected record was inspected; the digests were not independently recomputed, the benchmark logs were not independently re-parsed, and live branch/tag references were not re-verified in the follow-up. This record carries the same operator-relay provenance property as the attestation it describes.
