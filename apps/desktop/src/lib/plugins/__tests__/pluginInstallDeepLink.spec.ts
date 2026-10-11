import { describe, expect, it } from "vitest";
import { parsePluginInstallDeepLink } from "../pluginInstallDeepLink";

const PACKAGE_URL = "https://dl.dbxio.com/plugins/io.sqdesk.ssh/0.4.73/io.sqdesk.ssh-0.4.73-darwin-arm64.sqdeskp";

describe("parsePluginInstallDeepLink", () => {
  it("extracts the encoded package url", () => {
    const draft = parsePluginInstallDeepLink(`sqdesk://plugins/install?url=${encodeURIComponent(PACKAGE_URL)}`);
    expect(draft).toEqual({ url: PACKAGE_URL });
  });

  it("returns null for non-plugin links and junk", () => {
    expect(parsePluginInstallDeepLink("sqdesk://connection/new?type=mysql")).toBeNull();
    expect(parsePluginInstallDeepLink("sqdesk://plugins/installed?url=https://example.com/a.sqdeskp")).toBeNull();
    expect(parsePluginInstallDeepLink("sqdesk://plugins/installation?url=https://example.com/a.sqdeskp")).toBeNull();
    expect(parsePluginInstallDeepLink("")).toBeNull();
    expect(parsePluginInstallDeepLink("not a url")).toBeNull();
  });

  it("throws for matching links without a usable url param", () => {
    expect(() => parsePluginInstallDeepLink("sqdesk://plugins/install")).toThrow(/Missing url/);
    expect(() => parsePluginInstallDeepLink("sqdesk://plugins/install?url=")).toThrow(/Missing url/);
    expect(() => parsePluginInstallDeepLink(`sqdesk://plugins/install?url=${encodeURIComponent("ftp://example.com/a.sqdeskp")}`)).toThrow(/http/);
    expect(() => parsePluginInstallDeepLink(`sqdesk://plugins/install?url=${encodeURIComponent(`https://example.com/${"a".repeat(2100)}`)}`)).toThrow(/too long/);
  });
});
