<script setup lang="ts">
import { onBeforeUnmount, reactive } from "vue";
import type { CSSProperties } from "vue";
import { X } from "@lucide/vue";

/**
 * Filter boxes under the grid header. Typing state lives here so a keystroke
 * re-renders this row only, not the whole grid; the grid hears about a change
 * once the user pauses or presses Enter.
 */
defineProps<{
  columns: ReadonlyArray<{ key: string; name: string; style?: CSSProperties }>;
  beforeWidth: number;
  afterWidth: number;
  showScrollbarGutter: boolean;
  placeholder: string;
  hint: string;
}>();

const emit = defineEmits<{
  apply: [inputs: Record<string, string>];
}>();

const APPLY_DELAY_MS = 500;
const values = reactive<Record<string, string>>({});
let timer: ReturnType<typeof setTimeout> | undefined;

function applyNow() {
  clearTimeout(timer);
  emit("apply", { ...values });
}

function onInput(name: string, value: string) {
  if (value) values[name] = value;
  else delete values[name];
  clearTimeout(timer);
  timer = setTimeout(applyNow, APPLY_DELAY_MS);
}

function clearColumn(name: string) {
  if (!values[name]) return;
  delete values[name];
  applyNow();
}

/** Empties every box without notifying the grid; the caller already knows. */
function reset() {
  clearTimeout(timer);
  for (const name of Object.keys(values)) delete values[name];
}

onBeforeUnmount(() => clearTimeout(timer));

defineExpose({ reset });
</script>

<template>
  <div class="data-grid-inline-filter-row flex w-(--header-total-w) border-t border-border bg-background">
    <div class="shrink-0 border-r border-border w-(--row-num-w)" />
    <div class="shrink-0" :style="{ width: `${beforeWidth}px` }" />
    <div v-for="col in columns" :key="col.key" class="relative shrink-0 border-r border-border" :style="col.style">
      <input
        type="text"
        class="h-6 w-full min-w-0 bg-transparent px-2 pr-5 text-xs font-normal outline-none placeholder:text-muted-foreground/50 focus:bg-accent/40"
        :class="{ 'bg-primary/10 text-foreground': !!values[col.name] }"
        :value="values[col.name] ?? ''"
        :placeholder="placeholder"
        :title="hint"
        spellcheck="false"
        autocomplete="off"
        @input="onInput(col.name, ($event.target as HTMLInputElement).value)"
        @keydown.enter.prevent="applyNow"
        @keydown.escape.prevent="clearColumn(col.name)"
        @keydown.stop
        @keyup.stop
        @mousedown.stop
        @click.stop
        @dblclick.stop
        @contextmenu.stop
        @paste.stop
        @copy.stop
        @cut.stop
      />
      <button v-if="values[col.name]" type="button" class="absolute right-1 top-1/2 flex h-4 w-4 -translate-y-1/2 items-center justify-center rounded text-muted-foreground hover:text-foreground" :aria-label="placeholder" @mousedown.stop @click.stop="clearColumn(col.name)">
        <X class="h-3 w-3" />
      </button>
    </div>
    <div class="shrink-0" :style="{ width: `${afterWidth}px` }" />
    <div v-if="showScrollbarGutter" class="shrink-0 border-l border-border w-(--grid-scrollbar-gutter)" />
  </div>
</template>
