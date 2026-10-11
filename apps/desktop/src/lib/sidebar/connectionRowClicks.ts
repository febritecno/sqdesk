export type ConnectionRowClickAction = "open" | "rename" | "none";

/** Clicks in one burst that start an in-place rename of a connection. */
export const CONNECTION_RENAME_CLICK_COUNT = 4;

/**
 * What a burst of clicks on a connection row means, by its length: one click
 * opens the connection, exactly four rename it, and every other count does
 * nothing. The count is only final once the burst is over, so callers run the
 * action after a short wait and cancel it when another click arrives.
 */
export function connectionRowClickAction(clickCount: number): ConnectionRowClickAction {
  if (clickCount === 1) return "open";
  if (clickCount === CONNECTION_RENAME_CLICK_COUNT) return "rename";
  return "none";
}
