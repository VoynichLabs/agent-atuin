---
name: agent-atuin
description: Log shell commands and create structured memories via agent-atuin CLI. Use when finishing a task and wanting to record what commands were run, when searching past command history across sessions, or when creating/querying hierarchical task memories. Requires atuin binary at ~/bin/atuin.
---

# Agent-Atuin Skill

Structured command memory for OpenClaw agents using the agent-atuin CLI.

## Prerequisites

- `~/bin/atuin` binary (built from VoynichLabs/agent-atuin)
- Environment: `ATUIN_AGENT_ID` set to agent name (e.g., "bubba", "egon", "larry")

## Setup (run once per exec session)

Every exec call needs these env vars:

```bash
export PATH="$HOME/bin:$PATH"
export ATUIN_AGENT_ID="bubba"
export ATUIN_SESSION=$(atuin uuid)
```

## Core Operations

### Log a command to history

Wrap exec calls with start/end to capture them:

```bash
HIST_ID=$(atuin history start -- "the command you ran")
# ... run the actual command ...
atuin history end --exit $? --duration ${DURATION_NS} -- "$HIST_ID"
```

### Create a memory after finishing a task

```bash
atuin memory create "Description of what was accomplished" --link-last N --json
```

- `--link-last N` links the last N commands from history
- `--parent MEMORY_ID` nests under a parent task
- `--json` returns structured output

### Search memories

```bash
atuin memory search "keyword" --json
```

Note: hyphenated terms need quoting for FTS5 (`"agent"` not `agent-atuin`).

### Query command history

```bash
atuin history list --json
```

### View task tree

```bash
atuin memory tree --json
atuin memory tree --root MEMORY_ID --json
```

### Replay commands from a memory

```bash
atuin memory run MEMORY_ID --dry-run
```

## Workflow Pattern

1. At task start: create a parent memory
2. Run commands (optionally logging with history start/end)
3. At task end: create child memory with `--link-last N --parent PARENT_ID`
4. Pre-compaction: review and create any missing memories

## Security

- `secrets_filter = true` in atuin config blocks common credential patterns
- Custom filters in `~/.config/atuin/config.toml` for project-specific patterns
- History DB at `~/.local/share/atuin/history.db` is chmod 600
- No sync server — local only
- Never log commands containing secrets; verify with `atuin history list`
