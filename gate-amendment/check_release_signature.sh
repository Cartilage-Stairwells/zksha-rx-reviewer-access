#!/usr/bin/env bash
# Fail-closed release signature gate.
#   usage: check_release_signature.sh <repo> <tag> <release-branch> <allowlist-file> [--historical]
#
# Two authorities, never one:
#   release mode (default)  the signer must be on the release allowlist. A signer that appears
#                           only on the retired list FAILS: retirement means historical
#                           verification only, never release authority.
#   --historical            verifying an already-published tag. A retired signer is acceptable
#                           HERE and only here, and the result is never a release authorisation.
# Passes only when ALL of the following hold, and fails (non-zero) otherwise:
#   1. the tag exists and is an ANNOTATED tag object
#   2. it carries an OpenPGP signature that verifies cryptographically
#   3. the verifying key's fingerprint is on the pinned allowlist
#   4. the tag's target commit is reachable from the release branch
#   5. the allowlist itself is present and non-empty
# Every unmet condition is reported, not just the first, and any unmet condition is fatal.
set -uo pipefail
REPO=${1:?repo}; TAG=${2:?tag}; BRANCH=${3:?branch}; ALLOW=${4:?allowlist}
MODE=release; [ "${5:-}" = "--historical" ] && MODE=historical
RETIRED="$(dirname "$0")/allowlist.retired-only.txt"
F=0
bad(){ printf '  FAIL %s\n' "$*"; F=1; }
ok(){  printf '  ok   %s\n' "$*"; }
g(){ git -C "$REPO" "$@"; }

echo "gate: $TAG in $REPO (branch $BRANCH)"

# 5. configuration must be usable, else nothing below can be trusted
ENTRIES=$(grep -ve '^[[:space:]]*#' "$ALLOW" 2>/dev/null | grep -cve '^[[:space:]]*$')
if [ ! -s "$ALLOW" ] || [ "${ENTRIES:-0}" -eq 0 ]; then
  bad "allowlist '$ALLOW' has no fingerprint entries (comments only, or missing): refusing to pass anything"
else
  ok "allowlist present ($ENTRIES fingerprints)"
  # an identity may not be simultaneously authorised and retired: that is a contradiction,
  # not a policy, and it must fail closed rather than resolve in favour of the allowlist
  if [ -s "$RETIRED" ]; then
    OVERLAP=$(grep -v '^[[:space:]]*#' "$ALLOW" | grep -v '^[[:space:]]*$' | tr -d ' \t' | while read -r f; do
      [ -n "$f" ] && grep -qiF "$f" "$RETIRED" && echo "$f"; done | head -1)
    if [ -n "$OVERLAP" ]; then
      bad "configuration contradiction: signer $OVERLAP appears on BOTH the release and retired lists"
    else
      ok "no identity is both authorised and retired"
    fi
  fi
fi

# 1. tag exists and is annotated
if ! g rev-parse --verify -q "refs/tags/$TAG" >/dev/null; then
  bad "tag '$TAG' does not exist"
  echo "RESULT: FAIL"; exit 1
fi
TYPE=$(g cat-file -t "refs/tags/$TAG")
if [ "$TYPE" = tag ]; then ok "annotated tag object"; else bad "tag is '$TYPE', not an annotated tag object"; fi

# 4. target reachable from the release branch
COMMIT=$(g rev-list -n1 "refs/tags/$TAG" 2>/dev/null)
if [ -z "$COMMIT" ]; then bad "tag resolves to no commit"
elif ! g rev-parse --verify -q "refs/heads/$BRANCH" >/dev/null && ! g rev-parse --verify -q "$BRANCH" >/dev/null; then
  bad "release branch '$BRANCH' not found"
elif g merge-base --is-ancestor "$COMMIT" "$BRANCH" 2>/dev/null; then
  ok "target $(echo "$COMMIT"|cut -c1-10) is on $BRANCH"
else
  bad "target $(echo "$COMMIT"|cut -c1-10) is NOT reachable from $BRANCH"
fi

# A reviewer does not necessarily hold the release identity's public half, so any public
# halves shipped beside this gate are imported into a throwaway keyring inside the
# repository's git directory. Importing grants no trust; it only makes the signature
# checkable from a clean clone.
KEYDIR="$(dirname "$0")/keys"
if ls "$KEYDIR"/*.asc >/dev/null 2>&1; then
  KEYRING="$(g rev-parse --absolute-git-dir)/gate-keyring"
  mkdir -p "$KEYRING"; find "$KEYRING" -mindepth 1 -delete 2>/dev/null || true; chmod 700 "$KEYRING"
  export GNUPGHOME="$KEYRING"
  for k in "$KEYDIR"/*.asc; do gpg --batch --quiet --import "$k" >/dev/null 2>&1 || true; done
  trap 'find "$KEYRING" -mindepth 1 -delete 2>/dev/null || true' EXIT
  ok "shipped public halves imported into a throwaway keyring ($(ls "$KEYDIR"/*.asc | wc -l) file(s))"
fi

# 2 and 3. signature must verify, and by a pinned fingerprint
RAW=$(g verify-tag --raw "refs/tags/$TAG" 2>&1)
HUMAN=$(g verify-tag "refs/tags/$TAG" 2>&1)
if g verify-tag "refs/tags/$TAG" >/dev/null 2>&1; then
  ok "signature verifies"
else
  if echo "$HUMAN" | grep -q "No public key"; then
    bad "signature cannot be verified: the signer's public half is not present (fail closed, not a warning)"
  elif echo "$HUMAN" | grep -qi "no signature"; then
    bad "tag carries no signature"
  else
    bad "signature does not verify"
  fi
fi
FPR=$(echo "$RAW" | awk '/VALIDSIG/{print $3}' | head -1)
if [ -n "$FPR" ]; then
  IN_ALLOW=no; grep -qiF "$FPR" "$ALLOW" 2>/dev/null && IN_ALLOW=yes
  IN_RETIRED=no; [ -s "$RETIRED" ] && grep -qiF "$FPR" "$RETIRED" 2>/dev/null && IN_RETIRED=yes
  if [ "$MODE" = historical ]; then
    if [ "$IN_ALLOW" = yes ] || [ "$IN_RETIRED" = yes ]; then
      ok "historical verification: signer $FPR is known (retired=$IN_RETIRED release=$IN_ALLOW)"
      echo "  note this result verifies a published tag; it does not authorise anything new"
    else
      bad "signer fingerprint $FPR is on neither list"
    fi
  else
    if [ "$IN_ALLOW" = yes ]; then
      ok "signer fingerprint $FPR is pinned for release"
    elif [ "$IN_RETIRED" = yes ]; then
      bad "signer $FPR is RETIRED: authorised for historical verification only, never for release"
    else
      bad "signer fingerprint $FPR is NOT on the allowlist"
    fi
  fi
else
  bad "no verified signer fingerprint to compare against the allowlist"
fi

[ "$F" -eq 0 ] && { echo "RESULT: PASS"; exit 0; } || { echo "RESULT: FAIL"; exit 1; }
