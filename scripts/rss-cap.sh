#!/usr/bin/env bash
# Runs a command and kills it and every process below it once their summed resident set passes a
# bound, for a host with no memory cap of its own, such as macOS.
#
#   rss-cap.sh LIMIT_KIB COMMAND [ARGS...]
#
# Exits with the command's status, or 137 when the bound fired. The resident sets are read with
# `ps` twice a second, so a process that allocates faster than that can pass the bound by what it
# allocates in half a second before it is killed.
set -u

if [ "$#" -lt 2 ]; then
    echo "usage: rss-cap.sh LIMIT_KIB COMMAND [ARGS...]" >&2
    exit 2
fi
limit=$1
shift
"$@" &
root=$!
fired=0

# Each process at or below the command, with its resident set in KiB.
descendants() {
    ps -A -o pid=,ppid=,rss= | awk -v root="$root" '
        { parent[$1] = $2; rss[$1] = $3 }
        END {
            for (p in parent) {
                q = p
                while (q != "" && q != 0 && q != 1) {
                    if (q == root) { print p, rss[p]; break }
                    q = parent[q]
                }
            }
        }'
}

while kill -0 "$root" 2>/dev/null; do
    snapshot=$(descendants)
    total=$(awk '{ s += $2 } END { print s + 0 }' <<<"$snapshot")
    if [ "$total" -gt "$limit" ]; then
        echo "rss-cap: ${total} KiB over ${limit} KiB in $(wc -l <<<"$snapshot") processes, killing" >&2
        # The five largest, by resident set and then by command line.
        sort -k2 -n -r <<<"$snapshot" | head -5 >&2
        ps -A -o pid=,command= | awk 'NR == FNR { top[$1]; next } ($1 in top)' \
            <(sort -k2 -n -r <<<"$snapshot" | head -5) - | cut -c1-200 >&2
        awk '{ print $1 }' <<<"$snapshot" | xargs kill -9 2>/dev/null
        fired=1
        break
    fi
    sleep 0.5
done
wait "$root"
status=$?
[ "$fired" -eq 0 ] || exit 137
exit "$status"
