import { useEffect, useState } from "react";
import { version } from "../package.json";
import { call, native } from "./api";
import { Keyboard } from "./Keyboard";
import { Macros } from "./Macros";
import { copyName } from "./copyName";
import { ModelWorkbench } from "./ModelWorkbench";
import { OriginalNavigation, OriginalTop, UiIcon, useOriginalFrame } from "./OriginalChrome";
import { OriginalColors } from "./OriginalColors";
import { KEYBOARD, KEYBOARD_PREVIEW } from "./layout";
import {
  createProfile,
  changed,
  effects,
  parseProfile,
  validateMacro,
  fileName,
  lightingCapabilities,
  mediaActions,
  mouseActions,
} from "./model";
import type {
  Profile,
  Status,
  Preferences,
  Backup,
  ReadConfiguration,
  PreparedLighting,
  LightingWriteResult,
} from "./model";
export default function App() {
  const [page, setPage] = useState("Lighting"),
    [profile, setProfile] = useState<Profile>(createProfile()),
    [profiles, setProfiles] = useState<Profile[]>([]),
    [status, setStatus] = useState<Status>(),
    [preferences, setPreferences] = useState<Preferences>({
      demo_mode: false,
      reduce_motion: false,
    }),
    [selected, setSelected] = useState<string[]>([]),
    [message, setMessage] = useState(""),
    [error, setError] = useState(""),
    [busy, setBusy] = useState(false),
    [text, setText] = useState(""),
    [modal, setModal] = useState(""),
    [diagnostic, setDiagnostic] = useState(""),
    [backups, setBackups] = useState<Backup[]>([]),
    [target, setTarget] = useState(4),
    [actionType, setActionType] = useState<"keyboard" | "media" | "mouse">("keyboard"),
    [readConfiguration, setReadConfiguration] = useState<ReadConfiguration>(),
    [preparedLighting, setPreparedLighting] = useState<PreparedLighting>();
  const originalFrame=useOriginalFrame();
  const navigate=(next:string)=>{setPage(next);setMessage("");setError("");};
  const saved = profiles.find((p) => p.id === profile.id),
    dirty = changed(profile, saved);
  const refresh = async () => {
    const [s, p, b] = await Promise.all([
      call<Status>("get_status"),
      call<Profile[]>("list_profiles"),
      call<Backup[]>("list_backups"),
    ]);
    setStatus(s);
    setProfiles(p);
    setBackups(b);
  };
  const action = async (fn: () => Promise<unknown>) => {
    setBusy(true);
    setError("");
    setMessage("");
    try {
      await fn();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  useEffect(() => {
    void action(async () => {
      const p = await call<Preferences>("get_preferences");
      setPreferences(p);
      const list = await call<Profile[]>("list_profiles");
      if (list.length) setProfile(list[0]);
      await refresh();
    });
    const timer = setInterval(() => {
      call<Status>("get_status")
        .then(setStatus)
        .catch((e) => setError(String(e)));
    }, 3000);
    return () => clearInterval(timer);
  }, []);
  useEffect(() => {
    const handler = (e: BeforeUnloadEvent) => {
      if (dirty || busy) {
        e.preventDefault();
        e.returnValue = "";
      }
    };
    window.addEventListener("beforeunload", handler);
    return () => window.removeEventListener("beforeunload", handler);
  }, [dirty,busy]);
  useEffect(() => {
    if (!modal && !preparedLighting) return;
    const previous = document.activeElement as HTMLElement | null;
    const dialog = document.querySelector<HTMLElement>('[role="dialog"]');
    const focusable = () =>
      Array.from(
        dialog?.querySelectorAll<HTMLElement>(
          'button:not(:disabled), textarea, input:not(:disabled), select:not(:disabled), [tabindex="0"]',
        ) ?? [],
      );
    // Start at the review heading instead of scrolling a long review to Cancel.
    if (dialog) {
      dialog.tabIndex = -1;
      dialog.focus({ preventScroll: true });
      dialog.scrollTop = 0;
    }
    const handleKey = (event: globalThis.KeyboardEvent) => {
      if (event.key === "Escape" && !busy) {
        setModal("");
        setPreparedLighting(undefined);
      }
      if (event.key !== "Tab") return;
      const elements = focusable();
      const first = elements[0],
        last = elements.at(-1);
      if (document.activeElement === dialog) {
        event.preventDefault();
        (event.shiftKey ? last : first)?.focus();
      } else if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last?.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first?.focus();
      }
    };
    document.addEventListener("keydown", handleKey);
    return () => {
      document.removeEventListener("keydown", handleKey);
      previous?.focus();
    };
  }, [modal, preparedLighting, busy]);
  const patch = (p: Partial<Profile>) =>
    setProfile((v) => ({ ...v, ...p, updated_at: new Date().toISOString() }));
  const save = () =>
    action(async () => {
      for (const m of profile.macros) {
        const e = validateMacro(m.events);
        if (e) throw new Error(e);
      }
      await call("save_profile", { profile });
      await refresh();
      setMessage("Profile saved locally.");
    });
  const load = (p: Profile) => {
    if (
      dirty &&
      !window.confirm("Discard unsaved changes to the current profile?")
    )
      return;
    setProfile(structuredClone(p));
    setSelected([]);
  };
  const exportText = (name: string, value: string) => {
    setText(value);
    setModal(`Export ${name}`);
  };
  const select = (id: string, multi: boolean) =>
    setSelected((v) =>
      multi ? (v.includes(id) ? v.filter((k) => k !== id) : [...v, id]) : [id],
    );
  const toggleDemo = () =>
    action(async () => {
      await call("set_demo_mode", { enabled: !preferences.demo_mode });
      setReadConfiguration(undefined);
      setPreparedLighting(undefined);
      setPreferences((v) => ({ ...v, demo_mode: !v.demo_mode }));
      await refresh();
    });
  const readDevice = () =>
    action(async () => {
      const configuration = await call<ReadConfiguration>("read_configuration");
      setReadConfiguration(configuration);
      setDiagnostic(JSON.stringify(configuration, null, 2));
      setPage("Diagnostics");
      setMessage(
        "Configuration read. Use Load read lighting into draft to edit these settings locally.",
      );
    });
  const apply = () =>
    action(async () => {
      if (status?.mode === "demo") {
        const result = await call<{ message: string }>("apply_profile", {
          profile,
        });
        setMessage(result.message);
        return;
      }
      const capability =
        page === "Per-Key RGB"
          ? status?.capabilities.write_per_key
          : page === "Key Remapping"
            ? status?.capabilities.write_keymap
            : page === "Fn Layers"
              ? status?.capabilities.write_fn
              : status?.capabilities.write_lighting;
      if (!native || !capability)
        throw new Error(
          "A supported wired device is required to prepare a lighting write.",
        );
      const kind =
        page === "Per-Key RGB"
          ? "per_key"
          : page === "Key Remapping"
            ? "base"
            : page === "Fn Layers"
              ? "fn"
              : page === "Device Settings"
                ? "settings"
                : "lighting";
      const prepared = await call<PreparedLighting>(
        kind === "lighting"
          ? "prepare_lighting"
          : kind === "settings"
            ? "prepare_settings"
            : "prepare_mapping",
        kind === "lighting" || kind === "settings"
          ? { profile }
          : { profile, kind },
      );
      setPreparedLighting({ ...prepared, kind });
      await refresh();
    });
  const commitLighting = () =>
    action(async () => {
      if (!preparedLighting) return;
      const token = preparedLighting.token;
      // A preparation is single-use. A failed commit must be prepared again.
      setPreparedLighting(undefined);
      const result = await call<LightingWriteResult>(
        ["base", "fn", "per_key", "mapping_restore"].includes(
          preparedLighting.kind ?? "",
        )
          ? "commit_mapping"
          : "commit_lighting",
        {
          token,
        },
      );
      setMessage(
        `${result.message} ${result.readback_verified ? "Readback verified." : "Readback was not verified."} Persistence after reconnection has not been tested.`,
      );
      setReadConfiguration(undefined);
      await refresh();
    });
  const remap = page === "Key Remapping" || page === "Fn Layers",
    layer = page === "Fn Layers" ? "fn_layer" : "keymap";
  const consumerLayer = page === "Fn Layers" ? "consumer_fn_layer" : "consumer_keymap";
  const mouseLayer = page === "Fn Layers" ? "mouse_fn_layer" : "mouse_keymap";
  const available = selected.filter(
    (id) => page === "Per-Key RGB" || (id !== "fn" && !(+id >= 58 && +id <= 69)),
  );
  const lightingControls = lightingCapabilities(profile.lighting.effect);
  const scope =
    page === "Per-Key RGB"
      ? "per-key colors"
      : page === "Key Remapping"
        ? "base layer"
        : page === "Fn Layers"
          ? "Fn layer"
          : page === "Device Settings"
            ? "device settings"
            : "lighting";
  const reviewScope =
    preparedLighting?.kind === "brightness"
      ? "brightness"
      : preparedLighting?.kind === "mapping_restore"
        ? "bank restore"
        : preparedLighting?.kind === "per_key"
          ? "per-key colors"
          : preparedLighting?.kind === "base"
            ? "base layer"
            : preparedLighting?.kind === "fn"
              ? "Fn layer"
              : preparedLighting?.kind === "settings"
                ? "device settings"
                : preparedLighting?.kind === "restore"
                  ? "lighting restore"
                  : "lighting";
  const readBank = () =>
    action(async () => {
      const kind =
        page === "Per-Key RGB"
          ? "per_key"
          : page === "Fn Layers"
            ? "fn"
            : "base";
      const result = await call<{
        assignments?: Record<string, number>;
        consumer_assignments?: Record<string, number>;
        mouse_assignments?: Record<string, number>;
        colors?: Profile["per_key"];
      }>("read_mapping", { kind });
      setDiagnostic(JSON.stringify(result, null, 2));
      if (
        window.confirm(
          "Load the readable assignments/colors into this local draft? This replaces that draft bank and sends no writes.",
        )
      ) {
        if (kind === "per_key") patch({ per_key: result.colors ?? {} });
        else
          patch({
            [kind === "base" ? "keymap" : "fn_layer"]: result.assignments ?? {},
            [kind === "base" ? "consumer_keymap" : "consumer_fn_layer"]: result.consumer_assignments ?? {},
            [kind === "base" ? "mouse_keymap" : "mouse_fn_layer"]: result.mouse_assignments ?? {},
          });
        setMessage(
          "Loaded the readable bank into the draft. No keyboard settings were changed.",
        );
      }
    });
  if(page==="Models")return <ModelWorkbench demoMode={preferences.demo_mode} onBack={()=>navigate("Lighting")}/>;
  return (
    <div data-page={page} className={`app original-app ${preferences.reduce_motion ? "reduce-motion" : ""}`} style={originalFrame.style}>
      <div inert={!!modal || !!preparedLighting}><OriginalTop assets={originalFrame.assets} dirty={dirty} busy={busy} mode={status?.mode??"hardware"} connected={!!status?.connected} onNavigate={navigate} onClose={()=>{if(!busy&&(!dirty||window.confirm("Discard unsaved changes and close?")))void call("window_action",{action:"close"}).catch(e=>setError(String(e)));}}/></div>
      <OriginalNavigation page={page} effect={profile.lighting.effect} assets={originalFrame.assets} onNavigate={navigate} onEffect={id=>{patch({lighting:{...profile.lighting,effect:id}});navigate(id===21?"Per-Key RGB":"Lighting");}} demo={preferences.demo_mode} connected={!!status?.connected} busy={busy} toggleDemo={toggleDemo} inert={!!modal||!!preparedLighting}/>
      <main inert={!!modal || !!preparedLighting}>
        <div className="original-device-title"><UiIcon kind="keyboard"/> AULA F75 <span>{status?.mode==="demo"?"Demo":status?.connected?"USB":"No wired keyboard"}</span></div>
        <header className="original-actionbar">
          <h1 className="sr-only">{page}</h1>
          <div className="header-actions">
            <button className="original-local-save" title="Save profile locally" aria-label="Save profile" onClick={save} disabled={busy || !dirty}>
              <UiIcon kind="save"/><span className="sr-only">Save profile</span>
            </button>
            <button
              className="primary original-apply"
              title={status?.mode==="demo"?"Apply to demo":`Review ${scope} write`}
              aria-label={status?.mode==="demo"?"Apply to demo":`Review ${scope} write`}
              disabled={
                busy ||
                (page==="Macros"&&status?.mode!=="demo") ||
                !(page === "Per-Key RGB"
                  ? status?.capabilities.write_per_key
                  : page === "Key Remapping"
                    ? status?.capabilities.write_keymap
                    : page === "Fn Layers"
                      ? status?.capabilities.write_fn
                      : status?.capabilities.write_lighting) ||
                (status?.mode === "hardware" && !native)
              }
              onClick={apply}
            >
              <UiIcon kind="save"/><span className="sr-only">{status?.mode === "demo"
                ? "Apply to demo"
                : `Review ${scope} write`}</span>
            </button>
          </div>
        </header>
        {status?.notice && (
          <div className="notice original-connection-note" title={status.notice}>
            <span>i</span>
            <p>{status.notice}</p>
          </div>
        )}
        {error && (
          <div role="alert" className="notice error">
            {error}
            <button onClick={() => setError("")}>×</button>
          </div>
        )}
        {message && (
          <div role="status" className="notice success">
            {message}
            <button onClick={() => setMessage("")}>×</button>
          </div>
        )}
        {(page === "Dashboard" ||
          page === "Lighting" ||
          page === "Per-Key RGB" ||
          remap) && (
          <section className="panel keyboard-panel original-keyboard-panel">
            {remap && (
              <div className="original-layer-tabs" style={{paddingRight:KEYBOARD_PREVIEW.width-KEYBOARD_PREVIEW.right}} role="group" aria-label="Keyboard layer">
                <button aria-label="Default layer" aria-pressed={page === "Key Remapping"} onClick={() => navigate("Key Remapping")}>Default</button>
                <button aria-label="Fn Layers" aria-pressed={page === "Fn Layers"} onClick={() => navigate("Fn Layers")}>FN1</button>
              </div>
            )}
            <div className="panel-heading">
              <div>
                <h2>{profile.name}</h2>
                <p>
                  {remap
                    ? "Select a source key to edit this local layer."
                    : "A preview of your next setup."}
                </p>
              </div>
              <span className="badge">ANSI 75%</span>
            </div>
            <Keyboard
              profile={profile}
              selected={selected}
              onSelect={page === "Per-Key RGB" || remap ? select : undefined}
              showMapping={remap}
              fnLayer={page === "Fn Layers"}
            />
          </section>
        )}
        {page === "Dashboard" && (
          <>
            <div className="stats">
              <section className="panel">
                <p>CONNECTION</p>
                <h2>{status?.connected ? "Connected" : "Disconnected"}</h2>
                <small>
                  {status?.mode === "demo"
                    ? "In-memory simulation"
                    : "Wired USB detection"}
                </small>
              </section>
              <section className="panel">
                <p>ACTIVE LOCAL PROFILE</p>
                <h2>{profile.name}</h2>
                <small>Onboard profile identity unknown</small>
              </section>
              <section className="panel">
                <p>LIGHTING DRAFT</p>
                <h2>
                  {effects.find((e) => e.id === profile.lighting.effect)
                    ?.name ?? "Unknown"}
                </h2>
                <small>Firmware and hardware revision unread</small>
              </section>
            </div>
            <section className="panel">
              <h2>Make room for your workflow.</h2>
              <p>
                Build lighting, key mappings, and local macros, then save a
                versioned profile. Supported wired lighting changes require a
                configuration backup and a separate write confirmation.
              </p>
              <div className="toolbar">
                <button onClick={() => setPage("Lighting")}>
                  Customize lighting →
                </button>
                <button
                  disabled={busy || !status?.capabilities.read_config}
                  onClick={readDevice}
                >
                  Read configuration
                </button>
                <button
                  disabled={busy || !status?.capabilities.read_config}
                  onClick={() =>
                    action(async () => {
                      const r = await call<{ message: string }>(
                        "backup_configuration",
                      );
                      setMessage(r.message);
                      await refresh();
                    })
                  }
                >
                  Back up configuration
                </button>
              </div>
            </section>
          </>
        )}
        {(page === "Lighting" || page === "Per-Key RGB") && (
          <>
            <OriginalColors profile={profile} onChange={lighting=>patch({lighting})} perKey={page==="Per-Key RGB"} wheel={originalFrame.assets.colorwheel} customColors={preferences.custom_colors??[]} onAddColor={()=>void action(async()=>{
              const next={...preferences,custom_colors:[...(preferences.custom_colors??[]),profile.lighting.color]};
              await call("save_preferences",{preferences:next});setPreferences(next);setMessage("Custom color saved locally.");
            })}>
              {page==="Per-Key RGB"&&<div className="original-perkey-tools">
                <p>{available.length} keys selected</p>
                <button onClick={()=>setSelected(KEYBOARD.map(k=>k.id))}>Select all</button>
                <button onClick={()=>setSelected([])}>Clear selection</button>
                <button disabled={!available.length} onClick={()=>{
                  const next={...profile.per_key};available.forEach(id=>next[id]=profile.lighting.color);
                  patch({per_key:next,lighting:{...profile.lighting,effect:21}});
                }}>Assign color</button>
                <button disabled={!available.length} onClick={()=>{
                  const next={...profile.per_key};available.forEach(id=>delete next[id]);patch({per_key:next});
                }}>Clear colors</button>
              </div>}
            </OriginalColors>
            <details className="original-safe-tools"><summary>Read / recover</summary>
              <p>Preview is illustrative. Hardware changes require a backup and a separate review. Direction is unavailable.</p>
              <button disabled={busy||!status?.capabilities.read_config} onClick={readDevice}>Read configuration</button>
              {page==="Lighting"&&<button disabled={busy||!native||status?.mode!=="hardware"||!status.capabilities.write_lighting||!lightingControls.brightness} onClick={()=>void action(async()=>{
                const prepared=await call<PreparedLighting>("prepare_brightness",{value:profile.lighting.brightness});
                setPreparedLighting({...prepared,kind:"brightness"});await refresh();
              })}>Review brightness only</button>}
              <button disabled={!saved} onClick={()=>saved&&load(saved)}>Restore saved draft</button>
            </details>
          </>
        )}
        {remap && (
          <section className="panel original-remap-panel">
            <h2>
              {page === "Fn Layers" ? "Fn layer draft" : "Base layer draft"}
            </h2>
            <p>
              Edit this layer, then review its dedicated hardware write. Fn and
              function-key source positions are protected. Other assignments are
              preserved.
            </p>
            <div className="toolbar">
              <span>
                {selected
                  .map((id) => KEYBOARD.find((k) => k.id === id)?.label)
                  .join(", ") || "Select a key"}
              </span>
              <select
                aria-label="Action type"
                value={actionType}
                onChange={(e) => {
                  const kind = e.target.value as "keyboard" | "media" | "mouse";
                  setActionType(kind);
                  setTarget(kind === "media" ? 0xcd : kind === "mouse" ? 1 : 4);
                }}
              >
                <option value="keyboard">Keyboard key</option>
                <option value="media">Media control</option>
                <option value="mouse">Mouse button</option>
              </select>
              <select
                aria-label={actionType === "media" ? "Media action" : actionType === "mouse" ? "Mouse action" : "Replacement key"}
                value={target}
                onChange={(e) => setTarget(+e.target.value)}
              >
                {actionType === "media" ? mediaActions.map((a) => <option key={a.usage} value={a.usage}>{a.label}</option>) : actionType === "mouse" ? mouseActions.map(a => <option key={a.button} value={a.button}>{a.label}</option>) : KEYBOARD.filter((k) => k.usage !== undefined).map((k) => (
                  <option key={k.id} value={k.usage}>
                    {k.label} ({k.code})
                  </option>
                ))}
              </select>
              <button
                disabled={!available.length}
                onClick={() => {
                  const next = { ...profile[layer] };
                  const media = { ...profile[consumerLayer] };
                  const mouse = { ...profile[mouseLayer] };
                  available.forEach((id) => {
                    delete next[id]; delete media[id]; delete mouse[id];
                    if (actionType === "media") media[id] = target;
                    else if (actionType === "mouse") mouse[id] = target;
                    else next[id] = target;
                  });
                  patch({ [layer]: next, [consumerLayer]: media, [mouseLayer]: mouse });
                }}
              >
                Assign locally
              </button>
              <button
                disabled={!available.length}
                onClick={() => {
                  const next = { ...profile[layer] };
                  const media = { ...profile[consumerLayer] };
                  const mouse = { ...profile[mouseLayer] };
                  available.forEach((id) => { delete next[id]; delete media[id]; delete mouse[id]; });
                  patch({ [layer]: next, [consumerLayer]: media, [mouseLayer]: mouse });
                }}
              >
                Clear selected draft
              </button>
              <button
                onClick={() => {
                  if (
                    window.confirm("Clear all assignments in this local layer?")
                  )
                    patch({ [layer]: {}, [consumerLayer]: {}, [mouseLayer]: {} });
                }}
              >
                Clear layer draft
              </button>
            </div>
            <p className="muted">
              Keyboard, media and mouse assignments share this layer's backed-up review; physical
              behavior remains unverified. App-launch and display controls depend
              on your desktop shortcuts.
            </p>
          </section>
        )}
        {(remap || page === "Per-Key RGB") && (
          <details className="original-readbank-tools">
            <summary>Read this bank</summary>
            <p>
              Load supported assignments from the keyboard without changing it.
              Vendor actions and Fn controls stay in the raw backup.
            </p>
            <button
              disabled={
                busy ||
                status?.mode !== "hardware" ||
                !status.capabilities.read_config
              }
              onClick={readBank}
            >
              Read bank into draft
            </button>
          </details>
        )}
        {page === "Macros" && (
          <Macros
            macros={profile.macros}
            uiAssets={originalFrame.assets}
            onChange={(macros) => patch({ macros })}
            onExport={exportText}
            onWirePreview={native ? (localMacro) => void action(async () => {
              const report = await call("preview_f75_macro_wire", {localMacro});
              exportText("F75 macro wire inspection — offline", JSON.stringify(report,null,2));
            }) : undefined}
            onImport={() => {
              setText("");
              setModal("Import macro");
            }}
          />
        )}
        {page === "Profiles" && (
          <section className="panel">
            <div className="panel-heading">
              <h2>Local profile library</h2>
              <div className="toolbar">
                <button onClick={() => load(createProfile())}>+ New</button>
                <button
                  onClick={() => {
                    const p = structuredClone(profile);
                    p.id = crypto.randomUUID();
                    p.name = copyName(p.name);
                    load(p);
                  }}
                >
                  Duplicate
                </button>
                <button
                  onClick={() => {
                    setText("");
                    setModal("Import profile");
                  }}
                >
                  Import
                </button>
                <button
                  onClick={() =>
                    exportText(profile.name, JSON.stringify(profile, null, 2))
                  }
                >
                  Export draft
                </button>
              </div>
            </div>
            <label className="field">
              Current profile name
              <input
                value={profile.name}
                maxLength={80}
                onChange={(e) => patch({ name: e.target.value })}
              />
            </label>
            {profiles.length === 0 ? (
              <p>No saved profiles. Use Save profile to keep this draft.</p>
            ) : (
              profiles.map((p) => (
                <div className="profile-row" key={p.id}>
                  <div>
                    <b>{p.name}</b>
                    <small>{p.updated_at}</small>
                  </div>
                  <button onClick={() => load(p)}>Load</button>
                  <button
                    className="danger"
                    onClick={() =>
                      action(async () => {
                        if (
                          window.confirm(
                            `Delete ${p.name}? A history copy is retained.`,
                          )
                        ) {
                          await call("delete_profile", { id: p.id });
                          await refresh();
                        }
                      })
                    }
                  >
                    Delete
                  </button>
                </div>
              ))
            )}
          </section>
        )}
        {page === "Device Settings" && (
          <>
            <section className="panel">
              <h2>Application preferences</h2>
              <label className="field inline">
                <input
                  type="checkbox"
                  checked={preferences.demo_mode}
                  onChange={toggleDemo}
                />{" "}
                Demo mode — no hardware changes
              </label>
              <label className="field inline">
                <input
                  type="checkbox"
                  checked={preferences.reduce_motion}
                  onChange={(e) => {
                    const next = {
                      ...preferences,
                      reduce_motion: e.target.checked,
                    };
                    void action(async () => {
                      await call("save_preferences", { preferences: next });
                      setPreferences(next);
                      await refresh();
                    });
                  }}
                />{" "}
                Reduce motion
              </label>
              <p>
                Debounce, polling, OS layout, and onboard profile settings
                remain under investigation. The standard F75 sleep timer can be
                reviewed separately below.
              </p>
            </section>
            <section className="panel">
              <h2>Hardware sleep timer</h2>
              <p>
                Supported standard F75 setting. Save it with your profile, then
                use Review device settings write. Other settings are preserved.
              </p>
              <label className="field">
                Sleep after
                <select
                  aria-label="Hardware sleep timer"
                  value={profile.sleep_seconds ?? ""}
                  onChange={(e) => patch({ sleep_seconds: +e.target.value })}
                >
                  <option value="" disabled>
                    Choose a value
                  </option>
                  {[0, 60, 90, 120, 180, 240, 300, 600, 900, 1200].map((s) => (
                    <option key={s} value={s}>
                      {s === 0 ? "Sleep disabled" : `${s / 60} minutes`}
                    </option>
                  ))}
                </select>
              </label>
              <button
                disabled={
                  busy ||
                  status?.mode !== "hardware" ||
                  !status.capabilities.read_config
                }
                onClick={() =>
                  action(async () => {
                    const v =
                      await call<ReadConfiguration>("read_configuration");
                    if (
                      v.sleep_seconds === null ||
                      v.sleep_seconds === undefined
                    )
                      throw new Error(
                        "Sleep field does not match the supported resource table.",
                      );
                    patch({ sleep_seconds: v.sleep_seconds });
                    setMessage(
                      "Loaded sleep timer into the local draft. No settings changed.",
                    );
                  })
                }
              >
                Read sleep into draft
              </button>
            </section>
            <section className="panel">
              <h2>Configuration snapshots</h2>
              <button
                disabled={busy || !status?.capabilities.read_config}
                onClick={() =>
                  action(async () => {
                    const r = await call<{ message: string }>(
                      "backup_configuration",
                    );
                    setMessage(r.message);
                    await refresh();
                  })
                }
              >
                Create backup
              </button>
              {backups.map((b) => (
                <div className="profile-row" key={b.id}>
                  <span>
                    {b.id} · {b.kind}
                  </span>
                  {b.kind === "hardware" && b.mapping_kind && (
                    <button
                      disabled={
                        busy ||
                        status?.mode !== "hardware" ||
                        !status.capabilities.write_lighting
                      }
                      onClick={() =>
                        action(async () => {
                          const review = await call<PreparedLighting>(
                            "prepare_mapping_restore",
                            { id: b.id },
                          );
                          setPreparedLighting({
                            ...review,
                            kind: "mapping_restore",
                          });
                          await refresh();
                        })
                      }
                    >
                      Review bank restore
                    </button>
                  )}
                  {b.kind === "hardware" && (
                    <button
                      disabled={
                        busy ||
                        status?.mode !== "hardware" ||
                        !status.capabilities.write_lighting
                      }
                      onClick={() =>
                        action(async () => {
                          const review = await call<PreparedLighting>(
                            "prepare_restore",
                            { id: b.id },
                          );
                          setPreparedLighting({ ...review, kind: "restore" });
                          await refresh();
                        })
                      }
                    >
                      Review lighting restore
                    </button>
                  )}
                  <button
                    disabled={status?.mode !== "demo" || b.kind === "hardware"}
                    onClick={() =>
                      action(async () => {
                        const r = await call<{ message: string }>(
                          "restore_backup",
                          { id: b.id },
                        );
                        setMessage(r.message);
                      })
                    }
                  >
                    Restore demo
                  </button>
                </div>
              ))}
            </section>
          </>
        )}
        {page === "Diagnostics" && (
          <section className="panel">
            <div className="panel-heading">
              <h2>Device & protocol diagnostics</h2>
              <div className="toolbar">
                <button
                  onClick={() =>
                    action(async () =>
                      setDiagnostic(
                        JSON.stringify(await call("diagnostics"), null, 2),
                      ),
                    )
                  }
                >
                  Refresh
                </button>
                <button
                  disabled={busy || !status?.capabilities.read_config}
                  onClick={readDevice}
                >
                  Read configuration
                </button>
                <button
                  onClick={() =>
                    exportText(
                      "diagnostics",
                      diagnostic || JSON.stringify(status, null, 2),
                    )
                  }
                >
                  Export JSON
                </button>
              </div>
            </div>
            {readConfiguration?.lighting && (
              <div className="notice">
                <div>
                  <strong>Readable lighting settings are available</strong>
                  <p>
                    Loading updates this local draft. It does not write to the
                    keyboard.
                  </p>
                </div>
                <button
                  onClick={() => {
                    const lighting = readConfiguration.lighting!;
                    patch({
                      lighting: {
                        effect: lighting.effect ?? profile.lighting.effect,
                        brightness:
                          lighting.brightness ?? profile.lighting.brightness,
                        speed: lighting.speed ?? profile.lighting.speed,
                        color: lighting.color ?? profile.lighting.color,
                        colorful: lighting.colorful ?? false,
                        direction:
                          lighting.direction ?? profile.lighting.direction,
                      },
                    });
                    setPage("Lighting");
                    setMessage(
                      "Read lighting settings loaded into the local draft. Save the profile to keep them.",
                    );
                  }}
                >
                  Load read lighting into draft
                </button>
              </div>
            )}
            <pre>{diagnostic || JSON.stringify(status, null, 2)}</pre>
            <p className="muted">
              Device paths and descriptors are included; serial numbers and
              unrelated devices are omitted. On Linux, use the supplied narrow
              uaccess rule for access errors. On Windows and macOS, check device
              access and close other configurators.
            </p>
          </section>
        )}
        {page === "About" && (
          <section className="panel">
            <h2>AULA Studio {version}</h2>
            <p>
              An independent native desktop workspace for the original F75.
              Original code is MIT licensed; this project is not affiliated with
              AULA.
            </p>
            <p>
              Local profiles, RGB drafts, remapping drafts, macros and demo
              workflows are implemented. Read-only wired access is gated by
              interface identity and report descriptors. Supported wired
              lighting writes require a backup, concrete review and explicit
              confirmation. Theme switching has user confirmation; a scoped brightness change/readback/restoration test passed on wired ANSI F75. Other operations and power-cycle persistence await physical validation.
              Per-key and base/Fn mapping uploads have separate backed-up
              reviews. Onboard macro upload, wireless configuration and firmware
              operations remain unavailable.
            </p>
            <p>
              See README.md and docs/FEATURE_MATRIX.md in the distribution for
              the precise evidence and validation status.
            </p>
          </section>
        )}
      </main>
      {preparedLighting && (
        <div className="modal-backdrop">
          <section
            className="modal lighting-review"
            role="dialog"
            aria-modal="true"
            aria-label={`Review ${reviewScope} changes`}
          >
            <h2>Review {reviewScope} changes</h2>
            <p>{preparedLighting.summary}</p>
            <dl className="review-details">
              <dt>Device</dt>
              <dd>{preparedLighting.device}</dd>
              <dt>Backup</dt>
              <dd>{preparedLighting.backup_id}</dd>
              <dt>Validation</dt>
              <dd>{preparedLighting.kind === "brightness"
                ? "Brightness change/readback/restoration verified once on wired ANSI F75; persistence unverified."
                : preparedLighting.kind === "lighting"
                  ? "Theme switching has user confirmation; other lighting controls require physical validation."
                  : "Protocol verified; this operation still awaits physical validation."}</dd>
              <dt>Scope</dt>
              <dd>{preparedLighting.summary} Macros are not uploaded.</dd>
            </dl>
            {preparedLighting.changes.length ? (
              <table className="change-table">
                <thead>
                  <tr>
                    <th>Setting</th>
                    <th>Current device</th>
                    <th>Requested</th>
                  </tr>
                </thead>
                <tbody>
                  {preparedLighting.changes.map((change, i) => (
                    <tr key={`${change.field}-${i}`}>
                      <td>{change.field}</td>
                      <td>
                        {typeof change.before === "string"
                          ? change.before
                          : JSON.stringify(change.before)}
                      </td>
                      <td>
                        {typeof change.after === "string"
                          ? change.after
                          : JSON.stringify(change.after)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            ) : (
              <p>There are no lighting changes to write.</p>
            )}
            {preparedLighting.warnings.map((warning, i) => (
              <p className="notice" key={i}>
                {warning}
              </p>
            ))}
            <p className="muted">
              Writing changes the physical keyboard. The application checks
              readback; persistence after reconnecting remains unverified.
            </p>
            <div className="toolbar end">
              <button
                disabled={busy}
                onClick={() => setPreparedLighting(undefined)}
              >
                Cancel
              </button>
              <button
                className="primary"
                disabled={
                  busy ||
                  !preparedLighting.changes.length ||
                  status?.mode !== "hardware" ||
                  !native
                }
                onClick={commitLighting}
              >
                Write {reviewScope} to keyboard
              </button>
            </div>
          </section>
        </div>
      )}
      {modal && (
        <div className="modal-backdrop">
          <section
            className="modal"
            role="dialog"
            aria-modal="true"
            aria-label={modal}
          >
            <h2>{modal}</h2>
            <textarea
              aria-label="JSON content"
              value={text}
              onChange={(e) => setText(e.target.value)}
            />
            {modal.startsWith("Import") && (
              <input
                type="file"
                accept=".json,application/json"
                onChange={(e) => {
                  const f = e.target.files?.[0];
                  if (f && f.size <= 524288) void f.text().then(setText);
                  else setError("Choose a JSON file smaller than 512 KiB.");
                }}
              />
            )}
            <div className="toolbar end">
              <button onClick={() => setModal("")}>Close</button>
              {modal.startsWith("Import") ? (
                <button
                  className="primary"
                  onClick={() =>
                    action(async () => {
                      if (modal === "Import macro") {
                        const v = JSON.parse(text);
                        const m = v.macro;
                        if (
                          v.schema_version !== 1 ||
                          profile.macros.length >= 64 ||
                          !m ||
                          typeof m.name !== "string" ||
                          !m.name.trim() ||
                          new TextEncoder().encode(m.name).length > 120 ||
                          !Array.isArray(m.events) ||
                          validateMacro(m.events)
                        )
                          throw new Error("Invalid macro");
                        patch({
                          macros: [
                            ...profile.macros,
                            { name: m.name, events: m.events, id: crypto.randomUUID() },
                          ],
                        });
                      } else {
                        parseProfile(text);
                        const p = await call<Profile>("import_profile", {
                          json: text,
                        });
                        await refresh();
                        load(p);
                      }
                      setModal("");
                      setMessage("Imported locally.");
                    })
                  }
                >
                  Import
                </button>
              ) : (
                <button
                  className="primary"
                  onClick={() => {
                    const a = document.createElement("a");
                    const url = URL.createObjectURL(
                      new Blob([text], { type: "application/json" }),
                    );
                    a.href = url;
                    a.download = fileName(modal) + ".json";
                    a.click();
                    setTimeout(() => URL.revokeObjectURL(url), 1000);
                  }}
                >
                  Download JSON
                </button>
              )}
            </div>
          </section>
        </div>
      )}
    </div>
  );
}
