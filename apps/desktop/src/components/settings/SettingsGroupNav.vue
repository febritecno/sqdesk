<script setup lang="ts">
import { nextTick, onMounted, ref } from "vue";

/**
 * Jump bar for the current settings tab: one chip per group. It reads the
 * groups from the DOM so tabs do not have to declare their sections twice.
 * Mount it with a `key` per tab so it rescans when the tab changes.
 */
const rootRef = ref<HTMLElement>();
const groups = ref<Array<{ key: string; title: string }>>([]);

function groupElements(): HTMLElement[] {
  const container = rootRef.value?.parentElement;
  return container ? Array.from(container.querySelectorAll<HTMLElement>("[data-settings-group]")) : [];
}

function scan() {
  groups.value = groupElements().map((element) => ({ key: element.dataset.settingsGroup ?? "", title: element.dataset.settingsGroupTitle ?? "" }));
}

function jumpTo(key: string) {
  const element = groupElements().find((candidate) => candidate.dataset.settingsGroup === key);
  if (!element) return;
  element.dispatchEvent(new CustomEvent("settings-group:open"));
  void nextTick(() => element.scrollIntoView({ behavior: "smooth", block: "start" }));
}

// The tab's groups render as siblings after this bar; wait for them.
onMounted(() => void nextTick(scan));
</script>

<template>
  <nav ref="rootRef" class="settings-group-nav sticky top-0 z-10 -mx-1 flex gap-1.5 overflow-x-auto bg-background/95 px-1 py-2 backdrop-blur" :class="groups.length < 2 && 'hidden'">
    <button
      v-for="group in groups"
      :key="group.key"
      type="button"
      class="shrink-0 rounded-md border border-border/70 px-2.5 py-1 text-xs text-muted-foreground transition-colors hover:bg-accent hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      @click="jumpTo(group.key)"
    >
      {{ group.title }}
    </button>
  </nav>
</template>
