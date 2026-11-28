#!/usr/bin/env bash
# Exercise the renderer through its CLI against an isolated tracked inventory.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
rexx=$(command -v "${REXX:-rexx}")
case "$rexx" in
    /*) ;;
    *) rexx="$PWD/$rexx" ;;
esac
scratch=$(mktemp -d)
trap 'rm -rf -- "$scratch"' EXIT
repo="$scratch/inventory checkout"
mkdir -p "$repo/rust/scripts" "$repo/rust/corpus"
mkdir -m 700 "$scratch/api"

export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
export REXX_API_DIR="$scratch/api"
cp "$here/conformance_summary.rex" "$repo/rust/scripts/"
cd "$repo/rust/corpus"
printf '%s\n' 13624 > suite-revision.txt
printf '%s\n' '# included paths' '' lang/one.rex lang/two.rex lang/three.rex > differential.txt
printf 'lang/four.rex\tRecorded Deviation\n' > unfiled.txt
printf 'LENGTH\timplemented\nBEEP\tlater-level\n' > builtin-status.txt
printf '1\tactive\t-\t-\n2\twithdrawn\t-\t-\n' > deviations.tsv
cat > suite-licences.txt <<'TSV'
# ootest revision 13624
base/bif/LENGTH	TEST_A	Unspecified	Wording is unspecified
base/class/Stream	TEST_B	Deviation 1	Stream differs
TSV
cat > suite-not-run.txt <<'TSV'
# ootest revision 13624
base/rexxutil/platform/windows/SysIni	TEST_C	NotRun	restrictOS line 12
extensions/platform/windows/ole/OLEObject	TEST_D	NotRun	platform windows
TSV
cd "$repo"
git init -q --object-format=sha1
git add rust
pin=1111111111111111111111111111111111111111
git update-index --add --cacheinfo "160000,$pin,third_party/ooRexx"
git -c user.name=Fixture -c user.email=fixture@example.invalid -c commit.gpgsign=false commit -qm Inventory
commit=$(git rev-parse HEAD)
# The renderer reads the Git snapshot even if its source files are dirty.
printf '%s\n' 'uncommitted inventory edit' >> rust/corpus/differential.txt

cd "$scratch"
cat > 'suite results.tsv' <<'TSV'
# ootest revision 13624
# groups base/class/Stream,API/oo/METHOD
group	test	row	oracle	crate
base/class/Stream	TEST_A	-	passed	passed
base/class/Stream	TEST_B	Deviation(1)	passed	not started
base/class/Stream	TEST_C	-	passed	failed
base/class/Stream	TEST_D	-	passed	blocked: timeout
base/class/Stream	TEST_E	-	passed	started
API/oo/METHOD	TEST_A	-	passed	blocked at load: no summary
TSV
shard() {
    printf 'interrexx differential corpus report -- rust/corpus/differential.txt -- shard %s of 2\n' "$1"
    printf 'mode: STRICT (the gate) -- REXX_CORPUS_GATE is set\n%s of %s matching\n' "$2" "$3"
}
shard 0 1 2 > 'corpus first.log'
{ cat 'corpus first.log'; shard 1 1 1; } > 'corpus complete.log'

render() {
    "$rexx" "$repo/rust/scripts/conformance_summary.rex" "$@" > summary.md 2> summary.err
}
shows() {
    grep -Fqx -- "$1" summary.md
}
clean_stderr() {
    if test -s summary.err; then
        cat summary.err >&2
        return 1
    fi
}
rejects() {
    local needle=$1
    shift
    local status=0
    render "$@" || status=$?
    test "$status" = 1
    test ! -s summary.md
    grep -Fq -- "$needle" summary.err
}

render
clean_stderr
shows "- Interrexx inventory commit \`$commit\`"
shows "- ooRexx pin \`$pin\` from \`third_party/ooRexx\`"
shows '- ooTest pin r13624'
shows '- 3 programs are listed in `differential.txt`.'
shows '- `deviations.tsv` lists 1 active and 1 withdrawn Deviations.'
shows '| later-level | 1 | BEEP |'
shows '| Built-in functions | 0 | 1 | 0 |'
shows '| Streams | 1 | 0 | 0 |'
shows '| RexxUtil | 0 | 0 | 1 |'
shows '| Platform extensions | 0 | 0 | 1 |'
shows '| RexxUtil | NotRun | restrictOS line 12 | 1 |'
shows '| lang/four.rex | Recorded Deviation |'
shows 'No strict corpus log supplied.'
shows 'No ooTest per-test report supplied.'
if grep -Fq 'Program comparisons reported' summary.md; then exit 1; fi

render --run-commit="$commit" --corpus-log 'corpus first.log' --suite-report 'suite results.tsv'
clean_stderr
shows 'Strict differential reports cover 1 of 2 shards.'
shows '| 2 | 1 | 1 |'
shows 'ooTest r13624, selected groups (base/class/Stream,API/oo/METHOD). These are per-test outcomes as recorded by the suite gate.'
shows '| Streams | interrexx | 4 | 1 | 1 | 1 | 1 | 0 | 1 |'
shows '| Streams | ooRexx | 5 | 5 | 0 | 0 | 0 | 0 | 0 |'
shows '| Native API | interrexx | 0 | 0 | 0 | 0 | 0 | 1 | 0 |'

render --run-commit "$commit" --corpus-log 'corpus complete.log'
clean_stderr
shows 'Strict differential reports cover 2 of 2 shards.'
shows '| 3 | 2 | 1 |'

rejects 'observations require --run-commit' --corpus-log 'corpus first.log'
sed 's/revision 13624/revision 13625/' 'suite results.tsv' > invalid.tsv
rejects 'does not match the ooTest pin' --run-commit "$commit" --suite-report invalid.tsv
sed 's/blocked: timeout/passed eventually/' 'suite results.tsv' > invalid.tsv
rejects 'unknown verdict' --run-commit "$commit" --suite-report invalid.tsv
cat 'corpus first.log' 'corpus first.log' > invalid.log
rejects 'repeated or inconsistent shard indices' --run-commit "$commit" --corpus-log invalid.log
shard 0 1 1 > invalid.log
rejects 'does not match the tracked inventory size' --run-commit "$commit" --corpus-log invalid.log
sed 's/mode: STRICT/mode: REPORT/' 'corpus first.log' > invalid.log
rejects 'no strict differential report' --run-commit "$commit" --corpus-log invalid.log
rejects 'cannot read' --run-commit "$commit" --suite-report missing.tsv

printf '%s\n' 'Conformance summary CLI checks passed.'
