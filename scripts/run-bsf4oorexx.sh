#!/usr/bin/env bash
# Runs BSF4ooRexx's test suite, every .testGroup under bsf4oorexx.dev/testUnits, on a release build
# of interrexx, and fails on any group outside the expected-failure list that does not pass, and on
# any listed group that does.
#
#   run-bsf4oorexx.sh            fetch, build and run the suite
#   run-bsf4oorexx.sh --revision print the pinned BSF4ooRexx revision and stop
#   run-bsf4oorexx.sh --fetch    export the sources to BSF_SOURCES afresh, replacing any tree there,
#                                and stop
#
# BSF4ooRexx comes from its SourceForge svn repository at the pinned revision below; nothing of it is
# committed here. Its Java sources are compiled with the JDK of JAVA_HOME, and its JNI library
# libBSF4ooRexx850.so is compiled against this repository's include/ and linked against interrexx's
# librexx.so.4 and librexxapi.so.4. The library is also reachable as libBSF4ooRexx.so, the name a
# BSF4ooRexx install gives it too, which the groups that call rxFuncAdd with library "BSF4ooRexx" load.
#
# Environment (all optional):
#   REXX_RELEASE   directory holding the release rexx, librexx.so and librexxapi.so.
#                  Default: ${CARGO_TARGET_DIR:-rust/target}/release.
#   BSF_SOURCES    the exported BSF4ooRexx tree; exported there when absent.
#                  Default: rust/target/bsf4oorexx/r<revision>.
#   BSF_WORK       compiled classes, the library directory, logs and results.tsv.
#                  Default: rust/target/bsf4oorexx/work.
#   BSF_EXPECTED   the expected-failure list. Default: ci/bsf-expected-failures.txt.
#   BSF_GROUPS     the list of every group the sources hold. Default: ci/bsf-groups.txt.
#   BSF_TIMEOUT    seconds one group may run. Default: 180.
#   OOTEST         the ooTest checkout whose framework runs the groups. Default: ootest.
#   JAVA_HOME      the JDK. Default: the one `javac` on PATH belongs to.
#
# Each group runs from its own directory under `memcap 8G`. A group passes when it exits 0 and its
# summary reports at least one test ran, no failures and no errors. A listed group is an expected
# failure only when it ran normally, exited 0, and the tests it failed are exactly the ones the list
# names for it: a crash, a kill, a timeout or a run without tests fails the job whether the group is
# listed or not. The verdict table goes to stdout and, when set, to $GITHUB_STEP_SUMMARY.
set -euo pipefail

# The pinned BSF4ooRexx revision, the one place it is written.
BSF_REVISION=1131
BSF_URL=https://svn.code.sf.net/p/bsf4oorexx/code/branches/850

mode=run
case ${1:-} in
    --revision)
        echo "$BSF_REVISION"
        exit 0
        ;;
    --fetch) mode=fetch ;;
    '') ;;
    *)
        echo "run-bsf4oorexx.sh: unknown argument $1" >&2
        exit 2
        ;;
esac
if [ $# -gt 1 ]; then
    echo "run-bsf4oorexx.sh: unknown argument $2" >&2
    exit 2
fi

die() {
    echo "run-bsf4oorexx.sh: $*" >&2
    exit 1
}

root=$(cd "$(dirname "$0")/.." && pwd)
sources=${BSF_SOURCES:-$root/rust/target/bsf4oorexx/r$BSF_REVISION}
# Exports into a temporary directory, retrying a failed export, and renames the result into place,
# replacing any tree there. --fetch always does, so a partly restored cache is never kept. A run
# reuses a tree that is there, and the group list below stops it if that tree lost groups.
fetch() {
    local attempt
    mkdir -p "$(dirname "$sources")"
    for attempt in 1 2 3; do
        rm -rf "${sources:?}.tmp"
        if svn export -q -r "$BSF_REVISION" "$BSF_URL@$BSF_REVISION" "$sources.tmp"; then
            rm -rf "${sources:?}"
            mv "$sources.tmp" "$sources"
            return
        fi
        echo "run-bsf4oorexx.sh: svn export failed (attempt $attempt)" >&2
        [ "$attempt" = 3 ] || sleep 10
    done
    rm -rf "${sources:?}.tmp"
    die "svn export of $BSF_URL@$BSF_REVISION failed on every attempt"
}
if [ "$mode" = fetch ]; then
    fetch
    exit 0
fi
[ -d "$sources" ] || fetch

release=${REXX_RELEASE:-${CARGO_TARGET_DIR:-$root/rust/target}/release}
work=${BSF_WORK:-$root/rust/target/bsf4oorexx/work}
expected=${BSF_EXPECTED:-$root/ci/bsf-expected-failures.txt}
group_list=${BSF_GROUPS:-$root/ci/bsf-groups.txt}
group_timeout=${BSF_TIMEOUT:-180}
ootest=${OOTEST:-$root/ootest}
if [ -z "${JAVA_HOME:-}" ]; then
    JAVA_HOME=$(dirname "$(dirname "$(readlink -f "$(command -v javac)")")")
fi
export JAVA_HOME

for f in "$release/rexx" "$release/librexx.so" "$release/librexxapi.so"; do
    [ -e "$f" ] || die "$f is missing; build with cargo build --release"
done
[ -f "$ootest/framework/OOREXXUNIT.CLS" ] || die "no ooTest framework under $ootest"
[ -f "$JAVA_HOME/include/jni.h" ] || die "no JDK at JAVA_HOME=$JAVA_HOME"
[ -f "$expected" ] || die "no expected-failure list at $expected"
[ -f "$group_list" ] || die "no group list at $group_list"
command -v memcap >/dev/null || die "memcap is not on PATH"

dev=$sources/bsf4oorexx.dev
units=$dev/testUnits

# Every group the sources hold must be one the list names, and the other way round: a tree that lost
# or gained groups stops the run before anything is built.
[ -d "$units" ] || die "no $units"
mapfile -t groups < <(cd "$units" && find . -name '*.testGroup' | sed 's|^\./||' | LC_ALL=C sort)
listed_groups=$(grep -a -v -E '^(#|$)' "$group_list" | LC_ALL=C sort) || die "$group_list names no group"
missing=$(LC_ALL=C comm -23 <(printf '%s\n' "$listed_groups") <(printf '%s\n' "${groups[@]}"))
added=$(LC_ALL=C comm -13 <(printf '%s\n' "$listed_groups") <(printf '%s\n' "${groups[@]}"))
if [ -n "$missing$added" ]; then
    while read -r group; do
        [ -z "$group" ] || echo "missing from $units: $group" >&2
    done <<< "$missing"
    while read -r group; do
        [ -z "$group" ] || echo "not in $group_list: $group" >&2
    done <<< "$added"
    die "the groups under $units differ from $group_list"
fi

# The expected-failure list: a group path relative to testUnits, then its reason, and under it an
# indented line for each test the group fails, `failure` or `error` and then the test's name.
declare -A listed failing
group=
while IFS= read -r line; do
    case ${line#"${line%%[![:space:]]*}"} in '' | '#'*) continue ;; esac
    case $line in
        [[:space:]]*)
            [ -n "$group" ] || die "$expected: a test before any group: $line"
            read -r kind name rest <<< "$line"
            case $kind in failure | error) ;; *) die "$expected: not failure or error: $line" ;; esac
            [ -n "$name" ] && [ -z "$rest" ] || die "$expected: not a kind and one name: $line"
            failing[$group]+="$kind $name"$'\n'
            ;;
        *)
            read -r group reason <<< "$line"
            [ -n "$reason" ] || die "$expected: $group has no reason"
            listed[$group]=$reason
            failing[$group]=
            ;;
    esac
done < "$expected"
for group in "${!listed[@]}"; do
    [ -n "${failing[$group]}" ] || die "$expected: $group names no failing test"
done

mkdir -p "$work"
rm -rf "${work:?}/classes" "${work:?}/lib" "${work:?}/logs" "${work:?}/api" "${work:?}/source_cc"
mkdir -p "$work/classes" "$work/lib" "$work/logs" "$work/api"
work=$(cd "$work" && pwd)
lib=$work/lib

# The engine classes, compiled fresh so they match the native library built below; the rest of
# BSF4ooRexx's classes come precompiled with the sources.
"$JAVA_HOME/bin/javac" -nowarn -Xlint:none --release 8 -encoding ISO-8859-1 \
    -cp "$sources:$sources/jars/Xalan/xalan.jar" -implicit:none -d "$work/classes" \
    "$dev"/source_java/*.java

for library in librexx librexxapi; do
    cp "$release/$library.so" "$lib/$library.so.4"
    ln -sfn "$library.so.4" "$lib/$library.so"
done
# BsfLoadJava sizes its JavaVMOption array for one entry beyond its arguments but, on Unix, adds two
# when the arguments set neither java.class.path nor java.library.path: a call without arguments, as
# lex/procedural/Bean and 201811_strictArg make, writes past the heap block. The copy built here
# sizes the array for both. The edit must apply exactly once, so a revision that changes the line
# stops the run until the edit is reviewed. The edit is temporary: drop it once BSF4ooRexx sizes
# the array itself.
cp -R "$dev/source_cc" "$work/source_cc"
undersized='malloc( sizeof( JavaVMOption ) * (numArgs+1) )'
[ "$(grep -a -o -F "$undersized" "$work/source_cc/BSF4ooRexx.cc" | wc -l)" = 1 ] ||
    die "BSF4ooRexx.cc no longer has one '$undersized'; review the BsfLoadJava edit"
sed -i 's/malloc( sizeof( JavaVMOption ) \* (numArgs+1) )/malloc( sizeof( JavaVMOption ) * (numArgs+2) )/' \
    "$work/source_cc/BSF4ooRexx.cc"
grep -a -q -F 'malloc( sizeof( JavaVMOption ) * (numArgs+2) )' "$work/source_cc/BSF4ooRexx.cc" ||
    die "the BsfLoadJava edit did not apply"
api=$root/include
. "$root/scripts/api-flags.sh"
(cd "$work/source_cc" && "$api_cxx" -w -shared -O2 -DUNIX -DUSE_OREXX -DBSF4REXX_64_BIT \
    "${api_flags[@]}" -I. -I"$JAVA_HOME/include" -I"$JAVA_HOME/include/linux" \
    -o "$lib/libBSF4ooRexx850.so" BSF4ooRexx.cc \
    -L"$lib" -Wl,--disable-new-dtags,-rpath,"$lib" -lrexx -lrexxapi -lpthread)
ln -sfn libBSF4ooRexx850.so "$lib/libBSF4ooRexx.so"
# The library needs librexx.so.4 (librexxapi.so.4 only if the linker kept it), each from $lib.
needed=$(ldd "$lib/libBSF4ooRexx850.so" | awk '$1 ~ /^librexx(api)?\.so/ { print $1, $3 }')
case $'\n'$needed$'\n' in
    *$'\n'"librexx.so.4 $lib/librexx.so.4"$'\n'*) ;;
    *) die "libBSF4ooRexx850.so does not resolve librexx.so.4 to $lib" ;;
esac
while read -r name path; do
    [ "$path" = "$lib/$name" ] || die "libBSF4ooRexx850.so resolves $name to '$path', not $lib"
done <<< "$needed"

export CLASSPATH=.:$work/classes:$sources:$sources/jars/Xalan/xalan.jar:$units/bsf4rexx
export REXX_PATH=$dev/bin:$ootest:$ootest/framework
export PATH=$dev/bin:$PATH
export LD_LIBRARY_PATH=$lib
export REXX_API_DIR=$work/api

results=$work/results.tsv
printf 'group\trc\tran\tassertions\tfailures\terrors\tsecs\tverdict\n' > "$results"
table="| group | rc | ran | assertions | failures | errors | secs | verdict |"$'\n'
table+="|---|---|---|---|---|---|---|---|"$'\n'
passed=0 total=0 bad=0
field() { grep -a -m1 "^$1:" "$log" | awk '{ print $NF }' || true; }
# The tests the log reports as failed, one `failure NAME` or `error NAME` per line, sorted. ooTest
# prints each after the summary: a `[failure]` or `[error]` line, then the test's `Test:` line.
failed_tests() {
    awk '/^Tests ran:/ { summary = 1; next }
        summary && /^\[(failure|error)\] / { kind = substr($1, 2, length($1) - 2); next }
        kind != "" { if ($1 == "Test:") print kind, $2; kind = "" }' "$log" | LC_ALL=C sort
}
declare -A seen
[ "${#groups[@]}" -gt 0 ] || die "no .testGroup under $units"
for group in "${groups[@]}"; do
    seen[$group]=1
    total=$((total + 1))
    log=$work/logs/$(echo "$group" | tr / _).log
    start=$(date +%s%N)
    rc=0
    (cd "$units/$(dirname "$group")" &&
        timeout -k 5 "$group_timeout" memcap 8G "$release/rexx" "$(basename "$group")" \
            </dev/null >"$log" 2>&1) || rc=$?
    elapsed=$(( ($(date +%s%N) - start) / 100000000 ))
    secs="$((elapsed / 10)).$((elapsed % 10))"
    ran=$(field 'Tests ran') assertions=$(field Assertions)
    failures=$(field Failures) errors=$(field Errors)
    # How the run ended when it did not end normally. timeout exits 124 when the group stopped on
    # its TERM, and 137 when the group outlived it and was killed; memcap reports its own kill in
    # the log. A program ended by a Rexx error exits with 256 minus the error number and reports
    # that error in the log; any other status above 128 is a signal.
    abnormal=
    if [ "$rc" = 124 ]; then
        abnormal="timeout after ${group_timeout}s"
    elif [ "$rc" = 137 ] && grep -a -q '^memcap: OOM-killed' "$log"; then
        abnormal="killed at the memory cap"
    elif [ "$rc" = 137 ] && [ "$elapsed" -ge $((group_timeout * 10)) ]; then
        abnormal="timeout after ${group_timeout}s, killed"
    elif [ "$rc" -gt 128 ] && grep -a -q -E "^Error $((256 - rc))[.:]" "$log"; then
        abnormal="ended by Rexx error $((256 - rc))"
    elif [ "$rc" -gt 128 ]; then
        abnormal="ended by signal $((rc - 128))"
    elif [ -z "$ran" ] || [ "$ran" = 0 ]; then
        abnormal="no tests ran"
    fi
    clean=
    if [ "$rc" = 0 ] && [ -z "$abnormal" ] && [ "$failures" = 0 ] && [ "$errors" = 0 ]; then
        clean=1
    fi
    if [ -n "${listed[$group]+x}" ]; then
        if [ -n "$abnormal" ]; then
            verdict="FAIL: $abnormal (listed for ${listed[$group]}, which covers failed tests only)"
            bad=$((bad + 1))
        elif [ -n "$clean" ]; then
            verdict="UNEXPECTED PASS: remove it from the expected failures"
            bad=$((bad + 1))
        elif [ "$rc" = 0 ] && { [ "${failures:-0}" != 0 ] || [ "${errors:-0}" != 0 ]; }; then
            # The Errors count includes exceptions, which ooTest reports without a test name, so
            # every failure and error must have its name in the log before the names are compared.
            found=$(failed_tests)
            named=$(grep -c . <<< "$found" || true)
            pinned=$(printf '%s' "${failing[$group]}" | LC_ALL=C sort)
            if [ "$named" != $((${failures:-0} + ${errors:-0})) ]; then
                verdict="FAIL: the log names $named of its $((${failures:-0} + ${errors:-0})) failed tests (listed for ${listed[$group]})"
                bad=$((bad + 1))
            elif [ "$found" = "$pinned" ]; then
                verdict="expected failure (${listed[$group]})"
            else
                new=$(LC_ALL=C comm -13 <(echo "$pinned") <(echo "$found") | paste -s -d, -)
                gone=$(LC_ALL=C comm -23 <(echo "$pinned") <(echo "$found") | paste -s -d, -)
                verdict="FAIL: failed tests differ from the list (listed for ${listed[$group]})."
                [ -z "$new" ] || verdict+=" Failed but not listed: ${new//,/, }."
                [ -z "$gone" ] || verdict+=" Listed but did not fail: ${gone//,/, }."
                bad=$((bad + 1))
            fi
        else
            verdict="FAIL: exit status $rc without a failed test (listed for ${listed[$group]})"
            bad=$((bad + 1))
        fi
    elif [ -n "$clean" ]; then
        verdict=pass
        passed=$((passed + 1))
    elif [ -n "$abnormal" ]; then
        verdict="FAIL: $abnormal"
        bad=$((bad + 1))
    else
        verdict="FAIL: exit status $rc"
        bad=$((bad + 1))
    fi
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$group" "$rc" "$ran" "$assertions" "$failures" \
        "$errors" "$secs" "$verdict" >> "$results"
    table+="| $group | $rc | $ran | $assertions | $failures | $errors | $secs | $verdict |"$'\n'
done
for group in "${!listed[@]}"; do
    if [ -z "${seen[$group]+x}" ]; then
        table+="| $group | | | | | | | FAIL: listed as an expected failure but no such group |"$'\n'
        bad=$((bad + 1))
    fi
done

summary="BSF4ooRexx r$BSF_REVISION: $passed of $total groups pass, ${#listed[@]} listed as expected failures, $bad wrong"
printf '%s\n%s\n' "$table" "$summary"
if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
    printf '## BSF4ooRexx test suite\n\n%s\n%s\n' "$table" "$summary" >> "$GITHUB_STEP_SUMMARY"
fi
if [ "$bad" -gt 0 ]; then
    awk -F'\t' 'NR > 1 && $8 ~ /^(FAIL|UNEXPECTED)/ { print $1 }' "$results" | while read -r group; do
        echo "::group::log of $group"
        tail -n 40 "$work/logs/$(echo "$group" | tr / _).log"
        echo "::endgroup::"
    done
    exit 1
fi
