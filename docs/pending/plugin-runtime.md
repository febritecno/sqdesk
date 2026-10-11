# Plugin runtime and marketplace

Lets third-party packages add connection types, UI panels and AI tools. A plugin is a signed archive with a manifest, an optional backend process and an optional web UI. The plugin center UI was removed; the runtime is in `sqdesk-plugin-runtime`.

## Package

A zip with the extension `.dbxp` (upstream) containing:

| File | Purpose |
| --- | --- |
| `manifest.json` | Identity, permissions, entry points, contributions |
| `checksums.json` | SHA-256 of every file |
| `signature.json` | Ed25519 signature over `checksums.json`, with a key id |

Limits: 512 MiB package, 1 GiB uncompressed, 256 MiB per file, 10,000 entries.

## Manifest (version 1)

`id`, `name`, `version` (semver), `publisher`, `description`, `icon`, `engines` (required app version and host API version), `permissions`, `entrypoints { backend?, ui? }`, `contributions`, `localizations`.

- `backend`: `executable`, `transport` (`stdio-jsonl` or `stdio-framed`), `protocol_versions`.
- `ui`: `entry` (an HTML file) and optional `root`.
- Permissions: `host.events`, `host.binary`, `host.workbench`, `host.filesystem`, `host.plans:read`, `host.schema:read`, `host.storage`, `host.ai`, `host.clipboard:read`, `host.data:read`, and up to 8 `host.network:<origin>` entries.

Unknown top-level fields are kept but ignored. Manifest version 0 (the old JDBC plugin shape with `executable` and `drivers`) is still readable.

## Install

1. Take the install lock.
2. Extract into a temporary directory, enforcing the limits and rejecting paths that escape it.
3. Verify every file against `checksums.json`.
4. Verify the signature against the trust store. Policy decides what an unsigned or unknown-key package means: marketplace installs require a trusted key; local development installs do not.
5. Parse the manifest; require manifest version 1 or newer, a valid id and a semver version.
6. Check compatibility with the running app and host API.
7. If the caller expected a specific plugin, version or key (marketplace install), check the package matches.
8. **Update continuity:** if a previous install recorded its repository, publisher and signing key, an update that changes any of them, or that downgrades, is refused unless the user confirmed.
9. Block if the plugin has operations in flight.
10. Move the directory to `versions/<version>/`, keeping the replaced version as the rollback target.
11. Write an activation record under `activations/`. The newest record defines the active version.
12. Prune old versions and records.

Layout:

```
<plugin store>/<plugin id>/
  versions/<version>/      extracted package
  activations/             activation records (newest wins)
<plugin store>/.trust/keys.json
```

**Rollback** activates the previous version if its directory is still valid. **Uninstall** stops the backend, renames the container aside, then deletes it, retrying on Windows file locks.

## Marketplace

- A repository is a URL serving a catalog (`index.json`, at most 4 MiB, catalog version 1) signed by a repository-level key.
- The official repository's catalog comes from the R2 CDN with a GitHub raw fallback; its keys are built in.
- Catalog entry: plugin metadata, localizations, and versions each with artifacts per target (`universal` or a platform), URL, size and hash.
- Install from the marketplace = download the artifact, then the install flow above with the expectation set from the catalog.

## Backend process

Started on demand, one per plugin, spoken to over stdin and stdout.

- `stdio-jsonl`: one JSON message per line.
- `stdio-framed`: 1 byte of kind (0 = JSON, 1 = binary), a big-endian `u32` length, then the payload. JSON is capped at 8 MiB and binary at 64 MiB, so large results travel as binary frames. An unknown kind ends the session.

Both directions share one stream. The host uses numeric request ids; the plugin uses string ids for calls it makes back into the host. That is how responses are told apart.

Host → plugin methods include `connection/test`, `connection/connect`, `connection/disconnect`, `connection/action`, plus whatever the plugin's contributions declare. Requests time out after 30 seconds.

Plugin → host methods live under `host/`. `host/requestUserInput` shows a prompt (for example an MFA code): at most 4 open per session, 5 seconds to 10 minutes each. While a prompt is open the deadline of the request that caused it is paused, so a user reading an MFA challenge does not time out the connect.

Stopping a plugin kills the whole process tree, not just the launcher, because script launchers leave children that keep files locked.

## UI

Plugin pages are served by a custom protocol (`<app>-plugin://localhost/<id>/…`) from the plugin's own directory, under a Content-Security-Policy built from its permissions and declared network origins. Pages talk to the host through `postMessage`.

## AI tools

In agent mode the assistant discovers tools that enabled plugins declare and offers them to the model next to the built-in database tools. Each plugin can be switched on or off for AI use, and data access is granted per connection.

## Code

- `crates/sqdesk-plugin-runtime/src/plugins/`: `manifest.rs`, `installer.rs`, `marketplace.rs`, `runtime.rs` (process and protocol), `host.rs` (sessions, connection handles), `lifecycle.rs` (update guards), `filesystem.rs`, `assets.rs`, `jdbc_sessions.rs`.
- App side: `src-tauri/src/commands/plugins.rs`, `plugin_storage.rs`, `plugin_media.rs`, `plugin_download.rs`, `plugin_download_file.rs`, `src-tauri/src/plugin_ui_protocol.rs`, `plugin_file/`.
- AI integration: `crates/sqdesk-core/src/ai/plugin_tools.rs`.
- Frontend: `components/plugins/`, `lib/plugins/`.
- Schema: `plugins/manifest.schema.json`.
