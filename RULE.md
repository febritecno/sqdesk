# Rules

Constraints that hold for every change in this repository. Each one exists because breaking it lost data, broke a working feature, or wasted a day. [AGENT.md](AGENT.md) describes how to work; this file says what must not happen.

## How development is organized

Work in this repository is planned and tracked in [`docs/todo/`](docs/todo/README.md). The concept is simple and it is the first thing to know:

- Every piece of work is **one Markdown file**: a problem goes in `docs/todo/issues/`, something new in `docs/todo/features/`.
- The file is written **before** the work, with what was measured or read in the code, the plan, and how to tell it is done.
- When the work is finished and tested, the file is updated with what was built and what is still unchecked, then **moved to `docs/todo/completed/`**. It is never deleted.
- Features that were removed from the app are described in [`docs/pending/`](docs/pending/README.md), so one can be rebuilt without rediscovering how it worked.

The full workflow and the file template are in [`docs/todo/README.md`](docs/todo/README.md). The rules below apply to every item.

## Data and names

1. **Never rename an on-disk or wire format.** These keep their original spelling even though the product is called SQDesk:

   | Name | What depends on it |
   | --- | --- |
   | `dbxenc1`, `dbx-local-secret-store-v1`, `dbx-secret-v1`, `dbx.global` | Decrypting saved passwords and API keys |
   | `dbx-config`, `dbx-encrypted` | Importing connection export files |
   | `DBXEI` | Recognizing an earlier export's destination |
   | `…/dbx/secret.key` | Finding an existing key file |

   Changing one makes existing data unreadable with no error at build time.

2. **Names shared with something outside this repository belong to the other side.** Agent artifact names, plugin package formats, npm package names and HTTP headers are listed in [`docs/pending/README.md`](docs/pending/README.md). Rename one only together with whatever is on the other end.

3. **A change to where or how data is stored ships with a migration** that reads the old form, and a test for it. Renaming the database file needed one; so will the next such change.

4. **Do not edit the user's app data** (`~/Library/Application Support/com.sqdesk.app/`) unless asked, and copy it first when you do.

5. **Back up before a bulk or destructive change.** There is no Git history here. Archive the source tree before a rename, a mass edit or a deletion that spans many files.

## Behavior

6. **Measure before optimizing, and again after.** A change justified by speed needs a number from before and a number from after. If the number does not move, the change is not kept.

7. **Do not remove a feature that other code is wired into** because it looks unused. Count the references first. Removing a feature starts with a document in `docs/pending/` describing how it worked.

8. **A guard that protects the user's data is enforced in code, not only in a prompt.** Models ignore instructions. The inline AI's rules against unrequested comments, unrequested write statements and truncated SQL are checked after the model answers.

9. **Never write to the editor at a remembered position without checking the document is unchanged.** Offsets go stale the moment the user types.

10. **Inline AI stays strict; chat stays free.** The inline panel returns SQL only. The chat assistant may explain, offer alternatives and ask. Their prompts live in separate files (`lib/ai/inlineSql.ts` and `lib/ai/ai.ts`) and must stay separate.

11. **AI writes need explicit confirmation**, and production-marked databases refuse them. Do not add a path around either.

## Code

12. **Match the code around the change**: its naming, its comment density, its structure. A comment says why, not what.

13. **Every user-visible string goes through i18n**, with English and Indonesian at minimum. English is the fallback for the other locales.

14. **The product name in user-visible text is "SQDesk".** Credits to the upstream project keep the name DBX.

15. **Behavior that changes gets a test that would have failed before.** Remove a test only when the behavior it checked was removed on purpose, and say so.

16. **Secrets never reach a log, an error message or a test fixture.** API keys and passwords are stored through the secret store, not in plain configuration.

## Reporting

17. **Say what was verified and what was not.** "Tests pass" and "I saw it work in the app" are different claims; make the one that is true.

18. **Report a failure with its output.** If a test fails, a step was skipped or a result could not be reproduced, that is stated, not rounded up to success.

19. **A finding outside the task is reported, not silently fixed or silently ignored.** Fix it only if it blocks the task; otherwise add it to `docs/todo/issues/`.

## Process

20. **No work without an item.** Anything larger than a one-line fix has a file in `docs/todo/` before the first change, and that file is kept current while the work goes on.

21. **Finished means moved.** An item is done when its file is in `docs/todo/completed/` with its status, the differences from the plan and the checks still owed. Until then it is open.
