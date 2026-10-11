# Issues to optimize

Problems and optimizations that have been investigated but not done. One file per item, with the findings needed to pick it up cold. How items move through these folders is described in [../README.md](../README.md).

| Item | File | Size |
| --- | --- | --- |
| Reduce MySQL connections held per app | [mysql-connection-pool.md](mysql-connection-pool.md) | Medium; touches the core connection layer |

## Smaller open items

Known, not yet written up in detail:

- **Core test `docs_export_bundle_is_current` fails.** It asks for `pnpm build:docs-export`, a script this fork does not have. Either restore the script or remove the test and the bundle it guards.
- **Redis pub/sub server always starts**, listening on `127.0.0.1:4224` even when no Redis connection exists. Start it on first use.
- **Two dead toggles in Settings → Appearance.** "Data Transfer" and "Driver Manager" toolbar visibility no longer do anything now that both live permanently under More.
- **Driver Manager: installing an agent driver is untested end to end.** Checked so far: the registry answers (55 drivers), the artifact and JRE URLs it lists return 200, and a macOS arm64 JRE exists. The `min_app_version` check (`ensure_driver_app_version`, all 55 drivers ask for 0.6.40, SQDesk is 0.6.34) is defined but never called, so it blocks nothing. Not yet shown: that a driver downloads, unpacks and connects after the `dbx` → `sqdesk` rename (the agent error marker and the `DBX_AGENT_*` variables the prebuilt jars use were renamed in the app). See `docs/pending/driver-agents.md`.
- **Inline AI cannot be cancelled on the backend.** Closing the panel ignores the answer, but the provider request runs to completion. Related to [inline-ai.md](../completed/inline-ai.md), but a backend change.
- **New UI strings exist only in English and Indonesian.** Other locales fall back to English for them, and still show the old labels for the renamed AI Provider tab.
- **`cargo fmt` reports differences across the workspace**, a side effect of the rename changing identifier lengths. One `cargo fmt` run would clear it, at the cost of a large mechanical diff.
