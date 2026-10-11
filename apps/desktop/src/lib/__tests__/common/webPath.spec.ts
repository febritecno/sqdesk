import { describe, expect, it } from "vitest";
import { apiUrl, apiWebSocketUrl, sqdeskWebBasePath, webPath } from "@/lib/common/webPath";

describe("webPath", () => {
  it("keeps root deployments on root-relative API paths", () => {
    expect(sqdeskWebBasePath("/", "/")).toBe("");
    expect(apiUrl("/api/auth/check", "")).toBe("/api/auth/check");
  });

  it("uses an explicit build base path", () => {
    expect(sqdeskWebBasePath("/", "/sqdesk/")).toBe("/sqdesk");
    expect(webPath("/login", "/sqdesk")).toBe("/sqdesk/login");
    expect(webPath("/", "/sqdesk")).toBe("/sqdesk/");
    expect(webPath("/icons/database/mysql.svg", "/sqdesk")).toBe("/sqdesk/icons/database/mysql.svg");
    expect(webPath("/icons/ai/openai.svg", "/sqdesk")).toBe("/sqdesk/icons/ai/openai.svg");
    expect(webPath("/logo.png", "/sqdesk")).toBe("/sqdesk/logo.png");
    expect(webPath("/logo-black.png", "/sqdesk")).toBe("/sqdesk/logo-black.png");
    expect(webPath("/icon-preview-default.png", "/sqdesk")).toBe("/sqdesk/icon-preview-default.png");
    expect(webPath("/icon-preview-black.png", "/sqdesk")).toBe("/sqdesk/icon-preview-black.png");
    expect(apiUrl("/auth/check", "/sqdesk")).toBe("/sqdesk/api/auth/check");
    expect(apiUrl("/api/auth/check", "/sqdesk")).toBe("/sqdesk/api/auth/check");
    expect(apiUrl("api/auth/check", "/sqdesk")).toBe("/sqdesk/api/auth/check");
  });

  it("infers the runtime base path from the login URL for relative builds", () => {
    expect(sqdeskWebBasePath("/sqdesk/login", "./")).toBe("/sqdesk");
    expect(sqdeskWebBasePath("/tools/sqdesk/login", "./")).toBe("/tools/sqdesk");
  });

  it("infers the runtime base path from a mounted relative build", () => {
    expect(sqdeskWebBasePath("/sqdesk/", "./")).toBe("/sqdesk");
    expect(sqdeskWebBasePath("/tools/sqdesk/", "./")).toBe("/tools/sqdesk");
  });

  it("builds websocket URLs with the configured base path", () => {
    expect(apiWebSocketUrl("/redis/session/123", "/sqdesk", { protocol: "https:", host: "example.test" })).toBe("wss://example.test/sqdesk/api/redis/session/123");
  });
});
