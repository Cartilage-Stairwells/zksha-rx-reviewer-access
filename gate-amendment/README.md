# Fail-closed release signature gate — prepared, not yet installed

Status: **rehearsed against real repository data, including the acceptance path. Not
committed to the docs/quickstart-release-and-count-pointer branch of the reviewer repository and not merged to main, with the pinned allowlist intentionally empty so the gate refuses every release.** No production identity
exists. One disposable fixture identity was generated under explicit approval, signed one
test tag, and had its signing capability destroyed; see below.

`check_release_signature.sh <repo> <tag> <release-branch> <allowlist>`

Passes only if every one of these holds, and fails (exit 1) otherwise:

1. the tag exists and is an **annotated tag object** (a lightweight tag fails even when the
   commit it points at is signed, because the signature then attests the commit, not the
   release name);
2. it carries an OpenPGP signature that **verifies cryptographically**;
3. the **verifying key's fingerprint** is on the pinned allowlist, taken from the
   `VALIDSIG` line so it is the key that actually verified and not merely a claim inside
   the packet;
4. the tag's **target commit is reachable from the release branch**;
5. the **allowlist is present and non-empty**: an unconfigured gate refuses to pass
   anything rather than passing everything.

It reports every unmet condition rather than stopping at the first, so a failure names all
of its causes. It never warns: there is no state in which a release without a valid,
authorised signature passes.

## Why the old check had to be replaced, not repaired

The shipped check greps for the lowercase string `good signature`; GnuPG writes
`Good signature`. The pass branch was therefore unreachable for any key, correctly used.
It also tested only that *some* signature existed, never whose, and never bound the tag to
the branch. Reproduced on a correctly signed tag: `⚠️ Tag signature inconclusive`.

## Mutation results, run against real data, no keys generated

| # | Case | Source | Result |
|---|---|---|---|
| 1 | unsigned annotated tag, on branch | `zksha-rx` `review-v0.1.22` | FAIL: carries no signature |
| 2 | signed tag, on branch, public half absent | `zksha-rx` `review-v0.1.10` | FAIL: signer's public half is not present |
| 3 | signed tag, on master, public half absent | anchor `TSCP-v2.4.0-formal-sealed` | FAIL: public half absent |
| 4 | signed tag, off-branch | anchor `v0.1.0` | FAIL: not reachable from `master`, and unverifiable |
| 5 | lightweight tag, commit-signed | anchor `v0.3-reproduction` | FAIL: not an annotated tag object |
| 6 | tag that does not exist | — | FAIL: does not exist |
| 7 | correct tag, empty allowlist | `zksha-rx` `review-v0.1.22` | FAIL: refusing to pass anything |
| 8 | foreign allowlist | `zksha-rx` `review-v0.1.10` | FAIL: allowlist cannot rescue an unverifiable tag |

Eight of eight behave as specified. Each failure names its cause.

## Acceptance, now proven (2026-11-04)

The pass path has run. A disposable fixture identity, labelled `FIXTURE TEST ONLY` and
pinned only in `allowlist.fixture-test-only.txt`, produced `RESULT: PASS` on a correctly
signed annotated tag on-branch. Its secret half was then **destroyed**: signing with the
fixture now fails, and the retained public half plus the signed test tag keep the acceptance
test repeatable without restoring the ability to sign. No production identity exists.

Thirteen cases now behave as intended: the fixture acceptance and a `--historical`
verification pass; eleven refusals each name their cause. See
`../gate-acceptance-and-public-half-recovery-2026-11-04.md`.

## Two authorities, which the first draft conflated

A defect found by this exercise: as written, the gate authorised any fingerprint on the
allowlist, so a **retired** identity could have authorised a release. Retirement is now
mechanical rather than documentary.

- release mode (default): the signer must be on the release list. A retired signer fails
  with `RETIRED: authorised for historical verification only, never for release`.
- `--historical`: verifying an already-published tag. A retired signer is acceptable here
  and only here, and the run states that it authorises nothing new.
- an identity present on **both** lists is a fatal configuration contradiction.

## Prerequisite, partly cleared

The later retired identity's public half was recovered from the repository itself
(`pubkey.asc`, `signer-public-key.asc`), so 16 anchor tags and 2 reviewer tags now verify.
The earlier identity and the third, commit-level identity remain without public halves, so
21 signed tags stay unverifiable. Both identities return 404 from the public keyservers:
they were never published there.
