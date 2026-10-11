<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { Check, ChevronLeft, ChevronRight, GripVertical, Loader2, Sparkles, X } from "@lucide/vue";
import type { EditorView } from "@codemirror/view";
import * as api from "@/lib/backend/api";
import { buildInlineSqlSystemPrompt, buildInlineSqlUserPrompt, cleanInlineSqlOptions, inlineSqlInsertion, pickInlineSqlTables, type InlineSqlOption, type InlineSqlTable } from "@/lib/ai/inlineSql";
import { appendInlineSqlTurn, emptyInlineSqlConversation, inlineAiKeyAction, inlineSqlConversationContext, previewedInlineSqlOption, previewedInlineSqlTurn, showInlineSqlTurn, stepInlineSqlOption, stepInlineSqlTurn, type InlineSqlConversation } from "@/lib/ai/inlineSqlConversation";
import { runtimeEffortFromPreference } from "@/lib/ai/aiEffortPreference";
import { useConnectionStore } from "@/stores/connectionStore";
import { normalizeAiConfig, useSettingsStore } from "@/stores/settingsStore";
import type { DatabaseType } from "@/types/database";

/** Tables listed by name only, so the model knows what exists without the cost of every column. */
const MAX_LISTED_TABLES = 300;
/** Tables sent with columns, indexes and foreign keys. */
const MAX_DETAILED_TABLES = 16;
/** Editor text around the cursor that the model sees as context. */
const CONTEXT_BEFORE_CHARS = 1500;
const CONTEXT_AFTER_CHARS = 600;
/** How much of the editor is scanned for table names the request did not spell out. */
const RELEVANCE_SCAN_CHARS = 20_000;
const MAX_ANSWER_TOKENS = 2000;
/** Reasoning models spend output tokens on thinking first; the chat assistant uses the same floor. */
const MAX_ANSWER_TOKENS_WITH_THINKING = 8192;
const MAX_PROMPT_HISTORY = 30;
/** Gap kept between the panel and the edge of the editor area. */
const PANEL_EDGE_GAP = 8;
/** Smallest size the options panel can be dragged down to: room for the prompt, the controls and a few preview lines. */
const PANEL_MIN_WIDTH = 280;
const PANEL_MIN_HEIGHT = 180;
// Shared by every editor in this window, for the session.
const promptHistory: string[] = [];

const props = defineProps<{
  view: EditorView | null;
  connectionId?: string;
  database?: string;
  schema?: string;
  databaseType?: DatabaseType;
}>();

const emit = defineEmits<{
  open: [];
}>();

const { t } = useI18n();
const connectionStore = useConnectionStore();
const settingsStore = useSettingsStore();

const visible = ref(false);
const prompt = ref("");
const loading = ref(false);
const error = ref("");
const status = ref("");
const hasSelection = ref(false);
const panelStyle = ref<Record<string, string>>({});
const panelRef = ref<HTMLElement>();
const inputRef = ref<HTMLTextAreaElement>();
/**
 * Alternatives the model offered, turn by turn. While it has turns the panel
 * is in options mode: they are previewed here and the editor is left alone
 * until one is accepted.
 */
const conversation = ref<InlineSqlConversation>(emptyInlineSqlConversation());
const optionsMode = computed(() => conversation.value.turns.length > 0);
const previewTurn = computed(() => previewedInlineSqlTurn(conversation.value));
const previewOption = computed(() => previewedInlineSqlOption(conversation.value));
const isDragging = ref(false);
let target = { from: 0, to: 0 };
/** Document the target offsets were measured against; offsets are meaningless once it changes. */
let targetDoc: EditorView["state"]["doc"] | null = null;
/** Text of the target when the conversation began, so follow-ups describe the same selection. */
let targetSql = "";
let requestId = 0;
let historyIndex = promptHistory.length;

const placeholder = computed(() => {
  if (optionsMode.value) return t("editor.inlineAi.optionPlaceholder");
  return t(hasSelection.value ? "editor.inlineAi.placeholderEdit" : "editor.inlineAi.placeholderGenerate");
});

function panelHost(view: EditorView): HTMLElement | null {
  return view.dom.closest("[data-query-editor-root]")?.parentElement ?? view.dom.parentElement;
}

function positionPanel(view: EditorView) {
  const anchor = view.coordsAtPos(target.to) ?? view.coordsAtPos(target.from);
  const host = panelHost(view);
  if (!anchor || !host) {
    panelStyle.value = { top: "12px", left: "48px" };
    return;
  }
  const hostRect = host.getBoundingClientRect();
  const width = Math.min(520, Math.max(280, hostRect.width - 32));
  const left = Math.min(Math.max(16, anchor.left - hostRect.left), Math.max(16, hostRect.width - width - 16));
  // Keep the panel inside the editor: flip above the line when there is no room below.
  const below = anchor.bottom - hostRect.top + 6;
  const top = below + 120 > hostRect.height ? Math.max(PANEL_EDGE_GAP, anchor.top - hostRect.top - 126) : below;
  panelStyle.value = { top: `${top}px`, left: `${left}px`, width: `${width}px` };
}

/** Pulls the panel back inside the editor area after it grew (preview, history) or the area shrank. */
function keepPanelInside() {
  const view = props.view;
  const panel = panelRef.value;
  const host = view ? panelHost(view) : null;
  if (!panel || !host) return;
  const hostRect = host.getBoundingClientRect();
  const panelRect = panel.getBoundingClientRect();
  const top = Math.max(PANEL_EDGE_GAP, Math.min(panelRect.top - hostRect.top, hostRect.height - panelRect.height - PANEL_EDGE_GAP));
  const left = Math.max(PANEL_EDGE_GAP, Math.min(panelRect.left - hostRect.left, hostRect.width - panelRect.width - PANEL_EDGE_GAP));
  panelStyle.value = { ...panelStyle.value, top: `${top}px`, left: `${left}px` };
}

function onDragPointerDown(event: PointerEvent) {
  const view = props.view;
  const panel = panelRef.value;
  const host = view ? panelHost(view) : null;
  if (!panel || !host || event.button !== 0) return;
  event.preventDefault();
  const hostRect = host.getBoundingClientRect();
  const panelRect = panel.getBoundingClientRect();
  const offsetX = event.clientX - panelRect.left;
  const offsetY = event.clientY - panelRect.top;
  const maxLeft = Math.max(PANEL_EDGE_GAP, hostRect.width - panelRect.width - PANEL_EDGE_GAP);
  const maxTop = Math.max(PANEL_EDGE_GAP, hostRect.height - panelRect.height - PANEL_EDGE_GAP);
  isDragging.value = true;
  const onMove = (moveEvent: PointerEvent) => {
    const left = Math.min(maxLeft, Math.max(PANEL_EDGE_GAP, moveEvent.clientX - hostRect.left - offsetX));
    const top = Math.min(maxTop, Math.max(PANEL_EDGE_GAP, moveEvent.clientY - hostRect.top - offsetY));
    panelStyle.value = { ...panelStyle.value, top: `${top}px`, left: `${left}px` };
  };
  const onStop = () => {
    isDragging.value = false;
    window.removeEventListener("pointermove", onMove);
    window.removeEventListener("pointerup", onStop);
    window.removeEventListener("pointercancel", onStop);
    focusInput();
  };
  window.addEventListener("pointermove", onMove);
  window.addEventListener("pointerup", onStop);
  window.addEventListener("pointercancel", onStop);
}

/**
 * Resizes the options panel from its bottom-right corner. The top-left corner
 * stays where it is, and the panel cannot grow past the editor area.
 */
function onResizePointerDown(event: PointerEvent) {
  const view = props.view;
  const panel = panelRef.value;
  const host = view ? panelHost(view) : null;
  if (!panel || !host || event.button !== 0) return;
  event.preventDefault();
  const hostRect = host.getBoundingClientRect();
  const panelRect = panel.getBoundingClientRect();
  const startX = event.clientX;
  const startY = event.clientY;
  const maxWidth = Math.max(PANEL_MIN_WIDTH, hostRect.right - panelRect.left - PANEL_EDGE_GAP);
  const maxHeight = Math.max(PANEL_MIN_HEIGHT, hostRect.bottom - panelRect.top - PANEL_EDGE_GAP);
  isDragging.value = true;
  const onMove = (moveEvent: PointerEvent) => {
    const width = Math.min(maxWidth, Math.max(PANEL_MIN_WIDTH, panelRect.width + moveEvent.clientX - startX));
    const height = Math.min(maxHeight, Math.max(PANEL_MIN_HEIGHT, panelRect.height + moveEvent.clientY - startY));
    // An explicit height replaces the content-sized default, so the cap that goes with that default is lifted.
    panelStyle.value = { ...panelStyle.value, width: `${Math.round(width)}px`, height: `${Math.round(height)}px`, maxHeight: "none" };
  };
  const onStop = () => {
    isDragging.value = false;
    window.removeEventListener("pointermove", onMove);
    window.removeEventListener("pointerup", onStop);
    window.removeEventListener("pointercancel", onStop);
    focusInput();
  };
  window.addEventListener("pointermove", onMove);
  window.addEventListener("pointerup", onStop);
  window.addEventListener("pointercancel", onStop);
}

function focusInput() {
  inputRef.value?.focus();
}

function openInlineAi(): boolean {
  const view = props.view;
  if (!view || view.state.readOnly) return false;
  const selection = view.state.selection.main;
  target = { from: selection.from, to: selection.to };
  targetDoc = view.state.doc;
  targetSql = "";
  hasSelection.value = !selection.empty;
  error.value = "";
  conversation.value = emptyInlineSqlConversation();
  historyIndex = promptHistory.length;
  visible.value = true;
  positionPanel(view);
  // Capture phase: the panel's keys must be seen before CodeMirror and the Vim keymap get them.
  window.addEventListener("keydown", onWindowKeydown, true);
  emit("open");
  void nextTick(() => {
    fitInputHeight();
    focusInput();
  });
  return true;
}

/** Grows the prompt box downward with its text instead of scrolling inside two lines. */
function fitInputHeight() {
  const input = inputRef.value;
  if (!input) return;
  input.style.height = "auto";
  input.style.height = `${input.scrollHeight}px`;
}

watch(prompt, () => void nextTick(fitInputHeight));

// The preview and the history change the panel's height; keep it on screen.
watch(conversation, () => void nextTick(keepPanelInside));

// The target offsets belong to one editor; a replaced view makes them stale.
watch(
  () => props.view,
  () => closeInlineAi(false),
);

function closeInlineAi(restoreFocus = true) {
  if (!visible.value) return;
  requestId += 1;
  visible.value = false;
  loading.value = false;
  conversation.value = emptyInlineSqlConversation();
  window.removeEventListener("keydown", onWindowKeydown, true);
  if (restoreFocus) props.view?.focus();
}

onBeforeUnmount(() => window.removeEventListener("keydown", onWindowKeydown, true));

/** Text an option occupies in the editor, including the line breaks that keep generated SQL on its own line. */
function optionText(view: EditorView, option: InlineSqlOption, from: number, to: number, rewriting: boolean): string {
  if (rewriting) return option.sql;
  const line = view.state.doc.lineAt(from);
  return inlineSqlInsertion(option.sql, view.state.sliceDoc(line.from, from), view.state.sliceDoc(to, view.state.doc.lineAt(to).to));
}

function writeOption(view: EditorView, option: InlineSqlOption, rewriting: boolean) {
  const { from, to } = target;
  const insert = optionText(view, option, from, to, rewriting);
  const start = from + (insert.length - insert.trimStart().length);
  view.dispatch({ changes: { from, to, insert }, selection: { anchor: start, head: start + option.sql.length }, scrollIntoView: true, userEvent: "input.ai" });
}

function showOption(step: -1 | 1) {
  conversation.value = stepInlineSqlOption(conversation.value, step);
}

function showTurn(turnIndex: number) {
  conversation.value = showInlineSqlTurn(conversation.value, turnIndex);
}

/** Writes the previewed alternative into the editor. Until this runs, options mode changes nothing. */
function acceptOption() {
  const view = props.view;
  const option = previewOption.value;
  if (!view || !option || loading.value) return;
  // The offsets were measured when the conversation began. If the text
  // changed since, they point somewhere else and writing would damage it.
  if (targetDoc && !view.state.doc.eq(targetDoc)) {
    error.value = t("editor.inlineAi.editorChangedBeforeAccept");
    return;
  }
  writeOption(view, option, targetSql !== "");
  closeInlineAi();
}

function resolveAiConfig() {
  const active = settingsStore.activeModel;
  const item = (active && settingsStore.aiConfigs.find((config) => config.id === active.configId)) || settingsStore.aiConfigs.find((config) => config.isDefault) || settingsStore.aiConfigs[0];
  if (!item) return null;
  const model = active && active.configId === item.id ? active.modelId : item.model;
  return normalizeAiConfig({ ...item, model, runtimeEffort: runtimeEffortFromPreference(settingsStore.activeEffort) });
}

async function loadSchema(connectionId: string, database: string, relevantText: string) {
  let listed = connectionStore.lookupLocalCompletionTables(connectionId, database, "", MAX_LISTED_TABLES, props.schema);
  if (listed.length === 0) listed = await connectionStore.listCompletionTables(connectionId, database, "", MAX_LISTED_TABLES, props.schema).catch(() => []);
  const picked = pickInlineSqlTables(listed, relevantText, MAX_DETAILED_TABLES);
  const pickedSet = new Set(picked);
  const tables = await Promise.all(
    picked.map(async (table): Promise<InlineSqlTable> => {
      const schema = table.schema ?? props.schema ?? "";
      // Indexes and foreign keys are what let the model write joins and filters the planner can use.
      const [columns, indexes, foreignKeys] = await Promise.all([
        api.getColumns(connectionId, database, schema, table.name, table.catalog).catch(() => []),
        api.listIndexes(connectionId, database, schema, table.name, table.catalog).catch(() => []),
        api.listForeignKeys(connectionId, database, schema, table.name, table.catalog).catch(() => []),
      ]);
      return { schema: table.schema, name: table.name, tableType: table.tableType, columns, indexes, foreignKeys };
    }),
  );
  // A table whose columns could not be read still exists: list it by name so
  // the model does not conclude it is missing and refuse the request.
  const unreadable = new Set(tables.filter((table) => table.columns.length === 0).map((table) => table.name));
  return {
    tables: tables.filter((table) => table.columns.length > 0),
    otherTableNames: listed.filter((table) => !pickedSet.has(table) || unreadable.has(table.name)).map((table) => table.name),
    truncated: listed.length >= MAX_LISTED_TABLES,
  };
}

async function submit() {
  const view = props.view;
  const request = prompt.value.trim();
  if (!view || !request || loading.value) return;
  const connectionId = props.connectionId;
  if (!connectionId) {
    error.value = t("editor.inlineAi.noConnection");
    return;
  }
  const currentRequest = ++requestId;
  const followUp = optionsMode.value;
  loading.value = true;
  error.value = "";
  status.value = t("editor.inlineAi.readingSchema");
  try {
    await settingsStore.initAiConfigs();
    const config = resolveAiConfig();
    if (!config) throw new Error(t("editor.inlineAi.noModel"));
    const document = view.state.doc;
    if (!followUp) {
      // The editor stays editable while the panel is open. If its text changed
      // since then, the remembered offsets point somewhere else: follow the
      // current selection instead of rewriting the wrong range.
      if (targetDoc && !document.eq(targetDoc)) {
        const selection = view.state.selection.main;
        target = { from: selection.from, to: selection.to };
        hasSelection.value = !selection.empty;
      }
      targetDoc = document;
      targetSql = view.state.sliceDoc(target.from, target.to);
    }
    // A follow-up keeps describing the selection the conversation started
    // from, even if the editor has changed underneath it since.
    const selectedSql = targetSql;
    const unchanged = !!targetDoc && document.eq(targetDoc);
    const textBefore = unchanged ? view.state.sliceDoc(Math.max(0, target.from - CONTEXT_BEFORE_CHARS), target.from) : "";
    const textAfter = unchanged ? view.state.sliceDoc(target.to, Math.min(document.length, target.to + CONTEXT_AFTER_CHARS)) : "";
    const database = props.database ?? "";
    const turns = followUp ? inlineSqlConversationContext(conversation.value) : [];
    // Tables already used elsewhere in the editor are likely relevant even when the request does not name them.
    const relevantText = [request, selectedSql, ...turns.flatMap((turn) => [turn.request, turn.sql]), view.state.sliceDoc(0, Math.min(document.length, RELEVANCE_SCAN_CHARS))].join("\n");
    const schema = await loadSchema(connectionId, database, relevantText);
    if (currentRequest !== requestId) return;
    status.value = t("editor.inlineAi.generatingWithTables", { count: schema.tables.length });
    const connection = connectionStore.getConfig(connectionId);
    const input = {
      request,
      selectedSql,
      textBefore,
      textAfter,
      databaseType: props.databaseType ?? connection?.db_type ?? "sql",
      serverVersion: connection?.database_info?.productVersion,
      database,
      schema: props.schema,
      ...schema,
      conversation: turns,
    };
    const answer = await api.aiComplete({
      config,
      systemPrompt: buildInlineSqlSystemPrompt(input),
      messages: [{ role: "user", content: buildInlineSqlUserPrompt(input) }],
      maxTokens: config.enableThinking ? MAX_ANSWER_TOKENS_WITH_THINKING : MAX_ANSWER_TOKENS,
    });
    if (currentRequest !== requestId) return;
    const { options: answered, note, truncated, unrequestedStatement } = cleanInlineSqlOptions(answer, { selectedSql, request });
    if (truncated) throw new Error(t("editor.inlineAi.truncated"));
    if (unrequestedStatement) throw new Error(t("editor.inlineAi.unrequestedStatement", { kind: unrequestedStatement }));
    if (answered.length === 0) throw new Error(note || t("editor.inlineAi.empty"));
    rememberPrompt(request);
    prompt.value = "";
    if (!followUp && answered.length === 1) {
      // Direct mode: one answer to a fresh request goes straight into the editor.
      // Typing in the editor while the model was running moved the text the
      // offsets referred to. Writing there now would overwrite unrelated SQL.
      if (!view.state.doc.eq(document)) throw new Error(t("editor.inlineAi.editorChanged"));
      writeOption(view, answered[0], !!selectedSql);
      closeInlineAi();
      return;
    }
    // Options mode: preview only. The editor changes when the user accepts.
    conversation.value = appendInlineSqlTurn(conversation.value, request, answered);
    void nextTick(focusInput);
  } catch (e: any) {
    if (currentRequest !== requestId) return;
    error.value = String(e?.message || e);
  } finally {
    if (currentRequest === requestId) {
      loading.value = false;
      status.value = "";
    }
  }
}

function rememberPrompt(request: string) {
  const existing = promptHistory.indexOf(request);
  if (existing >= 0) promptHistory.splice(existing, 1);
  promptHistory.push(request);
  if (promptHistory.length > MAX_PROMPT_HISTORY) promptHistory.shift();
  historyIndex = promptHistory.length;
}

/** Arrow up/down on a single-line prompt walks earlier requests, like a shell. */
function recallPrompt(step: -1 | 1): boolean {
  if (promptHistory.length === 0 || prompt.value.includes("\n")) return false;
  const next = historyIndex + step;
  if (next < 0 || next > promptHistory.length) return false;
  historyIndex = next;
  prompt.value = promptHistory[next] ?? "";
  return true;
}

/** Runs what a key means for the panel. Returns false when the key is not the panel's to handle. */
function handleKey(event: KeyboardEvent): boolean {
  const action = inlineAiKeyAction({
    key: event.key,
    shiftKey: event.shiftKey,
    altKey: event.altKey,
    isComposing: event.isComposing,
    keyCode: event.keyCode,
    promptEmpty: !prompt.value.trim(),
    optionsMode: optionsMode.value,
    loading: loading.value,
  });
  switch (action) {
    case "close":
      closeInlineAi();
      return true;
    case "send":
      void submit();
      return true;
    case "accept":
      acceptOption();
      return true;
    case "previous-option":
    case "next-option":
      showOption(action === "previous-option" ? -1 : 1);
      return true;
    case "previous-turn":
    case "next-turn":
      conversation.value = stepInlineSqlTurn(conversation.value, action === "previous-turn" ? -1 : 1);
      return true;
    case "history-previous":
    case "history-next":
      return recallPrompt(action === "history-previous" ? -1 : 1);
    default:
      return false;
  }
}

function onKeydown(event: KeyboardEvent) {
  event.stopPropagation();
  if (handleKey(event)) event.preventDefault();
}

/**
 * Keys pressed while focus is somewhere other than the prompt. Without this
 * the panel only listens while its textarea has focus, so a click on one of
 * its buttons, or back in the editor, leaves Esc and Enter doing nothing.
 *
 * From the editor only Esc is taken: Enter there must stay a line break. It is
 * stopped before CodeMirror sees it, so the Vim keymap does not also treat it
 * as "leave insert mode".
 */
function onWindowKeydown(event: KeyboardEvent) {
  if (!visible.value) return;
  const view = props.view;
  const eventTarget = event.target instanceof Node ? event.target : null;
  if (!view || !eventTarget || eventTarget === inputRef.value) return;
  const insidePanel = !!panelRef.value?.contains(eventTarget);
  const insideEditorArea = insidePanel || !!panelHost(view)?.contains(eventTarget);
  if (!insideEditorArea) return;
  if (!insidePanel && event.key !== "Escape") return;
  // A focused button answers Enter itself; acting here as well would do two things at once.
  if (event.key === "Enter" && eventTarget instanceof HTMLButtonElement) return;
  if (handleKey(event)) {
    event.preventDefault();
    event.stopPropagation();
  }
}

/** A click on the panel's own surface returns focus to the prompt, unless the user is selecting preview text. */
function onPanelClick() {
  if (window.getSelection()?.toString()) return;
  focusInput();
}

defineExpose({ openInlineAi, closeInlineAi });
</script>

<template>
  <div
    v-if="visible"
    ref="panelRef"
    data-editor-inline-ai
    tabindex="-1"
    class="absolute z-[9999] isolate flex max-h-[min(80%,34rem)] flex-col gap-1.5 rounded-lg border border-border bg-popover p-2 text-popover-foreground shadow-xl outline-none ring-1 ring-border/60"
    :class="{ 'select-none': isDragging }"
    :style="panelStyle"
    @mousedown.stop
    @click.stop="onPanelClick"
    @contextmenu.stop
  >
    <template v-if="optionsMode">
      <div v-if="conversation.turns.length > 1" class="editor-inline-ai-scroll flex max-h-20 shrink-0 flex-col gap-0.5 overflow-y-auto pl-6" data-inline-ai-history>
        <button
          v-for="(turn, turnIndex) in conversation.turns"
          :key="turnIndex"
          type="button"
          class="flex w-full items-center gap-1.5 rounded px-1.5 py-0.5 text-left text-[11px] hover:bg-accent hover:text-accent-foreground"
          :class="turnIndex === conversation.turnIndex ? 'bg-accent/70 text-foreground' : 'text-muted-foreground'"
          :title="turn.request"
          @click.stop="
            showTurn(turnIndex);
            focusInput();
          "
        >
          <span class="shrink-0 tabular-nums opacity-70">{{ turnIndex + 1 }}.</span>
          <span class="min-w-0 flex-1 truncate">{{ turn.request }}</span>
          <span v-if="turn.options.length > 1" class="shrink-0 tabular-nums opacity-70">{{ turn.options.length }}</span>
        </button>
      </div>
      <pre v-if="previewOption" class="editor-inline-ai-scroll ml-6 min-h-0 flex-1 select-text overflow-auto whitespace-pre rounded border border-border/70 bg-muted/30 p-2 font-mono text-xs leading-relaxed text-foreground" data-inline-ai-preview>{{ previewOption.sql }}</pre>
      <div class="flex shrink-0 items-center gap-1 pl-6 text-xs" data-inline-ai-options>
        <template v-if="(previewTurn?.options.length ?? 0) > 1">
          <button
            type="button"
            class="flex h-5 w-5 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground"
            :title="t('editor.inlineAi.optionPrevious')"
            :aria-label="t('editor.inlineAi.optionPrevious')"
            @click.stop="
              showOption(-1);
              focusInput();
            "
          >
            <ChevronLeft class="h-3.5 w-3.5" />
          </button>
          <span class="shrink-0 tabular-nums text-muted-foreground" aria-live="polite">{{ conversation.optionIndex + 1 }}/{{ previewTurn?.options.length }}</span>
          <button
            type="button"
            class="flex h-5 w-5 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground"
            :title="t('editor.inlineAi.optionNext')"
            :aria-label="t('editor.inlineAi.optionNext')"
            @click.stop="
              showOption(1);
              focusInput();
            "
          >
            <ChevronRight class="h-3.5 w-3.5" />
          </button>
        </template>
        <span class="min-w-0 flex-1 truncate font-medium text-foreground">{{ previewOption?.label || (previewTurn && previewTurn.options.length > 1 ? t("editor.inlineAi.optionUnnamed", { index: conversation.optionIndex + 1 }) : "") }}</span>
        <button type="button" class="inline-flex h-6 shrink-0 items-center gap-1 rounded bg-primary px-2 text-[11px] font-medium text-primary-foreground hover:opacity-90 disabled:opacity-50" :disabled="loading || !previewOption" data-inline-ai-accept @click.stop="acceptOption">
          <Check class="h-3 w-3" />
          {{ t("editor.inlineAi.accept") }}
        </button>
      </div>
    </template>
    <div class="flex shrink-0 items-start gap-2">
      <span class="mt-1 flex h-5 w-4 shrink-0 cursor-grab items-center justify-center text-primary active:cursor-grabbing" :class="{ 'cursor-grabbing': isDragging }" :title="t('editor.inlineAi.dragHandle')" data-inline-ai-drag-handle @pointerdown.stop="onDragPointerDown">
        <Loader2 v-if="loading" class="h-4 w-4 animate-spin" />
        <GripVertical v-else-if="isDragging" class="h-4 w-4" />
        <Sparkles v-else class="h-4 w-4" />
      </span>
      <textarea
        ref="inputRef"
        v-model="prompt"
        rows="1"
        autocapitalize="off"
        autocorrect="off"
        spellcheck="false"
        class="editor-inline-ai-scroll max-h-40 min-h-[1.75rem] min-w-0 flex-1 resize-none overflow-y-auto bg-transparent py-1 text-sm text-foreground outline-none placeholder:text-muted-foreground"
        :placeholder="placeholder"
        :aria-label="placeholder"
        :readonly="loading"
        :class="{ 'opacity-60': loading }"
        @keydown="onKeydown"
        @keyup.stop
        @paste.stop
        @copy.stop
        @cut.stop
      />
      <button type="button" class="flex h-6 w-6 shrink-0 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground" :title="t('editor.inlineAi.close')" :aria-label="t('editor.inlineAi.close')" @click.stop="closeInlineAi()">
        <X class="h-3.5 w-3.5" />
      </button>
    </div>
    <div class="flex shrink-0 items-center justify-between gap-2 pl-6 text-[11px] text-muted-foreground">
      <span v-if="error" class="min-w-0 flex-1 select-text break-words text-destructive" aria-live="polite">{{ error }}</span>
      <span v-else-if="loading" class="min-w-0 flex-1 truncate">{{ status || t("editor.inlineAi.generating") }}</span>
      <span v-else-if="optionsMode" class="min-w-0 flex-1 truncate">{{ t("editor.inlineAi.optionHint") }}</span>
      <span v-else class="min-w-0 flex-1 truncate">{{ t(hasSelection ? "editor.inlineAi.hintEdit" : "editor.inlineAi.hintGenerate") }}</span>
      <span class="shrink-0">{{ t(optionsMode ? "editor.inlineAi.optionKeys" : "editor.inlineAi.keys") }}</span>
    </div>
    <span v-if="optionsMode" class="editor-inline-ai-resize" :title="t('editor.inlineAi.resizeHandle')" data-inline-ai-resize-handle @pointerdown.stop="onResizePointerDown" />
  </div>
</template>

<style>
/* One slim scrollbar for everything that scrolls inside the panel: the prompt,
   the SQL preview (both directions) and the turn history. */
.editor-inline-ai-scroll {
  scrollbar-width: thin;
  scrollbar-color: color-mix(in srgb, var(--muted-foreground) 45%, transparent) transparent;
}

.editor-inline-ai-scroll::-webkit-scrollbar {
  width: 5px;
  height: 5px;
}

.editor-inline-ai-scroll::-webkit-scrollbar-track,
.editor-inline-ai-scroll::-webkit-scrollbar-corner {
  background: transparent;
}

.editor-inline-ai-scroll::-webkit-scrollbar-thumb {
  border-radius: 999px;
  background: color-mix(in srgb, var(--muted-foreground) 45%, transparent);
}

.editor-inline-ai-scroll::-webkit-scrollbar-thumb:hover {
  background: color-mix(in srgb, var(--muted-foreground) 70%, transparent);
}

/* Corner grip for resizing the options panel. */
.editor-inline-ai-resize {
  position: absolute;
  right: 1px;
  bottom: 1px;
  width: 14px;
  height: 14px;
  cursor: nwse-resize;
  touch-action: none;
  border-bottom-right-radius: 0.5rem;
  background: linear-gradient(135deg, transparent 0 55%, color-mix(in srgb, var(--muted-foreground) 55%, transparent) 55% 62%, transparent 62% 72%, color-mix(in srgb, var(--muted-foreground) 55%, transparent) 72% 79%, transparent 79%);
}

.editor-inline-ai-resize:hover {
  background: linear-gradient(135deg, transparent 0 55%, var(--primary) 55% 62%, transparent 62% 72%, var(--primary) 72% 79%, transparent 79%);
}
</style>
