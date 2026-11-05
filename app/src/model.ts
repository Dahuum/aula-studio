export type RGB = [number, number, number];
export type MacroEvent = {
  kind: "down" | "up" | "delay";
  key?: number;
  ms?: number;
};
export type Macro = { id: string; name: string; events: MacroEvent[] };
export type Profile = {
  schema_version: 1;
  id: string;
  name: string;
  lighting: {
    effect: number;
    brightness: number;
    speed: number;
    color: RGB;
    colorful?: boolean;
    direction: number;
  };
  per_key: Record<string, RGB>;
  keymap: Record<string, number>;
  fn_layer: Record<string, number>;
  consumer_keymap?: Record<string, number>;
  consumer_fn_layer?: Record<string, number>;
  mouse_keymap?: Record<string, number>;
  mouse_fn_layer?: Record<string, number>;
  macros: Macro[];
  updated_at: string;
  sleep_seconds?: number;
};
export const mediaActions = [
  { usage: 0x183, label: "Media player", short: "Media" },
  { usage: 0xcd, label: "Play / pause", short: "Play" },
  { usage: 0xe2, label: "Mute", short: "Mute" },
  { usage: 0xb7, label: "Stop playback", short: "Stop" },
  { usage: 0xb6, label: "Previous track", short: "Prev" },
  { usage: 0xb5, label: "Next track", short: "Next" },
  { usage: 0xe9, label: "Volume up", short: "Vol+" },
  { usage: 0xea, label: "Volume down", short: "Vol−" },
  { usage: 0x223, label: "Browser home", short: "Home" },
  { usage: 0x192, label: "Calculator", short: "Calc" },
  { usage: 0x18a, label: "Email reader", short: "Mail" },
  { usage: 0x194, label: "File browser", short: "Files" },
  { usage: 0x6f, label: "Display brightness up", short: "Disp+" },
  { usage: 0x70, label: "Display brightness down", short: "Disp−" },
] as const;
export const mouseActions = [
  { button: 1, label: "Left click", short: "Mouse1" },
  { button: 2, label: "Right click", short: "Mouse2" },
  { button: 3, label: "Middle click", short: "Mouse3" },
  { button: 4, label: "Mouse button 4", short: "Mouse4" },
  { button: 5, label: "Mouse button 5", short: "Mouse5" },
] as const;
export type Device = {
  path: string;
  vendor_id: number;
  product_id: number;
  product?: string;
  interface_number?: number;
  supported?: boolean;
  reason?: string;
};
export type Capabilities = {
  read_config: boolean;
  write_lighting: boolean;
  write_per_key: boolean;
  write_keymap: boolean;
  write_fn: boolean;
  onboard_macros: boolean;
};
export type Status = {
  mode: "demo" | "hardware";
  devices: Device[];
  connected: boolean;
  capabilities: Capabilities;
  firmware?: string;
  notice?: string;
};
export type Preferences = { demo_mode: boolean; reduce_motion: boolean; custom_colors?: RGB[] };
export type Backup = {
  id: string;
  created_at: string;
  device: string;
  kind: string;
  mapping_kind?: string;
};
export type ReadConfiguration = {
  sleep_seconds?: number | null;
  lighting?: Partial<Profile["lighting"]> | null;
  [key: string]: unknown;
};
export type PreparedLighting = {
  kind?:
    | "lighting"
    | "brightness"
    | "base"
    | "fn"
    | "per_key"
    | "settings"
    | "restore"
    | "mapping_restore";
  token: string;
  device: string;
  summary: string;
  backup_id: string;
  changes: { field: string; before: unknown; after: unknown }[];
  warnings: string[];
};
export type LightingWriteResult = {
  message: string;
  readback_verified: boolean;
  persistence: string;
};
export const effects = [
  {
    id: 0,
    name: "Lights off",
    description: "A quiet workspace",
    pattern: "off",
  },
  {
    id: 1,
    name: "Solid color",
    description: "One color, every key",
    pattern: "solid",
  },
  {
    id: 2,
    name: "Breathing",
    description: "A gentle pulse of light",
    pattern: "breathe",
  },
  {
    id: 3,
    name: "Rainbow",
    description: "The full spectrum",
    pattern: "rainbow",
  },
  {
    id: 4,
    name: "Flash away",
    description: "A trail across the board",
    pattern: "rainbow",
  },
  {
    id: 5,
    name: "Raindrops",
    description: "Scattered points of light",
    pattern: "scatter",
  },
  {
    id: 6,
    name: "Color wheel",
    description: "Colors in rotation",
    pattern: "rainbow",
  },
  {
    id: 7,
    name: "Ripple",
    description: "Waves from each press",
    pattern: "breathe",
  },
  {
    id: 8,
    name: "Stars",
    description: "A field of quiet light",
    pattern: "scatter",
  },
  {
    id: 9,
    name: "Fade",
    description: "Soft color transitions",
    pattern: "breathe",
  },
  {
    id: 10,
    name: "Snake",
    description: "A winding light trail",
    pattern: "rainbow",
  },
  {
    id: 11,
    name: "Neon",
    description: "Shifting color accents",
    pattern: "rainbow",
  },
  {
    id: 12,
    name: "Reaction",
    description: "Light responds to typing",
    pattern: "scatter",
  },
  {
    id: 13,
    name: "Sine wave",
    description: "A flowing wave",
    pattern: "rainbow",
  },
  {
    id: 14,
    name: "Scanning",
    description: "A sweeping light band",
    pattern: "rainbow",
  },
  {
    id: 15,
    name: "Windmill",
    description: "A rotating pattern",
    pattern: "rainbow",
  },
  {
    id: 16,
    name: "Waterfall",
    description: "Light flowing down",
    pattern: "rainbow",
  },
  {
    id: 17,
    name: "Blossom",
    description: "Color radiating outward",
    pattern: "breathe",
  },
  {
    id: 18,
    name: "Storm",
    description: "An energetic pattern",
    pattern: "scatter",
  },
  {
    id: 21,
    name: "Custom colors",
    description: "Your per-key palette",
    pattern: "custom",
  },
];
// Official effect flags (KB.ini): RGB option absent on the built-in rainbow,
// windmill, waterfall and blossom effects. Custom colors use a separate upload.
export function lightingCapabilities(effect: number) {
  return {
    brightness: effect >= 1 && effect <= 18,
    speed: effect >= 2 && effect <= 18,
    color: [1, 2, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 18].includes(effect),
  };
}
export function createProfile(name = "Untitled profile"): Profile {
  return {
    schema_version: 1,
    id: crypto.randomUUID(),
    name,
    lighting: {
      effect: 1,
      brightness: 6,
      speed: 2,
      color: [103, 232, 209],
      colorful: false,
      direction: 0,
    },
    per_key: {},
    keymap: {},
    fn_layer: {},
    macros: [],
    updated_at: new Date().toISOString(),
  };
}
export function rgbHex(color: RGB): string {
  return "#" + color.map((v) => v.toString(16).padStart(2, "0")).join("");
}
export function hexRgb(hex: string): RGB {
  return [
    parseInt(hex.slice(1, 3), 16),
    parseInt(hex.slice(3, 5), 16),
    parseInt(hex.slice(5, 7), 16),
  ];
}
export function changed(profile: Profile, saved: Profile | undefined): boolean {
  const sorted = (value: unknown): unknown => {
    if (Array.isArray(value)) return value.map(sorted);
    if (value && typeof value === "object")
      return Object.fromEntries(Object.entries(value).sort(([a], [b]) => a.localeCompare(b)).map(([key, item]) => [key, sorted(item)]));
    return value;
  };
  const comparable = (p: Profile) => sorted({
    ...p,
    updated_at: "",
    consumer_keymap: p.consumer_keymap ?? {},
    consumer_fn_layer: p.consumer_fn_layer ?? {},
    mouse_keymap: p.mouse_keymap ?? {},
    mouse_fn_layer: p.mouse_fn_layer ?? {},
    lighting: { ...p.lighting, colorful: p.lighting.colorful ?? false },
  });
  return (
    !saved ||
    JSON.stringify(comparable(profile)) !== JSON.stringify(comparable(saved))
  );
}
export function validateMacro(events: unknown): string | null {
  if (!Array.isArray(events)) return "Macro events must be a list.";
  if (events.length > 2048) return "A macro can contain at most 2,048 events.";
  const held = new Set<number>();
  for (const e of events) {
    if (!e || typeof e !== "object" ||
        !["down", "up", "delay"].includes(e.kind))
      return "Unknown macro event.";
    if (e.kind === "delay") {
      if (e.key !== undefined || !Number.isInteger(e.ms) || e.ms < 0 || e.ms > 60000)
        return "Each delay must be between 0 and 60,000 milliseconds.";
    } else {
      if (e.ms !== undefined || !Number.isInteger(e.key) || e.key < 4 || e.key > 231)
        return "Choose a valid keyboard key.";
      if (e.kind === "down") {
        if (held.has(e.key!))
          return "A key is pressed twice without being released.";
        held.add(e.key!);
      } else {
        if (!held.has(e.key!)) return "A key is released before it is pressed.";
        held.delete(e.key!);
      }
    }
  }
  return held.size
    ? "Release all pressed keys before saving this macro."
    : null;
}
export function parseProfile(text: string): Profile {
  const p = JSON.parse(text) as Profile;
  if (p.schema_version === undefined || (p.schema_version as number) === 0)
    p.schema_version = 1;
  if (p.schema_version !== 1)
    throw new Error(
      "Unsupported profile version. This application reads version 1.",
    );
  if (
    typeof p.id !== "string" ||
    typeof p.name !== "string" ||
    !p.name.trim() ||
    p.name.length > 80
  )
    throw new Error("Profile needs a valid ID and a name of 1–80 characters.");
  if (
    !p.lighting ||
    !effects.some((e) => e.id === p.lighting.effect) ||
    !Number.isInteger(p.lighting.brightness) ||
    p.lighting.brightness < 0 ||
    p.lighting.brightness > 9 ||
    !Number.isInteger(p.lighting.speed) ||
    p.lighting.speed < 0 ||
    p.lighting.speed > 4
  )
    throw new Error("Profile contains unsupported lighting settings.");
  if (
    p.sleep_seconds !== undefined &&
    (!Number.isInteger(p.sleep_seconds) ||
      ![0, 60, 90, 120, 180, 240, 300, 600, 900, 1200].includes(
        p.sleep_seconds,
      ))
  )
    throw new Error("Unsupported sleep setting.");
  const color = (v: unknown) =>
    Array.isArray(v) &&
    v.length === 3 &&
    v.every((c) => Number.isInteger(c) && c >= 0 && c <= 255);
  if (
    !color(p.lighting.color) ||
    (p.lighting.colorful !== undefined &&
      typeof p.lighting.colorful !== "boolean") ||
    !p.per_key ||
    !Object.values(p.per_key).every(color)
  )
    throw new Error("Profile contains an invalid RGB color.");
  if (
    !p.keymap ||
    !p.fn_layer ||
    !Array.isArray(p.macros) ||
    p.macros.length > 64
  )
    throw new Error("Profile is missing key mappings or macros.");
  for (const map of [p.keymap, p.fn_layer]) {
    if (
      Object.entries(map).some(
        ([k, v]) =>
          !/^\d+$/.test(k) ||
          +k < 1 ||
          +k > 255 ||
          !Number.isInteger(v) ||
          v < 1 ||
          v > 255,
      )
    )
      throw new Error("Invalid keyboard usage mapping.");
  }
  for (const [keyboard, media, mouse] of [[p.keymap, p.consumer_keymap, p.mouse_keymap], [p.fn_layer, p.consumer_fn_layer, p.mouse_fn_layer]] as const) {
    if (media !== undefined && (!media || typeof media !== "object" || Array.isArray(media)))
      throw new Error("Invalid media assignments.");
    const seen = new Set(Object.keys(keyboard).map(Number));
    for (const [source, usage] of Object.entries(media ?? {})) {
      if (!/^\d+$/.test(source) || +source < 1 || +source > 255 || seen.has(+source) ||
          !mediaActions.some(action => action.usage === usage))
        throw new Error("Unsupported or conflicting media assignment.");
      seen.add(+source);
    }
    if (mouse !== undefined && (!mouse || typeof mouse !== "object" || Array.isArray(mouse)))
      throw new Error("Invalid mouse assignments.");
    for (const [source, button] of Object.entries(mouse ?? {})) {
      if (!/^\d+$/.test(source) || +source < 1 || +source > 255 || seen.has(+source) ||
          !mouseActions.some(action => action.button === button))
        throw new Error("Unsupported or conflicting mouse assignment.");
      seen.add(+source);
    }
  }
  for (const m of p.macros) {
    if (!Array.isArray(m.events)) throw new Error("Invalid macro events.");
    const error = validateMacro(m.events);
    if (error) throw new Error(error);
  }
  return p;
}
export function fileName(name: string) {
  return name.replace(/[^a-z0-9_-]/gi, "-").slice(0, 60) || "aula-profile";
}
