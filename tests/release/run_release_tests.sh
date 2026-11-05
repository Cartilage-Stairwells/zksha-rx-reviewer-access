#!/usr/bin/env bash
# Adversarial baseline suite for the release gate. Implements docs/ADVERSARIAL-TEST-PLAN.md.
#
# Runs against the repository it lives in. Every case clones a fresh fixture, changes exactly
# one property, and records what the release path actually does. Nothing is asserted about
# what it ought to do without also recording what it does.
#
# Usage: run_release_tests.sh [--strict]
#   default   exit non-zero only on FAIL (observed matches neither documented state)
#   --strict  also exit non-zero on KNOWN-DEFECT, which is how a correction is demonstrated
set -uo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
REPO=$(cd "$HERE/../.." && pwd)
TAG="${RELEASE_TAG:-review-v0.1.27}"
STRICT=0; CAND=""; NOTRUN=0
while [ $# -gt 0 ]; do
    case "$1" in
        --strict)    STRICT=1 ;;
        --candidate) shift; CAND="${1:-}"; [ -n "$CAND" ] || { echo "--candidate needs a path" >&2; exit 2; } ;;
        *)           echo "unknown argument: $1" >&2; exit 2 ;;
    esac
    shift
done
WORK=$(mktemp -d); trap 'rm -rf "$WORK"' EXIT

PASS=0; KNOWN=0; FIXEDC=0; FAILD=0; FAILED_IDS=""

fixture() {   # fixture <id> [tag]
  local d="$WORK/case-$1"
  git clone -q --no-hardlinks "$REPO" "$d" 2>/dev/null || { echo "fixture clone failed" >&2; exit 2; }
  [ "${2:-}" = tag ] && git -C "$d" checkout -q "$TAG" 2>/dev/null
  # A candidate validator is dropped over the fixture's copy only. The repository's own
  # validate_release.sh is never touched by this harness.
  if [ -n "$CAND" ]; then
      cp "$CAND" "$d/validate_release.sh"
      # Keep the fixture internally consistent. validate_release.sh is itself in the manifest, so
      # a candidate dropped in without updating its entry fails RC-04 before any case is measured,
      # which is the mitigation T-12 predicted and here confirms.
      python3 -c 'import hashlib,pathlib,sys
d=pathlib.Path(sys.argv[1]); t=d/"SHA256SUMS"; v=d/"validate_release.sh"
h=hashlib.sha256(v.read_bytes()).hexdigest()
out=[]; hit=False
for l in t.read_text().splitlines():
    p=l.partition("  ")[2]
    if p=="validate_release.sh": out.append(h+"  "+p); hit=True
    else: out.append(l)
assert hit, "validate_release.sh not in fixture manifest"
t.write_text("\n".join(out)+"\n")' "$d"
  fi
  printf '%s' "$d"
}

step_block() {  # step_block <dir> <stepnum> -> that step's output lines
  ( cd "$1" && bash validate_release.sh 2>&1 || true ) |
  awk -v n="$2" '
    $0 ~ "^" n "\\."      { b=1; next }
    b && (/^[0-9]+\. / || /^=== Summary/) { b=0 }
    b { print }'
}
step_state() {  # step_state <dir> <stepnum> -> pass|fail|absent
  # absent is a distinct outcome on purpose. An empty block means the validator never reached
  # that step, and reporting that as "pass" would make every refusal case silently vacuous.
  local b
  b=$(step_block "$1" "$2")
  if [ -z "$b" ]; then echo absent; return; fi
  printf '%s\n' "$b" | grep -q '❌' && echo fail || echo pass
}
validator_exit() { ( cd "$1" && bash validate_release.sh >/dev/null 2>&1 ); echo $?; }
reproduce_exit() { ( cd "$1" && make reproduce >/dev/null 2>&1 ); echo $?; }

record() {  # record <id> <mutation> <observed> <expected_now> <expected_fixed> <defect>
  local id=$1 mut=$2 obs=$3 now=$4 fix=$5 defect=${6:-no} verdict
  if   [ "$obs" = "$fix" ] && [ "$obs" != "$now" ]; then verdict=FIXED;        FIXEDC=$((FIXEDC+1))
  elif [ "$obs" = "$now" ]; then
       if [ "$defect" = yes ]; then verdict=KNOWN-DEFECT; KNOWN=$((KNOWN+1))
       else verdict=PASS; PASS=$((PASS+1)); fi
  else verdict=FAIL; FAILD=$((FAILD+1)); FAILED_IDS="$FAILED_IDS $id"
  fi
  printf '  %-5s %-46s observed=%-4s %s\n' "$id" "$mut" "$obs" "$verdict"
}

# Preflight: the harness must be able to see the steps it is measuring. If a step produces no
# output at all, every refusal case below would record "pass" for the wrong reason.
pre=$(fixture 00)
for n in 3 4 7; do
  st=$(step_state "$pre" "$n")
  if [ "$st" = absent ]; then
      echo "❌ preflight: step $n produced no parseable output; the harness cannot measure anything"
      exit 2
  fi
done
echo "  preflight: steps 3, 4 and 7 parse, and observe '$([ "$(step_state "$pre" 7)" = pass ] && echo pass || echo fail)' on a clean fixture"
echo

echo "=== Release gate adversarial baseline ==="
echo "  repository: $REPO"
echo "  tag under test: $TAG   mode: $([ $STRICT -eq 1 ] && echo strict || echo baseline)"
echo

# ---------- Group A: the checker's logic ----------
d=$(fixture 01);                                   record T-01 "clean record, no mutation"                  "$(step_state "$d" 7)" pass pass
d=$(fixture 02); sed -i '/^CUSTODY-DISCLOSURE: /d' "$d/REVIEW_RELEASE.md"
                                                   record T-02 "declaration line removed"                   "$(step_state "$d" 7)" fail fail
d=$(fixture 03); sed -i 's/^CUSTODY-DISCLOSURE: joint/CUSTODY-DISCLOSURE: independent/' "$d/REVIEW_RELEASE.md"
                                                   record T-03 "value changed to independent"               "$(step_state "$d" 7)" fail fail
d=$(fixture 04); printf '\nCUSTODY-DISCLOSURE: independent\n' >> "$d/REVIEW_RELEASE.md"
                                                   record T-04 "second declaration appended"                "$(step_state "$d" 7)" fail fail
d=$(fixture 05); python3 - "$d/REVIEW_RELEASE.md" <<'PY'
import re,sys,pathlib
p=pathlib.Path(sys.argv[1]); p.write_text(re.sub(r'accepted\s+deviation','owner determination',p.read_text(),flags=re.I))
PY
                                                   record T-05 "accepted-deviation wording removed"         "$(step_state "$d" 7)" fail fail
d=$(fixture 06); sed -i 's/^CUSTODY-DISCLOSURE: joint/   CUSTODY-DISCLOSURE: joint/' "$d/REVIEW_RELEASE.md"
                                                   record T-06 "declaration indented off line start"        "$(step_state "$d" 7)" fail fail
d=$(fixture 07); sed -i 's/^CUSTODY-DISCLOSURE: joint/CUSTODY-DISCLOSURE: joint-ish/' "$d/REVIEW_RELEASE.md"
                                                   record T-07 "value changed to joint-ish"                 "$(step_state "$d" 7)" fail fail
d=$(fixture 08); python3 - "$d/REVIEW_RELEASE.md" <<'PY'
import sys,pathlib
p=pathlib.Path(sys.argv[1]); p.write_bytes(p.read_text().replace('\n','\r\n').encode())
PY
                                                   record T-08 "record rewritten with CRLF endings"         "$(step_state "$d" 7)" pass pass
d=$(fixture 09); python3 - "$d/REVIEW_RELEASE.md" <<'PY'
import re,sys,pathlib
p=pathlib.Path(sys.argv[1]); p.write_text(re.sub(r'accepted\s+deviation','accepted\tdeviation',p.read_text(),count=1,flags=re.I))
PY
                                                   record T-09 "tab inside the required wording"            "$(step_state "$d" 7)" pass pass
d=$(fixture 10); line=$(grep -m1 '^CUSTODY-DISCLOSURE: ' "$d/REVIEW_RELEASE.md"); sed -i '/^CUSTODY-DISCLOSURE: /d' "$d/REVIEW_RELEASE.md"; printf '\n%s\n' "$line" >> "$d/HISTORY.md"
                                                   record T-10 "declaration moved to another file"          "$(step_state "$d" 7)" fail fail

# ---------- Group B: the enforcement boundary ----------
d=$(fixture 11); cat > "$d/gate-amendment/check_custody_disclosure.sh" <<'EOF'
#!/usr/bin/env bash
echo "ok   custody mode 'joint' declared, deviation disclosed in words"
echo "RESULT: PASS"
exit 1
EOF
                                                   record T-11 "checker prints PASS and exits 1"             "$(step_state "$d" 7)" pass fail yes
d=$(fixture 12); cat > "$d/gate-amendment/check_custody_disclosure.sh" <<'EOF'
#!/usr/bin/env bash
echo "RESULT: PASS"
exit 0
EOF
s7=$(step_state "$d" 7); s3=$(step_state "$d" 3)
if [ "$s3" != fail ]; then printf '  T-12  MISMATCH: expected step 3 to catch the replaced checker, observed %s\n' "$s3"; fi
                                                   record T-12 "checker replaced by a PASS-printing wrapper" "$s7" pass pass
d=$(fixture 13); rm -f "$d/gate-amendment/check_custody_disclosure.sh"
                                                   record T-13 "checker file deleted"                        "$(step_state "$d" 7)" fail fail
d=$(fixture 14); printf 'x' >> "$d/README.md"; printf '\nCUSTODY-DISCLOSURE: independent\n' >> "$d/REVIEW_RELEASE.md"
e=$(validator_exit "$d"); [ "$e" -ne 0 ] && e=pass || e=fail
                                                   record T-14 "step 3 fails, step 7 passes, later cannot override" "$e" pass pass

# ---------- Group C: package integrity ----------
d=$(fixture 15); printf 'x' >> "$d/README.md"
                                                   record T-15 "one byte appended to a listed file"          "$(step_state "$d" 3)" fail fail
d=$(fixture 16); rm -f "$d/LICENSE"
                                                   record T-16 "one listed file deleted"                     "$(step_state "$d" 4)" fail fail
d=$(fixture 17); printf 'stray\n' > "$d/UNLISTED-ADDITION.txt"
                                                   record T-17 "one unlisted file added"                     "$(step_state "$d" 4)" pass fail yes

# ---------- Group D: the full pipeline ----------
d=$(fixture 18 tag); e=$(validator_exit "$d"); [ "$e" -eq 0 ] && e=pass || e=fail
                                                   record T-18 "tagged clone, no mutation"                   "$e" pass pass
d=$(fixture 19 tag); printf 'x' >> "$d/README.md"; e=$(reproduce_exit "$d"); [ "$e" -ne 0 ] && e=pass || e=fail
                                                   record T-19 "digest tampered, make reproduce"             "$e" pass pass
d=$(fixture 20 tag); sed -i '/^CUSTODY-DISCLOSURE: /d' "$d/REVIEW_RELEASE.md"; e=$(reproduce_exit "$d"); [ "$e" -ne 0 ] && e=pass || e=fail
                                                   record T-20 "disclosure removed, make reproduce"          "$e" pass pass

# ---------- Group E: closure of D1 and D2 ----------
# These run only against a candidate. They are the part that distinguishes "the original
# reproduction stopped reproducing" from "the property now holds": each defect has a negative
# control that the fix must still accept, so a validator that simply refuses everything fails here.
# Closure cases run against a candidate, or against the installed gate once it carries step 4b,
# so the same nine cases that proved the proposal now hold the installed implementation.
if [ -z "$CAND" ] && grep -q '^# 4b\. Manifest completeness' "$REPO/validate_release.sh" 2>/dev/null; then
    CAND="$REPO/validate_release.sh"
fi
if [ -n "$CAND" ]; then
    printf '  %s\n' "--- closure against: $CAND"
    d=$(fixture 21); cat > "$d/gate-amendment/check_custody_disclosure.sh" <<'EOF'
#!/usr/bin/env bash
echo "ok   custody mode 'joint' declared, deviation disclosed in words"
echo "RESULT: PASS"
exit 0
EOF
    record T-21 "control: honest checker, exits 0 and prints PASS"    "$(step_state "$d" 7)" pass pass
    d=$(fixture 22); cat > "$d/gate-amendment/check_custody_disclosure.sh" <<'EOF'
#!/usr/bin/env bash
echo "ok   custody mode 'joint' declared, deviation disclosed in words"
echo "RESULT: PASS"
exit 1
EOF
    record T-22 "D1 closed: prints PASS, exits 1"                     "$(step_state "$d" 7)" fail fail
    d=$(fixture 23); cat > "$d/gate-amendment/check_custody_disclosure.sh" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
    record T-23 "D1 closed: exits 0, prints nothing"                  "$(step_state "$d" 7)" fail fail
    d=$(fixture 24); cat > "$d/gate-amendment/check_custody_disclosure.sh" <<'EOF'
#!/usr/bin/env bash
echo "FAIL custody mode declared without disclosure"
echo "RESULT: REFUSED"
exit 0
EOF
    record T-24 "D1 closed: exits 0, prints REFUSED"                  "$(step_state "$d" 7)" fail fail
    d=$(fixture 25)
    record T-25 "control: clean tree, nothing added"                  "$(step_state "$d" 4b)" pass pass
    d=$(fixture 26); printf 'listed addition\n' > "$d/docs/ADDED-AND-LISTED.md"; h=$(sha256sum "$d/docs/ADDED-AND-LISTED.md" | cut -d' ' -f1); printf '%s  docs/ADDED-AND-LISTED.md\n' "$h" >> "$d/SHA256SUMS"
    record T-26 "control: file added and correctly listed"            "$(step_state "$d" 4b)" pass pass
    d=$(fixture 27); printf 'stray\n' > "$d/UNLISTED-AT-ROOT.txt"
    record T-27 "D2 closed: unlisted file at root"                    "$(step_state "$d" 4b)" fail fail
    d=$(fixture 28); mkdir -p "$d/docs/nested"; printf 'stray\n' > "$d/docs/nested/UNLISTED-NESTED.txt"
    record T-28 "D2 closed: unlisted file in a subdirectory"          "$(step_state "$d" 4b)" fail fail
    d=$(fixture 29); rm -f "$d/LICENSE"
    record T-29 "no regression: listed file deleted still refused"    "$(step_state "$d" 4)" fail fail
else
    for id in 21 22 23 24 25 26 27 28 29; do
        printf '  %-5s %-46s %s\n' "T-$id" "closure case" "NOT RUN: no --candidate supplied"
        NOTRUN=$((NOTRUN+1))
    done
fi

echo
echo "=== Summary ==="
echo "  PASS          $PASS"
echo "  KNOWN-DEFECT  $KNOWN"
echo "  FIXED         $FIXEDC"
echo "  FAIL          $FAILD"
echo "  NOT RUN       $NOTRUN"
[ -n "$FAILED_IDS" ] && echo "  failed cases:$FAILED_IDS"
if [ "$FAILD" -gt 0 ]; then echo "❌ BASELINE UNRELIABLE"; exit 1; fi
if [ "$STRICT" -eq 1 ] && [ "$KNOWN" -gt 0 ]; then
    echo "❌ STRICT: $KNOWN known defect(s) still reproduce; a correction has not landed"
    exit 1
fi
if [ "$KNOWN" -eq 0 ] && [ "$FIXEDC" -gt 0 ]; then
    echo "✅ CORRECTION DEMONSTRATED: $FIXEDC documented defect(s) now refused, controls still pass"
elif [ "$KNOWN" -eq 0 ]; then
    echo "✅ ALL CASES MATCH the required behaviour; nothing is recorded as a known defect"
else
    echo "✅ BASELINE RECORDED: observed behaviour matches the documented state of $TAG"
fi
exit 0
