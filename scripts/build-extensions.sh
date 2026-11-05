#!/usr/bin/env bash
# Compiles ooRexx's hostemu and orxncurses extensions, unmodified, against this workspace's
# librexx.so.4 and librexxapi.so.4, never the oracle's.
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
# LD_LIBRARY_PATH). The build fails if `ldd` resolves either library anywhere else, with or
# without the oracle's lib directory on LD_LIBRARY_PATH.
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
api=$root/third_party/ooRexx/api

build() {
    if [ -z "$libs" ]; then
        (cd "$root/rust" && cargo build --locked -p interrexx-capi -p interrexx-capi-rxapi)
        libs=${CARGO_TARGET_DIR:-$root/rust/target}/debug
    fi
    for library in librexx librexxapi; do
        cp -f "$libs/$library.so" "$out/$library.so.4.tmp"
        mv -f "$out/$library.so.4.tmp" "$out/$library.so.4"
        ln -sfn "$library.so.4" "$out/$library.so"
    done

    # --no-as-needed keeps both libraries recorded although a product may call into only one,
    # so the check below always has both to look at.
    local flags=(-w -I"$api" -I"$api/platform/unix" -fPIC -shared -L"$out"
        -Wl,--disable-new-dtags -Wl,-rpath,"$out" -Wl,--no-as-needed)

    c++ -o "$out/libhostemu.so" "$src/hostemu/platform/unix/hostemu.cpp" \
        "$src/hostemu/cmdparse.cpp" -I"$src/hostemu" "${flags[@]}" -pthread -lrexx -lrexxapi

    rm -f "$out/liborxncurses.so"
    if echo '#include <ncurses.h>' | c++ -fsyntax-only -x c++ - 2>/dev/null; then
        c++ -o "$out/liborxncurses.so" "$src/orxncurses/orxncurses.cpp" -DCURSES_HAVE_NCURSES_H \
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
    flock 9
    stamp=$(stat -c '%n %s %.9Y' "$libs/librexx.so" "$libs/librexxapi.so" "$0")
    if [ -f "$out/.stamp" ] && [ "$(cat "$out/.stamp")" = "$stamp" ]; then
        echo "build-extensions.sh: $out is up to date"
        exit 0
    fi
    rm -f "$out/.stamp"
fi
[ -n "$check_only" ] || build
products=()
for product in "$out/libhostemu.so" "$out/liborxncurses.so"; do
    [ -f "$product" ] && products+=("$product")
done
if [ ${#products[@]} -eq 0 ]; then
    echo "build-extensions.sh: no extension in $out" >&2
    exit 1
fi

# Both libraries, which every product records as needed, must resolve to --out, under the
# caller's environment and with the oracle's lib directory first on LD_LIBRARY_PATH.
check() {
    local product=$1 resolved library
    shift
    resolved=$(env "$@" ldd "$product")
    for library in librexx.so.4 librexxapi.so.4; do
        if ! grep -q "^[[:space:]]*$library => $out/$library " <<<"$resolved"; then
            echo "build-extensions.sh: $product resolves $library outside $out ($*):" >&2
            grep "$library" <<<"$resolved" >&2 || echo "  (not resolved)" >&2
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
