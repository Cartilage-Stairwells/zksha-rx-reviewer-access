#!/usr/bin/env bash
# Custody disclosure check. A release record must declare its custody mode, and must not assert
# independent control of the signing identity, because a release record cannot establish that.
#
# Ported from the tested reference implementation filed at 61c011b in tscp-workstream-records
# (whitespace-resilient revision 31cf3c8). One change on install: a duplicate declaration is now
# refused. The reference implementation took the first declaration and silently ignored later
# ones, so a record carrying "joint" and then "independent" passed while asserting something the
# check exists to prevent.
#
# Usage: check_custody_disclosure.sh <path to REVIEW_RELEASE.md>
set -uo pipefail
FILE="${1:?usage: check_custody_disclosure.sh <REVIEW_RELEASE.md>}"
[ -f "$FILE" ] || { echo "FAIL no such file: $FILE"; exit 1; }
rc=0

# Line endings normalised once, up front: a record authored with CRLF is compliant prose and must
# not be refused for its newlines.
DOC=$(tr -d '\r' < "$FILE")

# Prose matched with every run of whitespace flattened to one space, so a line break or an
# indentation tab inside the required wording cannot defeat the match.
PROSE=$(printf '%s' "$DOC" | tr -s '[:space:]' ' ')

DECLS=$(printf '%s\n' "$DOC" | grep -c '^CUSTODY-DISCLOSURE: ' || true)
if [ "$DECLS" -eq 0 ]; then
    echo "FAIL no CUSTODY-DISCLOSURE line at the start of a line"
    exit 1
fi
if [ "$DECLS" -gt 1 ]; then
    echo "FAIL $DECLS CUSTODY-DISCLOSURE declarations; exactly one is permitted"
    exit 1
fi

val=$(printf '%s\n' "$DOC" | grep -m1 '^CUSTODY-DISCLOSURE: ' | sed 's/^CUSTODY-DISCLOSURE:[[:space:]]*//; s/[[:space:]]*$//')

case "$val" in
    joint|provisional)
        if printf '%s' "$PROSE" | grep -Eiq 'accepted[[:space:]]+deviation'; then
            echo "ok   custody mode '$val' declared, deviation disclosed in words"
        else
            echo "FAIL custody mode '$val' declared without the accepted-deviation disclosure"
            rc=1
        fi ;;
    independent)
        echo "FAIL 'independent' must not be asserted: a release record cannot establish it"
        rc=1 ;;
    *)
        echo "FAIL unrecognised custody mode '$val' (expected joint, provisional, or independent)"
        rc=1 ;;
esac

[ "$rc" -eq 0 ] && echo "RESULT: PASS" || echo "RESULT: REFUSED"
exit "$rc"
