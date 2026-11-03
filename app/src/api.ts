import { invoke, isTauri } from "@tauri-apps/api/core";
import { createProfile, parseProfile } from "./model";
import type { Profile, Status, Preferences, Backup } from "./model";
import { modelRegistry, newModelDraft, parseModelDraft } from "./modelLibrary";
import type { ModelDraft } from "./modelLibrary";
export const native = isTauri();
const prefix = "aula-studio-preview-";
const read = <T>(key: string, fallback: T): T => {
  try {
    return JSON.parse(localStorage.getItem(prefix + key) ?? "null") ?? fallback;
  } catch {
    return fallback;
  }
};
const write = (key: string, value: unknown) =>
  localStorage.setItem(prefix + key, JSON.stringify(value));
let simulated: Profile | undefined;
const snapshots = new Map<string, Profile>();
async function preview<T>(
  command: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  const profiles = read<Profile[]>("profiles", []);
  const prefs = read<Preferences>("preferences", {
    demo_mode: true,
    reduce_motion: false,
  });
  let result: unknown;
  switch (command) {
    case "list_models":
      result = modelRegistry;
      break;
    case "model_devices":
      result = [];
      break;
    case "new_model_profile":
      result = newModelDraft(String(args.modelId), String(args.name));
      break;
    case "list_model_profiles":
      result = read<ModelDraft[]>("model-profiles", []).map((p) =>
        parseModelDraft(JSON.stringify(p)),
      );
      break;
    case "model_profile_json":
      result = JSON.stringify(
        parseModelDraft(JSON.stringify(args.profile)),
        null,
        2,
      );
      break;
    case "save_model_profile": {
      const p = parseModelDraft(JSON.stringify(args.profile));
      const all = read<ModelDraft[]>("model-profiles", []),
        old = all.find((v) => v.id === p.id);
      if (old && old.model_id !== p.model_id)
        throw new Error("Cannot overwrite another model's profile");
      if (old)
        write("model-history", [
          ...read<ModelDraft[]>("model-history", []),
          old,
        ]);
      write("model-profiles", [...all.filter((v) => v.id !== p.id), p]);
      break;
    }
    case "delete_model_profile": {
      const all = read<ModelDraft[]>("model-profiles", []),
        old = all.find((v) => v.id === args.id);
      if (!old) throw new Error("Unknown model profile");
      write("model-history", [...read<ModelDraft[]>("model-history", []), old]);
      write(
        "model-profiles",
        all.filter((v) => v.id !== args.id),
      );
      break;
    }
    case "import_model_profile": {
      const p = parseModelDraft(String(args.json));
      p.id = crypto.randomUUID();
      write("model-profiles", [...read<ModelDraft[]>("model-profiles", []), p]);
      result = p;
      break;
    }
    case "export_model_profile": {
      const p = read<ModelDraft[]>("model-profiles", []).find(
        (v) => v.id === args.id,
      );
      if (!p) throw new Error("Unknown model profile");
      result = JSON.stringify(parseModelDraft(JSON.stringify(p)), null, 2);
      break;
    }
    case "get_status":
      result = {
        mode: prefs.demo_mode ? "demo" : "hardware",
        devices: [],
        connected: prefs.demo_mode,
        capabilities: {
          read_config: prefs.demo_mode,
          write_lighting: prefs.demo_mode,
          write_per_key: prefs.demo_mode,
          write_keymap: prefs.demo_mode,
          write_fn: prefs.demo_mode,
          onboard_macros: false,
        },
        notice: prefs.demo_mode
          ? "Browser preview · simulation only. No hardware access."
          : "Hardware access requires the native desktop application.",
      } satisfies Status;
      break;
    case "get_preferences":
      result = prefs;
      break;
    case "save_preferences":
      write("preferences", args.preferences);
      result = args.preferences;
      break;
    case "set_demo_mode":
      write("preferences", { ...prefs, demo_mode: args.enabled });
      break;
    case "list_profiles":
      result = profiles;
      break;
    case "save_profile": {
      const p = parseProfile(JSON.stringify(args.profile));
      write("profiles", [...profiles.filter((v) => v.id !== p.id), p]);
      result = p;
      break;
    }
    case "delete_profile":
      write(
        "profiles",
        profiles.filter((v) => v.id !== args.id),
      );
      break;
    case "import_profile": {
      const p = parseProfile(String(args.json));
      p.id = crypto.randomUUID();
      write("profiles", [...profiles, p]);
      result = p;
      break;
    }
    case "export_profile": {
      const p = profiles.find((p) => p.id === args.id);
      if (!p) throw new Error("Save the profile before exporting it.");
      result = JSON.stringify(p, null, 2);
      break;
    }
    case "apply_profile":
      if (!prefs.demo_mode)
        throw new Error("Hardware writes are unavailable in browser preview.");
      simulated = structuredClone(args.profile as Profile);
      result = { message: "Applied to the simulated device only." };
      break;
    case "read_configuration":
      if (!prefs.demo_mode)
        throw new Error("Open the native application for device access.");
      simulated ??= createProfile("Simulated device");
      result = {
        kind: "mock",
        profile: simulated,
        lighting: simulated.lighting,
      };
      break;
    case "backup_configuration": {
      if (!prefs.demo_mode) throw new Error("No readable physical device.");
      const id = crypto.randomUUID();
      snapshots.set(
        id,
        structuredClone(simulated ?? createProfile("Simulated device")),
      );
      result = {
        id,
        message:
          "Created a simulated configuration backup for this preview session.",
      };
      break;
    }
    case "list_backups":
      result = [...snapshots.keys()].map(
        (id) =>
          ({
            id,
            created_at: new Date().toISOString(),
            device: "Simulated F75",
            kind: "mock",
          }) satisfies Backup,
      );
      break;
    case "restore_backup": {
      const p = snapshots.get(String(args.id));
      if (!p || !prefs.demo_mode)
        throw new Error("Simulated backup is unavailable.");
      simulated = structuredClone(p);
      result = { message: "Restored the simulated device configuration." };
      break;
    }
    case "diagnostics":
      result = {
        application: "AULA Studio browser preview",
        mode: prefs.demo_mode ? "demo" : "hardware",
        physical_access: false,
        profiles: profiles.length,
        platform: navigator.platform,
        notes: [
          "No HID packets are transmitted from this preview.",
          "Native builds store profiles in XDG data directories.",
        ],
      };
      break;
    default:
      throw new Error(`Unknown preview command: ${command}`);
  }
  return result as T;
}
export function call<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  return native ? invoke<T>(command, args) : preview<T>(command, args);
}
