<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { ChevronDown } from "@lucide/vue";

/**
 * A titled block of related settings, laid out as an annotated row: the title
 * and a one-line explanation sit beside a panel that lists the options as
 * divided rows. Collapsible groups keep rarely used options out of the way and
 * remember whether the user left them open.
 */
const props = withDefaults(
  defineProps<{
    title: string;
    description?: string;
    collapsible?: boolean;
    defaultOpen?: boolean;
    /** Identifies the group for the jump bar and persists its open state. */
    storageKey?: string;
    /** Opens the group regardless of its saved state, e.g. while a search result inside it is highlighted. */
    forceOpen?: boolean;
  }>(),
  { defaultOpen: true },
);

const { t } = useI18n();
const rootRef = ref<HTMLElement>();

function readStoredOpen(): boolean | null {
  if (!props.storageKey) return null;
  try {
    const stored = localStorage.getItem(`sqdesk-settings-group:${props.storageKey}`);
    return stored === null ? null : stored === "1";
  } catch {
    return null;
  }
}

const open = ref(readStoredOpen() ?? props.defaultOpen);
const isOpen = computed(() => !props.collapsible || props.forceOpen || open.value);

function setOpen(next: boolean) {
  open.value = next;
  if (!props.storageKey) return;
  try {
    localStorage.setItem(`sqdesk-settings-group:${props.storageKey}`, next ? "1" : "0");
  } catch {
    // Storage can be unavailable; the group still works for this session.
  }
}

function toggle() {
  if (props.collapsible) setOpen(!isOpen.value);
}

// The jump bar asks a group to reveal itself before scrolling to it.
const openFromJumpBar = () => setOpen(true);
onMounted(() => rootRef.value?.addEventListener("settings-group:open", openFromJumpBar));
onBeforeUnmount(() => rootRef.value?.removeEventListener("settings-group:open", openFromJumpBar));
</script>

<template>
  <section ref="rootRef" class="settings-group" :data-settings-group="storageKey" :data-settings-group-title="title">
    <div class="settings-group__layout">
      <component :is="collapsible ? 'button' : 'div'" :type="collapsible ? 'button' : undefined" class="settings-group__header" :aria-expanded="collapsible ? isOpen : undefined" @click="toggle">
        <span class="min-w-0">
          <span class="block text-sm font-semibold tracking-tight text-foreground">{{ title }}</span>
          <span v-if="description" class="mt-1 block text-xs leading-5 text-muted-foreground">{{ description }}</span>
        </span>
        <ChevronDown v-if="collapsible" class="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground transition-transform duration-200" :class="isOpen && 'rotate-180'" aria-hidden="true" />
      </component>
      <div class="min-w-0">
        <div v-show="isOpen" class="settings-group-body">
          <slot />
        </div>
        <button v-if="collapsible && !isOpen" type="button" class="settings-group__reveal" @click="toggle">
          {{ t("settings.groupShow") }}
        </button>
      </div>
    </div>
  </section>
</template>

<style>
.settings-group {
  container-type: inline-size;
  padding-block: 1.25rem;
  scroll-margin-top: 3.25rem;
}

.settings-group + .settings-group {
  border-top: 1px solid color-mix(in oklab, var(--border) 70%, transparent);
}

.settings-group__layout {
  display: grid;
  gap: 0.75rem;
}

.settings-group__header {
  display: flex;
  width: 100%;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.5rem;
  border-radius: 0.5rem;
  text-align: left;
}

button.settings-group__header {
  margin: -0.375rem;
  width: calc(100% + 0.75rem);
  padding: 0.375rem;
  cursor: pointer;
  transition: background-color 150ms;
}

button.settings-group__header:hover {
  background: color-mix(in oklab, var(--muted) 55%, transparent);
}

button.settings-group__header:focus-visible,
.settings-group__reveal:focus-visible {
  outline: 2px solid var(--ring);
  outline-offset: 2px;
}

/* Wide enough for two columns: explanation on the left, options on the right. */
@container (min-width: 46rem) {
  .settings-group__layout {
    grid-template-columns: minmax(0, 15rem) minmax(0, 1fr);
    align-items: start;
    gap: 2rem;
  }

  .settings-group__header {
    position: sticky;
    top: 3.25rem;
  }
}

.settings-group-body {
  border: 1px solid color-mix(in oklab, var(--border) 85%, transparent);
  border-radius: 0.75rem;
  background: color-mix(in oklab, var(--muted) 16%, transparent);
  padding-inline: 1rem;
}

/* Options read as rows of one list: a hairline between them, no box per row. */
.settings-group-body > * {
  padding-block: 0.875rem;
}

.settings-group-body > * + * {
  border-top: 1px solid color-mix(in oklab, var(--border) 60%, transparent);
}

.settings-group-body .settings-item {
  border-width: 0 !important;
  border-radius: 0 !important;
  background: transparent !important;
  padding-inline: 0 !important;
}

.settings-group-body > .settings-item {
  padding-block: 0.875rem !important;
}

.settings-group-body > .settings-item + *,
.settings-group-body > * + .settings-item {
  border-top: 1px solid color-mix(in oklab, var(--border) 60%, transparent) !important;
}

.settings-group__reveal {
  width: 100%;
  border: 1px dashed color-mix(in oklab, var(--border) 90%, transparent);
  border-radius: 0.75rem;
  padding: 0.625rem 1rem;
  text-align: left;
  font-size: 0.75rem;
  color: var(--muted-foreground);
  transition:
    background-color 150ms,
    color 150ms;
}

.settings-group__reveal:hover {
  background: color-mix(in oklab, var(--muted) 45%, transparent);
  color: var(--foreground);
}
</style>
