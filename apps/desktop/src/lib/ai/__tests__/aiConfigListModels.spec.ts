import { describe, expect, it } from "vitest";
import { editableAiModels, removeConfiguredAiModel } from "@/lib/ai/aiConfigList";

describe("editableAiModels", () => {
  it("lists the default first and each model once", () => {
    expect(editableAiModels("b", [{ name: "a" }, { name: "b", label: "Bee" }, { name: "a" }])).toEqual([{ name: "b", label: "Bee" }, { name: "a" }]);
  });

  it("includes a default that was never saved in the list", () => {
    expect(editableAiModels(" solo ", undefined)).toEqual([{ name: "solo" }]);
    expect(editableAiModels("", [{ name: " a " }, { name: "" }])).toEqual([{ name: "a" }]);
  });
});

describe("removeConfiguredAiModel", () => {
  it("keeps the default when another model is removed", () => {
    expect(removeConfiguredAiModel("a", [{ name: "a" }, { name: "b" }], "b")).toEqual({ defaultModel: "a", models: [{ name: "a" }] });
  });

  it("hands the default to the next model when the default is removed", () => {
    expect(removeConfiguredAiModel("a", [{ name: "a" }, { name: "b" }, { name: "c" }], "a")).toEqual({ defaultModel: "b", models: [{ name: "b" }, { name: "c" }] });
  });

  it("leaves no default once the last model is gone", () => {
    expect(removeConfiguredAiModel("a", [{ name: "a" }], "a")).toEqual({ defaultModel: "", models: [] });
  });
});
