import type { InlineSqlOption } from "@/lib/ai/inlineSql";

/**
 * State and key rules of the inline AI panel's options mode: the model's
 * alternatives are previewed and refined turn by turn, and nothing reaches the
 * editor until the user accepts one. Kept free of Vue so the rules that decide
 * what a key press does can be tested on their own.
 */

export interface InlineSqlTurn {
  request: string;
  options: InlineSqlOption[];
  /** Alternative that was on screen when the user sent the next message. */
  shownIndex: number;
}

export interface InlineSqlConversation {
  turns: InlineSqlTurn[];
  turnIndex: number;
  optionIndex: number;
}

/** Earlier turns sent back to the model. Older ones stay visible but are not resent. */
export const MAX_INLINE_SQL_CONTEXT_TURNS = 4;

export function emptyInlineSqlConversation(): InlineSqlConversation {
  return { turns: [], turnIndex: -1, optionIndex: 0 };
}

/** Records which alternative the user was looking at, then shows the new turn. */
export function appendInlineSqlTurn(conversation: InlineSqlConversation, request: string, options: InlineSqlOption[]): InlineSqlConversation {
  const turns = conversation.turns.map((turn, index) => (index === conversation.turnIndex ? { ...turn, shownIndex: conversation.optionIndex } : turn));
  return { turns: [...turns, { request, options, shownIndex: 0 }], turnIndex: turns.length, optionIndex: 0 };
}

export function previewedInlineSqlTurn(conversation: InlineSqlConversation): InlineSqlTurn | null {
  return conversation.turns[conversation.turnIndex] ?? null;
}

export function previewedInlineSqlOption(conversation: InlineSqlConversation): InlineSqlOption | null {
  return previewedInlineSqlTurn(conversation)?.options[conversation.optionIndex] ?? null;
}

/** Moves between alternatives of the shown turn, wrapping at either end. */
export function stepInlineSqlOption(conversation: InlineSqlConversation, step: -1 | 1): InlineSqlConversation {
  const count = previewedInlineSqlTurn(conversation)?.options.length ?? 0;
  if (count < 2) return conversation;
  return { ...conversation, optionIndex: (conversation.optionIndex + step + count) % count };
}

/** Moves between turns without wrapping, returning to the alternative last shown there. */
export function stepInlineSqlTurn(conversation: InlineSqlConversation, step: -1 | 1): InlineSqlConversation {
  return showInlineSqlTurn(conversation, conversation.turnIndex + step);
}

export function showInlineSqlTurn(conversation: InlineSqlConversation, turnIndex: number): InlineSqlConversation {
  if (turnIndex < 0 || turnIndex >= conversation.turns.length || turnIndex === conversation.turnIndex) return conversation;
  const turns = conversation.turns.map((turn, index) => (index === conversation.turnIndex ? { ...turn, shownIndex: conversation.optionIndex } : turn));
  return { turns, turnIndex, optionIndex: turns[turnIndex].shownIndex };
}

export interface InlineSqlContextTurn {
  request: string;
  sql: string;
}

/**
 * What the model is told about the conversation before a follow-up: each
 * request with the SQL the user was looking at when they replied. History
 * after the shown turn is left out, because a reply answers what is on screen.
 */
export function inlineSqlConversationContext(conversation: InlineSqlConversation, maxTurns = MAX_INLINE_SQL_CONTEXT_TURNS): InlineSqlContextTurn[] {
  return conversation.turns
    .slice(0, conversation.turnIndex + 1)
    .map((turn, index) => ({ request: turn.request, sql: turn.options[index === conversation.turnIndex ? conversation.optionIndex : turn.shownIndex]?.sql ?? "" }))
    .filter((turn) => turn.sql)
    .slice(-maxTurns);
}

export type InlineAiKeyAction = "send" | "accept" | "close" | "previous-option" | "next-option" | "previous-turn" | "next-turn" | "history-previous" | "history-next" | "none";

export interface InlineAiKeyInput {
  key: string;
  shiftKey?: boolean;
  altKey?: boolean;
  /** `KeyboardEvent.isComposing`. */
  isComposing?: boolean;
  /** `KeyboardEvent.keyCode`; 229 marks a key consumed by an input method. */
  keyCode?: number;
  promptEmpty: boolean;
  optionsMode: boolean;
  loading: boolean;
}

/**
 * Decides what a key does in the panel.
 *
 * - Esc always closes.
 * - Enter sends when there is text; on an empty prompt in options mode it
 *   accepts the previewed alternative. Shift+Enter is left to the textarea.
 * - Arrow keys pick alternatives and turns only while the prompt is empty, so
 *   they keep moving the caret while the user is typing. Alt+Left/Right picks
 *   an alternative either way.
 * - A key that confirms an input-method composition never acts: some WebKit
 *   versions report it with `isComposing` false and `keyCode` 229.
 */
export function inlineAiKeyAction(input: InlineAiKeyInput): InlineAiKeyAction {
  if (input.isComposing || input.keyCode === 229) return "none";
  if (input.key === "Escape") return "close";
  if (input.key === "Enter") {
    if (input.shiftKey || input.loading) return "none";
    if (!input.promptEmpty) return "send";
    return input.optionsMode ? "accept" : "none";
  }
  if (input.loading) return "none";
  const horizontal = input.key === "ArrowLeft" ? "previous-option" : input.key === "ArrowRight" ? "next-option" : null;
  const vertical = input.key === "ArrowUp" ? -1 : input.key === "ArrowDown" ? 1 : 0;
  if (input.optionsMode) {
    if (horizontal && (input.altKey || input.promptEmpty)) return horizontal;
    if (vertical && input.promptEmpty) return vertical < 0 ? "previous-turn" : "next-turn";
    return "none";
  }
  if (vertical) return vertical < 0 ? "history-previous" : "history-next";
  return "none";
}
