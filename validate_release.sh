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

# 5. Historical labeling
echo ""
echo "5. Historical labeling"
HIST_REFS=$(grep -rn "historical" --include="*.md" . 2>/dev/null | grep -v ".git/" | wc -l)
if [ "$HIST_REFS" -gt 0 ]; then
    ok "$HIST_REFS historical references labeled"
else
    warn "No historical labels found"
fi

# 6. Signature verification (additional trust layer)
echo ""
echo "6. Signature verification (additional trust layer)"
if [ "$CURRENT_TAG" != "unknown" ]; then
    TAG_SIG=$(git tag -v "$CURRENT_TAG" 2>&1 || true)
    if echo "$TAG_SIG" | grep -q "good signature"; then
        ok "Tag has valid GPG signature"
    elif echo "$TAG_SIG" | grep -q "no signature"; then
        warn "Tag is not signed — additional trust layer, not required for review"
    else
        warn "Tag signature inconclusive"
    fi
else
    warn "Cannot verify tag signature (no tag)"
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
