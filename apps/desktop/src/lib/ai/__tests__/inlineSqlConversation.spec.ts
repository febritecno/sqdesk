import { describe, expect, it } from "vitest";
import { appendInlineSqlTurn, emptyInlineSqlConversation, inlineAiKeyAction, inlineSqlConversationContext, previewedInlineSqlOption, showInlineSqlTurn, stepInlineSqlOption, stepInlineSqlTurn, type InlineAiKeyInput } from "@/lib/ai/inlineSqlConversation";

const option = (sql: string, label = "") => ({ sql, label });

function conversation() {
  let state = appendInlineSqlTurn(emptyInlineSqlConversation(), "sort by date", [option("A1"), option("A2"), option("A3")]);
  state = stepInlineSqlOption(state, 1);
  return appendInlineSqlTurn(state, "newest first", [option("B1"), option("B2")]);
}

describe("inline SQL conversation", () => {
  it("previews the first alternative of the newest turn", () => {
    const state = conversation();
    expect(state.turnIndex).toBe(1);
    expect(previewedInlineSqlOption(state)?.sql).toBe("B1");
  });

  it("wraps between alternatives and stays put with a single one", () => {
    const state = conversation();
    expect(previewedInlineSqlOption(stepInlineSqlOption(state, -1))?.sql).toBe("B2");
    expect(previewedInlineSqlOption(stepInlineSqlOption(stepInlineSqlOption(state, 1), 1))?.sql).toBe("B1");
    const single = appendInlineSqlTurn(emptyInlineSqlConversation(), "x", [option("only")]);
    expect(stepInlineSqlOption(single, 1)).toBe(single);
  });

  it("returns to the alternative last shown when moving between turns", () => {
    const state = conversation();
    const back = stepInlineSqlTurn(state, -1);
    expect(previewedInlineSqlOption(back)?.sql).toBe("A2");
    expect(stepInlineSqlTurn(back, -1)).toBe(back);
    const forward = stepInlineSqlTurn(stepInlineSqlOption(back, 1), 1);
    expect(previewedInlineSqlOption(forward)?.sql).toBe("B1");
    expect(previewedInlineSqlOption(showInlineSqlTurn(forward, 0))?.sql).toBe("A3");
  });

  it("tells the model what was on screen for each request, up to the shown turn", () => {
    const state = conversation();
    expect(inlineSqlConversationContext(state)).toEqual([
      { request: "sort by date", sql: "A2" },
      { request: "newest first", sql: "B1" },
    ]);
    expect(inlineSqlConversationContext(stepInlineSqlTurn(state, -1))).toEqual([{ request: "sort by date", sql: "A2" }]);
    expect(inlineSqlConversationContext(state, 1)).toEqual([{ request: "newest first", sql: "B1" }]);
  });
});

describe("inlineAiKeyAction", () => {
  const base: InlineAiKeyInput = { key: "", promptEmpty: true, optionsMode: false, loading: false };
  const action = (input: Partial<InlineAiKeyInput>) => inlineAiKeyAction({ ...base, ...input });

  it("closes on Escape in every state", () => {
    expect(action({ key: "Escape" })).toBe("close");
    expect(action({ key: "Escape", loading: true, optionsMode: true, promptEmpty: false })).toBe("close");
  });

  it("sends text, accepts on an empty prompt in options mode, and otherwise does nothing", () => {
    expect(action({ key: "Enter", promptEmpty: false })).toBe("send");
    expect(action({ key: "Enter", promptEmpty: false, optionsMode: true })).toBe("send");
    expect(action({ key: "Enter", optionsMode: true })).toBe("accept");
    expect(action({ key: "Enter" })).toBe("none");
    expect(action({ key: "Enter", shiftKey: true, promptEmpty: false })).toBe("none");
    expect(action({ key: "Enter", loading: true, promptEmpty: false })).toBe("none");
  });

  it("never acts on a key that belongs to an input method", () => {
    expect(action({ key: "Enter", promptEmpty: false, isComposing: true })).toBe("none");
    expect(action({ key: "Enter", promptEmpty: false, keyCode: 229 })).toBe("none");
    expect(action({ key: "Escape", keyCode: 229 })).toBe("none");
  });

  it("picks alternatives and turns with the arrows only while the prompt is empty", () => {
    expect(action({ key: "ArrowLeft", optionsMode: true })).toBe("previous-option");
    expect(action({ key: "ArrowRight", optionsMode: true })).toBe("next-option");
    expect(action({ key: "ArrowUp", optionsMode: true })).toBe("previous-turn");
    expect(action({ key: "ArrowDown", optionsMode: true })).toBe("next-turn");
    expect(action({ key: "ArrowLeft", optionsMode: true, promptEmpty: false })).toBe("none");
    expect(action({ key: "ArrowUp", optionsMode: true, promptEmpty: false })).toBe("none");
    expect(action({ key: "ArrowRight", optionsMode: true, promptEmpty: false, altKey: true })).toBe("next-option");
  });

  it("walks the prompt history outside options mode", () => {
    expect(action({ key: "ArrowUp" })).toBe("history-previous");
    expect(action({ key: "ArrowDown", promptEmpty: false })).toBe("history-next");
    expect(action({ key: "ArrowLeft" })).toBe("none");
    expect(action({ key: "ArrowUp", loading: true })).toBe("none");
  });
});
