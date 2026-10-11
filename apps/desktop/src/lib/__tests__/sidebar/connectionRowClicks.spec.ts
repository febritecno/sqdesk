import { describe, expect, it } from "vitest";
import { connectionRowClickAction } from "@/lib/sidebar/connectionRowClicks";

describe("connectionRowClickAction", () => {
  it("opens on a single click and renames on exactly four", () => {
    expect(connectionRowClickAction(1)).toBe("open");
    expect(connectionRowClickAction(4)).toBe("rename");
  });

  it("does nothing for two, three, or more than four clicks", () => {
    for (const count of [0, 2, 3, 5, 6, 12]) expect(connectionRowClickAction(count)).toBe("none");
  });
});
