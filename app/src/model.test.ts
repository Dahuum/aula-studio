import { describe, it, expect } from "vitest";
import { KEYBOARD, CODE_USAGE } from "./layout";
import { createProfile, parseProfile, validateMacro, changed } from "./model";
describe("F75 physical layout", () => {
  it("has 80 unique keys, a reserved Fn, and correct usages", () => {
    expect(KEYBOARD).toHaveLength(80);
    expect(new Set(KEYBOARD.map((k) => k.id)).size).toBe(80);
    expect(CODE_USAGE.KeyA).toBe(4);
    expect(CODE_USAGE.KeyZ).toBe(29);
    expect(CODE_USAGE.ControlRight).toBe(228);
    expect(CODE_USAGE.AltRight).toBeUndefined();
    expect(KEYBOARD.find(k=>k.id==="fn")!.x).toBeLessThan(KEYBOARD.find(k=>k.usage===228)!.x);
    expect(KEYBOARD.find((k) => k.id === "fn")?.usage).toBeUndefined();
    expect(KEYBOARD.filter((k) => k.label === "F12")).toHaveLength(1);
  });
});
describe("local profiles", () => {
  it("roundtrips optional media layers and rejects conflicts or unsupported actions", () => {
    const p = createProfile();
    p.consumer_keymap = { "4": 0xcd };
    p.consumer_fn_layer = { "4": 0xe2 };
    expect(parseProfile(JSON.stringify(p))).toEqual(p);
    p.keymap = { "04": 5 };
    expect(() => parseProfile(JSON.stringify(p))).toThrow(/conflicting/);
    p.keymap = {};
    p.consumer_keymap = { "4": 0xb0 };
    expect(() => parseProfile(JSON.stringify(p))).toThrow(/Unsupported/);
  });
  it("roundtrips and detects draft changes", () => {
    const p = createProfile();
    expect(parseProfile(JSON.stringify(p))).toEqual(p);
    expect(changed(p, p)).toBe(false);
    expect(changed({ ...p, name: "changed" }, p)).toBe(true);
  });
  it("compares saved profile content independently of Rust field order and omitted empty defaults", () => {
    const p = createProfile();
    p.consumer_keymap = { "4": 0xcd };
    p.consumer_fn_layer = {};
    const saved = Object.fromEntries(Object.entries(p).reverse()) as typeof p;
    delete saved.consumer_fn_layer;
    saved.updated_at = "different save time";
    expect(changed(p, saved)).toBe(false);
    expect(changed({ ...p, consumer_keymap: { "4": 0xe2 } }, saved)).toBe(true);
  });
  it("rejects unsupported versions and unsafe colors", () => {
    const p = createProfile();
    expect(() =>
      parseProfile(JSON.stringify({ ...p, schema_version: 99 })),
    ).toThrow();
    p.lighting.color = [-1, 0, 0];
    expect(() => parseProfile(JSON.stringify(p))).toThrow();
  });
});
describe("macro safety", () => {
  it("rejects malformed imported events before preview or saving", () => {
    for (const events of [null, {}, [null], [{ kind: "delay", ms: 1, key: 4 }],
      [{ kind: "down", key: 4, ms: 1 }],
      [{ kind: "down", key: 4 }, { kind: "unrecognized", key: 4 }]]) {
      expect(validateMacro(events)).toBeTruthy();
    }
  });
  it("requires balanced presses and bounded delays", () => {
    expect(
      validateMacro([
        { kind: "down", key: 4 },
        { kind: "delay", ms: 12 },
        { kind: "up", key: 4 },
      ]),
    ).toBeNull();
    expect(validateMacro([{ kind: "up", key: 4 }])).toBeTruthy();
    expect(validateMacro([{ kind: "down", key: 4 }])).toBeTruthy();
    expect(validateMacro([{ kind: "delay", ms: 60001 }])).toBeTruthy();
  });
});
