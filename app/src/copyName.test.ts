import { expect, it } from "vitest";
import { copyName } from "./copyName";
import { createProfile, parseProfile } from "./model";
import { newModelDraft, parseModelDraft, modelRegistry } from "./modelLibrary";

it("reserves the suffix within the profile editor's character limit", () => {
  const name = copyName("A".repeat(80));
  expect(name).toBe("A".repeat(75) + " copy");
  const profile = { ...createProfile(), name };
  expect(parseProfile(JSON.stringify(profile))).toEqual(profile);
});

it("keeps Unicode macro copy names within the shared 120-byte limit", () => {
  const name = copyName("🎹".repeat(30), " (copy)");
  expect(name).toBe("🎹".repeat(28) + " (copy)");
  expect(new TextEncoder().encode(name).length).toBe(119);
  expect(JSON.parse(JSON.stringify(name))).toBe(name);
});

it("preserves complete multibyte characters at the model name boundary", () => {
  const model = modelRegistry.find((m) => m.id === "f87")!;
  const draft = newModelDraft(model.id, "Test");
  draft.name = copyName("鍵".repeat(40), " copy", 120);
  expect(draft.name).toBe("鍵".repeat(38) + " copy");
  expect(parseModelDraft(JSON.stringify(draft))).toEqual(draft);
  expect(copyName("A".repeat(120), " copy", 120)).toHaveLength(120);
});

it("keeps short copy names intact", () => {
  expect(copyName("My profile")).toBe("My profile copy");
  expect(copyName("My macro", " (copy)")).toBe("My macro (copy)");
});
