# Cloud sync

Backed up and restored connections, saved SQL and settings through a remote file. Two transports: **WebDAV** and **code snippets** (GitHub Gist, Gitee, GitLab). Removed: the engine (`cloud_sync.rs`) and the import transaction it used in `storage.rs`.

## What a snapshot holds

`SyncSnapshot` (schema version 3; versions 1 and 2 still import):

| Field | Content |
| --- | --- |
| `schema_version`, `exported_at`, `app_version` | Metadata |
| `connections` | Connection configs with secrets removed |
| `tunnel_profiles` | Shared tunnel profiles, secrets removed (`None` = older snapshot, leave local ones alone) |
| `mqtt_subscriptions` | `None` = older snapshot, do not clear local subscriptions |
| `sidebar_layout`, `pinned_tree_node_ids` | Sidebar state |
| `saved_sql` | Saved SQL library (folders and files) |
| `desktop_settings`, `editor_settings` | Settings, minus device-local paths |
| `encrypted_secrets` | Optional `EncryptedSecretsBlob` |
| `selection` | What the user chose to export (`None` = everything) |

Device-local settings are never synced: `saved_sql_sync_dir`, `driver_store_dir`, `plugin_store_dir`, `agent_store_dir`, custom AI skill root.

## Export

1. Read connections, layout, saved SQL and settings from storage.
2. Apply the `SyncSelection`: per category, `None` means all, an explicit list means exactly those ids (an empty list omits the category).
3. Scrub secrets out of every connection. The scrubbed keys are `password`, `ssh_password`, `ssh_key_passphrase`, `proxy_password`, `redis_sentinel_password`, `connection_string`, `url_params`, `init_script`, the MQ / MQTT / Nacos / Cassandra credential keys, and anything under `ssh_tunnels.` or `transport_layers.`.
4. If the user set a sync passphrase and chose to include secrets: collect the scrubbed values into a `SensitiveSyncPayload` and encrypt it (below) into `encrypted_secrets`.
5. Serialize to JSON and upload.

## Encryption

- Key: Argon2id (version 0x13), memory 19 MiB, 2 iterations, parallelism 1, 32-byte output, 16-byte random salt.
- Cipher: AES-256-GCM with a 12-byte random nonce.
- Blob: `{ version, kdf: "argon2id", cipher: "aes-256-gcm", salt, nonce, ciphertext, payload_type?, aad? }`, all binary fields base64.
- Version 1 has no associated data. Version 2 authenticates a context string as AAD and repeats it in `payload_type`; the sync payload uses the context `<app>-sync-secrets`. Decryption rejects a v2 blob whose `kdf`, `cipher` or `payload_type` differ.

## WebDAV transport

- Default remote path `<App>/sync/snapshot.json`; normalized by dropping empty and `.` segments and resolving `..`.
- `test`: a request against the endpoint to check credentials.
- `put_snapshot`: `MKCOL` each parent collection in order (an existing collection is fine; `409 Conflict` is an error), then `PUT` the JSON.
- `get_snapshot`: `GET` and parse.
- Basic auth; the password can be stored in the local secret store, keyed by the config.

## Snippet transport

- Providers: GitHub (`api.github.com/gists`), Gitee (`gitee.com/api/v5/gists`), GitLab (`<instance>/api/v4/snippets`).
- One file, `<app>-sync.json`. The whole snapshot is encrypted, not just the secrets: `{ format: "dbx-encrypted-sync-snapshot", version: 1, payload: EncryptedSecretsBlob }`.
- `put_snapshot`: create the snippet if there is no id, otherwise update it; store the returned id locally per provider and instance.
- Legacy migration: an old plaintext snippet is replaced only when the user asks (`replace_legacy_snippet`), and the old one is deleted only after the encrypted one exists and only if it is unchanged. A failed delete is queued for retry.
- The token is stored in the local secret store.

## Import

1. Reject unknown schema versions; validate metadata.
2. Resolve the selection: the caller's restore selection, else the one stored in the snapshot.
3. Secrets: if restore was requested and the snapshot has `encrypted_secrets`, a passphrase is required; restoring without one is an error rather than a silent partial restore.
4. With a selection, merge the selected items into local data. Without one (legacy full snapshot), replace.
5. Write connections, tunnel profiles, saved SQL, layout and settings, then re-attach decrypted secrets to their connections.
6. Return `ApplySnapshotSummary` with counts.

## Code

- `crates/sqdesk-core/src/persistence/cloud_sync.rs` (everything above; about 5,500 lines, half of it tests).
- Storage side in `persistence/storage.rs`, also removed: `SyncImportPlan`, `SyncImportCredential`, `SyncImportSecret` and `apply_sync_import_transaction`, which wrote connections, tunnel profiles, AI configs, settings and saved SQL in one transaction so a failed import left nothing half-applied.
- `src-tauri/src/commands/cloud_sync.rs` is **not** cloud sync despite its name: it holds the secret-store migration commands and is still in use.
