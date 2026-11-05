#!/bin/sh
# Setup script: gives the method-body tests (one process each) one API service directory of their own for this run.
# Its `RexxQueue create` row is stable only against a service no other test holds open, and
# the target directory's shared service is held open by whichever test last left a queue.
# Under `cargo test`, where binaries run one after another, the shared service is used.
set -eu
root=${NEXTEST_WORKSPACE_ROOT:-$PWD}
tmp=$root/target/tmp
mkdir -p "$tmp"
find "$tmp" -maxdepth 1 -name 'method-bodies-api-*' -mmin +120 -exec rm -rf {} + 2>/dev/null || true
id=$(printf '%s' "${NEXTEST_RUN_ID:-$$}" | cut -c1-8)
echo "REXX_API_DIR=$tmp/method-bodies-api-$id" >> "$NEXTEST_ENV"
