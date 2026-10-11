# Agent (JDBC) drivers

About half of the database types SQDesk lists have no native Rust driver. They run through an **agent**: a small Java program that wraps the vendor's JDBC driver and speaks JSON-RPC to the app. Oracle, DB2, SAP HANA, Snowflake, Trino, Hive, Cassandra, Neo4j, Dameng, Kingbase and the rest of the `runtimeMode: agent` descriptors in `plugins/connection-types/` work this way.

The agents are prebuilt and downloaded from DBX's release server on first use. That is why the upstream names in [README.md](README.md) matter here.

## Pieces

| Piece | Role |
| --- | --- |
| Catalog | Maps a database type (and driver profile) to an agent key |
| Registry | Remote JSON listing every agent and JRE with URLs, sizes and SHA-256 |
| Manager | Installs, upgrades and removes agents and JREs; owns the on-disk state |
| Runtime | Starts agent processes and keeps them alive |
| Driver client | JSON-RPC over the process's stdin and stdout |
| Offline export | Bundles installed agents into a zip for machines without internet |

## Registry

Fetched from `agents/agent-registry.json` on the R2 CDN, with the GitHub and cnb.cool release copies as fallbacks.

```
AgentRegistry { jre?, jres: { key -> JreInfo }, drivers: { key -> DriverInfo } }
JreInfo       { version, platforms: { platform -> ArtifactInfo } }
DriverInfo    { version, label, min_app_version, jar?, native: { platform -> ArtifactInfo }, jre }
ArtifactInfo  { url, sha256?, size, format?: "tar_zstd", delta? }
DeltaInfo     { base_version, url, sha256, size }
```

A driver is a `jar`, a `native` binary per platform, or both. `jre` names the JRE it needs (default key `21`; Java 21 is the minimum).

## Install

1. Take the operation lock for the agent key, so two installs of one driver cannot interleave.
2. Read the registry entry; refuse if `min_app_version` is newer than the app.
3. If the driver's JRE is not installed, install it first: download the platform archive, verify, extract (`tar.zst`, or `tar.gz` for older entries).
4. Download the driver artifact into the download cache (kept for 7 days), reporting progress at most every 500 ms.
5. Verify SHA-256 when the registry provides one.
6. Move the artifact into the driver directory and record `{ version, installed_at, jre }` in `state.json`.
7. Cancellation is checked between steps; a cancelled download returns "Agent download canceled by user."

**Delta updates.** If the registry entry has a `delta` and the artifact of `base_version` is still on disk, download the delta (a zstd patch-from frame), verify its hash, and reconstruct the new artifact from the old one. On any failure fall back to the full download.

**Upgrade all** runs up to 4 driver updates at once and refuses while a driver is in use.

On Windows a JRE in use cannot be deleted: it is renamed aside and cleaned up on the next start.

## Launch

```
<java> -Dfile.encoding=UTF-8 [--add-opens …] -XX:TieredStopAtLevel=1 -XX:+UseSerialGC -jar <agent.jar>
```

- `<java>` is the managed JRE, or a user-configured Java if it is new enough (probed with a 5-second timeout).
- Extra JVM options come from the environment variable `DBX_AGENT_JAVA_OPTS` (upstream spelling) and per-driver settings.
- The `--add-opens` flags are needed by drivers that reach into JDK internals.
- Native agents are launched directly.

## Protocol

JSON-RPC 2.0, one message per line: the app writes a request to the agent's stdin and reads the response from its stdout.

Handshake first. An agent reporting protocol version 2 with the `MultiSession` capability is **shared**: one process serves many connections, each with its own session id. Older agents get one process per connection.

Methods:

- Lifecycle: `Handshake`, `Connect`, `Disconnect`, `Shutdown`, `TestConnection`, `ValidateConnection`, `ConnectionInfo`
- Sessions: `OpenSession`, `CloseSession`, `ValidateSession`, `CancelSession`
- Metadata: `ListDatabases`, `ListSchemas`, `ListTables`, `ListObjects`, `ListDataTypes`, `GetColumns`, `ListIndexes`, `ListForeignKeys`, `ListTriggers`, `ListConstraints`, `ListPartitions`, `ListSubpartitions`, `GetTableDdl`, `GetObjectSource`, `GetCustomTypeDetails`, `GetExplainInfo`
- Queries: `ExecuteQuery`, `ExecuteQueryPage`, `FetchQueryPage`, `CloseQuerySession`, `ExecuteBatch`, `ExecuteTransaction`
- Streaming reads: `StartTableRead`, `FetchTableReadPage`, `CloseTableReadSession`
- Manual transactions: `BeginManualTransaction`, `CommitManualTransaction`, `RollbackManualTransaction`

**Errors.** An agent error is a message optionally followed by a marker line and a JSON object:

```
<message>
DBX_AGENT_ERROR_DATA:{"agentSessionId":…, "sqlState":…, …}
```

The app splits on the marker, turns the JSON into a structured `BackendError` (code family `JDBC`), and hides the marker from the user.

## Recovery

When a call fails because the process died, the decision depends on what was running:

| Scope | Behavior |
| --- | --- |
| `ConnectionOpen` | Restart the agent and retry |
| `ReadOnlyMetadata` | Restart and retry once |
| `Keepalive` | Restart quietly |
| `UserOperation` | Do not retry: a write may already have happened |

## Connection quirks handled here

- Oracle: alternate connect attempts (service name vs SID, thin vs OCI) chosen from the error text.
- OceanBase Oracle mode: adds `compatibleOjdbcVersion=8`.
- H2: file paths converted to and from JDBC URLs.
- MongoDB: retry with the legacy driver on specific handshake errors.
- ZooKeeper: minimum connection timeout of 15 seconds.

## On disk

```
<agent store>/
  state.json                 installed drivers, JRE versions, pending cleanup
  jre/<key>/                 extracted JRE
  drivers/<agent key>/       agent jar or native binary
  download-cache/            artifacts kept for delta updates
```

## Code

- `crates/sqdesk-driver-agent/src/`: `agent_catalog.rs`, `agent_manager.rs`, `agent_service.rs` (install, upgrade, registry), `agent_runtime.rs`, `agent_driver.rs` (client, launch, protocol), `agent_connection.rs` (connect parameters and quirks), `agent_recovery.rs`, `agent_offline_export.rs`, `driver_delta.rs`, `oracle_oci.rs`.
- **Not agent-specific, despite living in this crate:** `backend_error.rs` (the structured error type used by every driver) and `database_capabilities.rs`. Keep both if the crate is ever removed.
- App side: `src-tauri/src/commands/agents.rs`; UI in `components/config/` (Driver Manager).
- The JDBC plugin in `plugins/jdbc/` is a separate, older mechanism for user-supplied JDBC drivers.
