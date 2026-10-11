# Working on an item

How to carry a todo item from its file to `docs/todo/completed/`. Written for an AI agent, and just as valid for a person. The rules that must not be broken are in [RULE.md](RULE.md); read that first. Items live in [`docs/todo/`](docs/todo/README.md).

## Before starting

1. Read the item's file in `docs/todo/issues/` or `docs/todo/features/` to the end.
2. Read [RULE.md](RULE.md).
3. Read the code the item names. A plan written weeks ago may describe code that has since changed; trust the code.
4. If the item removes or rebuilds something described in [`docs/pending/`](docs/pending/README.md), read that document too.

## The project in one screen

| Part | Where |
| --- | --- |
| Frontend (Vue 3, TypeScript) | `apps/desktop/src` — `components/`, `stores/`, `lib/` |
| Bridge to the backend | `apps/desktop/src/lib/backend/api.ts` |
| Tauri app and commands | `src-tauri/src`, `src-tauri/src/commands/` |
| Core logic | `crates/sqdesk-core` |
| Drivers | `crates/sqdesk-driver-*`, `crates/sqdesk-drivers` |
| SQL analysis | `crates/sqdesk-sql-*` |
| AI providers | `crates/sqdesk-ai-provider`; prompts in `apps/desktop/src/lib/ai` |

The top-level `README.md` has the full structure and the request flow.

## Commands

```bash
make dev                      # run the app (Vite + Tauri, debug build)
pnpm exec vitest run <path>   # frontend tests for one area
cargo check -p sqdesk --tests # Rust compiles, including tests
```

Three that fail in a misleading way if run plainly:

```bash
# vue-tsc runs out of memory with the default heap
NODE_OPTIONS=--max-old-space-size=8192 pnpm typecheck

# core tests overflow the default thread stack
RUST_MIN_STACK=16777216 cargo test -p sqdesk-core --lib

# format only what was touched; the whole workspace is not rustfmt-clean
pnpm exec oxfmt <files>
rustfmt --edition 2021 <files>
```

## Doing the work

1. **Reproduce or measure first.** For a bug, make it happen. For a performance item, take the number the item quotes again. If it cannot be reproduced, write that in the file and stop: the item may be stale.
2. **Put logic where it can be tested.** Decisions (what a key does, how an answer is cleaned, which rows to show) belong in a plain module under `lib/`, with the component only wiring them up.
3. **Change the smallest thing that fixes it.** If the fix needs a larger change than the item describes, update the item before going further.
4. **Test at the level the change lives.** A rule gets a unit test. An interaction gets a component test with real events. A prompt change gets run against a real model, because a prompt has no other meaningful test.
5. **Look at the result.** Tests passing is not the same as the feature working: open the app, or take a screenshot, for anything visible.

## Things that waste time here

- **Hot reload sometimes fails** after several files are saved in a row ("Failed to reload …" in the dev log). The files are fine; reload the window.
- **The full frontend suite is flaky under load.** A handful of unrelated suites time out when everything runs at once or while Rust is compiling. Rerun the failing files alone before believing a failure.
- **There is no Git history in this checkout.** Nothing can be reverted with a command. See the backup rule.
- **`make dev` rebuilds and restarts the app on every Rust change.** Stop it during a long Rust refactor.
- **A debug build is not the app's real speed.** Do not judge performance from `make dev`; use `make package`.
- **Test fixtures that depend on string length** (base64 of a name, a byte offset, an identifier budget) break silently when a name changes. Search for them after any rename.

## Finishing

1. Run the tests for the area, the type check, and `cargo check` if Rust changed.
2. Update the item's file: status, what was built, where it differs from the plan, what is left to check.
3. Move the file to `docs/todo/completed/` and update the tables in [`docs/todo/README.md`](docs/todo/README.md).
4. Report plainly: what works and was verified, what was not verified, and anything found along the way that is not part of the item. A failure is reported with its output, not softened.
