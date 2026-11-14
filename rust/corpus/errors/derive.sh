#!/usr/bin/env bash
# Usage: derive.sh SCRATCH_ROOT [CORPUS] > NEW_CORPUS
# Re-derives parse-errors.tsv from the oracle. CORPUS defaults to the
# parse-errors.tsv beside this script. The oracle's message text is not
# recorded: a row holds the code and the line.
set -u

scratch=${1:?usage: derive.sh SCRATCH_ROOT [CORPUS]}
here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
corpus=${2:-$here/parse-errors.tsv}
oracle=$(dirname "$0")/../../../third_party/oracle

# Prints `expect<TAB>line` for one rexxc stderr file.
extract='
undef $/;
my $e = <STDIN>;
my ($sub, $at);
$sub = "$1.$2" if $e =~ /^Error (\d+)\.(\d+):/m;
$at = $1 if $e =~ /^Error \d+ running .* line (\d+):/m;
if (!defined $sub) { print "ok\t-"; exit }
die "no main line\n" unless defined $at;
print "$sub\t$at";
'

fail() {
    echo "derive.sh: $*" >&2
    exit 1
}

row=0
header=
while IFS= read -r line || [ -n "$line" ]; do
    row=$((row + 1))
    case $line in
    '#'*)
        printf '%s\n' "$line"
        continue
        ;;
    class$'\t'*)
        printf 'class\texpect\tline\tprogram\n'
        header=1
        continue
        ;;
    esac
    [ -n "$header" ] || fail "row $row: no class header line before the first row"
    IFS=$'\t' read -r class expect at program <<<"$line"

    dir=$scratch/row$row
    mkdir -p "$dir" || fail "row $row: cannot make $dir"
    printf '%b' "$program" >"$dir/p.rex"
    (
        cd "$dir" || exit 99
        ulimit -v 1048576
        LD_LIBRARY_PATH=$oracle/lib timeout -k 5 20 "$oracle/bin/rexxc" "$dir/p.rex" -s \
            >"$dir/out" 2>"$dir/err"
        echo $? >"$dir/rc"
    )
    rc=$(<"$dir/rc")
    [ "$rc" = 99 ] && fail "row $row: cannot enter $dir"
    got=$(LC_ALL=C perl -e "$extract" <"$dir/err" || fail "row $row: unreadable oracle stderr, see $dir/err") || exit 1
    IFS=$'\t' read -r gexpect gat <<<"$got"

    if [ "$class" = '?' ]; then
        case $gexpect in
        ok) class=- ;;
        98.903 | 90.999) class=install ;;
        *) class=translation ;;
        esac
    fi
    [ "$expect" = '?' ] && expect=$gexpect
    [ "$at" = '?' ] && at=$gat
    [ "$expect" = "$gexpect" ] || fail "row $row: expect $expect, oracle $gexpect (rc $rc)"
    [ "$at" = "$gat" ] || fail "row $row: line $at, oracle $gat"
    if [ "$gexpect" = ok ]; then
        [ "$rc" = 0 ] || fail "row $row: accepted but rc $rc"
    else
        [ "$rc" != 0 ] || fail "row $row: rejected but rc 0"
    fi
    printf '%s\t%s\t%s\t%s\n' "$class" "$expect" "$at" "$program"
done <"$corpus"
