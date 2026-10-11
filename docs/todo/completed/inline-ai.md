# Inline AI: options preview with refinement, movable panel, reliable Enter and Esc

**Type:** feature · **Completed:** 2026-10-11

**Status:** implemented. Covered by unit and component tests; still to be confirmed by hand in the app (see "Left to check").

## What was built

- **Options mode previews.** Alternatives are shown in the panel and the editor is not written until Accept. A fresh request that returns one answer is still inserted directly.
- **Refinement by conversation.** The prompt stays editable under the preview; each message adds a turn, earlier turns are listed and can be revisited, and the last four are sent back to the model.
- **Movable panel.** Drag the sparkle icon. The panel is kept inside the editor area, also after it grows.
- **Keys.** Esc closes from the prompt, from any panel control and from the editor (stopped before CodeMirror, so Vim does not also react). Enter sends when there is text and accepts on an empty prompt. A key that belongs to an input method is ignored.

Code: `EditorInlineAiPanel.vue`, `lib/ai/inlineSqlConversation.ts` (state and key rules), `lib/ai/inlineSql.ts` (conversation section of the prompt). Tests: `inlineSqlConversation.spec.ts`, `inlineSql.spec.ts`, `EditorInlineAiPanel.options.spec.ts`.

## Where the result differs from the plan below

- **Enter in the editor is not taken over.** The plan said Enter should act on the panel from anywhere in the editor area. That would turn every line break typed in the editor into "accept" or "send" while the panel is open, so only Esc is global. Enter works in the prompt and on the panel's controls.
- **Arrow keys:** Left/Right choose an alternative and Up/Down move between turns only while the prompt is empty; Alt+Left/Right chooses an alternative while typing.
- **The preview is plain monospace text**, without syntax highlighting.
- **Accept after an edit is refused.** If the editor text changed after the conversation began, Accept shows a message instead of writing, because the remembered position is no longer trustworthy.
- **The panel position is not remembered** between openings.

## Left to check

- Dragging and the taller options layout in the real window.
- Esc in macOS fullscreen (`macos_escape_guard.rs` sits in front of the webview there).
- Esc with the Vim keymap enabled: the test covers the event being stopped, not Vim's own mode indicator.

The rest of this document is the original plan, kept for the reasoning.

The inline AI panel is `apps/desktop/src/components/editor/EditorInlineAiPanel.vue`; its prompt and answer handling is `apps/desktop/src/lib/ai/inlineSql.ts`. It opens with `Mod-i` (keymap entry in `QueryEditor.vue`) or from the editor context menu.

Three separate changes.

## 1. Options mode: preview first, refine by conversation

The panel has two modes, decided by what the model returns.

| Mode | When | Behavior |
| --- | --- | --- |
| Direct | One answer | Written into the editor at once, panel closes. **Unchanged from today.** |
| Options | Several alternatives | Shown as a preview inside the panel. The editor is not touched until the user accepts. |

### Today

Options mode writes the first alternative straight into the editor, and stepping through `‹ 1/3 ›` rewrites the editor each time. The user's SQL is replaced before they have chosen anything, and Esc is the only way back. The exchange also ends there: to refine a result the user must select the SQL again, reopen the panel and retype the context.

### Wanted

1. **Preview, not replacement.** In options mode the alternatives are shown in a read-only SQL preview inside the panel. The editor keeps the original text.
2. **Arrow keys choose.** Left and Right move between alternatives; the preview and the `‹ 1/3 ›` counter follow.
3. **Accept writes.** Only an explicit accept (Enter, or an Accept button) writes the shown alternative into the editor and closes the panel. Esc closes the panel and changes nothing.
4. **Refine by conversation.** Below the preview there is always a prompt input. Sending another message produces a new set of alternatives from the current context, and the panel keeps the earlier turns as a short history, like a chat. The user repeats until one fits, then accepts.

There is no separate "reply" button: the input is the reply.

### Design notes

- **State.** Replace `applied` (which tracked text already written into the editor) with a conversation: an ordered list of turns, each `{ request, options[] }`, plus the index of the turn and option being previewed. Nothing is dispatched to the editor until accept, so the stale-offset problem shrinks to one check at accept time (`view.state.doc.eq(documentAtOpen)`), and "restore the original" is no longer needed.
- **Preview.** A small read-only code block with the same SQL highlighting as the editor, a fixed maximum height and its own scroll. Show the option's label next to the counter.
- **Keys while the prompt is empty:** Left/Right change the option; Up/Down move between turns of the history. **While the prompt has text**, arrow keys must edit the text as usual, so option switching needs either an empty prompt or a modifier (for example Alt+Left/Right). Pick one rule and show it in the hint line.
- **Enter is overloaded** and needs one clear rule: with text in the prompt, Enter sends the message; with an empty prompt, Enter accepts the previewed option. Shift+Enter stays a newline.
- **Follow-up request.** Send the original selection, the earlier turns (request and the option that was on screen when the user replied), and the new message. `buildInlineSqlUserPrompt` has a "Selected SQL to rewrite" section; add a "Conversation so far" section rather than pretending the previous answer was the user's selection, so the model still knows what the editor really contains.
- **A follow-up may come back with a single answer.** That is still options mode for this panel: show it in the preview and wait for accept. Direct mode applies only to the first answer of a fresh panel.
- **Keep the output rules.** Every turn uses the strict system prompt and the answer cleaner, including the guard against turning a query into a write statement.
- **History size.** Cap the turns sent to the model (for example the last 4) so a long refinement does not grow the request without bound. The panel may still display more.
- **Layout.** The panel gets taller in options mode. Keep it inside the editor area (`positionPanel` flips above the line when there is no room below; with a preview and history it needs a maximum height and internal scrolling).

### Done when

- A request that returns alternatives leaves the editor text untouched.
- Left and Right change the previewed alternative; the editor still does not change.
- Accept writes exactly the previewed alternative and closes the panel; Esc closes it with the editor unchanged.
- A follow-up message produces new alternatives that take the earlier turns into account, and earlier turns stay visible.
- A request that returns one answer on a fresh panel is inserted directly, exactly as today.
- The prompt history (arrow up on an empty single-line prompt, outside options mode) still works.

## 2. Movable panel

### Today

`positionPanel` places the panel once, next to the selection, when it opens. It can cover the SQL the user wants to read while writing the request, and it does not follow scrolling.

### Wanted

The user can drag the panel to another place.

### Design notes

- Use the left strip (the sparkle icon) or a thin top edge as the drag handle; the textarea must stay a normal text field.
- Follow the pointer-event pattern already used in this app for the sidebar pane divider (`onConnectionPaneResizePointerDown` in `ConnectionTree.vue`): `pointerdown` on the handle, `pointermove` and `pointerup` on `window`, clean up on `pointercancel`.
- Clamp to the editor host's rectangle, the same bounds `positionPanel` uses, so the panel cannot be dragged out of reach.
- Once the user has moved it, stop auto-positioning for that opening. Whether the position is remembered across openings is a product choice; remembering an offset from the anchor is less surprising than remembering absolute coordinates, because the anchor moves with the selection.
- Dragging must not close the panel or steal focus from the textarea.

### Done when

- The panel can be dragged anywhere inside the editor area and stays where it is dropped while it is open.
- Typing, Enter and Esc work normally after a drag.
- Reopening the panel puts it somewhere sensible, never off-screen.

## 3. Enter and Esc must always work

### Today

Both keys are handled in `onKeydown` on the textarea only. They work while the textarea has focus and stop working in every case where it does not:

- After clicking anywhere else in the panel (the option buttons, the panel background) focus leaves the textarea, and the keys go nowhere.
- While alternatives are shown the textarea is read-only. It still receives keys, but only if it kept focus. (Item 1 makes the prompt editable in options mode, which removes this case but adds the Enter and arrow-key rules described there.)
- After clicking in the editor, keys go to CodeMirror. With the Vim keymap active (`@replit/codemirror-vim`, mounted ahead of the default keymaps), Esc is consumed by Vim to leave insert mode.
- On macOS, `src-tauri/src/macos_escape_guard.rs` intercepts Esc at the AppKit level to stop it leaving fullscreen. Check that the key still reaches the webview in fullscreen.
- IME composition: Enter that confirms a composition must not submit. The handler checks `event.isComposing`; some WebKit versions report the confirming Enter with `isComposing` false and `keyCode` 229, so that case needs a guard too.

### Wanted

While the panel is open, Enter and Esc act on the panel regardless of where focus is inside the editor area, in every keymap (default, Vim) and on macOS, Windows and Linux.

### Design notes

- Handle the keys at the panel level, not only on the textarea: a `keydown` listener in the capture phase on the editor host (or `window`) that is installed when the panel opens and removed when it closes. Capture phase is what lets it run before CodeMirror and Vim.
- Scope it tightly: act only on Enter and Esc, only while the panel is visible, and leave every other key alone so normal typing in the editor is unaffected.
- Keep Shift+Enter as a newline in the prompt.
- Enter should not submit an empty prompt, and should not fire while a request is running.
- Give the panel root `tabindex="-1"` and return focus to the textarea after clicks on the panel's buttons, so the simple case never loses focus in the first place.
- Test matrix: default keymap and Vim (normal and insert mode); focus in the textarea, on a panel button, and in the editor; macOS windowed and fullscreen; a CJK IME mid-composition.

### Done when

- Esc closes the panel (or restores the original text in the alternatives state) from any focus position inside the editor area, in both keymaps.
- Enter submits or accepts under the same conditions.
- In Vim mode, Esc with the panel open does not also change Vim's mode.
- Confirming an IME composition with Enter does not submit.

## Tests

`apps/desktop/src/lib/ai/__tests__/inlineSql.spec.ts` covers prompts and answer cleaning. The panel itself has no component test yet; the options conversation (preview, accept, follow-up) and the key handling are the right place to start one. Keep the conversation state and its key rules in a plain module next to `inlineSql.ts` so they can be tested without mounting the component.
