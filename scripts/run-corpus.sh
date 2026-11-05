#!/bin/bash
# Runs the corpus scripts/fetch-corpus.sh fetched on the oracle and on this crate's debug `rexx`
# and prints one TSV line per program: name, then rc and result for the oracle and for the crate.
# PPWizard's result is the sha256 of the page it writes; a Rexx Parser suite's is its "Tests ran"
# line. Each side runs from the same short directory: PPWizard writes its input's path into its
# output and builds variable names from it.
set -eu
cd "$(dirname "$0")/.."
root=$PWD
[ -d corpus/ppwizard ] && [ -d corpus/rexx-parser ] || {
    echo "run scripts/fetch-corpus.sh first" >&2
    exit 1
}
work=$(mktemp -d /tmp/corpus.XXXXXX)
oracle=$root/third_party/oracle
crate=$root/rust/target/debug
# The oracle's Rexx-only packages, as the suite harness lays them beside this crate's rexx, less
# the five this crate rewrites and embeds.
mkdir "$work/packages"
for package in "$oracle"/bin/*.cls; do
    case $(basename "$package") in
    rxregexp.cls | socket.cls | streamsocket.cls | mime.cls | smtp.cls) ;;
    *) cp "$package" "$work/packages/" ;;
    esac
done
mkdir -m 700 "$work/service"

capped() {
    if command -v memcap >/dev/null; then
        memcap 4G timeout -k 5 600 "$@"
    else
        (ulimit -v 4194304 && exec timeout -k 5 600 "$@")
    fi
}
side() { # oracle|crate DIR ARGS...
    local which=$1 dir=$2
    shift 2
    if [ "$which" = oracle ]; then
        (cd "$dir" && XDG_RUNTIME_DIR="$work/service" LD_LIBRARY_PATH="$oracle/lib" \
            PATH="$PATH_PREFIX$oracle/bin:/usr/bin:/bin" capped "$oracle/bin/rexx" "$@")
    else
        (cd "$dir" && REXX_API_DIR="$work/service" \
            PATH="$PATH_PREFIX$crate:$work/packages:/usr/bin:/bin" capped "$crate/rexx" "$@")
    fi
}
stop_services() {
    REXX_API_DIR="$work/service" "$crate/rxapi" --stop >/dev/null 2>&1 || true
    for pid in $(pgrep -x rxapi); do
        if tr '\0' '\n' <"/proc/$pid/environ" 2>/dev/null |
            grep -qx "XDG_RUNTIME_DIR=$work/service"; then
            kill "$pid"
        fi
    done
}
trap stop_services EXIT

ppwizard() {
    rm -rf "$work/pp"
    mkdir "$work/pp"
    cp corpus/ppwizard/ppwizard-oorexx.rex corpus/ppwizard/tryme.it corpus/ppwizard/htmlpre.ih \
        "$work/pp/"
    local rc=0
    PATH_PREFIX='' side "$1" "$work/pp" ppwizard-oorexx.rex tryme.it >"$work/pp.$1.out" 2>&1 ||
        rc=$?
    printf '%s\t%s' "$rc" "$(sha256sum <"$work/pp/tryme.htm" 2>/dev/null | cut -c1-64)"
}
printf 'ppwizard\t%s\t%s\n' "$(ppwizard oracle)" "$(ppwizard crate)"

for which in oracle crate; do
    rm -rf "$work/rp"
    cp -r corpus/rexx-parser "$work/rp"
    if [ "$which" = oracle ]; then
        cp "$oracle/bin/rxregexp.cls" "$work/rp/tests/framework/"
    else
        cp rust/crates/interrexx/packages/rxregexp.cls "$work/rp/tests/framework/"
    fi
    for suite in "$work"/rp/tests/suites/*.testGroup; do
        name=$(basename "$suite" .testGroup)
        rc=0
        PATH_PREFIX='framework:suites:../bin:' side "$which" "$work/rp/tests" \
            "suites/$name.testGroup" </dev/null >"$work/$name.$which.out" 2>&1 || rc=$?
        ran=$(grep -a -m1 'Tests ran' "$work/$name.$which.out" | tr -s ' ' || true)
        printf '%s\t%s\t%s\n' "$name" "$rc" "$ran"
    done >"$work/rp.$which.tsv"
done
join -t "$(printf '\t')" "$work/rp.oracle.tsv" "$work/rp.crate.tsv" | sed 's/^/rexx-parser /'
echo "outputs in $work" >&2
