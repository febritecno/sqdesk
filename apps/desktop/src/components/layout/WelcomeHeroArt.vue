<template>
  <!-- Decorative loop: a query is typed, then its result rows arrive. -->
  <svg class="welcome-art" viewBox="0 0 440 320" fill="none" aria-hidden="true">
    <defs>
      <pattern id="welcome-art-dots" width="16" height="16" patternUnits="userSpaceOnUse">
        <circle cx="1.5" cy="1.5" r="1" class="welcome-art-dot" />
      </pattern>
    </defs>
    <rect width="440" height="320" rx="18" fill="url(#welcome-art-dots)" />

    <!-- Editor -->
    <g>
      <rect x="28" y="26" width="300" height="132" rx="12" class="welcome-art-surface" />
      <circle cx="46" cy="44" r="3.5" class="welcome-art-chrome" />
      <circle cx="58" cy="44" r="3.5" class="welcome-art-chrome" />
      <circle cx="70" cy="44" r="3.5" class="welcome-art-chrome" />
      <rect x="46" y="66" width="52" height="8" rx="4" class="welcome-art-keyword welcome-art-line" style="--i: 0" />
      <rect x="106" y="66" width="118" height="8" rx="4" class="welcome-art-text welcome-art-line" style="--i: 1" />
      <rect x="46" y="88" width="40" height="8" rx="4" class="welcome-art-keyword welcome-art-line" style="--i: 2" />
      <rect x="94" y="88" width="84" height="8" rx="4" class="welcome-art-text welcome-art-line" style="--i: 3" />
      <rect x="46" y="110" width="46" height="8" rx="4" class="welcome-art-keyword welcome-art-line" style="--i: 4" />
      <rect x="100" y="110" width="150" height="8" rx="4" class="welcome-art-text welcome-art-line" style="--i: 5" />
      <rect x="46" y="132" width="2" height="12" rx="1" class="welcome-art-caret" />
    </g>

    <!-- Assistant prompt -->
    <g class="welcome-art-prompt">
      <rect x="236" y="118" width="176" height="44" rx="10" class="welcome-art-surface welcome-art-surface--raised" />
      <path d="M256 131l2.6 6.4 6.4 2.6-6.4 2.6-2.6 6.4-2.6-6.4-6.4-2.6 6.4-2.6z" class="welcome-art-spark" />
      <rect x="274" y="132" width="96" height="7" rx="3.5" class="welcome-art-text" />
      <rect x="274" y="145" width="58" height="7" rx="3.5" class="welcome-art-muted" />
    </g>

    <!-- Result grid -->
    <g>
      <rect x="64" y="178" width="348" height="118" rx="12" class="welcome-art-surface" />
      <rect x="64" y="178" width="348" height="26" rx="12" class="welcome-art-header" />
      <rect x="64" y="192" width="348" height="12" class="welcome-art-header" />
      <rect x="82" y="188" width="38" height="6" rx="3" class="welcome-art-muted" />
      <rect x="170" y="188" width="64" height="6" rx="3" class="welcome-art-muted" />
      <rect x="290" y="188" width="46" height="6" rx="3" class="welcome-art-muted" />
      <g v-for="row in 4" :key="row" class="welcome-art-row" :style="{ '--i': row - 1 }">
        <rect x="82" :y="198 + row * 20" width="26" height="7" rx="3.5" class="welcome-art-number" />
        <rect x="170" :y="198 + row * 20" :width="[92, 70, 104, 58][row - 1]" height="7" rx="3.5" class="welcome-art-text" />
        <rect x="290" :y="198 + row * 20" :width="[54, 76, 40, 66][row - 1]" height="7" rx="3.5" class="welcome-art-text" />
        <line x1="76" x2="400" :y1="211 + row * 20" :y2="211 + row * 20" class="welcome-art-rule" />
      </g>
    </g>
  </svg>
</template>

<style>
.welcome-art {
  display: block;
  width: 100%;
  height: auto;
}

.welcome-art-dot {
  fill: color-mix(in srgb, var(--foreground) 9%, transparent);
}

.welcome-art-surface {
  fill: var(--background);
  stroke: color-mix(in srgb, var(--border) 90%, transparent);
}

.welcome-art-surface--raised {
  filter: drop-shadow(0 6px 14px color-mix(in srgb, var(--primary) 18%, transparent));
}

.welcome-art-header {
  fill: color-mix(in srgb, var(--muted) 70%, var(--background));
}

.welcome-art-chrome,
.welcome-art-muted {
  fill: color-mix(in srgb, var(--muted-foreground) 32%, transparent);
}

.welcome-art-keyword {
  fill: color-mix(in srgb, var(--primary) 78%, transparent);
}

.welcome-art-text {
  fill: color-mix(in srgb, var(--foreground) 26%, transparent);
}

.welcome-art-number {
  fill: color-mix(in srgb, var(--primary) 42%, transparent);
}

.welcome-art-rule {
  stroke: color-mix(in srgb, var(--border) 60%, transparent);
}

.welcome-art-caret,
.welcome-art-spark {
  fill: var(--primary);
}

.welcome-art-line,
.welcome-art-row,
.welcome-art-prompt,
.welcome-art-spark {
  transform-box: fill-box;
}

.welcome-art-line {
  transform-origin: left center;
  animation: welcome-art-line 9s cubic-bezier(0.22, 1, 0.36, 1) infinite;
  animation-delay: calc(var(--i) * 0.28s);
}

.welcome-art-caret {
  animation: welcome-art-caret 1.1s steps(2, jump-none) infinite;
}

.welcome-art-prompt {
  transform-origin: center;
  animation: welcome-art-pop 9s cubic-bezier(0.22, 1, 0.36, 1) infinite;
  animation-delay: 2s;
}

.welcome-art-spark {
  transform-origin: center;
  animation: welcome-art-spark 2.4s ease-in-out infinite;
}

.welcome-art-row {
  animation: welcome-art-row 9s cubic-bezier(0.22, 1, 0.36, 1) infinite;
  animation-delay: calc(3.2s + var(--i) * 0.22s);
}

@keyframes welcome-art-line {
  0% {
    opacity: 0;
    transform: scaleX(0);
  }
  4% {
    opacity: 1;
  }
  10%,
  88% {
    opacity: 1;
    transform: scaleX(1);
  }
  96%,
  100% {
    opacity: 0;
    transform: scaleX(1);
  }
}

@keyframes welcome-art-row {
  0% {
    opacity: 0;
    transform: translateY(8px);
  }
  8%,
  60% {
    opacity: 1;
    transform: translateY(0);
  }
  68%,
  100% {
    opacity: 0;
    transform: translateY(0);
  }
}

@keyframes welcome-art-pop {
  0% {
    opacity: 0;
    transform: translateY(6px) scale(0.96);
  }
  7%,
  70% {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
  78%,
  100% {
    opacity: 0;
    transform: translateY(0) scale(1);
  }
}

@keyframes welcome-art-spark {
  0%,
  100% {
    transform: scale(0.85) rotate(0deg);
  }
  50% {
    transform: scale(1.12) rotate(18deg);
  }
}

@keyframes welcome-art-caret {
  0% {
    opacity: 1;
  }
  100% {
    opacity: 0;
  }
}

/* Motion is decoration here: show the finished picture instead. */
@media (prefers-reduced-motion: reduce) {
  .welcome-art-line,
  .welcome-art-row,
  .welcome-art-prompt,
  .welcome-art-spark,
  .welcome-art-caret {
    animation: none;
  }
}
</style>
