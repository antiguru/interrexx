#!/bin/bash
# Run in the foreground under an outer memcap 16G, with normal login environment.
set -eu
D=$(cd "$(dirname "$0")" && pwd)
exec python3 "$D/runner.py" "$@"
