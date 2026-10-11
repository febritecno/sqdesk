// @vitest-environment happy-dom

import { beforeEach, describe, expect, it, vi } from "vitest";
import * as api from "@/lib/backend/api";
import type { InstalledPlugin } from "@/types/database";
import { clearPluginIconCache, resolvePluginIcon } from "@/lib/plugins/pluginIconResolver";

vi.mock("@/lib/backend/api", () => ({ listPlugins: vi.fn() }));

function plugin(id: string, icon: string): InstalledPlugin {
  return {
    manifest: { id, name: id, version: "1.0.0", icon, contributions: [] },
    compatibility: { compatible: true, errors: [], warnings: [] },
  } as unknown as InstalledPlugin;
}

describe("pluginIconResolver cache", () => {
  beforeEach(() => {
    clearPluginIconCache();
    vi.mocked(api.listPlugins).mockReset();
  });

  it("memoizes the installed-plugin list across resolves", async () => {
    vi.mocked(api.listPlugins).mockResolvedValue([plugin("a", "assets/a.svg")]);
    expect(await resolvePluginIcon("a")).toBe("assets/a.svg");
    expect(await resolvePluginIcon("a")).toBe("assets/a.svg");
    expect(api.listPlugins).toHaveBeenCalledTimes(1);
  });

  it("drops the stale cache on sqdesk:plugins-changed (batch uninstall path)", async () => {
    vi.mocked(api.listPlugins).mockResolvedValueOnce([plugin("old", "assets/old.svg")]);
    expect(await resolvePluginIcon("old")).toBe("assets/old.svg");
    expect(api.listPlugins).toHaveBeenCalledTimes(1);

    // Batch uninstall refreshes via sqdesk:plugins-changed only; the removed plugin must not
    // resolve from the cached list anymore.
    vi.mocked(api.listPlugins).mockResolvedValueOnce([]);
    window.dispatchEvent(new Event("sqdesk:plugins-changed"));

    expect(await resolvePluginIcon("old")).toBeUndefined();
    expect(api.listPlugins).toHaveBeenCalledTimes(2);
  });
});
