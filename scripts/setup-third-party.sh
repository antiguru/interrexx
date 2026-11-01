#!/bin/bash
# Fetches and builds the unmodified third-party trees the tests compare against.
set -eu
cd "$(dirname "$0")/.."
OOTEST_REVISION=13178
OODOCS_REVISION=13198
SVN=https://svn.code.sf.net/p/oorexx/code-0

git submodule update --init third_party/ooRexx
cmake -S third_party/ooRexx -B third_party/oracle \
    -DCMAKE_POLICY_VERSION_MINIMUM=3.5 -DCMAKE_BUILD_TYPE=Release
cmake --build third_party/oracle -j"${JOBS:-4}"
ln -sfn third_party/oracle build
[ -d ootest ] || svn checkout -q -r "$OOTEST_REVISION" "$SVN/test/trunk" ootest
[ -d oodocs/rexxref ] || svn checkout -q -r "$OODOCS_REVISION" "$SVN/docs/trunk/rexxref" oodocs/rexxref
[ -d oodocs/rexxpg ] || svn checkout -q -r "$OODOCS_REVISION" "$SVN/docs/trunk/rexxpg" oodocs/rexxpg
