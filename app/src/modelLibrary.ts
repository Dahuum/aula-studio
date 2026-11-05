import type { Macro, RGB } from "./model";
export type ModelKey = {
  id: string;
  label: string;
  usage: number | null;
  matrix_index: number | null;
  light_index: number | null;
  x: number;
  y: number;
  width: number;
  height: number;
  protected: boolean;
};
export type ModelDefinition = {
  id: string;
  name: string;
  device_kind: string;
  family: string;
  status: string;
  usb: {
    vendor_id: number;
    product_id: number;
    interface_number: number | null;
    connection: string;
    product_name?: string;
  }[];
  model_identity: number[] | null;
  keys: ModelKey[];
  lighting: {
    brightness_max: number;
    speed_max: number;
    default_effect: number;
    effects: {
      id: number;
      name: string;
      controls?: { brightness: boolean; speed: boolean; color: boolean };
    }[];
    per_key: boolean;
  } | null;
  fn_layer: boolean;
  notes: string[];
  evidence: {
    source_url: string;
    installer_sha256: string;
    configuration_path: string;
    configuration_sha256: string;
    level: string;
  }[];
};
export type ModelAssignment = { kind: "keyboard" | "consumer"; usage: number };
export type ModelDraft = {
  schema_version: 1;
  id: string;
  name: string;
  model_id: string;
  lighting: {
    effect: number;
    brightness: number;
    speed: number;
    color: RGB;
  } | null;
  per_key: Record<string, RGB>;
  assignments: Record<string, ModelAssignment>;
  fn_assignments: Record<string, ModelAssignment>;
  macros: Macro[];
};
export type ModelDevice = {
  path: string;
  vendor_id: number;
  product_id: number;
  interface_number: number;
  product_name: string;
  candidates: string[];
  hardware_writes_enabled: false;
  read_candidate?: boolean;
  product_name_matched?: boolean;
  note: string;
};
export function modelCode(usage: number): string {
  if (usage >= 4 && usage <= 29) return "Key" + String.fromCharCode(usage + 61);
  if (usage >= 30 && usage <= 38) return "Digit" + (usage - 29);
  if (usage === 39) return "Digit0";
  if (usage >= 58 && usage <= 69) return "F" + (usage - 57);
  if (usage >= 89 && usage <= 97) return "Numpad" + (usage - 88);
  return (
    (
      {
        40: "Enter",
        41: "Escape",
        42: "Backspace",
        43: "Tab",
        44: "Space",
        45: "Minus",
        46: "Equal",
        47: "BracketLeft",
        48: "BracketRight",
        49: "Backslash",
        51: "Semicolon",
        52: "Quote",
        53: "Backquote",
        54: "Comma",
        55: "Period",
        56: "Slash",
        57: "CapsLock",
        70: "PrintScreen",
        71: "ScrollLock",
        72: "Pause",
        73: "Insert",
        74: "Home",
        75: "PageUp",
        76: "Delete",
        77: "End",
        78: "PageDown",
        79: "ArrowRight",
        80: "ArrowLeft",
        81: "ArrowDown",
        82: "ArrowUp",
        83: "NumLock",
        84: "NumpadDivide",
        85: "NumpadMultiply",
        86: "NumpadSubtract",
        87: "NumpadAdd",
        88: "NumpadEnter",
        98: "Numpad0",
        99: "NumpadDecimal",
        101: "ContextMenu",
        224: "ControlLeft",
        225: "ShiftLeft",
        226: "AltLeft",
        227: "MetaLeft",
        228: "ControlRight",
        229: "ShiftRight",
        230: "AltRight",
        231: "MetaRight",
      } as Record<number, string>
    )[usage] ?? ""
  );
}

// The preview uses the same public, factual registry as the native library.
// Native commands independently validate every profile in Rust.
import registryData from "../../data/models.json";
import { validateMacro } from "./model";
export const modelRegistry = registryData as ModelDefinition[];
const validId = (v: unknown): v is string =>
  typeof v === "string" && /^[A-Za-z0-9_-]{1,64}$/.test(v);
const color = (v: unknown) =>
  Array.isArray(v) &&
  v.length === 3 &&
  v.every((n) => Number.isInteger(n) && n >= 0 && n <= 255);
const object = (v: unknown): v is Record<string, unknown> =>
  v !== null && typeof v === "object" && !Array.isArray(v);
export function parseModelDraft(text: string): ModelDraft {
  if (new TextEncoder().encode(text).length > 512 * 1024)
    throw new Error("Model profile exceeds 512 KiB");
  const p: unknown = JSON.parse(text);
  if (
    !object(p) ||
    Object.keys(p).sort().join(",") !==
      "assignments,fn_assignments,id,lighting,macros,model_id,name,per_key,schema_version" ||
    p.schema_version !== 1 ||
    !validId(p.id) ||
    typeof p.name !== "string" ||
    !p.name.trim() ||
    new TextEncoder().encode(p.name).length > 120
  )
    throw new Error("Invalid model profile schema or identity");
  const m = modelRegistry.find((m) => m.id === p.model_id);
  if (!m?.keys.length || m.id === "f75-ansi")
    throw new Error("No recovered model layout");
  // Schema1 F98 profiles predating the recovered lighting definitions stay
  // readable. This adds local intent only; no saved file/device is changed.
  if (m.id === "f98-pro" && p.lighting === null)
    p.lighting = defaultModelLighting(m);
  if (m.lighting) {
    const l = p.lighting,
      c = m.lighting;
    if (
      !object(l) ||
      Object.keys(l).sort().join(",") !== "brightness,color,effect,speed" ||
      !c.effects.some((e) => e.id === l.effect) ||
      !Number.isInteger(l.brightness) ||
      Number(l.brightness) < 0 ||
      Number(l.brightness) > c.brightness_max ||
      !Number.isInteger(l.speed) ||
      Number(l.speed) < 0 ||
      Number(l.speed) > c.speed_max ||
      !color(l.color)
    )
      throw new Error("Lighting exceeds documented model bounds");
  } else if (p.lighting !== null)
    throw new Error("Lighting is not supported by this model");
  if (
    !object(p.per_key) ||
    Object.entries(p.per_key).some(
      ([id, c]) =>
        !m.lighting?.per_key || !m.keys.some((k) => k.id === id) || !color(c),
    )
  )
    throw new Error("Invalid per-key colors for this model");
  for (const field of ["assignments", "fn_assignments"] as const) {
    const mappings = p[field];
    if (
      !object(mappings) ||
      (!m.fn_layer &&
        field === "fn_assignments" &&
        Object.keys(mappings).length)
    )
      throw new Error("Unsupported model layer");
    for (const [id, a] of Object.entries(mappings)) {
      if (
        !m.keys.some((k) => k.id === id && k.usage !== null && !k.protected) ||
        !object(a) ||
        Object.keys(a).sort().join(",") !== "kind,usage" ||
        !Number.isInteger(a.usage) ||
        !(a.kind === "keyboard"
          ? Number(a.usage) >= 1 && Number(a.usage) <= 255
          : a.kind === "consumer" &&
            [0xcd, 0xb7, 0xb6, 0xb5, 0xe9, 0xea, 0xe2].includes(
              Number(a.usage),
            ))
      )
        throw new Error("Unsupported or protected key assignment");
    }
  }
  if (!Array.isArray(p.macros) || p.macros.length > 64)
    throw new Error("Invalid macro list");
  const ids = new Set<string>();
  for (const macro of p.macros) {
    if (
      !object(macro) ||
      Object.keys(macro).sort().join(",") !== "events,id,name" ||
      !validId(macro.id) ||
      ids.has(macro.id) ||
      typeof macro.name !== "string" ||
      !macro.name.trim() ||
      new TextEncoder().encode(macro.name).length > 120 ||
      validateMacro(macro.events)
    )
      throw new Error("Invalid or unbalanced local macro");
    ids.add(macro.id);
  }
  return p as unknown as ModelDraft;
}
function defaultModelLighting(m: ModelDefinition): ModelDraft["lighting"] {
  return m.lighting
    ? {
        effect: m.lighting.default_effect,
        brightness: m.lighting.brightness_max,
        speed: Math.floor(m.lighting.speed_max / 2),
        color: [255, 96, 32],
      }
    : null;
}
export function newModelDraft(modelId: string, name: string): ModelDraft {
  const m = modelRegistry.find((m) => m.id === modelId);
  if (!m?.keys.length || m.id === "f75-ansi")
    throw new Error("No recovered layout");
  return parseModelDraft(
    JSON.stringify({
      schema_version: 1,
      id: crypto.randomUUID(),
      name,
      model_id: modelId,
      lighting: defaultModelLighting(m),
      per_key: {},
      assignments: {},
      fn_assignments: {},
      macros: [],
    }),
  );
}
