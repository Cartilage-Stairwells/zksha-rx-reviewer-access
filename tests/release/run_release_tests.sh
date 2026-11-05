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
TAG="${RELEASE_TAG:-review-v0.1.29}"
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
    b && (/^[0-9]+[a-z]?\. / || /^=== Summary/) { b=0 }
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
overall() {  # overall <dir> -> pass|fail, the validator's own verdict rather than a step's
    local e; e=$(validator_exit "$1"); [ "$e" -eq 0 ] && echo pass || echo fail
}
broken_comm_state() {  # broken_comm_state <dir> -> pass|fail with a comm that cannot work
    # A completeness check that reports success when the comparison itself failed is worse than no
    # check, so this proves the refusal rather than assuming it.
    local bin="$WORK/brokenbin" e
    mkdir -p "$bin"; printf '#!/bin/sh\nexit 1\n' > "$bin/comm"; chmod +x "$bin/comm"
    ( cd "$1" && PATH="$bin:$PATH" bash validate_release.sh >/dev/null 2>&1 ); e=$?
    [ "$e" -eq 0 ] && echo pass || echo fail
}

CONTROLC=0
record() {  # record <id> <mutation> <observed> <expected_now> <expected_fixed> <defect>
  local id=$1 mut=$2 obs=$3 now=$4 fix=$5 defect=${6:-no} verdict
  if   [ "$obs" = "$fix" ] && [ "$obs" != "$now" ]; then verdict=FIXED;        FIXEDC=$((FIXEDC+1))
  elif [ "$obs" = "$now" ] && [ "$defect" = control ]; then
       # A control passes because its behaviour did not change, which is not evidence that
       # anything was closed. Counting it as PASS would credit it toward closure.
       verdict=CONTROL; CONTROLC=$((CONTROLC+1))
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
# T-14 asserts the ordering claim itself: an EARLY failure must not be overridden by a LATER pass.
# The fixture therefore carries the README tamper ONLY. An added prohibited declaration made step 7
# fail too, so no step passed and the ordering was never exercised. Measured against two mutants, the
# README tamper is also caught by other checks, so this remains a control: it now EXERCISES the named
# ordering but cannot discriminate. T-34 is the discriminating aggregate case for a late refusal.
d=$(fixture 14); printf 'x' >> "$d/README.md"
s3=$(step_state "$d" 3); s7=$(step_state "$d" 7)
[ "$s3" != fail ] && printf '  T-14  MISMATCH: expected step 3 to fail, observed %s\n' "$s3"
[ "$s7" != pass ] && printf '  T-14  MISMATCH: expected step 7 to pass, observed %s\n' "$s7"
e=$(validator_exit "$d"); [ "$e" -ne 0 ] && e=pass || e=fail
                                                   record T-14 "early step fails, later step passes, still refused" "$e" pass pass control

# ---------- Group C: package integrity ----------
d=$(fixture 15); printf 'x' >> "$d/README.md"
                                                   record T-15 "one byte appended to a listed file"          "$(step_state "$d" 3)" fail fail
d=$(fixture 16); rm -f "$d/LICENSE"
                                                   record T-16 "one listed file deleted"                     "$(step_state "$d" 4)" fail fail
d=$(fixture 17 tag); printf 'stray\n' > "$d/UNLISTED-ADDITION.txt"
                                                   record T-17 "one unlisted file added"                     "$(overall "$d")" pass fail yes

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
    record T-22 "D1 closed: prints PASS, exits 1"                     "$(step_state "$d" 7)" pass fail
    d=$(fixture 23); cat > "$d/gate-amendment/check_custody_disclosure.sh" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
    record T-23 "D1 closed: exits 0, prints nothing"                  "$(step_state "$d" 7)" fail fail control
    d=$(fixture 24); cat > "$d/gate-amendment/check_custody_disclosure.sh" <<'EOF'
#!/usr/bin/env bash
echo "FAIL custody mode declared without disclosure"
echo "RESULT: REFUSED"
exit 0
EOF
    record T-24 "D1 closed: exits 0, prints REFUSED"                  "$(step_state "$d" 7)" fail fail control
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

# ---------- Group F: completeness fail-closed (D3) and non-regular entries (D4) ----------
# Both were found by a third-party run and confirmed by probe. They are the same failure class as
# D1 and D2: a check that reports success without having established anything.
d=$(fixture 30 tag); ln -s /etc/hostname "$d/UNLISTED-SYMLINK"
                                                   record T-30 "unlisted symlink (D4)"                         "$(overall "$d")" pass fail yes
d=$(fixture 31 tag); mkfifo "$d/UNLISTED-FIFO"
                                                   record T-31 "unlisted fifo (D4)"                            "$(overall "$d")" pass fail yes
d=$(fixture 32 tag)
                                                   record T-32 "failing comm fails closed (D3)"                "$(broken_comm_state "$d")" pass fail yes
d=$(fixture 33 tag); printf 'stray\n' > "$d/UNLISTED-PARSER.txt"
if step_block "$d" 4 | grep -q '4b\. Manifest completeness'; then leak=fail; else leak=pass; fi
                                                   record T-33 "parser keeps step 4b out of step 4"             "$leak" fail pass yes
else
    for id in 21 22 23 24 25 26 27 28 29; do
        printf '  %-5s %-46s %s\n' "T-$id" "closure case" "NOT RUN: no --candidate supplied"
        NOTRUN=$((NOTRUN+1))
    done
fi

# ---------- Group G: the aggregation lens ----------
# Every case above that touches the release record reads one step. Nothing asserted that a
# LATE-step refusal reaches the aggregate verdict while the earlier checks stay clean, so a
# validator that stopped propagating late failures would keep the suite green. This case holds
# that line: the declaration is the only mutation, the manifest is re-cut so step 3 stays clean,
# and the verdict is read through the aggregate lens rather than through step 7.
d=$(fixture 34 tag)
printf '\nCUSTODY-DISCLOSURE: independent\n' >> "$d/REVIEW_RELEASE.md"
python3 - "$d" <<'RECUT'
import hashlib, pathlib, sys
d = pathlib.Path(sys.argv[1]); t = d / "SHA256SUMS"; target = "REVIEW_RELEASE.md"
h = hashlib.sha256((d / target).read_bytes()).hexdigest()
out = []; hit = False
for line in t.read_text().splitlines():
    path = line.partition("  ")[2]
    if path == target:
        out.append(h + "  " + path); hit = True
    else:
        out.append(line)
assert hit, "REVIEW_RELEASE.md not in the fixture manifest"
t.write_text("\n".join(out) + "\n")
RECUT
s3=$(step_state "$d" 3); s7=$(step_state "$d" 7)
# The case is only meaningful in the state it names. If either guard is off, the row below
# would be measuring some other refusal, which is the defect this case exists to catch.
[ "$s3" = pass ] || printf '  T-34  MISMATCH: expected step 3 clean, observed %s\n' "$s3"
[ "$s7" = fail ] || printf '  T-34  MISMATCH: expected step 7 refused, observed %s\n' "$s7"
                                                   record T-34 "late-step refusal reaches the aggregate verdict" "$(overall "$d")" fail fail

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
[ "$CONTROLC" -gt 0 ] && printf '  controls (unchanged on both implementations, cannot discriminate): %s\n' "$CONTROLC"
if [ "$KNOWN" -eq 0 ] && [ "$FIXEDC" -gt 0 ]; then
    echo "✅ CORRECTION DEMONSTRATED: $FIXEDC documented defect(s) now refused, controls still pass"
elif [ "$KNOWN" -eq 0 ]; then
    echo "✅ ALL CASES MATCH the required behaviour; nothing is recorded as a known defect"
else
    echo "✅ BASELINE RECORDED: observed behaviour matches the documented state of $TAG"
fi
exit 0
