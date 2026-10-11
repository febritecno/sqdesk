const LEGACY_PREFIX = "dbx";
const CURRENT_PREFIX = "sqdesk";

/**
 * Storage keys carried the upstream prefix before the SQDesk rename. Move each
 * one to its current name once, so saved layouts and preferences survive the
 * rename. A key that already exists under the current name wins.
 */
export function migrateLegacyStorageKeys(storage: Pick<Storage, "length" | "key" | "getItem" | "setItem" | "removeItem">): number {
  const legacyKeys: string[] = [];
  for (let index = 0; index < storage.length; index += 1) {
    const key = storage.key(index);
    if (key?.startsWith(LEGACY_PREFIX)) legacyKeys.push(key);
  }
  let moved = 0;
  for (const key of legacyKeys) {
    const currentKey = CURRENT_PREFIX + key.slice(LEGACY_PREFIX.length);
    const value = storage.getItem(key);
    if (value !== null && storage.getItem(currentKey) === null) {
      storage.setItem(currentKey, value);
      moved += 1;
    }
    storage.removeItem(key);
  }
  return moved;
}
