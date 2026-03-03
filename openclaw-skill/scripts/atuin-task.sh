#!/bin/bash
# atuin-task.sh — Create a structured memory from recent commands
# Usage: atuin-task.sh "Description of task" [num_commands_to_link] [parent_memory_id]
#
# Example:
#   atuin-task.sh "Built agent-atuin from source" 5
#   atuin-task.sh "Fixed FTS5 search bug" 3 019cb0c90ef07221

set -euo pipefail
export PATH="$HOME/bin:$PATH"
export ATUIN_AGENT_ID="${ATUIN_AGENT_ID:-$(whoami)}"
export ATUIN_SESSION="${ATUIN_SESSION:-$(atuin uuid)}"

DESC="${1:?Usage: atuin-task.sh \"description\" [link_count] [parent_id]}"
LINK_COUNT="${2:-5}"
PARENT="${3:-}"

ARGS=(memory create "$DESC" --link-last "$LINK_COUNT" --json)
[ -n "$PARENT" ] && ARGS+=(--parent "$PARENT")

atuin "${ARGS[@]}"
