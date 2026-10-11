import { describe, expect, it } from "vitest";
import { isPluginGraphicsEngineEnabled, normalizePluginGraphicsEngineIds, withPluginGraphicsEngineEnabled } from "./pluginGraphicsEngine";

describe("pluginGraphicsEngine", () => {
  it("defaults to no grants and drops junk entries", () => {
    expect(normalizePluginGraphicsEngineIds(undefined)).toEqual([]);
    expect(normalizePluginGraphicsEngineIds("io.github.t8y2.s3")).toEqual([]);
    expect(normalizePluginGraphicsEngineIds([null, 7, "", "  ", "io.github.t8y2.s3", " io.github.t8y2.s3 "])).toEqual(["io.github.t8y2.s3"]);
  });

  it("grants and revokes a single plugin without touching the others", () => {
    const granted = withPluginGraphicsEngineEnabled(["io.sqdesk.chart"], "io.github.t8y2.s3", true);
    expect(granted).toEqual(["io.sqdesk.chart", "io.github.t8y2.s3"]);
    expect(isPluginGraphicsEngineEnabled(granted, "io.github.t8y2.s3")).toBe(true);
    expect(isPluginGraphicsEngineEnabled(granted, "io.sqdesk.other")).toBe(false);

    const revoked = withPluginGraphicsEngineEnabled(granted, "io.github.t8y2.s3", false);
    expect(revoked).toEqual(["io.sqdesk.chart"]);
    expect(isPluginGraphicsEngineEnabled(revoked, "io.github.t8y2.s3")).toBe(false);
  });

  it("never grants an empty plugin id", () => {
    expect(withPluginGraphicsEngineEnabled([], "", true)).toEqual([]);
    expect(isPluginGraphicsEngineEnabled([""], "")).toBe(false);
  });

  it("keeps a re-grant idempotent", () => {
    const once = withPluginGraphicsEngineEnabled([], "io.sqdesk.chart", true);
    expect(withPluginGraphicsEngineEnabled(once, "io.sqdesk.chart", true)).toEqual(once);
  });
});
