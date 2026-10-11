# Documentation

Working documents for SQDesk. User-facing setup, structure and architecture are in the [top-level README](../README.md); this folder is for the people and agents changing the code.

| Folder | What it is | Start with |
| --- | --- | --- |
| [`todo/`](todo/) | Planned and finished work, one Markdown file per item | [todo/README.md](todo/README.md) |
| [`pending/`](pending/) | How features removed from the app used to work, in case one is rebuilt | [pending/README.md](pending/README.md) |

Two files at the repository root go with these:

| File | What it is |
| --- | --- |
| [`RULE.md`](../RULE.md) | Constraints that hold for every change, and the development concept in short |
| [`AGENT.md`](../AGENT.md) | How to carry one item from its file to done: commands, order of work, known time-wasters |

## How work moves

```text
idea or problem
      │  write it down, with what is known
      ▼
todo/issues/<name>.md        something wrong or wasteful
todo/features/<name>.md      something new
      │  investigate → do → test → look at the result
      ▼
todo/completed/<name>.md     updated with what was built and what is still unchecked
```

A file is never deleted when its work is done: it is moved, so the reasoning stays available.

Removing a feature is the one case with an extra step. Its algorithm is written into `pending/` first, while the code still exists to read.

## Reading order for someone new

1. [`../README.md`](../README.md) — what the app is, how to run it, how it is built.
2. [`../RULE.md`](../RULE.md) — what must not happen.
3. [`todo/README.md`](todo/README.md) — the workflow and the list of open items.
4. [`../AGENT.md`](../AGENT.md) — before starting on an item.
