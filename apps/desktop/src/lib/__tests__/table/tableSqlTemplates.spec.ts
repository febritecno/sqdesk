import { describe, expect, it } from "vitest";

import { buildTableSelectTemplate, buildTableUpdateTemplate } from "@/lib/table/tableSqlTemplates";

describe("buildTableSelectTemplate", () => {
  it("can omit identifier quotes for a new-query table reference", () => {
    expect(
      buildTableSelectTemplate({
        databaseType: "postgres",
        schema: "public",
        tableName: "sqdesk_smoke",
        quoteIdentifiers: false,
      }),
    ).toBe("SELECT *\nFROM public.sqdesk_smoke;");
  });

  it("keeps required quotes for columns when identifier quoting is disabled", () => {
    expect(
      buildTableSelectTemplate({
        databaseType: "dameng",
        tableName: "SQDESK_TEST",
        columns: [
          { name: "ORDER", data_type: "VARCHAR" },
          { name: "CUSTOMER_NAME", data_type: "VARCHAR" },
        ],
        quoteIdentifiers: false,
      }),
    ).toBe('SELECT "ORDER", CUSTOMER_NAME\nFROM SQDESK_TEST;');
  });

  it("applies the same identifier policy to update templates", () => {
    expect(
      buildTableUpdateTemplate({
        databaseType: "oracle",
        tableName: "SQDESK_TEST",
        columns: [
          { name: "ID", data_type: "NUMBER", is_primary_key: true },
          { name: "ORDER", data_type: "VARCHAR" },
        ],
        quoteIdentifiers: false,
      }),
    ).toBe("UPDATE SQDESK_TEST\nSET \"ORDER\" = 'ORDER_value'\nWHERE ID = 0;");
  });
});
