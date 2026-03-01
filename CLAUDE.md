# agent-atuin

Fork of [Atuin](https://github.com/atuinsh/atuin) with AI agent support. Adds agent identification (`ATUIN_AGENT_ID`), structured JSON output (`--json`), and a memory store (`atuin memory`).

@AGENTS.md — crate map, sync protocols, encryption, conventions
@docs/AGENT_SETUP.md — agent CLI reference and workflows

## Build & check

```sh
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

Runner: `cargo nextest` (preferred over `cargo test`).

## Key rules

### Hot paths — no DB calls
`history start`, `history end`, and `init` skip database initialization for latency. Never add DB calls to these code paths.

### Database migrations
Never modify existing migrations. Only add new ones. Client uses SQLite (WAL mode); server uses Postgres or SQLite.

### History struct construction
Always include `agent_id` when building `History` structs — use `agent_id: None` if no agent context. Forgetting this field causes compile errors since the agent-atuin fork added it.

## Fork-specific additions

### `ATUIN_AGENT_ID` environment variable
Tags all commands with the agent's identifier. Set early in a session.

### `--json` flag
All agent-facing commands support `--json` for structured output.

### `atuin memory` subcommand
Creates searchable memories linked to commands. Lives in `crates/atuin-memory/`.
- Schema: `memories` table + `memory_commands` join table + FTS5 index
- Database: `~/.local/share/atuin/memory.db`

## Crate quick reference

| Crate | Role |
|-------|------|
| `atuin` | CLI binary + TUI |
| `atuin-client` | Local DB, encryption, sync, settings |
| `atuin-common` | Shared types and API models |
| `atuin-daemon` | Background gRPC daemon |
| `atuin-server` | HTTP sync server (axum) |
| `atuin-server-postgres` | Postgres backend |
| `atuin-memory` | Memory store crate (this fork) |

## Testing quick reference

- Unit tests: inline `#[cfg(test)]`, async via `#[tokio::test]`
- Integration tests: `crates/atuin/tests/` (need Postgres via `ATUIN_DB_URI`)
- Use `":memory:"` SQLite for unit tests needing a database
- New `History` structs in tests must include `agent_id: None`

## Conventions

- Rust 2024 edition, toolchain 1.93.1
- Errors: `eyre::Result` in binaries, `thiserror` in libraries
- IDs: UUIDv7 with newtype wrappers (`HistoryId`, `RecordId`, `HostId`)
- `#![deny(unsafe_code)]` on client/common, `#![forbid(unsafe_code)]` on server
- Clippy: `pedantic` + `nursery`, CI enforces `-D warnings`
