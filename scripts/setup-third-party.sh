#!/bin/bash
# Fetches and builds the unmodified third-party trees the tests compare against.
set -eu
cd "$(dirname "$0")/.."
OOTEST_REVISION=13347
OODOCS_REVISION=13347
SVN=https://svn.code.sf.net/p/oorexx/code-0

git submodule update --init third_party/ooRexx
# The compile date is observable through .RexxInfo and PARSE VERSION.
export SOURCE_DATE_EPOCH=1785412800
cmake -S third_party/ooRexx -B third_party/oracle \
    -DCMAKE_POLICY_VERSION_MINIMUM=3.5 -DCMAKE_BUILD_TYPE=Release
cmake --build third_party/oracle -j"${JOBS:-4}"
ln -sfn third_party/oracle build
[ -d ootest ] || svn checkout -q -r "$OOTEST_REVISION" "$SVN/test/trunk" ootest
# The books the spec is read against: the language reference, the programming
# guide, and the books of the native API and of the extension libraries.
for book in rexxref rexxpg rexxapi rexxextensions rxmath rxsock unixextensions orxncurses; do
    [ -d "oodocs/$book" ] || svn checkout -q -r "$OODOCS_REVISION" "$SVN/docs/trunk/$book" "oodocs/$book"
done
