const STORAGE_KEY = "sqdesk.tabGroupAutoColors";

let assignedColors: Map<string, string> | null = null;

function loadAssignedColors(): Map<string, string> {
  if (assignedColors) return assignedColors;
  assignedColors = new Map();
  try {
    const stored: unknown = JSON.parse(window.localStorage.getItem(STORAGE_KEY) ?? "{}");
    if (stored && typeof stored === "object" && !Array.isArray(stored)) {
      for (const [key, color] of Object.entries(stored)) {
        if (typeof color === "string") assignedColors.set(key, color);
      }
    }
  } catch {
    // Storage can be unavailable or hold stale data; start with no assignments.
  }
  return assignedColors;
}

function storeAssignedColors(colors: Map<string, string>) {
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(Object.fromEntries(colors)));
  } catch {
    // Persisting the assignment is best-effort.
  }
}

/**
 * Default color for a tab group that has no custom color. The first group
 * gets a random palette color; every later group gets a random pick among the
 * colors used least so far, so neighbouring groups do not share a color until
 * the palette is exhausted. An assignment sticks to its key across restarts.
 */
export function autoTabGroupColor(key: string, palette: readonly string[]): string {
  const colors = loadAssignedColors();
  const existing = colors.get(key);
  if (existing && palette.includes(existing)) return existing;

  const usage = new Map(palette.map((color) => [color, 0]));
  for (const color of colors.values()) {
    if (usage.has(color)) usage.set(color, usage.get(color)! + 1);
  }
  const leastUsed = Math.min(...usage.values());
  const candidates = palette.filter((color) => usage.get(color) === leastUsed);
  const picked = candidates[Math.floor(Math.random() * candidates.length)] ?? palette[0]!;
  colors.set(key, picked);
  storeAssignedColors(colors);
  return picked;
}

/** Test hook: forget every assignment. */
export function resetAutoTabGroupColors() {
  assignedColors = new Map();
  storeAssignedColors(assignedColors);
}
