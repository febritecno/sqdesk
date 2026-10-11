# Removed features

SQDesk is a trimmed fork of [DBX](https://github.com/t8y2/dbx). This folder records how each removed feature worked, so it can be rebuilt without reverse-engineering it again.

Each document covers: what the feature did, its algorithm, the data it stored, the contracts it had with the outside world, and where its code lived.

| Feature | Document | Status |
| --- | --- | --- |
| MCP server and bridge | [mcp-server.md](mcp-server.md) | Removed |
| CLI-based AI providers | [cli-ai-providers.md](cli-ai-providers.md) | Removed; the provider enum variants remain so saved configs still load |
| Cloud sync (WebDAV, snippets) | [cloud-sync.md](cloud-sync.md) | Removed |
| Update checker | [updater.md](updater.md) | Unused by the app; still in the tree because the JDBC plugin installer reads the same release manifest |
| Plugin runtime and marketplace | [plugin-runtime.md](plugin-runtime.md) | UI removed; runtime still in the tree (wired into connection, query and schema code) |
| Agent (JDBC) drivers | [driver-agents.md](driver-agents.md) | Still in the tree and still the only way to reach about half of the listed database types |

## Getting the original code back

The fastest route is upstream: every feature here still exists in DBX. The documents name the upstream paths; in SQDesk the crates are called `sqdesk-*` where upstream says `dbx-*`.

## The naming trap

SQDesk renamed `DBX` / `dbx` to `SQDESK` / `sqdesk` everywhere in its own code. Anything that talks to something **outside** this repository still uses the upstream spelling, and a rebuilt feature has to use it too:

| Contract | Upstream spelling | Who is on the other side |
| --- | --- | --- |
| Agent artifacts | `dbx-agent-<name>.jar` | DBX release server |
| Agent error marker | `\nDBX_AGENT_ERROR_DATA:` | Prebuilt agent jars |
| Agent environment | `DBX_AGENT_JAVA_OPTS`, `DBX_JAVA_BIN` | Prebuilt agent jars |
| Plugin package | `.dbxp`, plugin ids `io.dbx.*` | DBX plugin store |
| Plugin signing keys | `dbx-store-release-2026` | DBX plugin store |
| MCP npm package | `@dbx-app/mcp-server`, binary `dbx-mcp-server` | npm registry |
| MCP tool names | `dbx_execute_query`, ... | External MCP clients |
| HTTP tunnel | header `X-DBX-Tunnel-Token`, script `dbx_tunnel.php` | The user's web server |

Three on-disk formats were deliberately **not** renamed and still say `dbx`: the secret envelope prefix `dbxenc1`, the connection export formats `dbx-config` / `dbx-encrypted`, and the export identity magic `DBXEI`.
