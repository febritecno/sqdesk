import { describe, expect, it } from "vitest";
import { parseDataGridInlineFilter } from "@/lib/dataGrid/dataGridInlineFilter";

describe("parseDataGridInlineFilter", () => {
  it("ignores blank input and operators without a value", () => {
    expect(parseDataGridInlineFilter("   ", "text")).toBeNull();
    expect(parseDataGridInlineFilter(undefined, "integer")).toBeNull();
    expect(parseDataGridInlineFilter(">=", "integer")).toBeNull();
  });

  it("uses contains on text columns and equals elsewhere", () => {
    expect(parseDataGridInlineFilter("abc", "character varying")).toEqual({ mode: "like", rawValue: "abc" });
    expect(parseDataGridInlineFilter("42", "integer")).toEqual({ mode: "equals", rawValue: "42" });
    expect(parseDataGridInlineFilter("true", "boolean")).toEqual({ mode: "equals", rawValue: "true" });
    expect(parseDataGridInlineFilter("abc", undefined)).toEqual({ mode: "equals", rawValue: "abc" });
  });

  it("honours comparison operators on any column type", () => {
    expect(parseDataGridInlineFilter(">= 10", "integer")).toEqual({ mode: "greater-than-or-equal", rawValue: "10" });
    expect(parseDataGridInlineFilter("<2026-01-01", "timestamp")).toEqual({ mode: "less-than", rawValue: "2026-01-01" });
    expect(parseDataGridInlineFilter("!=x", "text")).toEqual({ mode: "not-equals", rawValue: "x" });
    expect(parseDataGridInlineFilter("<>x", "text")).toEqual({ mode: "not-equals", rawValue: "x" });
    expect(parseDataGridInlineFilter("=abc", "text")).toEqual({ mode: "equals", rawValue: "abc" });
  });

  it("supports null checks and text shortcuts", () => {
    expect(parseDataGridInlineFilter("NULL", "integer")).toEqual({ mode: "is-null", rawValue: "" });
    expect(parseDataGridInlineFilter("!null", "text")).toEqual({ mode: "is-not-null", rawValue: "" });
    expect(parseDataGridInlineFilter("!abc", "text")).toEqual({ mode: "not-like", rawValue: "abc" });
    expect(parseDataGridInlineFilter("^abc", "varchar")).toEqual({ mode: "begins-with", rawValue: "abc" });
    expect(parseDataGridInlineFilter("abc$", "varchar")).toEqual({ mode: "ends-with", rawValue: "abc" });
  });
});
