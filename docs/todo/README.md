# Todo

Planned work for SQDesk, one Markdown file per item. A file is written so that someone with no memory of the discussion can pick the item up: what is wrong or wanted, what is already known, and how to tell when it is done.

## Folders

| Folder | Holds |
| --- | --- |
| [`issues/`](issues/) | Something that is wrong or wasteful: bugs, regressions, performance problems |
| [`features/`](features/) | Something new: a capability or a change in how the app behaves |
| [`completed/`](completed/) | Items that are done, kept for the reasoning and for what was left unchecked |

Related: [`../pending/`](../pending/) describes features that were removed from the app and how they worked, in case one is rebuilt.

Before working on an item, read [`AGENT.md`](../../AGENT.md) (how to carry an item through) and [`RULE.md`](../../RULE.md) (what must not happen), both at the repository root.

## Workflow

1. **Write the item.** Create `issues/<name>.md` or `features/<name>.md`. Use a short lowercase name with hyphens, for example `mysql-connection-pool.md`.
2. **Investigate before planning.** Record what was measured or read in the code, with file paths, so the plan rests on facts rather than guesses.
3. **Do the work.** Keep the file current: if the result differs from the plan, say where and why.
4. **Close it.** When the work is done and tested, set the status, add what is still unverified, and **move the file to `completed/`**. Do not delete it.
5. **Keep the indexes right.** Add the item to the table below when it is created, and move its row when it is completed.

An item that turns out not to be worth doing is also moved to `completed/`, with the status `dropped` and the reason.

## What a file contains

```markdown
# <Title: the outcome, not the activity>

**Type:** issue | feature · **Status:** not started | in progress | implemented | dropped

## Problem            (issue)   or   ## Wanted   (feature)
What is wrong or missing, in terms a user would recognize. Measurements if there are any.

## Cause / Today
What the code does now, with file paths.

## Options / Design notes
The approaches considered and the one recommended. Decisions that still need to be made.

## Things to watch
What could break, and what must not change.

## Done when
Checkable conditions. "It works" is not one.

## How to verify
Commands, queries or steps that prove it.
```

On completion, add at the top:

- **What was built**, in a few lines.
- **Where the result differs from the plan.**
- **Left to check**: anything not verified, especially what needs a person at the app.

## Current items

### Open

| Item | Type | File |
| --- | --- | --- |
| Reduce MySQL connections held per app | Issue | [issues/mysql-connection-pool.md](issues/mysql-connection-pool.md) |

Smaller items that do not have a file yet are listed in [issues/README.md](issues/README.md).

### Completed

| Item | Type | Completed | File |
| --- | --- | --- | --- |
| Inline AI: options preview with refinement, movable panel, reliable Enter and Esc | Feature | 2026-10-11 | [completed/inline-ai.md](completed/inline-ai.md) |
