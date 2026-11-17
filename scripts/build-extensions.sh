#!/usr/bin/env bash
# Compiles ooRexx's hostemu and orxncurses extensions, unmodified, against this workspace's
# librexx.so.4 and librexxapi.so.4, never the oracle's, and against this repository's include/.
#
#   build-extensions.sh [--libs DIR] [--out DIR] [--check-only]
#
# --libs        a directory holding librexx.so and librexxapi.so as cargo built them. Default:
#               build both with `cargo build` and use rust/target/debug (or
#               $CARGO_TARGET_DIR/debug).
# --out         where the libraries go. Default: rust/target/extensions.
# --check-only  build nothing: run the check below on the products already in --out.
#
# hostemu's grammar is the generated cmdparse.cpp beside cmdparse.ypp, as the oracle's own build
# uses it, so no bison is needed. A host without the ncurses headers skips orxncurses and says so.
#
# Every product records only --out as its search path (DT_RPATH, which the loader consults before
# LD_LIBRARY_PATH, or LC_RPATH on macOS). The build fails if either library resolves anywhere
# else, with or without the oracle's lib directory on LD_LIBRARY_PATH (api_resolves in
# api-flags.sh).
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
libs=
out=$root/rust/target/extensions
check_only=
while [ $# -gt 0 ]; do
    case $1 in
        --libs) libs=$2; shift 2 ;;
        --out) out=$2; shift 2 ;;
        --check-only) check_only=1; shift ;;
        *) echo "build-extensions.sh: unknown argument $1" >&2; exit 2 ;;
    esac
done

src=$root/third_party/ooRexx/extensions
api=$root/include
. "$root/scripts/api-flags.sh"

build() {
    if [ -z "$libs" ]; then
        (cd "$root/rust" && cargo build --locked -p interrexx-capi -p interrexx-capi-rxapi)
        libs=${CARGO_TARGET_DIR:-$root/rust/target}/debug
    fi
    for library in librexx librexxapi; do
        versioned=$(api_versioned "$library")
        cp -f "$libs/$library.$api_dylib" "$out/$versioned.tmp"
        mv -f "$out/$versioned.tmp" "$out/$versioned"
        ln -sfn "$versioned" "$out/$library.$api_dylib"
    done

    # api_keep_needed keeps both libraries recorded although a product may call into only one,
    # so the check below always has both to look at. It is empty on macOS, which the bash there
    # (3.2) takes for unset under `set -u` unless the expansion is guarded.
    api_rpath "$out"
    local flags=(-w "${api_flags[@]}" -shared -L"$out" "${api_rpath_flags[@]}"
        ${api_keep_needed[@]+"${api_keep_needed[@]}"})

    c++ -o "$out/libhostemu.$api_dylib" "$src/hostemu/platform/unix/hostemu.cpp" \
        "$src/hostemu/cmdparse.cpp" -I"$src/hostemu" "${flags[@]}" -pthread -lrexx -lrexxapi

    rm -f "$out/liborxncurses.$api_dylib"
    if echo '#include <ncurses.h>' | c++ -fsyntax-only -x c++ - 2>/dev/null; then
        c++ -o "$out/liborxncurses.$api_dylib" "$src/orxncurses/orxncurses.cpp" -DCURSES_HAVE_NCURSES_H \
            "${flags[@]}" -lrexx -lrexxapi -lncurses
    else
        echo "build-extensions.sh: no ncurses headers (<ncurses.h>); orxncurses is not built" >&2
    fi
}

mkdir -p "$out"
out=$(cd "$out" && pwd)

# Callers that name --libs may run at once (cargo nextest runs every test as a process of its
# own): they take turns, and one whose inputs are those of the build already in --out has
# nothing left to do.
stamp=
if [ -n "$libs" ] && [ -z "$check_only" ]; then
    exec 9>"$out.lock"
    api_lock 9
    stamp=$(api_file_stamp "$libs/librexx.$api_dylib" "$libs/librexxapi.$api_dylib" "$0" "$root/scripts/api-flags.sh"
        api_stamp
        find "$src" -type f -print0 | LC_ALL=C sort -z | xargs -0 "${api_stat[@]}"
        c++ --version | head -n 1)
    if [ -f "$out/.stamp" ] && [ "$(cat "$out/.stamp")" = "$stamp" ]; then
        echo "build-extensions.sh: $out is up to date"
        exit 0
    fi
    rm -f "$out/.stamp"
fi
[ -n "$check_only" ] || build
products=()
for product in "$out/libhostemu.$api_dylib" "$out/liborxncurses.$api_dylib"; do
    [ -f "$product" ] && products+=("$product")
done
if [ ${#products[@]} -eq 0 ]; then
    echo "build-extensions.sh: no extension in $out" >&2
    exit 1
fi

# Both libraries, which every product records as needed, must resolve to --out, under the
# caller's environment and with the oracle's lib directory first on LD_LIBRARY_PATH
# (api_resolves says what the check reads on each host).
check() {
    local product=$1 library status
    shift
    for library in $(api_versioned librexx) $(api_versioned librexxapi); do
        status=0
        api_resolves "$product" "$library" "$out" "$@" || status=$?
        if [ "$status" -ne 0 ]; then
            echo "build-extensions.sh: $product resolves $library outside $out ($*)" >&2
            [ "$status" -ne 2 ] || echo "  (not needed)" >&2
            exit 1
        fi
    done
}
oracle_lib=$root/third_party/oracle/lib
for product in "${products[@]}"; do
    check "$product"
    check "$product" LD_LIBRARY_PATH="$oracle_lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
done
echo "build-extensions.sh: built into $out"
[ -z "$stamp" ] || printf '%s\n' "$stamp" > "$out/.stamp"
