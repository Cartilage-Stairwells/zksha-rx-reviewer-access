#!/usr/bin/env bash
# validate_release.sh — Mechanical release identity gate
# Run from repo root after: git clone --branch <tag> <repo>
set -euo pipefail

PASS=0; FAIL=0; WARN=0
ok()   { echo "  ✅ $1"; PASS=$((PASS+1)); }
fail() { echo "  ❌ $1"; FAIL=$((FAIL+1)); }
warn() { echo "  ⚠️  $1"; WARN=$((WARN+1)); }

echo "=== Release Validation Gate ==="
echo ""

# 1. Tag identity
echo "1. Tag identity"
CURRENT_TAG=$(git describe --tags --exact-match 2>/dev/null || echo "none")
if [ "$CURRENT_TAG" != "none" ]; then
    ok "Checked out at tag: $CURRENT_TAG"
else
    warn "Not on a tagged commit (detached HEAD or branch)"
    CURRENT_TAG="unknown"
fi
CURRENT_COMMIT=$(git rev-parse HEAD)
ok "Commit: ${CURRENT_COMMIT:0:12}"

# 2. Release document identity
echo ""
echo "2. Release document identity"
if [ -f "REVIEW_RELEASE.md" ]; then
    DOC_TAG=$(grep -oP 'review-v[0-9]+\.[0-9]+\.[0-9]+' REVIEW_RELEASE.md | head -1 || echo "none")
    if [ "$DOC_TAG" = "$CURRENT_TAG" ]; then
        ok "REVIEW_RELEASE.md references matching tag: $DOC_TAG"
    else
        fail "REVIEW_RELEASE.md references '$DOC_TAG' but checked out tag is '$CURRENT_TAG'"
    fi
else
    fail "REVIEW_RELEASE.md not found"
fi
if [ -f "HISTORY.md" ]; then ok "HISTORY.md exists"; else warn "HISTORY.md not found"; fi

# 3. Checksum integrity
echo ""
echo "3. Checksum integrity"
if [ -f "SHA256SUMS" ]; then
    if sha256sum -c SHA256SUMS > /dev/null 2>&1; then
        CHECKSUM_COUNT=$(wc -l < SHA256SUMS)
        ok "SHA256SUMS verified ($CHECKSUM_COUNT files)"
    else
        fail "SHA256SUMS verification failed"
    fi
else
    fail "SHA256SUMS not found"
fi

# 4. Dead path check — verify every file in SHA256SUMS exists
echo ""
echo "4. Dead path check"
DEAD=0
while IFS= read -r line; do
    filepath=$(echo "$line" | awk '{print $2}' | sed 's|^\./||')
    if [ -n "$filepath" ] && [ ! -e "$filepath" ]; then
        fail "SHA256SUMS references missing file: $filepath"
        DEAD=$((DEAD+1))
    fi
done < SHA256SUMS
if [ "$DEAD" -eq 0 ]; then
    ok "All SHA256SUMS paths exist"
fi

# 4b. Manifest completeness, present to listed
echo ""
echo "4b. Manifest completeness (present to listed)"
if [ -f "SHA256SUMS" ]; then
    LISTED=$(awk '{print $2}' SHA256SUMS | sort -u)
    # Non-regular entries are enumerated before the file list, because a digest manifest cannot
    # represent a symlink, fifo, socket or device: no content digest means for those what a regular
    # file's digest means. Enumerating and refusing them is the only honest option, and silently
    # skipping them is what let an unlisted symlink pass this step.
    NONREG=$(find . \( -type l -o -type p -o -type s -o -type b -o -type c \) -not -path './.git/*' | sed 's|^\./||' | sort -u)
    if [ -n "$NONREG" ]; then
        printf '%s\n' "$NONREG" | sed 's/^/     unsupported: /'
        fail "$(printf '%s\n' "$NONREG" | wc -l) non-regular entr(ies) present; a release tree may hold regular files only"
    fi
    # Empty directories are deliberately not enumerated: git cannot carry one, so a tree taken from
    # the repository cannot contain one, and refusing them would fail a working copy over a state
    # the artifact cannot express.
    PRESENT=$(find . -type f -not -path './.git/*' | sed 's|^\./||' | grep -vx 'SHA256SUMS' | sort -u)
    # Fail closed. This comparison used to carry "|| true", so a failing comm produced an empty
    # unlisted set and a printed pass, which is the same silent-degradation class as defect D1.
    if ! UNLISTED=$(comm -13 <(printf '%s\n' "$LISTED") <(printf '%s\n' "$PRESENT")); then
        fail "completeness comparison failed (comm exited non-zero); completeness is NOT established"
    elif [ -z "$UNLISTED" ]; then
        ok "Every present file is listed in SHA256SUMS"
    else
        printf '%s\n' "$UNLISTED" | sed 's/^/     unlisted: /'
        fail "$(printf '%s\n' "$UNLISTED" | wc -l) present file(s) absent from SHA256SUMS"
    fi
else
    fail "SHA256SUMS missing; completeness cannot be established"
fi

# 5. Historical labeling
echo ""
echo "5. Historical labeling"
HIST_REFS=$(grep -rn "historical" --include="*.md" . 2>/dev/null | grep -v ".git/" | wc -l)
if [ "$HIST_REFS" -gt 0 ]; then
    ok "$HIST_REFS historical references labeled"
else
    warn "No historical labels found"
fi

# 6. Release signature (fail-closed; see gate-amendment/README.md)
echo ""
echo "6. Release signature (fail-closed)"
GATE="./gate-amendment/check_release_signature.sh"
ALLOWLIST="./gate-amendment/allowlist.pinned.txt"
if [ ! -x "$GATE" ]; then
    fail "signature gate missing or not executable at $GATE"
else
    if GATE_OUT=$("$GATE" . "$CURRENT_TAG" HEAD "$ALLOWLIST" 2>&1); then
        ok "Signature gate passed: tag signed by the pinned release identity"
    else
        echo "$GATE_OUT" | sed 's/^/     /'
        fail "Signature gate refused this release (reasons above)"
    fi
fi

# 7. Custody disclosure
echo ""
echo "7. Custody disclosure"
if [ -f "gate-amendment/check_custody_disclosure.sh" ]; then
    CD_RC=0
    CD_OUT=$(bash gate-amendment/check_custody_disclosure.sh REVIEW_RELEASE.md 2>&1) || CD_RC=$?
    if [ "$CD_RC" -eq 0 ] && printf '%s\n' "$CD_OUT" | grep -q 'RESULT: PASS'; then
        ok "Custody disclosure declared and consistent with the frozen position"
    else
        echo "$CD_OUT" | sed 's/^/     /'
        fail "Custody disclosure check refused this release (exit status $CD_RC)"
    fi
else
    fail "gate-amendment/check_custody_disclosure.sh missing; disclosure is unenforced"
fi

# Summary
echo ""
echo "=== Summary ==="
echo "  Passed:   $PASS"
echo "  Failed:   $FAIL"
echo "  Warnings: $WARN"
echo ""
if [ "$FAIL" -gt 0 ]; then
    echo "❌ RELEASE VALIDATION FAILED — $FAIL issue(s)"
    exit 1
else
    echo "✅ RELEASE VALIDATION PASSED"
    exit 0
fi
