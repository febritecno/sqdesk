import type { DataGridContextFilterMode } from "@/lib/dataGrid/dataGridSql";

export interface DataGridInlineFilterExpression {
  mode: DataGridContextFilterMode;
  rawValue: string;
}

const TEXT_COLUMN_TYPE = /char|text|string|clob/i;

const OPERATOR_MODES: ReadonlyArray<readonly [string, DataGridContextFilterMode]> = [
  [">=", "greater-than-or-equal"],
  ["<=", "less-than-or-equal"],
  ["!=", "not-equals"],
  ["<>", "not-equals"],
  [">", "greater-than"],
  ["<", "less-than"],
  ["=", "equals"],
];

/**
 * Parses what the user typed into a column's inline filter box.
 *
 * Plain text means "contains" on text columns and "equals" elsewhere. A leading
 * operator overrides that: `=`, `!=`, `<>`, `>`, `>=`, `<`, `<=`, `!text` (does
 * not contain), `^text` (starts with), `text$` (ends with), `null`, `!null`.
 */
export function parseDataGridInlineFilter(input: string | undefined, dataType?: string): DataGridInlineFilterExpression | null {
  const text = (input ?? "").trim();
  if (!text) return null;
  const lower = text.toLowerCase();
  if (lower === "null") return { mode: "is-null", rawValue: "" };
  if (lower === "!null" || lower === "not null") return { mode: "is-not-null", rawValue: "" };
  for (const [operator, mode] of OPERATOR_MODES) {
    if (!text.startsWith(operator)) continue;
    const rawValue = text.slice(operator.length).trim();
    return rawValue ? { mode, rawValue } : null;
  }
  const isText = !!dataType && TEXT_COLUMN_TYPE.test(dataType);
  if (isText) {
    if (text.startsWith("!") && text.length > 1) return { mode: "not-like", rawValue: text.slice(1) };
    if (text.startsWith("^") && text.length > 1) return { mode: "begins-with", rawValue: text.slice(1) };
    if (text.endsWith("$") && text.length > 1) return { mode: "ends-with", rawValue: text.slice(0, -1) };
    return { mode: "like", rawValue: text };
  }
  return { mode: "equals", rawValue: text };
}
