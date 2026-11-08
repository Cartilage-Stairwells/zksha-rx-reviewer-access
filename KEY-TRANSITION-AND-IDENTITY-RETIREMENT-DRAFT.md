# Key transition and identity retirement — DRAFT, PARTIALLY EXECUTED 2026-11-08

Status: **partially executed, updated 2026-11-08.** The former seat key `CAF6A411CD9AD65E2557E3DA7C65D2F3BA9969AD` is revoked and its revocation-bearing public half is the shipped copy; it stays on the retired list for historical verification only. An intermediate replacement, `7FD1A718D855DD4D481E9FDBE70FE87308B04A22`, was generated and ships as a candidate; it was never pinned and is superseded. The pinned release signer is now `D6581E111DA3F1C69759D6F50C4C0A104AD7B940`, whose public half ships as `gate-amendment/keys/seat-2026-11-08-public.asc`; its secret key is passphrase protected and was restored and used to sign from its encrypted export, so protection and recovery are demonstrated. ACTIVATION IS NOT DEMONSTRATED: no tag signed by this identity has been accepted by the gate, so release mode has not been seen to pass on a genuine git-signed tag. Historical signatures made by the retired identities have NOT been independently re-verified against the retired key. Nothing has been merged, no tag created or moved, and no release published. Earlier text below is preserved as written at drafting time; the drafting-time sentence "No key generated, nothing signed, nothing committed, nothing pushed, no tag created or moved" was true of that draft and is superseded by this status.

The two decisions below are recorded as *recommended by
the owner in the conversation* and marked unratified, because the same message that made
the recommendations said they were not his to ratify. Execution waits on an explicit
statement of decision.

## Decisions, as recommended and not yet ratified

| Decision | Recommendation | State |
|---|---|---|
| Signing identity | **SEAT** (release authority survives device and personnel changes; personal identity stays distinct) | UNRATIFIED |
| Missing signature | **HARD FAILURE** (fail closed) | UNRATIFIED |

Both are implemented in the prepared gate, so ratifying them costs no further work.

## Stage 1: historical verification, performed against real data

60 tags in `Cartilage-Stairwells/tscp-anchor`, read from tag objects, not from summaries.

| Reading | Count |
|---|---|
| Tags signed (OpenPGP, on the tag object) | 37 |
| Tags unsigned | 23 |
| Reachable from `master` | 17 |
| NOT reachable from `master` | 43 |
| Cryptographically verifiable today | **0** |

Three corrections to earlier statements in this thread, each found by checking rather than
restating:

1. **The "unknown issuer" is resolved, and it is not an unknown key.** `v0.3-reproduction`
   is a *lightweight* tag; the signature is an inline `gpgsig` on the **commit** it points
   at, made by a **third** identity, `B5690EEEBB952194`. Different object, different
   signer, same conclusion: unverifiable without a public half, but no longer unexplained.
2. **43 of 60 tags sit outside the branch lineage.** `master`'s root is `e27b79d8`
   (2026-07-17), a rebuilt history, so every tag predating the rebuild points at commits
   `master` no longer contains. Confirmed two ways: ancestry test and a direct scan of
   `master`'s commit list. **The proposed "tag must be on the release branch" rule cannot
   be applied retroactively** without declaring 43 historical tags invalid, which the
   non-negotiables forbid. Scope it to tags created after the amendment.
3. **The reviewer repository is not unsigned throughout.** `review-v0.1.10` and
   `review-v0.1.13` are signed, both by the later identity, both 2026-08-05. Signing did
   not need inventing; it lapsed after `review-v0.1.13` and was never resumed.

### The identities

| Role | Fingerprint (full) | Tags | Last use |
|---|---|---|---|
| retired earlier | `901EEB9EB4A9F380ABCA41D07A69CA58234E4798` | 20 | 2026-07-08 |
| retired later (the pen) | `84692E6294128CC1C4ACCD15E747C3AF22573539` | 16 anchor + 2 reviewer | 2026-07-25 |
| commit-level, third | `B5690EEEBB952194` (long id only; full fingerprint not yet recoverable from a tag packet) | 1 commit | 2026-07-23 |

Full fingerprints for the first two were read from the signatures' issuer-fingerprint
subpackets, so they are the identities that actually signed, not transcriptions.

## Non-negotiables, adopted as written

No backdating. No retroactive signatures. No rewriting of historical tags. No silent change
to the meaning of any previous verification result. The retired identities remain available
for **historical verification only, never for future signing**. Both historical fingerprints
must remain independently verifiable, which makes recovery of their public halves a
precondition rather than a nicety.

## Retirement terms

- The pen's identity is recorded as **retired for loss**, not as compromised, unless it was
  stored without a PIN; the owner must confirm which, since the two have different
  consequences for the 18 tags it signed.
- Retirement is a statement about **future signing only**. It does not withdraw, weaken, or
  re-date any signature already made.
- The third, commit-level identity is recorded as **unresolved**: it signed once, its full
  fingerprint is not established, and it is neither retired nor authorised.

## Replacement identity, when authorised

Requirements, all enforceable by the prepared gate:

1. **Seat-held.** Signing authority attaches to the seat, not to a device or a person.
2. **Independently controlled**, meaning the private half is not held by whoever operates
   the release automation, and custody is documented in writing.
3. **Fingerprint pinned** in the release identity document and repeated in the gate's
   allowlist, so pinning is a filed artifact and not a memory.
4. **Custody documented**: where the private half lives, who can use it, how it is
   recovered, and what happens on loss. A second loss must not repeat this thread.

What a signature will attest, stated plainly so nobody reads more into it: **continuity of
an authorised identity**, nothing about authorship by a named human, and nothing about the
correctness of what it signs.

## Proposed execution order, with one correction

| Stage | State |
|---|---|
| 1 Historical verification | **DONE as inventory; BLOCKED as verification** (no public halves) |
| 2 Identity retirement | drafted above; awaiting ratification |
| 3 Transition record | this document |
| 4 Gate amendment | **prepared and rehearsed**, eight mutants, not installed |
| 5 Mutation testing | 8/8 refusals proven; **acceptance path unrun** |
| 6 Replacement key | not started; withheld by the authorisation boundary |
| 7 New signed snapshot | not started; depends on 6 |

The correction: Stage 5's pinned-key acceptance case cannot precede Stage 6. Either run it
against a disposable fixture now (recommended) or move it after Stage 6 and accept a
half-tested gate until then.

## Executed evidence, appended 2026-11-04

The three corrections above stand unchanged. Two statements elsewhere in this draft are now
superseded by evidence rather than edited away:

1. **"0 cryptographically verifiable today" is no longer true.** The later retired
   identity's public half was recovered from the repository (`pubkey.asc`,
   `signer-public-key.asc`), so **16 anchor tags and 2 reviewer tags verify**, 11 of the
   anchor ones on `master`. 21 signed tags remain unverifiable: 20 from the earlier
   identity and 1 from the third, commit-level identity.
2. **The retirement is now mechanically enforced**, not merely written down. As first
   drafted, the gate authorised any fingerprint on its allowlist, so a retired identity
   could have authorised a release. Release authority and `--historical` verification are
   now separate, and an identity on both lists is a fatal configuration contradiction.

One finding to record against the SEAT decision: the recovered half's user ID is the
owner's **personal** identity (`SEAN CHRISTOPHER SOUTHWICK <schlagetorren@gmail.com>`), so
project release signatures have in fact been made with personal signing authority. The
decision to move to a seat identity corrects a real practice rather than anticipating one.

Recovery note for whoever repeats this: both identities return 404 from
keys.openpgp.org and keyserver.ubuntu.com, so neither was ever published to a public
keyserver. The recovered half survived only as a file in the repository. Publishing a
public half is what keeps an identity's history verifiable after its pen is gone.

## Prerequisites before anything is installed

1. Recover the public halves of both retired identities, or the 37 historical signatures
   become permanently unverifiable even though they remain unaltered. The anchor's own
   history points at where one lived: `ci: import signing key before release verification`,
   2026-07-20.
2. Confirm the loss classification of the pen (lost and protected, versus lost and
   unprotected).
3. Ratify the two decisions explicitly, in the thread, since delegation is not ratification.
