#!/bin/sh
# Decides whether a nightly workflow has anything new to test. Prints `run=true` when the
# commit under test differs from the commit of the workflow's last finished scheduled run,
# and `run=false` when they are the same.
#
# Usage: nightly-changed.sh WORKFLOW_FILE
#
# Reads GITHUB_REPOSITORY, GITHUB_SHA and GH_TOKEN (a token with `actions: read`), as a
# workflow step has them. A run that was cancelled, timed out or did not start does not
# count as a finished run, so the next night runs again on the same commit. A failed run
# does count: the same commit fails the same way, and a dispatch reruns it by hand. When
# the last run cannot be looked up, it prints `run=true`: a lookup error must not
# silently stop the nightly.
set -eu
workflow=$1
last=$(gh api "repos/$GITHUB_REPOSITORY/actions/workflows/$workflow/runs?event=schedule&status=completed&per_page=30" \
  --jq '[.workflow_runs[] | select(.conclusion == "success" or .conclusion == "failure")][0].head_sha // ""') || {
  echo "Could not look up the last scheduled run; running." >&2
  echo "run=true"
  exit 0
}
if [ -z "$last" ]; then
  echo "No finished scheduled run of $workflow; running." >&2
  echo "run=true"
elif [ "$last" = "$GITHUB_SHA" ]; then
  echo "The last scheduled run of $workflow tested $last already; skipping." >&2
  echo "run=false"
else
  echo "The last scheduled run of $workflow tested $last; $GITHUB_SHA is new; running." >&2
  echo "run=true"
fi
