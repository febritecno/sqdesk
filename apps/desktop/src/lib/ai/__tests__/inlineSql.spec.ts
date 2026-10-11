import { describe, expect, it } from "vitest";
import { buildInlineSqlSystemPrompt, buildInlineSqlUserPrompt, cleanInlineSqlAnswer, cleanInlineSqlOptions, inlineSqlInsertion, formatInlineSqlTable, pickInlineSqlTables, scoreInlineSqlTable, type InlineSqlTable } from "@/lib/ai/inlineSql";
import type { ColumnInfo } from "@/types/database";

function column(name: string, dataType: string, extra: Partial<ColumnInfo> = {}): ColumnInfo {
  return { name, data_type: dataType, is_nullable: false, column_default: null, is_primary_key: false, extra: null, ...extra };
}

const orders: InlineSqlTable = {
  schema: "public",
  name: "orders",
  columns: [column("id", "bigint", { is_primary_key: true }), column("customer_id", "bigint"), column("status", "varchar(20)", { is_nullable: true, comment: "pending, paid or void" }), column("created_at", "timestamp")],
  indexes: [
    { name: "orders_pkey", columns: ["id"], is_unique: true, is_primary: true },
    { name: "orders_customer_created_idx", columns: ["customer_id", "created_at"], is_unique: false, is_primary: false },
  ],
  foreignKeys: [{ name: "orders_customer_fk", column: "customer_id", ref_table: "customers", ref_column: "id" }],
};

describe("scoreInlineSqlTable", () => {
  it("ranks an exact mention above a plural match above shared name parts", () => {
    const exact = scoreInlineSqlTable("orders", "select from orders where id = 1");
    const plural = scoreInlineSqlTable("orders", "total per order last month");
    const partial = scoreInlineSqlTable("customer_orders", "orders per customer");
    const oneWord = scoreInlineSqlTable("customer_addresses", "orders per customer");
    expect(exact).toBeGreaterThan(plural);
    expect(plural).toBeGreaterThan(0);
    expect(partial).toBeGreaterThan(oneWord);
    expect(oneWord).toBeGreaterThan(0);
  });

  it("ignores unrelated tables and filler words", () => {
    expect(scoreInlineSqlTable("audit_log", "orders per customer")).toBe(0);
    expect(scoreInlineSqlTable("data_rows", "show all rows of data")).toBe(0);
  });

  it("does not treat a name inside a longer identifier as a mention", () => {
    expect(scoreInlineSqlTable("order", "select * from order_items")).toBeLessThan(100);
  });
});

describe("pickInlineSqlTables", () => {
  const tables = ["customers", "orders", "order_items", "audit_log", "sessions"].map((name) => ({ name }));

  it("sends a small database whole", () => {
    expect(pickInlineSqlTables(tables, "anything", 10)).toHaveLength(5);
  });

  it("keeps only relevant tables, best match first, when the database is large", () => {
    expect(pickInlineSqlTables(tables, "revenue per customer from orders", 2).map((table) => table.name)).toEqual(["orders", "customers"]);
  });

  it("returns nothing rather than guessing when no table matches", () => {
    expect(pickInlineSqlTables(tables, "what time is it", 2)).toEqual([]);
  });
});

describe("formatInlineSqlTable", () => {
  it("shows keys, nullability, references, comments and indexes", () => {
    const text = formatInlineSqlTable(orders);
    expect(text).toContain("table public.orders (");
    expect(text).toContain("  id bigint PK NOT NULL");
    expect(text).toContain("  customer_id bigint NOT NULL -> customers.id");
    expect(text).toContain("  status varchar(20) NULL  -- pending, paid or void");
    expect(text).toContain("indexes: PRIMARY orders_pkey (id); INDEX orders_customer_created_idx (customer_id, created_at)");
  });
});

describe("inline SQL prompts", () => {
  const base = { request: "paid orders this month", selectedSql: "", databaseType: "postgres", serverVersion: "16.2", database: "shop", schema: "public", tables: [orders], otherTableNames: ["customers", "audit_log"] };

  it("states the dialect, the output contract and the efficiency rules", () => {
    const system = buildInlineSqlSystemPrompt(base);
    expect(system).toContain("runnable on postgres 16.2");
    expect(system).toContain("exactly one ```sql fenced block");
    expect(system).toContain("Do not add comments of any kind");
    expect(system).toContain("Always return a rewritten query");
    expect(system).toContain("reply with up to 3 fenced blocks instead, best first");
    expect(system).toContain("No SELECT * unless asked for");
    expect(system).toContain("sargable");
    expect(system).toContain("never write an UPDATE or DELETE without a WHERE clause");
    expect(system).toContain("uppercase keywords");
  });

  it("follows the keyword case of the selected SQL", () => {
    expect(buildInlineSqlSystemPrompt({ ...base, selectedSql: "select id from orders where id = 1" })).toContain("lowercase keywords");
  });

  it("carries the conversation into a follow-up and asks to revise the last proposal", () => {
    const prompt = buildInlineSqlUserPrompt({
      ...base,
      selectedSql: "select id from orders",
      request: "newest first",
      conversation: [
        { request: "only paid", sql: "select id from orders where paid" },
        { request: "skipped", sql: "  " },
      ],
    });
    expect(prompt).toContain("Selected SQL to rewrite:");
    expect(prompt).toContain("Conversation so far (proposals the user has seen; none is in the editor yet):");
    expect(prompt).toContain("Request 1: only paid\nProposed SQL 1:\n```sql\nselect id from orders where paid\n```");
    expect(prompt).not.toContain("skipped");
    expect(prompt.endsWith("Revise proposed SQL 1 as follows: newest first")).toBe(true);
    expect(prompt).not.toContain("Rewrite the selected SQL as follows");
  });

  it("asks for a new query when nothing is selected", () => {
    const prompt = buildInlineSqlUserPrompt({ ...base, textBefore: "SELECT 1;" });
    expect(prompt).toContain("Database type: postgres 16.2; database: shop; default schema: public");
    expect(prompt).toContain("table public.orders (");
    expect(prompt).toContain("Other tables that exist (names only, columns not shown): customers, audit_log");
    expect(prompt).toContain("Editor text before the cursor (context only, do not repeat it):");
    expect(prompt).not.toContain("Selected SQL to rewrite");
    expect(prompt.endsWith("Write SQL for this request: paid orders this month")).toBe(true);
  });

  it("asks for a rewrite when SQL is selected", () => {
    const prompt = buildInlineSqlUserPrompt({ ...base, request: "add a date filter", selectedSql: "select * from orders" });
    expect(prompt).toContain("Selected SQL to rewrite:\n```sql\nselect * from orders\n```");
    expect(prompt.endsWith("Rewrite the selected SQL as follows: add a date filter")).toBe(true);
  });

  it("says so when no table details could be loaded", () => {
    expect(buildInlineSqlUserPrompt({ ...base, tables: [], otherTableNames: [] })).toContain("no table details are available");
  });
});

describe("cleanInlineSqlAnswer", () => {
  it("takes the SQL out of a fenced block", () => {
    expect(cleanInlineSqlAnswer("```sql\nSELECT 1;\n```")).toEqual({ sql: "SELECT 1;", note: "" });
    expect(cleanInlineSqlAnswer("Here you go:\n```postgresql\nSELECT 1\n```\nHope it helps.")).toEqual({ sql: "SELECT 1", note: "" });
  });

  it("drops a preamble when the model forgot the fence", () => {
    expect(cleanInlineSqlAnswer("Sure, here is the query:\nSELECT id\nFROM orders").sql).toBe("SELECT id\nFROM orders");
  });

  it("reports a comment-only reply as a note instead of SQL", () => {
    expect(cleanInlineSqlAnswer("```sql\n-- No table holds invoices in this schema.\n```")).toEqual({ sql: "", note: "No table holds invoices in this schema." });
  });

  it("drops comments the model added on its own", () => {
    expect(cleanInlineSqlAnswer("```sql\n-- half-open range keeps the index usable\nSELECT 1\n```").sql).toBe("SELECT 1");
    expect(cleanInlineSqlAnswer("```sql\n/* tidied */\nSELECT id\n/*\n  two-line\n  remark\n*/\nFROM orders\n```").sql).toBe("SELECT id\nFROM orders");
  });

  it("keeps comments from the selection, optimizer hints and trailing comments", () => {
    const selectedSql = "-- monthly report\nselect id from orders";
    expect(cleanInlineSqlAnswer("```sql\n-- monthly report\n-- reformatted\nselect id\nfrom orders\n```", { selectedSql }).sql).toBe("-- monthly report\nselect id\nfrom orders");
    expect(cleanInlineSqlAnswer("```sql\nSELECT\n/*+ INDEX(o idx_orders_paid) */\nid -- key\nFROM orders o\n```").sql).toBe("SELECT\n/*+ INDEX(o idx_orders_paid) */\nid -- key\nFROM orders o");
  });

  it("cuts the explanation a model appends after unfenced SQL", () => {
    expect(cleanInlineSqlAnswer("SELECT id\nFROM orders\n\nThis query returns every order id. Let me know if you need more.").sql).toBe("SELECT id\nFROM orders");
    // A blank line inside the statement is not the end of it.
    expect(cleanInlineSqlAnswer("SELECT id\nFROM orders\n\nWHERE paid = 1\n\nUNION ALL\n\nSELECT id FROM refunds").sql).toBe("SELECT id\nFROM orders\n\nWHERE paid = 1\n\nUNION ALL\n\nSELECT id FROM refunds");
  });

  it("refuses a reply that stopped inside its code block", () => {
    expect(cleanInlineSqlAnswer("```sql\nSELECT id, name\nFROM orders\nWHERE status = 'pa")).toEqual({ sql: "", note: "", truncated: true });
    expect(cleanInlineSqlAnswer("```sql\nSELECT 1\n```").truncated).toBeUndefined();
  });

  it("treats MySQL hash comments like dash comments, but not JSON operators", () => {
    expect(cleanInlineSqlAnswer("```sql\n# pick paid orders\nSELECT 1\n```").sql).toBe("SELECT 1");
    expect(cleanInlineSqlAnswer("```sql\n# orders has no invoice column\n```")).toEqual({ sql: "", note: "orders has no invoice column" });
    expect(cleanInlineSqlAnswer("```sql\nSELECT data\n#>> '{a,b}'\nFROM docs\n```").sql).toBe("SELECT data\n#>> '{a,b}'\nFROM docs");
  });

  it("holds back a rewrite that turns a query into a write statement", () => {
    const selectedSql = "SELECT id, name FROM absens_area";
    expect(cleanInlineSqlAnswer("```sql\nALTER TABLE absens_area ADD COLUMN phone VARCHAR(20);\n\nSELECT id, name, phone FROM absens_area;\n```", { selectedSql, request: "add the phone column" })).toEqual({ sql: "", note: "", unrequestedStatement: "ALTER" });
    // Named in the request, already in the selection, or only inside a string: allowed.
    expect(cleanInlineSqlAnswer("```sql\nDELETE FROM absens_area WHERE id = 1\n```", { selectedSql, request: "turn this into a delete for id 1" }).sql).toBe("DELETE FROM absens_area WHERE id = 1");
    expect(cleanInlineSqlAnswer("```sql\nUPDATE t SET a = 2 WHERE id = 1\n```", { selectedSql: "update t set a = 1 where id = 1", request: "set a to 2" }).sql).toBe("UPDATE t SET a = 2 WHERE id = 1");
    expect(cleanInlineSqlAnswer("```sql\nSELECT id FROM logs WHERE action = 'delete'\n```", { selectedSql: "SELECT id FROM logs", request: "only deletions" }).sql).toBe("SELECT id FROM logs WHERE action = 'delete'");
    // Generating from scratch is not a rewrite, so nothing is held back.
    expect(cleanInlineSqlAnswer("```sql\nINSERT INTO t (a) VALUES (1)\n```", { selectedSql: "", request: "add a row" }).sql).toBe("INSERT INTO t (a) VALUES (1)");
  });

  it("keeps added comments when the request asks for them", () => {
    const answer = "```sql\n-- paid orders only\nSELECT 1\n```";
    expect(cleanInlineSqlAnswer(answer, { request: "add comments explaining each clause" }).sql).toBe("-- paid orders only\nSELECT 1");
    expect(cleanInlineSqlAnswer(answer, { request: "tambahkan komentar penjelasan" }).sql).toBe("-- paid orders only\nSELECT 1");
    expect(cleanInlineSqlAnswer(answer, { request: "filter paid orders" }).sql).toBe("SELECT 1");
  });

  it("matches the selection's statement terminator", () => {
    expect(cleanInlineSqlAnswer("```sql\nSELECT 1\n```", { selectedSql: "select 2;" }).sql).toBe("SELECT 1;");
    expect(cleanInlineSqlAnswer("```sql\nSELECT 1;\n```", { selectedSql: "select 2" }).sql).toBe("SELECT 1");
    expect(cleanInlineSqlAnswer("```sql\nSELECT 1;\nSELECT 2;\n```", { selectedSql: "select 2" }).sql).toBe("SELECT 1;\nSELECT 2;");
  });

  it("returns nothing for an empty reply", () => {
    expect(cleanInlineSqlAnswer("   ")).toEqual({ sql: "", note: "" });
  });
});

describe("inlineSqlInsertion", () => {
  it("leaves SQL alone on an empty line", () => {
    expect(inlineSqlInsertion("SELECT 2", "", "")).toBe("SELECT 2");
    expect(inlineSqlInsertion("SELECT 2", "   ", "\t")).toBe("SELECT 2");
  });

  it("moves SQL onto its own line when the cursor's line has content", () => {
    expect(inlineSqlInsertion("SELECT 2", "SELECT 1;", "")).toBe("\nSELECT 2");
    expect(inlineSqlInsertion("SELECT 2", "", "SELECT 3;")).toBe("SELECT 2\n");
    expect(inlineSqlInsertion("SELECT 2", "SELECT 1;", " -- tail")).toBe("\nSELECT 2\n");
  });
});

describe("cleanInlineSqlOptions", () => {
  it("returns a single answer as one unnamed option", () => {
    expect(cleanInlineSqlOptions("```sql\nSELECT 1\n```")).toEqual({ note: "", options: [{ sql: "SELECT 1", label: "" }] });
  });

  it("reads labelled alternatives, best first", () => {
    const answer = "### Newest first\n```sql\nSELECT id FROM orders ORDER BY created_at DESC\n```\n\n### Option 2: Largest first\n```sql\nSELECT id FROM orders ORDER BY total DESC\n```";
    expect(cleanInlineSqlOptions(answer).options).toEqual([
      { sql: "SELECT id FROM orders ORDER BY created_at DESC", label: "Newest first" },
      { sql: "SELECT id FROM orders ORDER BY total DESC", label: "Largest first" },
    ]);
  });

  it("cleans each alternative, drops duplicates and keeps at most three", () => {
    const block = (sql: string) => `\`\`\`sql\n-- remark\n${sql}\n\`\`\``;
    const answer = [block("SELECT 1"), block("SELECT 1"), block("SELECT 2"), block("SELECT 3"), block("SELECT 4")].join("\n");
    expect(cleanInlineSqlOptions(answer).options.map((option) => option.sql)).toEqual(["SELECT 1", "SELECT 2", "SELECT 3"]);
  });

  it("skips an alternative that would change the statement type", () => {
    const answer = "```sql\nALTER TABLE t ADD COLUMN phone VARCHAR(20)\n```\n```sql\nSELECT id FROM t\n```";
    expect(cleanInlineSqlOptions(answer, { selectedSql: "SELECT id, name FROM t", request: "add phone" }).options).toEqual([{ sql: "SELECT id FROM t", label: "" }]);
  });

  it("reports why nothing is usable when every alternative is rejected", () => {
    const answer = "```sql\n-- t has no phone column\n```\n```sql\n-- nothing to add\n```";
    expect(cleanInlineSqlOptions(answer)).toEqual({ note: "t has no phone column", options: [] });
  });
});
