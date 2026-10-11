import { describe, expect, it } from "vitest";
import { isKeyInKvExportScope, kvDirectoryPrefix, kvExportFilenameStem, kvValueByteIdentity } from "@/lib/kv/kvExportScope";

describe("KV export scope", () => {
  it("matches a directory itself and descendants without leaking into adjacent prefixes", () => {
    const scope = { path: "/sqdesk", kind: "prefix" as const };

    expect(isKeyInKvExportScope("/sqdesk", scope)).toBe(true);
    expect(isKeyInKvExportScope("/sqdesk/app/name", scope)).toBe(true);
    expect(isKeyInKvExportScope("/sqdesk-other", scope)).toBe(false);
  });

  it("matches only the selected key for a key export", () => {
    const scope = { path: "sqdesk/key", kind: "key" as const };

    expect(isKeyInKvExportScope("sqdesk/key", scope)).toBe(true);
    expect(isKeyInKvExportScope("sqdesk/key/child", scope)).toBe(false);
  });

  it("builds stable directory prefixes and safe filenames", () => {
    expect(kvDirectoryPrefix("/sqdesk")).toBe("/sqdesk/");
    expect(kvDirectoryPrefix("sqdesk/")).toBe("sqdesk/");
    expect(kvExportFilenameStem("/应用 配置/prod")).toBe("应用-配置-prod");
    expect(kvExportFilenameStem("/")).toBe("root");
  });

  it("compares UTF-8 and Base64 representations by their original bytes", () => {
    expect(kvValueByteIdentity({ encoding: "utf8", data: "/sqdesk/app" })).toBe(kvValueByteIdentity({ encoding: "base64", data: "L3NxZGVzay9hcHA=" }));
    expect(kvValueByteIdentity({ encoding: "utf8", data: "/sqdesk/app" })).not.toBe(kvValueByteIdentity({ encoding: "utf8", data: "/sqdesk-other" }));
  });
});
