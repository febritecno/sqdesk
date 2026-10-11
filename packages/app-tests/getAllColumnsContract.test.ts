import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "vitest";

function source(relativePath: string): string {
  return readFileSync(path.resolve(relativePath), "utf8");
}

test("getAllColumns is exported from both backends with table_name", () => {
  const http = source("apps/desktop/src/lib/backend/http.ts");
  const tauri = source("apps/desktop/src/lib/backend/tauri.ts");
  const api = source("apps/desktop/src/lib/backend/api.ts");

  assert.match(http, /export async function getAllColumns/);
  assert.match(tauri, /export async function getAllColumns/);
  assert.match(api, /getAllColumns/);

  assert.match(http, /export interface TableColumnsResult \{\n  table_name: string;/);
  assert.match(tauri, /export interface TableColumnsResult \{\n  table_name: string;/);
  assert.doesNotMatch(http, /export interface TableColumnsResult \{\n  tableName:/);
  assert.doesNotMatch(tauri, /export interface TableColumnsResult \{\n  tableName:/);

  assert.match(tauri, /invoke\("get_all_columns"/);
  assert.match(http, /\/api\/schema\/all-columns/);
});