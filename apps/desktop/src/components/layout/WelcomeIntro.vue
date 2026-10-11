<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { ArrowRight, Code2, DatabaseZap, Download, History, Plus, Settings2, ShieldCheck, Sparkles, Table2 } from "@lucide/vue";
import AppLogo from "@/components/icons/AppLogo.vue";
import WelcomeHeroArt from "@/components/layout/WelcomeHeroArt.vue";

defineProps<{
  appVersion: string;
}>();

const emit = defineEmits<{
  "new-connection": [];
  "import-config": [];
  "show-history": [];
  "open-settings": [];
}>();

const { t } = useI18n();

const features = computed(() => [
  { icon: DatabaseZap, title: t("welcome.introFeatureConnections"), description: t("welcome.introFeatureConnectionsDescription") },
  { icon: Code2, title: t("welcome.introFeatureSql"), description: t("welcome.introFeatureSqlDescription") },
  { icon: Table2, title: t("welcome.introFeatureData"), description: t("welcome.introFeatureDataDescription") },
  { icon: Sparkles, title: t("welcome.introFeatureAi"), description: t("welcome.introFeatureAiDescription") },
]);
</script>

<template>
  <div class="welcome-intro mx-auto flex min-h-full w-full min-w-0 flex-col justify-center px-6 py-10 sm:px-10">
    <section class="welcome-intro-hero grid items-center gap-10">
      <div class="min-w-0">
        <div class="welcome-rise flex items-center gap-3" style="--i: 0">
          <div class="flex h-10 w-10 items-center justify-center rounded-xl border bg-background p-1.5"><AppLogo class="h-full w-full object-contain" /></div>
          <div class="text-xs font-medium text-muted-foreground">{{ t("welcome.introEyebrow") }}</div>
        </div>
        <h1 class="welcome-rise mt-6 text-3xl font-semibold leading-[1.1] tracking-tight sm:text-[2.5rem]" style="--i: 1; text-wrap: balance">{{ t("welcome.introTitle") }}</h1>
        <p class="welcome-rise mt-4 max-w-[56ch] text-sm leading-6 text-muted-foreground sm:text-[0.95rem]" style="--i: 2; text-wrap: pretty">{{ t("welcome.introDescription") }}</p>
        <div class="welcome-rise mt-7 flex flex-wrap items-center gap-2" style="--i: 3">
          <button type="button" class="welcome-intro-primary inline-flex h-9 items-center gap-2 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground" @click="emit('new-connection')">
            <Plus class="h-4 w-4" /> {{ t("welcome.introNewConnection") }} <ArrowRight class="welcome-intro-arrow h-3.5 w-3.5" />
          </button>
          <button type="button" class="welcome-intro-secondary inline-flex h-9 items-center gap-2 rounded-md border px-4 text-sm font-medium" @click="emit('import-config')"><Download class="h-4 w-4" /> {{ t("welcome.introImport") }}</button>
        </div>
      </div>
      <div class="welcome-rise welcome-intro-art min-w-0" style="--i: 2">
        <WelcomeHeroArt />
      </div>
    </section>

    <dl class="welcome-intro-features mt-12 grid gap-x-12">
      <div v-for="(feature, index) in features" :key="feature.title" class="welcome-rise welcome-intro-feature flex items-start gap-3 py-4" :style="{ '--i': 4 + index }">
        <div class="mt-0.5 flex h-8 w-8 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary"><component :is="feature.icon" class="h-4 w-4" /></div>
        <div class="min-w-0">
          <dt class="text-sm font-medium">{{ feature.title }}</dt>
          <dd class="mt-1 text-xs leading-5 text-muted-foreground">{{ feature.description }}</dd>
        </div>
      </div>
    </dl>

    <footer class="welcome-rise mt-8 flex flex-wrap items-center justify-between gap-3 text-xs text-muted-foreground" style="--i: 8">
      <div class="flex items-center gap-2"><ShieldCheck class="h-4 w-4 text-primary" /> {{ t("welcome.introPrivacyHint") }}</div>
      <div class="flex flex-wrap items-center gap-4">
        <button type="button" class="welcome-intro-link inline-flex items-center gap-1.5" @click="emit('show-history')"><History class="h-3.5 w-3.5" /> {{ t("history.title") }}</button>
        <button type="button" class="welcome-intro-link inline-flex items-center gap-1.5" @click="emit('open-settings')"><Settings2 class="h-3.5 w-3.5" /> {{ t("welcome.introSettings") }}</button>
        <span class="tabular-nums text-muted-foreground/60">SQDesk {{ appVersion ? "v" + appVersion : "" }}</span>
      </div>
    </footer>
  </div>
</template>

<style>
.welcome-intro {
  max-width: 68rem;
}

@media (min-width: 1024px) {
  .welcome-intro-hero {
    grid-template-columns: minmax(0, 1.05fr) minmax(0, 0.95fr);
  }
}

.welcome-intro-art {
  border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
  border-radius: 1.25rem;
  background: linear-gradient(160deg, color-mix(in srgb, var(--muted) 45%, var(--background)), var(--background) 70%);
  padding: 0.75rem;
}

/* A divided list rather than four equal cards. */
.welcome-intro-features {
  border-top: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
}

.welcome-intro-feature {
  border-bottom: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
}

@media (min-width: 640px) {
  .welcome-intro-features {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

.welcome-intro-primary,
.welcome-intro-secondary,
.welcome-intro-link {
  transition:
    background-color 180ms,
    color 180ms,
    transform 120ms;
}

.welcome-intro-primary:hover {
  background: color-mix(in srgb, var(--primary) 90%, var(--foreground));
}

.welcome-intro-secondary:hover {
  background: color-mix(in srgb, var(--muted) 70%, transparent);
}

.welcome-intro-link:hover {
  color: var(--foreground);
}

.welcome-intro-primary:active,
.welcome-intro-secondary:active {
  transform: translateY(1px);
}

.welcome-intro-primary:focus-visible,
.welcome-intro-secondary:focus-visible,
.welcome-intro-link:focus-visible {
  outline: 2px solid var(--ring);
  outline-offset: 2px;
}

.welcome-intro-arrow {
  transition: transform 180ms;
}

.welcome-intro-primary:hover .welcome-intro-arrow {
  transform: translateX(2px);
}

.welcome-rise {
  animation: welcome-rise 520ms cubic-bezier(0.22, 1, 0.36, 1) both;
  animation-delay: calc(var(--i, 0) * 60ms);
}

@keyframes welcome-rise {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (prefers-reduced-motion: reduce) {
  .welcome-rise {
    animation: none;
  }
}
</style>
