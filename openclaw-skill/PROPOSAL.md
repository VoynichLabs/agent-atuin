# Agent-Atuin: Structured Command Memory for Lobsters

**Date:** 2026-03-02  
**Status:** Layer 1 complete, working demo on Mac Mini  
**Authors:** Bubba + Egon (reviewed by the boss)

## What It Does (30 seconds)

- Every shell command an agent runs gets logged to a local SQLite database
- Agents create named "memories" linked to the actual commands that did the work
- Memories form parent-child trees (task → subtasks)
- Everything is searchable and queryable with `--json` output
- Each agent is tagged (`ATUIN_AGENT_ID=bubba`) so you know who did what

## What Problem It Solves

Today when a lobster molts, every command it ran is gone. If Larry ran 15 commands to build charts yesterday, nobody can recover those commands. We write prose to MEMORY.md but lose the actual execution trail.

With agent-atuin: `atuin memory search "charts" --json` returns the structured memory with every linked command, exit code, and duration — even after a molt.

## Security

- **Local only** — no sync server, database never leaves the machine
- **DB permissions** — `chmod 600` on history.db, owner-only access
- **Built-in secrets filter** — auto-blocks AWS keys, GitHub PATs, Slack tokens, Stripe keys
- **Custom filters added** — blocks `sshpass`, `export.*KEY`, `Bearer`, `sk-*`, `ghp_*`
- **What humans should check periodically:**
  - Run `atuin history list` and scan for anything sensitive that slipped through
  - Review `~/.config/atuin/config.toml` filters if new credential formats are introduced

## How It Works (for agents)

```
1. Agent starts a task
2. Agent runs commands via exec (optionally logging with atuin history start/end)
3. Agent finishes task → runs: atuin memory create "What I did" --link-last N --json
4. Memory is stored in SQLite with links to actual command history
5. Next session: atuin memory search "keyword" --json recovers it
```

No zsh hook needed. Direct CLI calls from exec work perfectly.

## What Humans Need To Do

1. **Nothing day-to-day** — agents handle memory creation automatically
2. **Periodic audit** — `atuin history list` to check nothing sensitive leaked
3. **Update filters** — if new credential formats appear, add regex to config.toml
4. **Simon:** review the Rust source if desired (`/tmp/agent-atuin/crates/atuin-memory/`)

## What's Built

- ✅ atuin v18.13.0-beta.2 binary at `~/bin/atuin`
- ✅ Rust 1.93.1 toolchain on Mac Mini
- ✅ Security filters configured
- ✅ OpenClaw skill at `~/bubba-workspace/skills/agent-atuin/`
- ✅ Working demo with parent-child task tree + linked commands

## Architecture: Bubba Is the Memory Bank

- **Egon's Linode** has 2GB RAM — can't compile Rust, can't run atuin locally
- **Cross-compiling** from Mac → Linux needs a linker toolchain we don't have
- **Solution:** Bubba is the single atuin node. When Egon or Larry need command history, they ask Bubba in Discord. Bubba queries atuin and replies. Simple.
- No SSH tunneling needed — just lobsters talking to each other in the channel

## What Humans Need To Do

1. **Nothing day-to-day** — Bubba handles memory creation during pre-compaction flush
2. **Periodic audit** — run `atuin history list` on the Mac Mini to check for leaked secrets
3. **Update filters** — add new regex patterns to `~/.config/atuin/config.toml` if new credential formats appear

## What's Next (needs Simon's input)

- Review the Rust crate quality (`/tmp/agent-atuin/crates/atuin-memory/`)
- Integrate with OpenClaw's pre-compaction flush for automatic memory creation
- Consider: export atuin memories to Markdown so `memory_search` can find them too

## Demo Output

```
Tree:
└── Demo: Check VoynichLabs repos (0 commands)
    └── Listed repos and checked recent commits (2 commands)
        [0] ✓ gh repo list VoynichLabs --limit 5
        [1] ✓ git -C /tmp/agent-atuin log --oneline -3
```
