#!/usr/bin/env bash
# Compiles ooRexx's native-API test libraries and drivers, unmodified, against this workspace's
# librexx.so.4 and librexxapi.so.4, never the oracle's.
#
#   build-testbinaries.sh [--libs DIR] [--out DIR]
#
# --libs  a directory holding librexx.so and librexxapi.so as cargo built them. Default: build
#         both with `cargo build` and use rust/target/debug (or $CARGO_TARGET_DIR/debug).
# --out   where the libraries and binaries go. Default: rust/target/testbinaries.
#
# Every product records only --out as its search path (DT_RPATH, which the loader consults before
# LD_LIBRARY_PATH). The build fails if `ldd` resolves either library anywhere else, with or
# without the oracle's lib directory on LD_LIBRARY_PATH.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
libs=
out=$root/rust/target/testbinaries
while [ $# -gt 0 ]; do
    case $1 in
        --libs) libs=$2; shift 2 ;;
        --out) out=$2; shift 2 ;;
        *) echo "build-testbinaries.sh: unknown argument $1" >&2; exit 2 ;;
    esac
done

if [ -z "$libs" ]; then
    (cd "$root/rust" && cargo build --locked -p interrexx-capi -p interrexx-capi-rxapi)
    libs=${CARGO_TARGET_DIR:-$root/rust/target}/debug
fi

src=$root/third_party/ooRexx/testbinaries
api=$root/third_party/ooRexx/api
samples=$root/third_party/ooRexx/samples/api/classic/unix/callrexx

mkdir -p "$out"
out=$(cd "$out" && pwd)
for library in librexx librexxapi; do
    cp -f "$libs/$library.so" "$out/$library.so.4.tmp"
    mv -f "$out/$library.so.4.tmp" "$out/$library.so.4"
    ln -sfn "$library.so.4" "$out/$library.so"
done

flags=(-I"$api" -I"$api/platform/unix" -fPIC -L"$out" -Wl,--disable-new-dtags -Wl,-rpath,"$out")

shared() {
    local compiler=$1 name=$2
    shift 2
    "$compiler" -shared -o "$out/lib$name.so" "$@" "${flags[@]}"
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
if [ -f "$samples/callrexx1.cpp" ]; then
    c++ -o "$out/callrexx1" "$samples/callrexx1.cpp" "${flags[@]}" -lrexx -lrexxapi
fi

products=("$out"/liborx*.so "$out/rexxinstance" "$out/provoke_locks")
[ -f "$out/callrexx1" ] && products+=("$out/callrexx1")

# Each of the two libraries a product needs must resolve to --out, under the caller's environment
# and with the oracle's lib directory first on LD_LIBRARY_PATH.
check() {
    local product=$1 resolved library
    shift
    resolved=$(env "$@" ldd "$product")
    for library in librexx.so.4 librexxapi.so.4; do
        grep -q "^[[:space:]]*$library " <<<"$resolved" || continue
        if ! grep -q "^[[:space:]]*$library => $out/$library " <<<"$resolved"; then
            echo "build-testbinaries.sh: $product resolves $library outside $out ($*):" >&2
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
echo "build-testbinaries.sh: built into $out"
