<script setup lang="ts">
import type { SwitchRootEmits, SwitchRootProps } from "reka-ui";
import type { HTMLAttributes } from "vue";
import { reactiveOmit } from "@vueuse/core";
import { SwitchRoot, SwitchThumb, useForwardPropsEmits } from "reka-ui";
import { cn } from "@/lib/common/utils";

const props = withDefaults(
  defineProps<
    SwitchRootProps & {
      class?: HTMLAttributes["class"];
      size?: "sm" | "default";
    }
  >(),
  {
    size: "default",
  },
);

const emits = defineEmits<SwitchRootEmits>();

const delegatedProps = reactiveOmit(props, "class", "size");

const forwarded = useForwardPropsEmits(delegatedProps, emits);
</script>

<template>
  <SwitchRoot
    v-slot="slotProps"
    data-slot="switch"
    :data-size="size"
    v-bind="forwarded"
    :class="
      cn(
        'sqdesk-switch data-checked:bg-primary data-unchecked:bg-input focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:aria-invalid:border-destructive/50 dark:data-unchecked:bg-input/80 shrink-0 rounded-full border border-transparent focus-visible:ring-3 aria-invalid:ring-3 data-[size=default]:h-[18.4px] data-[size=default]:w-[32px] data-[size=sm]:h-[14px] data-[size=sm]:w-[24px] peer group/switch relative inline-flex items-center transition-colors outline-none after:absolute after:-inset-x-3 after:-inset-y-2 data-disabled:cursor-not-allowed data-disabled:opacity-50',
        props.class,
      )
    "
  >
    <SwitchThumb data-slot="switch-thumb" class="sqdesk-switch-thumb bg-background dark:data-unchecked:bg-foreground dark:data-checked:bg-primary-foreground rounded-full group-data-[size=default]/switch:size-4 group-data-[size=sm]/switch:size-3 pointer-events-none block ring-0 transition-transform">
      <slot name="thumb" v-bind="slotProps" />
    </SwitchThumb>
  </SwitchRoot>
</template>

<style>
.sqdesk-switch {
  box-sizing: border-box;
  display: inline-flex !important;
  align-items: center !important;
  position: relative;
  flex-shrink: 0;
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--border) 80%, transparent) !important;
  background-color: var(--input) !important;
  vertical-align: middle;
}

.sqdesk-switch[data-size="default"] {
  width: 32px !important;
  height: 18.4px !important;
}

.sqdesk-switch[data-size="sm"] {
  width: 24px !important;
  height: 14px !important;
}

.sqdesk-switch-thumb {
  display: block !important;
  border-radius: 9999px;
  background-color: var(--background) !important;
  box-shadow: 0 1px 2px color-mix(in srgb, var(--foreground) 18%, transparent);
  transform: translateX(0) !important;
}

.sqdesk-switch[data-size="default"] .sqdesk-switch-thumb {
  width: 16px !important;
  height: 16px !important;
}

.sqdesk-switch[data-size="sm"] .sqdesk-switch-thumb {
  width: 12px !important;
  height: 12px !important;
}

.sqdesk-switch[data-state="checked"],
.sqdesk-switch[data-checked],
.sqdesk-switch[aria-checked="true"] {
  border-color: var(--primary) !important;
  background-color: var(--primary) !important;
}

.sqdesk-switch[data-size="default"][data-state="checked"] .sqdesk-switch-thumb,
.sqdesk-switch[data-size="default"][data-checked] .sqdesk-switch-thumb,
.sqdesk-switch[data-size="default"][aria-checked="true"] .sqdesk-switch-thumb {
  transform: translateX(14px) !important;
}

.sqdesk-switch[data-size="sm"][data-state="checked"] .sqdesk-switch-thumb,
.sqdesk-switch[data-size="sm"][data-checked] .sqdesk-switch-thumb,
.sqdesk-switch[data-size="sm"][aria-checked="true"] .sqdesk-switch-thumb {
  transform: translateX(10px) !important;
}

.sqdesk-switch[data-state="checked"] .sqdesk-switch-thumb,
.sqdesk-switch[data-checked] .sqdesk-switch-thumb,
.sqdesk-switch[aria-checked="true"] .sqdesk-switch-thumb {
  background-color: var(--primary-foreground) !important;
}

.dark .sqdesk-switch {
  border-color: color-mix(in srgb, var(--border) 90%, transparent) !important;
  background-color: color-mix(in srgb, var(--input) 80%, transparent) !important;
}

.dark .sqdesk-switch[data-state="checked"],
.dark .sqdesk-switch[data-checked],
.dark .sqdesk-switch[aria-checked="true"] {
  border-color: var(--primary) !important;
  background-color: var(--primary) !important;
}

.dark .sqdesk-switch[data-state="checked"] .sqdesk-switch-thumb,
.dark .sqdesk-switch[data-checked] .sqdesk-switch-thumb,
.dark .sqdesk-switch[aria-checked="true"] .sqdesk-switch-thumb {
  background-color: var(--primary-foreground) !important;
}

html.sqdesk-legacy-webview.dark .sqdesk-switch[data-state="unchecked"] .sqdesk-switch-thumb,
html.sqdesk-legacy-webview.dark .sqdesk-switch[data-unchecked] .sqdesk-switch-thumb,
html.sqdesk-legacy-webview.dark .sqdesk-switch[aria-checked="false"] .sqdesk-switch-thumb {
  background-color: rgb(215, 215, 219) !important;
}

html.sqdesk-legacy-webview.dark .sqdesk-switch {
  border-color: rgba(180, 180, 190, 0.46) !important;
  background-color: rgba(110, 110, 114, 0.44) !important;
}

html.sqdesk-legacy-webview.dark .sqdesk-switch[data-state="checked"],
html.sqdesk-legacy-webview.dark .sqdesk-switch[data-checked],
html.sqdesk-legacy-webview.dark .sqdesk-switch[aria-checked="true"] {
  border-color: rgb(208, 208, 214) !important;
  background-color: rgb(208, 208, 214) !important;
}

html.sqdesk-legacy-webview.dark .sqdesk-switch[data-state="checked"] .sqdesk-switch-thumb,
html.sqdesk-legacy-webview.dark .sqdesk-switch[data-checked] .sqdesk-switch-thumb,
html.sqdesk-legacy-webview.dark .sqdesk-switch[aria-checked="true"] .sqdesk-switch-thumb {
  background-color: rgb(19, 20, 22) !important;
}
</style>
