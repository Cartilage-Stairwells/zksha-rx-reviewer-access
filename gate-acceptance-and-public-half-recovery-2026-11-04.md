# Gate acceptance proven, public halves partly recovered

Executed under the owner's approval of 2026-11-04: fixture acceptance test, the eight
rejection mutants, and historical public-half recovery. **No production identity was
generated, nothing was signed, nothing was committed or pushed, no tag was altered, and
branch protection stayed on throughout.**

## 1. The acceptance path now runs

A disposable fixture identity was generated in a scratch directory, signed one annotated
tag on a scratch branch, and the gate returned:

```
ok   allowlist present (1 fingerprints)
ok   annotated tag object
ok   target c891df65e9 is on main
ok   signature verifies
ok   signer fingerprint 5D38FB8C4F0110B1B74B119F09F2C11E64773952 is pinned
RESULT: PASS
```

The fixture is labelled `FIXTURE TEST ONLY <fixture-test-only@invalid>` and its fingerprint
appears only in `allowlist.fixture-test-only.txt`, never in the production list.

**Discarded as required, in a way that keeps the test re-runnable:** the secret half was
destroyed (`delete-secret-and-public-keys`, then re-import of the public half). Signing
with the fixture now fails, verified by attempting a new signature. The public half and the
signed test tag are retained because they are inert: they let the acceptance test be
repeated by anyone without restoring the ability to sign. Secret identities remaining: 0.

## 2. Thirteen cases, two passes, eleven refusals

| # | Case | Outcome |
|---|---|---|
| 1 | fixture-signed, test-only allowlist | **PASS** (acceptance) |
| 2 | unsigned tag, valid allowlist | FAIL: carries no signature |
| 3 | comment-only allowlist | FAIL: no fingerprint entries |
| 4 | retired signer placed inside the release list | FAIL: configuration contradiction |
| 5 | anchor tag signed by the retired identity | FAIL: RETIRED, historical only |
| 6 | anchor tag signed, off-branch | FAIL: not reachable from `master` |
| 7 | lightweight tag, commit-signed | FAIL: not an annotated tag object |
| 8 | nonexistent tag | FAIL: does not exist |
| 9 | empty allowlist | FAIL: no fingerprint entries |
| 10 | foreign allowlist | FAIL: RETIRED |
| 11 | fixture-signed against a non-test list | FAIL: not on the allowlist |
| 12 | authorised signer, tag off-branch | FAIL: not reachable from `main` |
| 13 | published tag in `--historical` mode | **PASS** (verification, not authorisation) |

The eight original mutants still fail independently. Cases 13 and 4 are new, and they came
from a defect this exercise exposed.

## 3. Defect found in my own gate: retirement was not enforceable

As first written, the gate authorised **any** fingerprint on the allowlist, so a retired
identity whose fingerprint sat in that file would have passed a release. That contradicts
the retirement terms directly. Two changes:

- **Two authorities.** Release mode and `--historical` mode are now distinct. A signer that
  is retired and NOT on the release list fails release mode with
  `RETIRED: authorised for historical verification only, never for release`, and may pass
  `--historical`, which prints that verification authorises nothing new.
- **Contradiction is fatal.** An identity appearing on both lists now fails closed rather
  than resolving in favour of the allowlist.

A third defect was found in my own tests, not the gate: a comment-only allowlist counted as
configured, because the check tested file size rather than entry count. Now zero entries is
refused. Two of my earlier "failing" cases were also vacuous, one because the tag was never
off-branch, one because the "production" allowlist I built contained the fixture's own
fingerprint. Both are corrected above; the first is why case 4 exists.

## 4. Public half recovery: half the history is verifiable again

| Identity | State | Consequence |
|---|---|---|
| later, the pen `84692E62…E747C3AF…22573539` | **RECOVERED**, published in the repository as `pubkey.asc` and `signer-public-key.asc` | 16 anchor tags + 2 reviewer tags now verify |
| earlier `901EEB9E…7A69CA58…234E4798` | **NOT FOUND** | 20 tags remain unverifiable |
| commit-level `B5690EEEBB952194` | not searched for yet, no packet-level fingerprint available | 1 tag remains unverifiable |

Both identities returned **404 from keys.openpgp.org and keyserver.ubuntu.com**, so neither
was ever published to a public keyserver. The one that was recovered survived only because
the repository carries its public half as a file. That is the general lesson: publication in
a keyserver or a repository file is what keeps history verifiable after a pen is lost.

Sweep of all 60 anchor tags after the recovery: **16 verify** (11 of them on `master`),
21 signed but unverifiable, 23 unsigned.

## 5. What the recovered identity actually was

The recovered half's user ID reads:

```
SEAN CHRISTOPHER SOUTHWICK (https://toolintell.com) <schlagetorren@gmail.com>
```

The pen was the owner's **personal** signing identity, not a seat identity. Project release
authority has therefore been exercised through a personal identity, which is precisely the
conflation the SEAT decision is meant to end. Recorded as a finding, not as a fault: the
retirement stands, and the replacement being seat-held is now a correction of an actual
practice rather than a preference.

## 6. State of the package

Still uncommitted, still uninstalled, and now stronger: acceptance proven, eleven refusals
independently demonstrated, retirement mechanically enforced, and the historical
verification mode separated from release authorisation.

Still open: the earlier identity's public half, the third identity's full fingerprint, the
ratification of the two identity-policy amendments, and any production identity at all.
