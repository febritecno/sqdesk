import { describe, expect, it } from "vitest";
import { migrateLegacyStorageKeys } from "@/lib/startup/legacyStorageKeys";

function memoryStorage(entries: Record<string, string>) {
  const map = new Map(Object.entries(entries));
  return {
    map,
    get length() {
      return map.size;
    },
    key: (index: number) => [...map.keys()][index] ?? null,
    getItem: (key: string) => map.get(key) ?? null,
    setItem: (key: string, value: string) => void map.set(key, value),
    removeItem: (key: string) => void map.delete(key),
  };
}

describe("migrateLegacyStorageKeys", () => {
  it("moves legacy keys to the current prefix and leaves other keys alone", () => {
    const storage = memoryStorage({ "dbx-recent-connection-ids-v1": "[1]", "dbx:code-snapshot-settings": "{}", theme: "dark" });

    expect(migrateLegacyStorageKeys(storage)).toBe(2);
    expect(Object.fromEntries(storage.map)).toEqual({ "sqdesk-recent-connection-ids-v1": "[1]", "sqdesk:code-snapshot-settings": "{}", theme: "dark" });
  });

  it("keeps a value already stored under the current name", () => {
    const storage = memoryStorage({ "dbx-app-state:tabs": "old", "sqdesk-app-state:tabs": "new" });

    expect(migrateLegacyStorageKeys(storage)).toBe(0);
    expect(Object.fromEntries(storage.map)).toEqual({ "sqdesk-app-state:tabs": "new" });
  });
});
