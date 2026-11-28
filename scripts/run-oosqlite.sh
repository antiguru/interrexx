#!/usr/bin/env bash
# Runs ooSQLite's example programs on interrexx and on the oracle and fails on any program whose
# exit status, standard output or error codes differ.
#
#   run-oosqlite.sh             fetch (when absent), build and run the examples
#   run-oosqlite.sh --source-id print the pinned ooSQLite revision as one word, and stop
#   run-oosqlite.sh --fetch     export the sources to OOSQLITE_SOURCES afresh, replacing any tree
#                               there, and stop
#
# ooSQLite and the APICommon helper it compiles with come from ooRexx's SourceForge svn repository
# at the pinned revision below. Nothing of either is committed here: both are under ooRexx's own
# licence, which no file of this repository carries (the SQLite engine they bundle is public
# domain). liboosqlite.so and the example extension libraries are compiled,
# unmodified, against this repository's include/ only. They link nothing from an interpreter, as
# every extension reaches it through the tables handed in at call time, so one build serves both
# interpreters.
#
# Each example runs twice, on the release rexx and on the oracle, each time from a fresh copy of
# the examples tree (they write to its database), with standard input empty, under an 8 GiB
# memory cap and a time bound. The cap is a systemd scope (MemoryMax=8G, no swap) where the user
# manager can make one, as docs/developing.md asks on Linux, and scripts/rss-cap.sh's watchdog
# otherwise, as on macOS or a CI runner with no user session; the script says which. A run that
# the time bound or the cap ended (status 124 or 137) fails the example whatever the other
# interpreter did, so two runs that both hang do not agree. Both interpreters keep their API
# service, interrexx's rxapi beside the release rexx, in one directory of the run's own, whose
# daemons are stopped when the script ends. Standard output and the exit status must match byte
# for byte. Of standard
# error only the code of each `Error` line is compared: the specification leaves message texts
# unspecified, and no tool here compares texts with ooRexx's. The programs found must be the ones
# ci/oosqlite-examples.txt lists, so an example added or lost upstream fails the run rather than
# narrowing it. An example ci/oosqlite-expected-differences.txt lists, with its reason, must match
# up to the order of the words on each line of its output instead, and must not match exactly.
#
# Environment (all optional):
#   REXX_RELEASE      directory holding the release rexx. Default: ${CARGO_TARGET_DIR:-rust/target}/release.
#   ORACLE            the oracle's build directory, holding bin/rexx and lib/.
#                     Default: third_party/oracle.
#   OOSQLITE_SOURCES  the exported ooSQLite and APICommon trees, exported there when absent.
#                     Default: rust/target/oosqlite/r<revision>.
#   OOSQLITE_WORK     the built libraries, the per-run copies and the outputs.
#                     Default: rust/target/oosqlite/work.
#   OOSQLITE_TIMEOUT  seconds one program may run. Default: 120.
set -euo pipefail

# The pinned revision of ooRexx's svn repository, the one place it is written. It holds ooSQLite
# 1.0.2 and the APICommon it builds with.
OOSQLITE_REVISION=13646
SVN=https://svn.code.sf.net/p/oorexx/code-0

mode=run
case ${1:-} in
    --source-id)
        echo "r$OOSQLITE_REVISION"
        exit 0
        ;;
    --fetch) mode=fetch ;;
    '') ;;
    *)
        echo "run-oosqlite.sh: unknown argument $1" >&2
        exit 2
        ;;
esac
if [ $# -gt 1 ]; then
    echo "run-oosqlite.sh: unknown argument $2" >&2
    exit 2
fi

die() {
    echo "run-oosqlite.sh: $*" >&2
    exit 1
}

root=$(cd "$(dirname "$0")/.." && pwd)
sources=${OOSQLITE_SOURCES:-$root/rust/target/oosqlite/r$OOSQLITE_REVISION}
work=${OOSQLITE_WORK:-$root/rust/target/oosqlite/work}
release=${REXX_RELEASE:-${CARGO_TARGET_DIR:-$root/rust/target}/release}
oracle=${ORACLE:-$root/third_party/oracle}
timeout_s=${OOSQLITE_TIMEOUT:-120}
list=$root/ci/oosqlite-examples.txt
expected=$root/ci/oosqlite-expected-differences.txt
api=$root/include

fetch() {
    command -v svn >/dev/null || die "svn is not on PATH"
    rm -rf "$sources"
    mkdir -p "$sources"
    svn export -q -r "$OOSQLITE_REVISION" "$SVN/incubator/ooSQLite" "$sources/ooSQLite"
    svn export -q -r "$OOSQLITE_REVISION" "$SVN/incubator/APICommon" "$sources/APICommon"
}

if [ "$mode" = fetch ]; then
    fetch
    echo "run-oosqlite.sh: exported r$OOSQLITE_REVISION to $sources"
    exit 0
fi
[ -d "$sources/ooSQLite" ] && [ -d "$sources/APICommon" ] || fetch

oo=$sources/ooSQLite
examples=$oo/examples
[ -x "$release/rexx" ] || die "no release rexx in $release: cargo build --release -p interrexx -p interrexx-capi"
[ -x "$release/rxapi" ] ||
    die "no rxapi beside the release rexx in $release: cargo build --release -p interrexx-rxapi"
[ -x "$oracle/bin/rexx" ] || die "no oracle in $oracle: scripts/setup-third-party.sh"

# The examples the tree holds must be the listed ones.
found=$(cd "$examples" && find . -name '*.rex' | sed 's#^\./##' | LC_ALL=C sort)
listed=$(grep -v '^#' "$list" | sed '/^$/d' | LC_ALL=C sort)
if [ "$found" != "$listed" ]; then
    diff <(echo "$listed") <(echo "$found") >&2 || true
    die "the examples in $examples differ from $list (< listed, > found)"
fi

while IFS=$'\t' read -r example _; do
    echo "$listed" | grep -qxF "$example" || die "$expected names $example, which is not an example"
done < <(grep -v '^#' "$expected" | sed '/^$/d')

# Build. The flags are those of ooSQLite's Makefile.lin; the extensions zipfile and compress, which
# need zlib, are not in the exported tree and are left out, as are the readline shell and the
# Windows-only parts.
lib=$work/lib
rm -rf "$work"
mkdir -p "$lib" "$work/obj"
version=$(sed -n 's/^OOSQLITE_VER_STR=//p' "$oo/src/ooSQLite.ver" | tr -d '"')
flags=(-O2 -fPIC -w -I"$api" -I"$api/platform/unix" -I"$oo/src" -I"$oo/src/sqlite"
    -I"$oo/src/platform/unix" -I"$sources/APICommon/src"
    -DSQLITE_ENABLE_COLUMN_METADATA -DSQLITE_ENABLE_MEMORY_MANAGEMENT -DSQLITE_ENABLE_FTS3
    -DSQLITE_ENABLE_FTS3_PARENTHESIS -DSQLITE_ENABLE_FTS4 -DSQLITE_ENABLE_FTS5
    -DSQLITE_ENABLE_GEOPOLY -DSQLITE_ENABLE_MATH_FUNCTIONS -DSQLITE_ENABLE_RTREE
    -DSQLITE_DEFAULT_FOREIGN_KEYS=1 -DSQLITE_CORE -DHAVE_USLEEP=1 -DHAVE_LOCALTIME_R=1
    -DHAVE_GMTIME_R=1 -DHAVE_FDATASYNC=1 -D_REENTRANT=1
    -DOOSQLITE_VER_MAJOR="${version%%.*}" -DOOSQLITE_VER_MINOR="$(echo "$version" | cut -d. -f2)"
    -DOOSQLITE_VER_LEVEL="$(echo "$version" | cut -d. -f3)"
    -DOOSQLITE_VER_BUILD="$(echo "$version" | cut -d. -f4)"
    -DOOSQLITE_VER_STRING="\"$version\"" -DOOSQLITE_COPYRIGHT_YEAR="\"2012-2026\"")
objects=()
compile() { # language source object [extra flags]
    local language=$1 source=$2 object=$work/obj/$3
    shift 3
    "$language" -c "${flags[@]}" "$@" -o "$object" "$source"
    objects+=("$object")
}
# sqlite3.c, by far the largest, compiles while the rest do; its own status is waited for below,
# so a failure there stops the build here rather than at the link.
compile cc "$oo/src/sqlite/sqlite3.c" sqlite3.o -DOOSQLITE_BUILD &
sqlite_pid=$!
compile c++ "$oo/src/ooSQLite.cpp" ooSQLite.o -DOOSQLITE_BUILD
compile c++ "$oo/src/platform/unix/ooSqlSysLibrary.cpp" ooSqlSysLibrary.o \
    -DOOSQLITE_SHARED_LIBRARY_EXT='".so"'
compile c++ "$sources/APICommon/src/APICommon.cpp" APICommon.o
for extension in "$oo"/src/sqlite/extensions/*.c; do
    compile cc "$extension" "$(basename "$extension" .c).o"
done
wait "$sqlite_pid" || die "sqlite3.c did not compile"
objects+=("$work/obj/sqlite3.o")
c++ -shared -o "$lib/liboosqlite.so" "${objects[@]}" -ldl -lpthread -lm
cp "$oo/src/rexx/ooSQLite.cls" "$lib/"
# The example extension libraries, built where the examples look for them. They reach SQLite
# through the routine table liboosqlite.so hands them, so they are compiled without SQLITE_CORE,
# which would bind them to SQLite's symbols instead.
ext_flags=(-O2 -fPIC -w -I"$api" -I"$api/platform/unix" -I"$oo/src" -I"$oo/src/sqlite")
built=$work/user.extensions
mkdir -p "$built/autoPackages"
for source in "$examples"/user.extensions/*.c "$examples"/user.extensions/*.cpp; do
    name=$(basename "${source%.*}" | tr '[:upper:]' '[:lower:]')
    c++ -shared "${ext_flags[@]}" -x "$([ "${source##*.}" = c ] && echo c || echo c++)" "$source" \
        -o "$built/lib$name.so"
done
for source in "$examples"/user.extensions/autoPackages/*.cpp; do
    name=$(basename "${source%.*}" | tr '[:upper:]' '[:lower:]')
    c++ -shared "${ext_flags[@]}" "$source" -o "$built/autoPackages/lib$name.so"
done
echo "run-oosqlite.sh: built ooSQLite $version (r$OOSQLITE_REVISION) into $lib"

# One API service directory for every run of both interpreters, outside the per-example copies
# (which are removed while a daemon could still be using them), and the daemons serving it
# stopped when the script ends, however it ends.
service=$work/service
mkdir -m 700 "$service"
stop_services() {
    REXX_API_DIR="$service" "$release/rxapi" --stop >/dev/null 2>&1 || true
    local pid
    for pid in $(pgrep -x rxapi); do
        if tr '\0' '\n' <"/proc/$pid/environ" 2>/dev/null |
            grep -qx -e "XDG_RUNTIME_DIR=$service" -e "REXX_API_DIR=$service"; then
            kill "$pid" 2>/dev/null || true
        fi
    done
}
trap stop_services EXIT

# The memory cap every run is under: a systemd scope where the user manager can make one, the
# RSS watchdog otherwise.
if systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 true >/dev/null 2>&1; then
    capped=(systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0)
    echo "run-oosqlite.sh: runs are capped by a systemd scope (MemoryMax=8G)"
else
    capped=("$root/scripts/rss-cap.sh" 8388608)
    echo "run-oosqlite.sh: no systemd user manager; runs are capped by scripts/rss-cap.sh (8 GiB RSS)"
fi

# Runs example $2 on interpreter $1 (ix or oo) from a fresh copy numbered $3, writing
# $work/out/$3.$1.{out,err,rc}. Both interpreters run from the same path, which some examples
# print. The copies are numbered so that the rxapi socket paths under them stay short.
run() {
    local who=$1 example=$2 n=$3 copy=$work/run/$n status=0
    rm -rf "$copy"
    mkdir -p "$copy"
    cp -r "$examples/." "$copy/"
    cp "$lib/ooSQLite.cls" "$copy/"
    cp "$lib/ooSQLite.cls" "$(dirname "$copy/$example")/"
    cp "$built"/*.so "$copy/user.extensions/"
    cp "$built"/autoPackages/*.so "$copy/user.extensions/autoPackages/"
    (
        cd "$(dirname "$copy/$example")"
        # The interpreter's variables are set inside the cap, through env: systemd-run --user
        # finds the user manager by the caller's XDG_RUNTIME_DIR, which the oracle's must not
        # replace.
        if [ "$who" = oo ]; then
            exec timeout -k 5 "$timeout_s" "${capped[@]}" env XDG_RUNTIME_DIR="$service" \
                LD_LIBRARY_PATH="$lib:$oracle/lib" PATH="$lib:$oracle/bin:$PATH" \
                "$oracle/bin/rexx" "$copy/$example"
        else
            exec timeout -k 5 "$timeout_s" "${capped[@]}" env REXX_API_DIR="$service" \
                LD_LIBRARY_PATH="$lib" PATH="$lib:$PATH" "$release/rexx" "$copy/$example"
        fi
    ) > "$work/out/$n.$who.out" 2> "$work/out/$n.$who.err" < /dev/null || status=$?
    echo "$status" > "$work/out/$n.$who.rc"
}

# The lines of $1 with the words of each sorted: what output whose columns are reordered keeps.
words() {
    python3 -c 'import sys
for line in open(sys.argv[1], encoding="latin-1"):
    print(" ".join(sorted(line.split())))' "$1"
}

# The codes of the `Error` lines of standard error, one per line.
codes() {
    sed -n 's/^Error \([0-9][0-9.]*\)[ :].*/\1/p' "$1"
}

mkdir -p "$work/out" "$work/run"
failed=0
n=0
printf '%-40s %8s %8s  %s\n' program oracle interrexx verdict
while read -r example; do
    n=$((n + 1))
    id=$(printf '%02d' "$n")
    run oo "$example" "$id"
    run ix "$example" "$id"
    o=$work/out/$id.oo
    i=$work/out/$id.ix
    reason=$(awk -F'\t' -v e="$example" '$1 == e { print $2 }' "$expected")
    verdict=same
    if [ "$(cat "$o.rc")" = 124 ] || [ "$(cat "$o.rc")" = 137 ]; then
        verdict="the oracle was ended by the time bound or the memory cap"
    elif [ "$(cat "$i.rc")" = 124 ] || [ "$(cat "$i.rc")" = 137 ]; then
        verdict="interrexx was ended by the time bound or the memory cap"
    elif [ "$(cat "$o.rc")" != "$(cat "$i.rc")" ]; then
        verdict="exit status differs"
    elif [ "$(codes "$o.err")" != "$(codes "$i.err")" ]; then
        verdict="error codes differ"
    elif [ -z "$reason" ] && ! cmp -s "$o.out" "$i.out"; then
        verdict="stdout differs"
    elif [ -n "$reason" ] && cmp -s "$o.out" "$i.out"; then
        verdict="listed as differing, but matches: remove it from $(basename "$expected")"
    elif [ -n "$reason" ] && [ "$(words "$o.out")" != "$(words "$i.out")" ]; then
        verdict="stdout differs beyond word order"
    elif [ -n "$reason" ]; then
        verdict="same but for word order (${reason%%:*})"
    fi
    line=$(printf '%-40s %8s %8s  %s' "$example" "$(cat "$o.rc")" "$(cat "$i.rc")" "$verdict")
    echo "$line"
    [ -z "${GITHUB_STEP_SUMMARY:-}" ] || echo "    $line" >> "$GITHUB_STEP_SUMMARY"
    if [ "$verdict" != same ] && [ "${verdict#same but}" = "$verdict" ]; then
        failed=$((failed + 1))
        diff "$o.out" "$i.out" | head -20 >&2 || true
    fi
done <<< "$listed"

if [ "$failed" -ne 0 ]; then
    die "$failed of $n examples differ; outputs are in $work/out"
fi
echo "run-oosqlite.sh: all $n examples agree"
