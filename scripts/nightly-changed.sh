#!/bin/sh
# Decides whether a nightly workflow has anything new to test. Prints `run=false` when the
# workflow's last successful scheduled run was on the commit under test, and `run=true`
# otherwise.
#
# Usage: nightly-changed.sh WORKFLOW_FILE
#
# Reads GITHUB_REPOSITORY, GITHUB_SHA and GH_TOKEN (a token with `actions: read`), as a
# workflow step has them. Only a successful run counts. A run that failed, was cancelled,
# timed out or did not start does not, so the next night runs again on the same commit: a
# workflow also fails on a setup or runner error, and a rerun is what shows it recovered.
# A night this script skipped also ends in success, but on a commit a successful run had
# already tested, so it moves the baseline nowhere. When the runs cannot be looked up, it
# prints `run=true`: a lookup error must not silently stop the nightly.
set -eu
workflow=$1
last=$(gh api "repos/$GITHUB_REPOSITORY/actions/workflows/$workflow/runs?event=schedule&status=completed&per_page=30" \
  --jq '[.workflow_runs[] | select(.conclusion == "success")][0].head_sha // ""') || {
  echo "Could not look up the last scheduled run; running." >&2
  echo "run=true"
  exit 0
}
if [ -z "$last" ]; then
  echo "No successful scheduled run of $workflow; running." >&2
  echo "run=true"
elif [ "$last" = "$GITHUB_SHA" ]; then
  echo "The last successful scheduled run of $workflow tested $last already; skipping." >&2
  echo "run=false"
else
  echo "The last successful scheduled run of $workflow tested $last; $GITHUB_SHA is not it; running." >&2
  echo "run=true"
fi
