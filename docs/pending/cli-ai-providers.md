# CLI-based AI providers

Let the AI assistant run through a locally installed coding agent instead of an HTTP API: Claude Code, Codex, Cursor, OpenCode, Grok, Qoder, CodeBuddy, Pi.

Status: removed. The provider list hides them and every backend entry point answers with `CLI_PROVIDER_UNSUPPORTED`. The `AiProvider` enum keeps its CLI variants and `AiConfig` keeps the per-CLI path and environment fields, so configurations saved earlier still deserialize. They depended on the MCP server, which is also gone; see [mcp-server.md](mcp-server.md).

## Why they need MCP

An API provider gets database tools from the app in-process. A CLI agent is a separate program: the only way to give it database access is to hand it an MCP server. Without one it can chat but cannot read a schema or run a query.

## Algorithm

1. **Resolve the program.** Use the configured path, else the default name (`claude`, `codex`, `agent`, `opencode`, `grok`, `qodercli`, `codebuddy`, `pi`). A bare name is searched on `PATH` and in common install directories, because a GUI app does not inherit the shell's `PATH`.
2. **Resolve the MCP server command** (`program`, `args`).
3. **Build the scope environment** (`sqdesk_mcp_scope_env`): connection id and name, database, schema, `ALLOW_WRITES`, `ALLOW_DANGEROUS_SQL`, and the confirmed write SQL if any. With several selected databases the database and schema scopes are sent empty, which lifts the single-database bound.
4. **Choose the enabled tools** (`sqdesk_mcp_enabled_tools`): always `list_connections`, `list_tables`, `describe_table`, `get_schema_context`; in agent mode also `execute_query`, `execute_redis_command`, `salesforce_current_user`. The Salesforce write pair is deliberately left out: the agents run without an interactive permission prompt and a Salesforce write cannot be rolled back.
5. **Build the prompt** from the system prompt and the message history as one text; the CLI keeps no session between runs.
6. **Spawn the CLI** in non-interactive, streaming-JSON mode with the MCP config and an allowlist of exactly the enabled tools.
7. **Read stdout line by line** and translate each JSON line into `AgentEvent`s for the UI: text delta, reasoning delta, tool call start, tool result, end (with token usage), error.
8. **Cancel** by killing the process; **classify failures** from stderr into user-readable messages (not installed, not logged in, command line too long, rate limited).

### Claude Code invocation

```
claude --print --output-format stream-json --verbose --input-format text
       --no-session-persistence --permission-mode dontAsk
       --mcp-config '{"mcpServers":{"sqdesk":{"command":…,"args":[…],"env":{…}}}}'
       --strict-mcp-config
       --tools <tools…> --allowedTools <tools…>
       [--model <model>] [--effort <level>]
```

Tools are named `mcp__sqdesk__<tool>`. The prompt goes in on stdin.

### Output dialects

Each CLI has its own JSON-lines shape, selected by `CliAgentJsonlDialect`: `CodexExec`, `ClaudeCodePrint`, `CodeBuddyPrint`, `QoderPrint`, `OpenCodeRun`, `CursorPrint`, `GrokStreamingJson`. `parse_cli_jsonl_event` maps a line to events; lines it does not recognize are ignored.

### Environment rules

Users can add environment variables per provider. Names matching the app's MCP control variables are rejected, so a user config cannot widen the scope the app set.

### Models and effort

Model lists come from the CLI itself where it can print them as JSON (`list_json_models_or_default`), otherwise from a built-in default. Effort levels map to the CLI's own flag.

## Code

All of the Rust modules below were deleted; take them from upstream.

- `crates/sqdesk-ai-provider/src/ai_cli_agent.rs` — shared options, scope env, tool list, JSONL parsing.
- One module per CLI: `ai_claude_code_cli.rs`, `ai_codex_cli.rs`, `ai_cursor_cli.rs`, `ai_opencode_cli.rs`, `ai_grok_cli.rs`, `ai_qoder_cli.rs`, `ai_codebuddy_cli.rs`, `ai_pi_agent_cli.rs`.
- Dispatch: `run_agent_loop` in `crates/sqdesk-core/src/ai/agent_loop.rs` (the `is_cli_provider` branch).
- Entry: `ai_agent_stream` in `src-tauri/src/commands/ai.rs`.
- Frontend: `CLI_AI_PROVIDERS` in `EditorSettingsDialog.vue`, `lib/ai/aiConfigCandidates.ts`, presets in `stores/settingsStore.ts`.

## To bring them back

1. Restore the MCP server and the resolver.
2. Restore the nine modules and their `pub mod` lines in `sqdesk-ai-provider/src/lib.rs`, and the re-exports in `sqdesk-core/src/lib.rs`.
3. In `ai.rs`, put the per-provider arms back in `list_models_core`, `resolve_model_effort_core` and `test_cli_provider_connection`.
4. In `run_agent_loop`, replace the early `CLI_PROVIDER_UNSUPPORTED` return with the per-provider dispatch, and add `cli_mcp_server_command` back to `AgentLoopContext`.
5. In `ai_agent_stream`, replace `reject_cli_provider` with the resolver call.
6. Remove the CLI filter from `builtinAiProviderOptions`.
