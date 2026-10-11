# SQDesk

A minimal desktop SQL client with an AI assistant.
- [Download](#download)
- [Stack](#stack)
- [Requirements](#requirements)
- [Set up and run](#set-up-and-run)
- [Build and package](#build-and-package)
- [Checks](#checks)
- [Project structure](#project-structure)
- [Architecture](#architecture)

## Download

Installers for macOS (Apple Silicon and Intel), Windows and Linux are attached to each release:

**[Download the latest release](https://github.com/febritecno/sqdesk/releases/latest)** · [all releases](https://github.com/febritecno/sqdesk/releases)

| OS      | File                                                    |
| ------- | ------------------------------------------------------- |
| macOS   | `SQDesk_<version>_aarch64.dmg` (Apple Silicon) or `SQDesk_<version>_x64.dmg` (Intel) |
| Windows | `SQDesk_<version>_x64-setup.exe`                        |
| Linux   | `.AppImage`, `.deb` or `.rpm`                           |

### macOS: "SQDesk.app is damaged" or "cannot be verified"

The builds are ad-hoc signed, not notarized by Apple, so Gatekeeper blocks them after download. Either open **System Settings → Privacy & Security** and choose **Open Anyway**, or clear the quarantine flag:

```bash
xattr -cr /Applications/SQDesk.app
```

## Stack

| Layer    | Technology                                                             |
| -------- | ---------------------------------------------------------------------- |
| Shell    | Tauri 2 (`src-tauri`), system webview                                  |
| Backend  | Rust workspace, 23 library crates under `crates/sqdesk-*`              |
| Frontend | Vue 3 + TypeScript + Pinia, Vite 8, Tailwind CSS 4 (`apps/desktop`)    |
| Editor   | CodeMirror 6                                                           |
| Tooling  | pnpm workspace, Vitest, vue-tsc, oxlint, oxfmt, Make                   |

## Requirements

| Tool    | Version    | Notes                                         |
| ------- | ---------- | --------------------------------------------- |
| Node.js | >= 22.13.0 | `.nvmrc` pins the version                     |
| pnpm    | 10.27.0    | Declared in `packageManager`; use Corepack    |
| Rust    | >= 1.88    | Install with [rustup](https://rustup.rs)      |
| Make    | any        | Entry point for every dev task                |

You also need the Tauri 2 system dependencies for your OS (Xcode Command Line Tools on macOS, WebView2 on Windows, WebKitGTK and friends on Linux). Follow the [Tauri prerequisites guide](https://tauri.app/start/prerequisites/).

## Set up and run

```bash
nvm use                 # or any Node >= 22.13.0
corepack enable         # provides the pinned pnpm
make                    # install dependencies and start the desktop app
```

`make` is the same as `make dev`. `make help` lists every target.

| Command         | What it does                                                         |
| --------------- | -------------------------------------------------------------------- |
| `make` / `make dev` | Install dependencies if needed, then start the app in development mode |
| `make dev-fast` | Same, with a smaller feature set (no DuckDB sidecar) for faster builds |
| `make install`  | `pnpm install --frozen-lockfile` only                                |
| `pnpm dev`      | Vite dev server only, without the Tauri shell                        |

### What `make dev` does

1. Runs `pnpm install --frozen-lockfile` when `package.json` or `pnpm-lock.yaml` changed.
2. Checks that port **1420** is free and stops with a message if it is not.
3. Runs `pnpm dev:tauri` (`scripts/dev-tauri.mjs`), which starts `tauri dev`.
4. Tauri runs `pnpm dev` first, which serves the frontend at `http://localhost:1420`.
5. Cargo compiles the Rust workspace and launches the `sqdesk` debug binary.

The first Rust build compiles about a thousand crates and takes several minutes. Later builds are incremental. The dev profile uses `debug = "line-tables-only"` (see `Cargo.toml`), which keeps an incremental rebuild at around 20 seconds instead of about 3 minutes; set `debug = true` under `[profile.dev]` locally when you need to inspect variables in a debugger. Switching between `make dev` and `make dev-fast` changes the feature set and forces a recompile, so stick to one. The frontend hot-reloads through Vite, and Tauri rebuilds the binary when Rust sources under `src-tauri` or `crates/` change.

### macOS signing prompt

On macOS the dev entry point signs the debug binary with a local-only identity stored under `~/Library/Application Support/SQDesk/development-signing/`. Pick **Always Allow** on the first Keychain prompt. Without that signature the keychain entry holding the secret-store key is unreadable and the migration wizard reappears on every launch.

Release builds must go through `tauri build`; the dev signer refuses `--release`.

### Cargo features

`src-tauri` builds with these features by default:

| Feature                   | Purpose                                             |
| ------------------------- | --------------------------------------------------- |
| `duckdb-sidecar`          | DuckDB support through a worker process             |
| `dynamodb`                | DynamoDB driver                                     |
| `mq-admin`                | Message queue administration                        |
| `sqlite-multiple-ciphers` | Encrypted SQLite databases                          |
| `system-fonts`            | Lists installed fonts for the editor settings       |
| `os-keyring`              | Stores the secret-store key in the OS keychain      |

`make dev-fast` builds with `dynamodb,sqlite-bundled,os-keyring` only. Pass your own set with:

```bash
pnpm dev:tauri -- --no-default-features --features dynamodb,sqlite-bundled,os-keyring
```

Keep `os-keyring` in any custom set, for the reason given above.

### App data

Connections, history, saved SQL and settings live in a SQLite database in the app data directory, for example `~/Library/Application Support/com.sqdesk.app/` on macOS. Set `SQDESK_DATA_DIR` to use a different directory.

## Build and package

```bash
make build     # type-check, then build the frontend into dist/
make package   # build the desktop bundle with `tauri build`
make clean     # remove Rust build artifacts
```

`make package` runs the frontend build first and writes the installers under `target/release/bundle/`.

### Release

Pushing a tag that matches `v*` runs `.github/workflows/release.yml`, which builds the installers for every platform and attaches them to a draft GitHub release. Keep the tag in step with the app version in `package.json`, `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`, then publish the draft:

```bash
git tag v0.6.40
git push origin v0.6.40
```

## Checks

```bash
make check        # format check, lint, type-check and tests in one run
make test         # vitest
make cargo-check  # cargo check -p sqdesk
```

Individual steps:

```bash
pnpm typecheck    # vue-tsc
pnpm lint         # oxlint
pnpm fmt          # oxfmt (writes changes)
pnpm test         # vitest run
pnpm test:watch   # vitest in watch mode
```

`vue-tsc` can run out of memory on this codebase. If `pnpm typecheck` crashes, raise the heap:

```bash
NODE_OPTIONS=--max-old-space-size=8192 pnpm typecheck
```

## Project structure

```text
sqdesk/
├── apps/desktop/           Vue 3 frontend
│   ├── index.html
│   ├── vite.config.ts
│   └── src/
│       ├── main.ts, App.vue, StartupGate.vue
│       ├── components/     UI by feature: layout, sidebar, editor, grid, ai, connection, ...
│       ├── stores/         Pinia stores: connection, query, settings, history, savedSql, ...
│       ├── lib/            Non-UI logic by domain (see src/lib/README.md)
│       ├── composables/    Shared Vue composables
│       ├── i18n/           Locale files; English is the fallback
│       ├── types/          Shared types; types/generated is produced by a script
│       └── styles/         Global CSS
├── src-tauri/              Tauri application crate (`sqdesk`)
│   ├── tauri.conf.json     Window, bundle and dev-server configuration
│   ├── capabilities/       Tauri permission sets
│   └── src/
│       ├── lib.rs          App setup, plugin registration, command handler list
│       └── commands/       One module per Tauri command group
├── crates/                 Rust library crates (see Architecture)
├── packages/
│   ├── app-tests/          Vitest suites that span several frontend modules
│   └── mongo-shell/        MongoDB shell syntax support used by the frontend
├── plugins/
│   ├── connection-types/   YAML descriptor per database type (source of truth)
│   ├── dialects/           YAML SQL dialect definitions
│   ├── jdbc/               JDBC agent (Gradle project) for agent-based drivers
│   └── mappings/           Type mapping data
├── scripts/                Dev runner, macOS dev signing, check runner, type generator
├── tests/fixtures/         Fixtures shared by Rust and frontend tests
├── vendor/                 Patched upstream crates (wry, tiberius, rumqttc, ...)
├── Cargo.toml              Rust workspace and crates.io patches
├── package.json            Frontend dependencies and scripts
└── Makefile                Development entry points
```

Tests sit next to the code as `*.spec.ts` under `apps/desktop/src`, plus the cross-module suites in `packages/app-tests`.

## Architecture

SQDesk is a Tauri app: a Vue frontend in the system webview talks to a Rust backend in the same process through Tauri commands.

```text
┌──────────────────────────── Webview (apps/desktop) ────────────────────────────┐
│  components  →  stores (Pinia)  →  lib/backend (invoke bridge)                 │
└───────────────────────────────────────┬────────────────────────────────────────┘
                                        │ Tauri IPC (commands + events)
┌───────────────────────────────────────▼────────────────────────────────────────┐
│  src-tauri        command handlers, window and app lifecycle                   │
│  sqdesk-core      connections, query execution, persistence, safety, AI        │
│  sqdesk-drivers   driver registry, tunnels  →  sqdesk-driver-*                 │
│  sqdesk-sql-*     SQL analysis, dialects, generated SQL                        │
└───────────────────────────────────────┬────────────────────────────────────────┘
                                        │
                     databases, SSH/proxy tunnels, AI providers
```

### Frontend

- **Components** render the UI and hold view state only. They are grouped by feature under `components/`.
- **Stores** hold application state. `connectionStore` owns connections and the sidebar tree, `queryStore` owns tabs and results, `settingsStore` owns user settings.
- **`lib/backend`** is the bridge to Tauri. `api.ts` wraps the backend commands, and components and stores call those wrappers instead of `invoke`. A few window-level helpers outside it still call Tauri directly.
- **`lib/*`** holds the rest of the non-UI logic, organised by domain. `apps/desktop/src/lib/README.md` describes the layout and its rules.

### Backend

`src-tauri` is a thin layer. `src/lib.rs` configures the app and registers every command; each module in `src/commands/` exposes one command group (`connection`, `query`, `ai`, `transfer`, ...) and delegates to the library crates.

| Crate                         | Responsibility                                                         |
| ----------------------------- | ---------------------------------------------------------------------- |
| `sqdesk-core`                 | Connection management, query execution, persistence, safety checks, AI agent loop |
| `sqdesk-types`                | Shared data models                                                     |
| `sqdesk-drivers`              | Driver registry, SSH and proxy tunnels, smaller built-in drivers       |
| `sqdesk-driver-*`             | One crate per larger driver: MySQL family, PostgreSQL, SQL Server, MongoDB, Redis, Elasticsearch |
| `sqdesk-driver-agent`         | Runs agent-based drivers (JDBC) as separate processes                  |
| `sqdesk-driver-support`       | Helpers shared by drivers: execution, SSH config, tunnels              |
| `sqdesk-sql-core`             | SQL analysis, risk detection, diagnostics, editability                 |
| `sqdesk-sql-dialect`          | Dialect capabilities and DML binding                                   |
| `sqdesk-sql-data`             | SQL for the data grid and database search                              |
| `sqdesk-sql-schema`           | Table structure SQL, schema diff, admin SQL                            |
| `sqdesk-sql`                  | Facade over the `sqdesk-sql-*` crates                                  |
| `sqdesk-formats`              | CSV, XLSX and text export                                              |
| `sqdesk-ai-provider`          | AI provider clients (API providers; requests go straight to the provider) |
| `sqdesk-platform`             | Paths, processes, proxy and download helpers                           |
| `sqdesk-plugin-runtime`       | Plugin loading and runtime                                             |
| `sqdesk-sqlite-worker`        | Short-lived worker process for SQLite files reached over SSH           |
| `sqdesk-tauri-consul`, `sqdesk-tauri-schema` | Tauri command sets kept outside `src-tauri`             |

### Request flow

Running a query goes through these steps:

1. `QueryEditor` asks `queryStore` to execute the statement.
2. `queryStore` calls the wrapper in `lib/backend/api.ts`, which invokes a Tauri command.
3. `src-tauri/src/commands/query.rs` receives it and calls `sqdesk-core`.
4. `sqdesk-core` checks the statement with `sqdesk-sql-core`, picks the driver for the connection, and executes.
5. The result returns over IPC; `queryStore` stores it and the grid renders it.

### Connection types

Database types are declared once, in `plugins/connection-types/*.yaml`. Both sides generate code from those files:

- `scripts/sync-connection-types.mjs` writes `apps/desktop/src/types/generated/*.ts` and `crates/sqdesk-core/assets/database-drivers.manifest.json`.
- `crates/sqdesk-core/build.rs` generates the Rust `DatabaseType` enum.

The generator runs automatically before `pnpm dev`, `pnpm typecheck` and `pnpm test`. Run `pnpm generate:connection-types` to refresh by hand, and never edit the generated files.

### Vendored crates

`vendor/` holds patched copies of upstream crates, wired in through `[patch.crates-io]` in `Cargo.toml`. Each patch is explained in a comment there: Windows 7 compatibility for the webview stack, MQTT 3.1 support, and SQL Server result decoding.

## License

Apache-2.0, same as DBX. See `LICENSE`.
