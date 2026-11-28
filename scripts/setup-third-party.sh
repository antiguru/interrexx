#!/bin/bash
# Fetches the unmodified test inputs. --standalone skips the oracle build.
set -eu
if [ "$#" -ne 0 ] && { [ "$#" -ne 1 ] || [ "$1" != --standalone ]; }; then
    echo "usage: setup-third-party.sh [--standalone]" >&2
    exit 2
fi
cd "$(dirname "$0")/.."
OOTEST_REVISION=13624
OODOCS_REVISION=13624
SVN=https://svn.code.sf.net/p/oorexx/code-0

books=(rexxref)
if [ "$#" -eq 0 ]; then
    git submodule update --init third_party/ooRexx
    # The compile date is observable through .RexxInfo and PARSE VERSION.
    export SOURCE_DATE_EPOCH=1785412800
    cmake -S third_party/ooRexx -B third_party/oracle \
        -DCMAKE_POLICY_VERSION_MINIMUM=3.5 -DCMAKE_BUILD_TYPE=Release
    cmake --build third_party/oracle -j"${JOBS:-4}"
    ln -sfn third_party/oracle build
    # The other books the spec is read against.
    books+=(rexxpg rexxapi rexxextensions rxmath rxsock unixextensions orxncurses)
fi
[ -d ootest ] || svn checkout -q -r "$OOTEST_REVISION" "$SVN/test/trunk" ootest
for book in "${books[@]}"; do
    [ -d "oodocs/$book" ] || svn checkout -q -r "$OODOCS_REVISION" "$SVN/docs/trunk/$book" "oodocs/$book"
done
