#!/bin/bash
# Fetches the real-program corpus (Phase 10 design, "Corpus") into corpus/, checked against pinned
# sha256 hashes. Nothing fetched is committed: PPWizard's licence allows distributing its archive
# only in full, and CI does not run the corpus.
set -eu
cd "$(dirname "$0")/.."
PPWIZARD_URL=http://dennisbareis.com/zips_fw/ppwall.zip
PPWIZARD_SHA256=c538b4d899a88f612c108470f91fd6aee82d7e3d335b0332f75b3055a031ec8c
PPWIZARD_REX_SHA256=d6853c2959ab8d75e778b3419ab701fe576f65f7d6ee4ef07b6ad1906b88b235
PARSER_COMMIT=33cf252de3a7ca99b0dbe89a8fe2adb81c9c824b
PARSER_URL=https://github.com/JosepMariaBlasco/rexx-parser/archive/$PARSER_COMMIT.tar.gz
PARSER_SHA256=7b6c0a8e0a78beed1eee6c7ecb52b3407120a1200741751e3766427b877ee8ae

fetch() { # URL SHA256 FILE
    [ -f "$3" ] || curl -sSfL -o "$3.part" "$1"
    [ -f "$3" ] || mv "$3.part" "$3"
    echo "$2  $3" | sha256sum -c --quiet
}

mkdir -p corpus/downloads
fetch "$PPWIZARD_URL" "$PPWIZARD_SHA256" corpus/downloads/ppwall.zip
fetch "$PARSER_URL" "$PARSER_SHA256" corpus/downloads/rexx-parser.tar.gz

if [ ! -d corpus/ppwizard ]; then
    mkdir corpus/ppwizard.part
    unzip -q corpus/downloads/ppwall.zip -d corpus/ppwizard.part
    echo "$PPWIZARD_REX_SHA256  corpus/ppwizard.part/ppwizard.rex" | sha256sum -c --quiet
    # The licence allows modifying the code. PPWizard's interpreter detection takes ooRexx for
    # OS/2: these two one-word changes (docs/superpowers/plans/ppwizard-gate.md) make it treat
    # Linux as UNIX and read the environment through ooRexx's selector.
    LC_ALL=C sed \
        -e 's/^if RexSystemOpSys="BEOS" then/if RexSystemOpSys="BEOS" | RexSystemOpSys="LINUX" | RexSystemOpSys="AIX" | RexSystemOpSys="DARWIN" then/' \
        -e "s/^RexEnvVarPool='OS2ENVIRONMENT'/RexEnvVarPool='ENVIRONMENT'/" \
        corpus/ppwizard.part/ppwizard.rex >corpus/ppwizard.part/ppwizard-oorexx.rex
    [ "$(LC_ALL=C grep -ac 'RexSystemOpSys="LINUX"\|^RexEnvVarPool=.ENVIRONMENT.' \
        corpus/ppwizard.part/ppwizard-oorexx.rex)" = 2 ]
    mv corpus/ppwizard.part corpus/ppwizard
fi

if [ ! -d corpus/rexx-parser ]; then
    mkdir corpus/rexx-parser.part
    tar -xzf corpus/downloads/rexx-parser.tar.gz -C corpus/rexx-parser.part --strip-components=1
    mv corpus/rexx-parser.part corpus/rexx-parser
fi
echo "corpus/ppwizard and corpus/rexx-parser are ready"
