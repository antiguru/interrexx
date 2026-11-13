#!/usr/bin/env bash
# Compiles the spec's fixture libraries from spec/fixture/specfixture.cpp, against the public API
# headers only:
#
#   build-spec-fixture.sh [--out DIR]
#
# --out  where the libraries go. Default: rust/target/spec-fixture.
#
# The products are libspecfixture.so, libspecclassic.so and libspecnewer.so. They link nothing
# from an interpreter, so the same files load into interrexx and into the oracle; the build fails
# if `ldd` shows either interpreter library among a product's dependencies.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
out=$root/rust/target/spec-fixture
while [ $# -gt 0 ]; do
    case $1 in
        --out) out=$2; shift 2 ;;
        *) echo "build-spec-fixture.sh: unknown argument $1" >&2; exit 2 ;;
    esac
done

src=$root/spec/fixture/specfixture.cpp
api=$root/third_party/ooRexx/api
. "$root/scripts/api-flags.sh"

mkdir -p "$out"
out=$(cd "$out" && pwd)

# Test processes that build at once take turns, and one whose inputs are those of the build
# already in --out has nothing left to do.
exec 9>"$out.lock"
flock 9
stamp=$(stat -c '%n %s %.9Y' "$src" "$0" "$root/scripts/api-flags.sh"
    git -C "$root/third_party/ooRexx" rev-parse HEAD
    "$api_cxx" --version | head -n 1)
if [ -f "$out/.stamp" ] && [ "$(cat "$out/.stamp")" = "$stamp" ]; then
    echo "build-spec-fixture.sh: $out is up to date"
    exit 0
fi
rm -f "$out/.stamp"

for variant in PACKAGE:specfixture CLASSIC:specclassic NEWER:specnewer; do
    define=${variant%%:*}
    name=${variant#*:}
    "$api_cxx" -shared -o "$out/lib$name.so.tmp" "$src" "${api_flags[@]}" -pthread \
        -Wall -Werror -Wno-unused-parameter -DSPEC_FIXTURE_"$define"
    if ldd "$out/lib$name.so.tmp" | grep -Eq 'librexx(api)?\.so'; then
        echo "build-spec-fixture.sh: lib$name.so depends on an interpreter library" >&2
        exit 1
    fi
    mv -f "$out/lib$name.so.tmp" "$out/lib$name.so"
done
echo "build-spec-fixture.sh: built into $out"
printf '%s\n' "$stamp" > "$out/.stamp"
