#!/usr/bin/env bash
# Compiles ooRexx's native-API test libraries and drivers, unmodified, against this workspace's
# librexx.so.4 and librexxapi.so.4, never the oracle's, and against this repository's include/.
#
#   build-testbinaries.sh [--libs DIR] [--out DIR] [--stage-only]
#
# --libs        a directory holding librexx.so and librexxapi.so as cargo built them. Default:
#               build both with `cargo build` and use rust/target/debug (or
#               $CARGO_TARGET_DIR/debug).
# --out         where the libraries and binaries go. Default: rust/target/testbinaries.
# --stage-only  put only librexx.so.4 and librexxapi.so.4 into --out and stop: no upstream
#               source is read, so the mode runs in a checkout without the submodule. Drivers
#               this repository owns link against that directory.
#
# Every product records only --out as its search path (DT_RPATH, which the loader consults before
# LD_LIBRARY_PATH, or LC_RPATH on macOS). The build fails if either library resolves anywhere
# else, with or without the oracle's lib directory on LD_LIBRARY_PATH (api_resolves in
# api-flags.sh).
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
libs=
out=$root/rust/target/testbinaries
stage_only=
while [ $# -gt 0 ]; do
    case $1 in
        --libs) libs=$2; shift 2 ;;
        --out) out=$2; shift 2 ;;
        --stage-only) stage_only=1; shift ;;
        *) echo "build-testbinaries.sh: unknown argument $1" >&2; exit 2 ;;
    esac
done

if [ -z "$libs" ]; then
    (cd "$root/rust" && cargo build --locked -p interrexx-capi -p interrexx-capi-rxapi)
    libs=${CARGO_TARGET_DIR:-$root/rust/target}/debug
fi

src=$root/third_party/ooRexx/testbinaries
api=$root/include
. "$root/scripts/api-flags.sh"
samples=$root/third_party/ooRexx/samples/api/classic/unix/callrexx

mkdir -p "$out"
out=$(cd "$out" && pwd)

# Callers that name --libs may run at once (cargo nextest runs every test as a process of its
# own): they take turns, and one whose inputs are those of the build already in --out has
# nothing left to do.
stamp=
if [ -n "$libs" ]; then
    exec 9>"$out.lock"
    api_lock 9
fi
stage() {
    for library in librexx librexxapi; do
        versioned=$(api_versioned "$library")
        cp -f "$libs/$library.$api_dylib" "$out/$versioned.tmp"
        mv -f "$out/$versioned.tmp" "$out/$versioned"
        ln -sfn "$versioned" "$out/$library.$api_dylib"
    done
}
if [ -n "$stage_only" ]; then
    stage
    echo "build-testbinaries.sh: staged the libraries into $out"
    exit 0
fi

if [ -n "$libs" ]; then
    # The stamp names the inputs of the build: the libraries, this script, the headers, and the
    # upstream sources it compiles (size and time, as a submodule update rewrites them).
    stamp=$(api_file_stamp "$libs/librexx.$api_dylib" "$libs/librexxapi.$api_dylib" "$0" "$root/scripts/api-flags.sh"
        api_stamp
        find "$src" "$samples" -type f -print0 | LC_ALL=C sort -z | xargs -0 "${api_stat[@]}"
        c++ --version | head -n 1)
    if [ -f "$out/.stamp" ] && [ "$(cat "$out/.stamp")" = "$stamp" ]; then
        echo "build-testbinaries.sh: $out is up to date"
        exit 0
    fi
    rm -f "$out/.stamp"
fi
stage

api_rpath "$out"
flags=("${api_flags[@]}" -L"$out" "${api_rpath_flags[@]}")

shared() {
    local compiler=$1 name=$2
    shift 2
    "$compiler" -shared -o "$out/lib$name.$api_dylib" "$@" "${flags[@]}"
}

shared c++ orxfunction "$src/orxfunction.cpp" -lrexx -lrexxapi
shared c++ orxmethod "$src/orxmethod.cpp" -lrexx -lrexxapi
shared c++ orxclassic "$src/orxclassic.cpp" -lrexx -lrexxapi
shared cc orxclassic1 "$src/orxclassic1.c" -funwind-tables -pthread -lrexx -lrexxapi
shared c++ orxexits "$src/orxclassicexits.cpp" "$src/orxinstance.cpp" -lrexx -lrexxapi
shared c++ orxinvocation "$src/orxinvocation.cpp" -lorxexits -lrexx -lrexxapi
c++ -o "$out/rexxinstance" "$src/rexxinstance.cpp" "${flags[@]}" -lorxexits -lrexx -lrexxapi
c++ -o "$out/provoke_locks" "$src/provoke_locks.cpp" "${flags[@]}" -pthread -lrexx -lrexxapi
cp -f "$src/provoke_locks.rex" "$out/provoke_locks.rex"
c++ -o "$out/detach_root_terminate" "$src/detach_root_terminate.cpp" "${flags[@]}" -pthread \
    -lrexx -lrexxapi
if [ -f "$samples/callrexx1.cpp" ]; then
    c++ -o "$out/callrexx1" "$samples/callrexx1.cpp" "${flags[@]}" -lrexx -lrexxapi
fi

products=("$out"/liborx*."$api_dylib" "$out/rexxinstance" "$out/provoke_locks"
    "$out/detach_root_terminate")
[ -f "$out/callrexx1" ] && products+=("$out/callrexx1")

# Each of the two libraries a product needs must resolve to --out, under the caller's environment
# and with the oracle's lib directory first on LD_LIBRARY_PATH (api_resolves says what the check
# reads on each host).
check() {
    local product=$1 library status
    shift
    for library in $(api_versioned librexx) $(api_versioned librexxapi); do
        status=0
        api_resolves "$product" "$library" "$out" "$@" || status=$?
        if [ "$status" -eq 1 ]; then
            echo "build-testbinaries.sh: $product resolves $library outside $out ($*)" >&2
            exit 1
        fi
    done
}
oracle_lib=$root/third_party/oracle/lib
for product in "${products[@]}"; do
    check "$product"
    check "$product" LD_LIBRARY_PATH="$oracle_lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
done
echo "build-testbinaries.sh: built into $out"
[ -z "$stamp" ] || printf '%s\n' "$stamp" > "$out/.stamp"
