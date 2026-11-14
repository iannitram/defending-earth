#!/bin/bash
# usage: docs/briefs/run_job.sh <job> <model>     e.g.  G3_fire claude-fable-5-2
# Runs one brief with the claude command line, from the project folder.
# The run is not held back by permission checks, only by the brief's own
# wording, so check `git status` after every job.
HERE="$(cd "$(dirname "$0")" && pwd)"
RESULTS="${RESULTS:-/tmp/de_results}"
mkdir -p "$RESULTS"
cd "$HERE/../.." || exit 1
claude -p "$(cat "$HERE/$1.md")" --model "$2" --dangerously-skip-permissions --output-format json > "$RESULTS/$1.json" 2> "$RESULTS/$1.err"
echo "$1 exit=$?"
python3 -I -c "
import json,sys;d=json.load(open(sys.argv[1]))
print({k:d.get(k) for k in ('is_error','num_turns','duration_ms','total_cost_usd')});print(d.get('result'))" "$RESULTS/$1.json"
git status --short; git log -1 --oneline
