# MCP server and bridge

Exposed SQDesk's connections to AI clients over the Model Context Protocol: list connections, read schema, run queries. It was also how the CLI AI providers reached the database (see [cli-ai-providers.md](cli-ai-providers.md)).

Removed: the `sqdesk-mcp` crate, the bridge, the HTTP server, and the npm installer logic. About 22,000 lines.

## Pieces

| Piece | Role |
| --- | --- |
| `sqdesk-mcp` binary | The MCP server. Opens the app's database file directly to read connections and run queries. |
| MCP bridge (in the app) | Loopback HTTP listener the MCP process calls for things that need the running app. |
| MCP HTTP server (in the app) | Optional: the same server embedded in the app over streamable HTTP. |
| Resolver | Finds the installed MCP server binary so the app can hand it to a CLI agent. |

## MCP server

Transports, chosen by flag or environment:

- `--stdio` (default) or `--http` / `--transport streamable-http`.
- HTTP options: `--http-host`, `--http-port`, `--http-path` (must be absolute, for example `/mcp`), `--http-allow-remote`.
- Binding a non-loopback address requires **both** `--http-allow-remote` and `<APP>_MCP_HTTP_ALLOW_REMOTE=1`, plus `<APP>_MCP_HTTP_ALLOWED_HOSTS` and `<APP>_MCP_HTTP_ALLOWED_ORIGINS`.
- HTTP always needs a bearer token: `<APP>_MCP_HTTP_TOKEN` or `<APP>_MCP_HTTP_TOKEN_FILE`, never both.

Tools (prefix `sqdesk_`, upstream `dbx_`): `list_connections`, `list_databases`, `list_tables`, `describe_table`, `table_schema`, `list_routines`, `get_routine_source`, `get_schema_context`, `execute_query`, `execute_batch`, `execute_and_show`, `execute_redis_command`, `open_table`, `add_connection`, `remove_connection`, `duplicate_connection`, `open_session`, `close_session`, `begin_transaction`, `commit_transaction`, `rollback_transaction`, the Salesforce trio (`salesforce_current_user`, `salesforce_prepare_write`, `salesforce_apply_write`), message-queue tools (`peek_messages`, `send_message`), and plugin tools exposed as `sqdesk_<plugin>__<tool>`.

Resources use the `sqdesk://` scheme.

### Scope and safety

A session can be narrowed with environment variables, which is how the app confined a CLI agent to the chat's connection:

| Variable | Effect |
| --- | --- |
| `<APP>_MCP_SCOPE_CONNECTION_ID`, `_NAME` | Only this connection is visible |
| `<APP>_MCP_SCOPE_DATABASE` | Only this database (empty = no single-database bound) |
| `<APP>_MCP_SCOPE_SCHEMA` | Default schema |
| `<APP>_MCP_ALLOW_WRITES` | `1` allows INSERT / UPDATE / DELETE |
| `<APP>_MCP_ALLOW_DANGEROUS_SQL` | `1` allows DDL and other dangerous statements |
| `<APP>_MCP_CONFIRMED_WRITE_SQL` | A write must equal this text exactly (after trimming) |

On top of the session scope sits a persisted global policy (allowed connections, databases and tools) checked by `ensure_tool_allowed` and the connection checks. Statement risk comes from `sqdesk-sql-core::sql_risk`. Production-marked databases refuse writes.

Reading credentials: on macOS the server reads the app's keychain item only if its own code signature matches the app's identity.

## Bridge

1. On startup the app binds `127.0.0.1:0` and writes the chosen port to `<data dir>/mcp-bridge-port`.
2. The MCP process reads that file and sends plain HTTP/1.1 `POST` requests.
3. The bridge rejects anything that is not a local client, reads the header block and then exactly `Content-Length` bytes (a single read truncated large SQL), and routes by path:
   - `/open-table` — open a table tab in the UI
   - `/call-plugin-tool`, `/list-plugin-connections`
   - `/data/list-tables`, `/data/describe-table`
   - `/data/mongo/*` — collection listing, counts, find, aggregate, stats
4. Tauri commands that could be reached this way took an `mcp_request` flag and called `ensure_mcp_read_allowed_by_id`, `ensure_mcp_write_allowed_by_id` or `ensure_mcp_dangerous_write_allowed_by_id` before doing anything.

History rows written through MCP carry `source = mcp` and `mcp_tool_name`, with their own retention limit.

## Resolver

`resolve_mcp_server_command` returned `(program, args)` in this order:

1. A native installation next to the app.
2. A Node runtime plus the globally installed npm package `@dbx-app/mcp-server` (npm, pnpm and bun global directories were all probed).
3. A `mise` shim resolved through `mise which`.
4. A launcher found on `PATH` (`dbx-mcp-server` or `mcp-server`).

## Code

- Crate `crates/sqdesk-mcp/` — `server.rs` (tools), `backend.rs` (data access), `http.rs`, `http_auth.rs`, `session.rs`, `transaction.rs`, `runtime.rs` (flags), `credentials.rs`.
- App side: `src-tauri/src/commands/mcp.rs` (resolver), `mcp_bridge.rs`, `mcp_http_server.rs`, `mcp_native_installation.rs`.
- Still present: `crates/sqdesk-core/src/ai/mcp_policy.rs` and the MCP settings and history columns in storage.
