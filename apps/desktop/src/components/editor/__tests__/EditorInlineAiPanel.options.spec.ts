// @vitest-environment happy-dom
//
// The inline AI panel's two modes, driven the way a user drives them: a real
// CodeMirror view, real key events, and only the model call replaced. The point
// is the contract with the editor: options mode must not touch the document
// until an alternative is accepted, and direct mode must write at once.
import { createApp, h, nextTick, shallowRef, type App } from "vue";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia } from "pinia";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import i18n from "@/i18n";
import EditorInlineAiPanel from "@/components/editor/EditorInlineAiPanel.vue";
import { useSettingsStore } from "@/stores/settingsStore";
import { useConnectionStore } from "@/stores/connectionStore";

const backend = vi.hoisted(() => ({ aiComplete: vi.fn<(request: { messages: { content: string }[] }) => Promise<string>>() }));

vi.mock("@/lib/backend/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@/lib/backend/api")>()),
  aiComplete: backend.aiComplete,
  getColumns: vi.fn(async () => []),
  listIndexes: vi.fn(async () => []),
  listForeignKeys: vi.fn(async () => []),
}));

const ORIGINAL = "select id from orders";
const fenced = (...sql: string[]) => sql.map((statement, index) => `### Option ${index + 1}\n\`\`\`sql\n${statement}\n\`\`\``).join("\n\n");

let app: App | null = null;
let view: EditorView;
let host: HTMLElement;
let panel: { openInlineAi: () => boolean; closeInlineAi: () => void };

async function settle() {
  // Schema loading and the model call each resolve over a few microtask turns.
  for (let turn = 0; turn < 8; turn += 1) {
    await Promise.resolve();
    await nextTick();
  }
}

function mount() {
  host = document.createElement("div");
  const root = document.createElement("div");
  root.setAttribute("data-query-editor-root", "");
  host.append(root);
  document.body.append(host);
  view = new EditorView({ parent: root, state: EditorState.create({ doc: ORIGINAL, selection: { anchor: 0, head: ORIGINAL.length } }) });

  const pinia = createPinia();
  const mountPoint = document.createElement("div");
  host.append(mountPoint);
  const viewRef = shallowRef<EditorView | null>(view);
  app = createApp({
    setup() {
      return () =>
        h(EditorInlineAiPanel, {
          ref: (instance: unknown) => {
            if (instance) panel = instance as typeof panel;
          },
          view: viewRef.value,
          connectionId: "conn-1",
          database: "shop",
          databaseType: "mysql",
        });
    },
  });
  app.use(pinia).use(i18n).mount(mountPoint);

  const settings = useSettingsStore();
  settings.initAiConfigs = vi.fn(async () => {});
  settings.aiConfigs = [{ id: "cfg", name: "Test", provider: "openai-compatible", apiKey: "", authMethod: "bearer", endpoint: "http://localhost/v1", model: "m", apiStyle: "completions", isDefault: true }] as never;
  const connections = useConnectionStore();
  connections.lookupLocalCompletionTables = vi.fn(() => []) as never;
  connections.listCompletionTables = vi.fn(async () => []) as never;
}

const input = () => document.querySelector<HTMLTextAreaElement>("[data-editor-inline-ai] textarea")!;
const preview = () => document.querySelector<HTMLElement>("[data-inline-ai-preview]")?.textContent ?? null;
const isOpen = () => document.querySelector("[data-editor-inline-ai]") !== null;
const press = (target: EventTarget, key: string, init: KeyboardEventInit = {}) => target.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...init }));

async function send(text: string) {
  input().value = text;
  input().dispatchEvent(new Event("input", { bubbles: true }));
  await nextTick();
  press(input(), "Enter");
  await settle();
}

beforeEach(async () => {
  backend.aiComplete.mockReset();
  mount();
  await nextTick();
  expect(panel.openInlineAi()).toBe(true);
  await nextTick();
});

afterEach(() => {
  app?.unmount();
  app = null;
  view.destroy();
  host.remove();
});

describe("EditorInlineAiPanel", () => {
  it("writes a single answer straight into the editor", async () => {
    backend.aiComplete.mockResolvedValueOnce(fenced("select id from orders where paid = 1"));
    await send("only paid");

    expect(view.state.doc.toString()).toBe("select id from orders where paid = 1");
    expect(isOpen()).toBe(false);
  });

  it("previews alternatives without touching the editor, and writes only on accept", async () => {
    backend.aiComplete.mockResolvedValueOnce(fenced("select id from orders order by created_at", "select id from orders order by updated_at"));
    await send("sort by date");

    expect(view.state.doc.toString()).toBe(ORIGINAL);
    expect(preview()).toBe("select id from orders order by created_at");

    press(input(), "ArrowRight");
    await nextTick();
    expect(preview()).toBe("select id from orders order by updated_at");
    expect(view.state.doc.toString()).toBe(ORIGINAL);

    press(input(), "Enter");
    await nextTick();
    expect(view.state.doc.toString()).toBe("select id from orders order by updated_at");
    expect(isOpen()).toBe(false);
  });

  it("refines by conversation and keeps previewing until accepted", async () => {
    backend.aiComplete.mockResolvedValueOnce(fenced("select id from orders order by created_at", "select id from orders order by updated_at"));
    await send("sort by date");
    press(input(), "ArrowRight");
    await nextTick();

    // A follow-up that returns one answer is still a preview, not a direct write.
    backend.aiComplete.mockResolvedValueOnce(fenced("select id from orders order by updated_at desc"));
    await send("newest first");

    const followUp = backend.aiComplete.mock.calls[1][0].messages[0].content;
    expect(followUp).toContain("Request 1: sort by date");
    expect(followUp).toContain("select id from orders order by updated_at");
    expect(followUp).not.toContain("order by created_at");
    expect(followUp).toContain("Revise proposed SQL 1 as follows: newest first");

    expect(view.state.doc.toString()).toBe(ORIGINAL);
    expect(preview()).toBe("select id from orders order by updated_at desc");
    expect(document.querySelectorAll("[data-inline-ai-history] button")).toHaveLength(2);

    press(input(), "ArrowUp");
    await nextTick();
    expect(preview()).toBe("select id from orders order by updated_at");

    press(input(), "ArrowDown");
    await nextTick();
    document.querySelector<HTMLButtonElement>("[data-inline-ai-accept]")!.click();
    await nextTick();
    expect(view.state.doc.toString()).toBe("select id from orders order by updated_at desc");
  });

  it("leaves the editor alone when the panel is closed with Escape", async () => {
    backend.aiComplete.mockResolvedValueOnce(fenced("select 1", "select 2"));
    await send("anything");

    press(input(), "Escape");
    await nextTick();
    expect(isOpen()).toBe(false);
    expect(view.state.doc.toString()).toBe(ORIGINAL);
  });

  it("closes on Escape pressed in the editor, but leaves Enter there to the editor", async () => {
    backend.aiComplete.mockResolvedValueOnce(fenced("select 1", "select 2"));
    await send("anything");

    press(view.contentDOM, "Enter");
    await nextTick();
    expect(isOpen()).toBe(true);
    expect(view.state.doc.toString()).toBe(ORIGINAL);

    const escape = new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true });
    view.contentDOM.dispatchEvent(escape);
    await nextTick();
    expect(isOpen()).toBe(false);
    expect(escape.defaultPrevented).toBe(true);
  });

  it("does not submit while an input method is composing", async () => {
    input().value = "sort";
    input().dispatchEvent(new Event("input", { bubbles: true }));
    await nextTick();
    press(input(), "Enter", { isComposing: true });
    press(input(), "Enter", { keyCode: 229 });
    await settle();
    expect(backend.aiComplete).not.toHaveBeenCalled();
  });

  it("refuses to accept after the editor text changed", async () => {
    backend.aiComplete.mockResolvedValueOnce(fenced("select 1", "select 2"));
    await send("anything");

    view.dispatch({ changes: { from: 0, insert: "-- note\n" } });
    press(input(), "Enter");
    await nextTick();

    expect(view.state.doc.toString()).toBe(`-- note\n${ORIGINAL}`);
    expect(isOpen()).toBe(true);
    expect(document.querySelector("[data-editor-inline-ai] .text-destructive")?.textContent).toBeTruthy();
  });
});
