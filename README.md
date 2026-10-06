# bluesky-mcp-rs

🦋 **Bluesky + AT Protocol MCP server in Rust.** Stateless. Fast. MCP 2026-07-28.

## Implementation status

This is a prototype. Bluesky, couchsky, and music advancement are not implemented
and return explicit errors without performing external actions. The two music
read tools return labeled static samples. Full MCP interoperability remains unverified.

## Tools

| Tool | Description |
|------|-------------|
| `bluesky_post` | Post to Bluesky |
| `bluesky_timeline` | Get home timeline |
| `bluesky_profile` | Get a profile |
| `bluesky_search` | Search posts |
| `couchsky_*` | Public read-only (3 tools) |
| `music_now_playing` | Get now playing from music.vaked.dev |
| `music_choreographies` | List all choreographies |
| `music_advance` | Advance to next track |

## Quick Start

```bash
cargo build --release
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | ./target/release/bluesky-mcp-rs
```

## Ecosystem

| Repo | Role |
|------|------|
| [bluesky-mcp](https://github.com/8b-is/bluesky-mcp) | TypeScript reference |
| [honest-irc-mcp](https://github.com/8b-is/honest-irc-mcp) | Quantum-proof messaging |
| [ayeos-mcp](https://github.com/8b-is/ayeos-mcp) | Ternary inference |
| [mlx-quant-mcp](https://github.com/8b-is/mlx-quant-mcp) | Ternary quantization |
