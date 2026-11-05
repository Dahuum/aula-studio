# Model workspaces — 0.5.0

The table below records the published0.5 scope. Unreleased0.6 adds an explicitly partial103-position F2088 workspace and [F98 Pro lighting drafts](F98_RESEARCH.md):20 presets,0–5 sliders, per-key colors and per-effect disabled states, preserving older local profiles. Neither model gains physical access. Five exact G5 variants separately gain experimental read-only captures as documented in [G5_READ_CONTRACTS.md](G5_READ_CONTRACTS.md); those reads are not new-model write support.

Open **More views → Device library** in the native application. This is a local editor for additional resource-derived layouts, separate from the F75 hardware controller. It works without owning those keyboards. It does not configure them.

## Availability

| Model/resource variant | Resource keys | Local workspace | Hardware access |
| --- | ---: | --- | --- |
| Original ANSI F75 | 80 | Existing F75 controller | Existing scoped controller; identity checked before configuration access |
| F87 | 87 | Lighting/color drafts, assignments, macros, profiles | Disabled |
| F87 Pro identity010b | 87 | Same local workflows with separate identity/profile binding | Disabled |
| F87 Pro Wired | 87 | Same local workflows with separate identity/profile binding | Disabled |
| F108 | 108 | Lighting/color drafts, assignments, macros, profiles | Disabled |
| H108 | 108 | Lighting/color drafts, assignments, macros, profiles | Disabled |
| S75 Pro | 80 | Lighting/color drafts, base/Fn intent, macros, profiles | Disabled |
| S99 Pro | 99 | Lighting/color drafts, assignments, macros, profiles | Disabled |
| F108 Pro | 104 | Lighting/color drafts, base/Fn intent, macros, profiles | Disabled |
| F98 Pro | 95 | Assignment/macro/profile editor; lighting definitions unavailable | Disabled |
| F2088, SC620, SC580/580 X, S12 Pro | Unknown | Research metadata only | Disabled |

Counts describe visible entries in the acquired resources, not verified physical models. S99 contains conflicting matrix/LED indices; ambiguous indices remain null. F98 resource count95 has not been reconciled with physical hardware. Do not infer layouts from marketing names. Only explicitly recovered lighting modes/ranges are offered; this can be a single default mode.

## Editing

Select a model, click keys, use Shift/Ctrl-click for multiple selection, and edit a local draft. Assignments are HID intent, not guessed vendor encodings. Protected controls cannot be remapped. Fn editing appears only for resource-declared models; no Fn hardware behavior is asserted. Color previews show the draft, not physical readback or a simulated animation.

Local macros use the selected layout, including numpad keys. Recording works only inside the focused recording area. Preview highlights events and does not type into other applications. Import/export uses the same version1 macro wrapper as the F75 editor. Onboard uploading is unavailable.

Save, duplicate, rename, load, delete, import and export profiles on the Profiles tab. Unsaved changes are indicated; switching models or leaving prompts before discarding edits. Export can validate/export an unsaved draft. Failed validation preserves the saved profile. Import creates a fresh ID. Changing a model ID in an existing profile cannot overwrite a profile belonging to another model.

## Storage and safety

Native model profiles are separate version1 documents under `$XDG_DATA_HOME/aula-studio/model-profiles/` (default `~/.local/share/aula-studio/model-profiles/`). `AULA_DATA_DIR` redirects the root for isolated QA. Files are atomically written with mode0600, new directories0700; replacement/deletion retains a history copy. They cannot be imported as legacy F75 hardware profiles. Profile format keys include `model_id`, local colors, base/Fn assignments and balanced macro events. Imports/exports are bounded to512KiB and validated by Rust.

No new-model Apply/Upload action or hardware transport is exposed. Discovery reads USB metadata and preserves multiple candidates for shared IDs. The F75 controller independently checks its six-byte model identity before configuration access, so other `258a:010c` keyboards cannot inherit F75 write permissions. Receiver discovery never enables wireless configuration.

Browser preview uses a separate localStorage namespace and explicitly has no physical access. Native storage validation remains authoritative. No backend success from saving a profile is presented as a physical device change.

## CLI

```sh
./target/release/aula models
./target/release/aula model-devices
./target/release/aula model-profile exported-model-profile.json
./target/release/aula identity /dev/hidrawN
```

The last command is an explicit bounded G5 identity read, restricted to the known wired descriptor/interface. Enumerate first: hidraw numbers change. It does not probe Sonix/S99/receiver commands or write configuration. The exact identity of the local wired F75 was successfully read as `03 00 00 00 00 cd`; that does not validate any other device.

See [catalog acquisition](DOWNLOAD_CATALOG.md), [static protocol evidence](MULTI_MODEL_RESEARCH.md), [model evidence requirements](MODEL_SUPPORT.md) and [tests](TESTING.md). Hardware validation must be tracked separately from local editing and packet tests.
