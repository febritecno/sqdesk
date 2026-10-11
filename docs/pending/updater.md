# Update checker

Checks whether a newer release exists and returns its notes. The in-app installer was removed earlier; what remained only *checked*.

## Algorithm

1. Build the candidate URL list from the user's download source:
   - `Official`: R2 CDN `releases/latest/latest.json`, then GitHub `releases/latest/download/latest.json`.
   - `Cnb`: the cnb.cool mirror first, then R2, then GitHub.
2. Request each candidate in order with `User-Agent: <app>-update-checker` and `Accept-Encoding: identity`. The first 2xx wins; every failure is collected and reported together if all fail.
3. Parse the body as `TauriRelease { version, notes?, jdbc_plugin? { version, protocol_version, url } }`.
4. Best effort: fetch `https://api.github.com/repos/<owner>/<repo>/releases/tags/v<version>` for `name`, `html_url`, `body`.
5. For non-Chinese locales, best effort: fetch `changelog/latest-en.json` from R2 and use its `notes` only if its `version` equals the release version after normalization.
6. Compare with the running version using `is_newer_version` (semantic comparison after stripping a leading `v`).
7. Return `UpdateInfo { current_version, latest_version, update_available, portable_mode, manual_update_only, release_name, release_url, release_notes }`.

Release notes preference: English notes (step 5), then `notes` from `latest.json`, then the GitHub release body.

## Contracts

- `latest.json` is the Tauri updater manifest published with each release.
- Hosts: `dl.dbxio.com` (R2), `github.com/t8y2/dbx`, `cnb.cool/dbxio.com/dbx`.

## Code

- `crates/sqdesk-core/src/host/update.rs` (check logic), `host/changelog.rs` (changelog fetch).
- `crates/sqdesk-platform/src/version.rs` (`is_newer_version`, `normalize_version`, `parse_version`) and `download.rs` (`R2_CDN_BASE`, `DownloadSource`, `race_download`).
- Frontend: update events `sqdesk:update-release`, `sqdesk:update-prepare`, `sqdesk:update-prepared`; strings under `settings.*Update*`.
