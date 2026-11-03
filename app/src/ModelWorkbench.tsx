import { useEffect, useRef, useState } from "react";
import { call, native } from "./api";
import { Macros } from "./Macros";
import { mediaActions, rgbHex, validateMacro } from "./model";
import { modelCode } from "./modelLibrary";
import type {
  ModelDefinition,
  ModelDraft,
  ModelKey,
  ModelDevice,
  ModelAssignment,
} from "./modelLibrary";
import type { Key } from "./layout";
import type { RGB } from "./model";
import "./models.css";

const serialise = (p: ModelDraft | undefined) => JSON.stringify(p);
const usb = (v: number) => v.toString(16).padStart(4, "0");
export function ModelWorkbench({ onBack }: { onBack: () => void }) {
  const [models, setModels] = useState<ModelDefinition[]>([]),
    [devices, setDevices] = useState<ModelDevice[]>([]),
    [profiles, setProfiles] = useState<ModelDraft[]>([]);
  const [modelId, setModelId] = useState(""),
    [draft, setDraft] = useState<ModelDraft>(),
    [baseline, setBaseline] = useState<string>(),
    [selected, setSelected] = useState<string[]>([]);
  const [tab, setTab] = useState("Lighting"),
    [busy, setBusy] = useState(false),
    [error, setError] = useState(""),
    [message, setMessage] = useState(""),
    [query, setQuery] = useState("");
  const [assignmentKind, setAssignmentKind] = useState<"keyboard" | "consumer">(
      "keyboard",
    ),
    [assignment, setAssignment] = useState(4),
    [jsonText, setJsonText] = useState(""),
    [jsonTitle, setJsonTitle] = useState("");
  const importFile = useRef<HTMLInputElement>(null);
  const modalRef = useRef<HTMLDivElement>(null);
  const pendingProfile = useRef<ModelDraft | undefined>(undefined);
  const model = models.find((m) => m.id === modelId),
    dirty = serialise(draft) !== baseline,
    stored = profiles.some((p) => p.id === draft?.id);
  const perform = async (fn: () => Promise<void>) => {
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
  const refreshProfiles = async () =>
    setProfiles(await call<ModelDraft[]>("list_model_profiles"));
  useEffect(() => {
    void perform(async () => {
      const [m, p, d] = await Promise.all([
        call<ModelDefinition[]>("list_models"),
        call<ModelDraft[]>("list_model_profiles"),
        call<ModelDevice[]>("model_devices"),
      ]);
      setModels(m);
      setProfiles(p);
      setDevices(d);
      setModelId(
        m.find((v) => v.id !== "f75-ansi" && v.keys.length)?.id ??
          m[0]?.id ??
          "",
      );
    });
  }, []);
  useEffect(() => {
    if (!model || !model.keys.length || model.id === "f75-ansi") {
      setDraft(undefined);
      setBaseline(undefined);
      return;
    }
    void perform(async () => {
      const pending =
        pendingProfile.current?.model_id === model.id
          ? pendingProfile.current
          : undefined;
      pendingProfile.current = undefined;
      const existing = pending ?? profiles.find((p) => p.model_id === model.id);
      const p =
        existing ??
        (await call<ModelDraft>("new_model_profile", {
          modelId: model.id,
          name: model.name + " profile",
        }));
      setDraft(p);
      setBaseline(serialise(p));
      setSelected([]);
    });
  }, [modelId, models]);
  useEffect(() => {
    const handler = (e: BeforeUnloadEvent) => {
      if (dirty) {
        e.preventDefault();
        e.returnValue = "";
      }
    };
    window.addEventListener("beforeunload", handler);
    return () => window.removeEventListener("beforeunload", handler);
  }, [dirty]);
  useEffect(() => {
    if (!jsonTitle) return;
    const previous = document.activeElement as HTMLElement | null;
    const root = modalRef.current;
    root?.querySelector<HTMLElement>("textarea,button")?.focus();
    const handle = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        if (!busy) setJsonTitle("");
      }
      if (e.key === "Tab" && root) {
        const targets = Array.from(
          root.querySelectorAll<HTMLElement>(
            "textarea:not(:disabled),button:not(:disabled),input:not(:disabled)",
          ),
        );
        const first = targets[0],
          last = targets[targets.length - 1];
        if (e.shiftKey && document.activeElement === first) {
          e.preventDefault();
          last?.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
          e.preventDefault();
          first?.focus();
        }
      }
    };
    document.addEventListener("keydown", handle);
    return () => {
      document.removeEventListener("keydown", handle);
      previous?.focus();
    };
  }, [jsonTitle, busy]);
  const chooseModel = (id: string) => {
    if (busy || id === modelId) return;
    if (
      dirty &&
      !window.confirm("Discard unsaved changes to this model profile?")
    )
      return;
    pendingProfile.current = undefined;
    setDraft(undefined);
    setBaseline(undefined);
    setModelId(id);
    setTab("Lighting");
    setError("");
    setMessage("");
  };
  const patch = (v: Partial<ModelDraft>) =>
    setDraft((p) => (p ? { ...p, ...v } : p));
  const selectKey = (key: ModelKey, multi: boolean) =>
    setSelected((old) =>
      multi
        ? old.includes(key.id)
          ? old.filter((v) => v !== key.id)
          : [...old, key.id]
        : [key.id],
    );
  const chooseProfile = (p: ModelDraft) => {
    if (dirty && !window.confirm("Discard unsaved changes?")) return;
    pendingProfile.current =
      p.model_id !== modelId ? structuredClone(p) : undefined;
    setModelId(p.model_id);
    setDraft(structuredClone(p));
    setBaseline(serialise(p));
    setSelected([]);
  };
  const save = () =>
    perform(async () => {
      if (!draft) return;
      await call("save_model_profile", { profile: draft });
      setBaseline(serialise(draft));
      await refreshProfiles();
      setMessage(
        "Saved to this model’s local profile storage. No hardware operation.",
      );
    });
  const download = (text: string, name: string) => {
    const url = URL.createObjectURL(
      new Blob([text], { type: "application/json" }),
    );
    const a = document.createElement("a");
    a.href = url;
    a.download = name;
    a.click();
    URL.revokeObjectURL(url);
  };
  const exportDraft = () =>
    perform(async () => {
      if (!draft) return;
      const text = await call<string>("model_profile_json", { profile: draft });
      setJsonTitle("Export local model profile");
      setJsonText(text);
    });
  const importText = () =>
    perform(async () => {
      if (
        dirty &&
        !window.confirm(
          "Import this profile and discard current unsaved edits?",
        )
      )
        return;
      const p = await call<ModelDraft>("import_model_profile", {
        json: jsonText,
      });
      await refreshProfiles();
      pendingProfile.current = p.model_id !== modelId ? p : undefined;
      setModelId(p.model_id);
      setDraft(p);
      setBaseline(serialise(p));
      setSelected([]);
      setJsonTitle("");
      setMessage(
        "Imported as a new local profile. Nothing was applied to hardware.",
      );
    });
  const setBinding = (
    layer: "assignments" | "fn_assignments",
    clear = false,
  ) => {
    if (!draft || !model) return;
    const next = { ...draft[layer] };
    for (const id of selected) {
      const key = model.keys.find((k) => k.id === id);
      if (!key || key.protected || key.usage === null) continue;
      if (clear) delete next[id];
      else
        next[id] = {
          kind: assignmentKind,
          usage: assignment,
        } as ModelAssignment;
    }
    patch({ [layer]: next });
  };
  const macroLayout: Key[] =
    model?.keys.map((k) => ({
      id: k.id,
      label: k.label,
      usage: k.usage ?? undefined,
      code: k.usage ? modelCode(k.usage) : "",
      x: k.x,
      y: k.y,
      width: k.width,
    })) ?? [];
  const exit = () => {
    if (!dirty || window.confirm("Discard unsaved model-profile changes?"))
      onBack();
  };
  const close = () => {
    if (
      !dirty ||
      window.confirm("Discard unsaved model-profile changes and close?")
    )
      void call("window_action", { action: "close" }).catch((e) =>
        setError(String(e)),
      );
  };
  return (
    <div className="model-workbench">
      <header className="models-top">
        <button onClick={exit} disabled={busy}>
          ← F75 controls
        </button>
        <h1
          onMouseDown={(e) => {
            if (native && e.button === 0)
              void call("window_action", { action: "drag" }).catch(() => {});
          }}
        >
          AULA Studio <span>Device library</span>
        </h1>
        <span className="models-mode">Local workspaces</span>
        <button
          aria-label="Close application"
          disabled={!native || busy}
          onClick={close}
        >
          ×
        </button>
      </header>
      <div className="models-body">
        <aside className="models-sidebar">
          <label>
            Find a model
            <input
              aria-label="Find a model"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
          </label>
          <nav aria-label="Keyboard and mouse models">
            {models
              .filter((m) => m.name.toLowerCase().includes(query.toLowerCase()))
              .map((m) => (
                <button
                  key={m.id}
                  aria-label={m.name}
                  aria-pressed={m.id === modelId}
                  disabled={busy}
                  onClick={() => chooseModel(m.id)}
                >
                  <strong>{m.name}</strong>
                  <span>
                    {m.id === "f75-ansi"
                      ? "Existing hardware controller"
                      : m.keys.length
                        ? `${m.keys.length} resource keys · offline editor`
                        : "Research only · no controller"}
                  </span>
                </button>
              ))}
          </nav>
          <p>
            Model resources do not establish physical compatibility. New-model
            workspaces never send HID commands.
          </p>
        </aside>
        <main className="models-main">
          {error && (
            <div role="alert" className="models-alert">
              {error}
              <button onClick={() => setError("")}>Dismiss</button>
            </div>
          )}
          {message && (
            <div role="status" className="models-message">
              {message}
            </div>
          )}
          {!model ? (
            <p role="status">
              {busy ? "Loading model library…" : "No matching model."}
            </p>
          ) : (
            <>
              <div className="models-heading">
                <div>
                  <p className="models-eyebrow">{model.family}</p>
                  <h2>{model.name}</h2>
                  <p>
                    {model.id === "f75-ansi"
                      ? "Original ANSI F75 uses the existing reviewed controller."
                      : "Hardware access disabled · resource-derived layout · physical validation pending"}
                  </p>
                </div>
                <span className="model-key-count">
                  {model.keys.length
                    ? `${model.keys.length} keys`
                    : "No recovered layout"}
                </span>
              </div>
              {model.id === "f75-ansi" ? (
                <section className="models-panel">
                  <h3>Open the existing F75 controller</h3>
                  <p>
                    Its bounded reads, reviewed writes and evidence remain
                    separate from new-model research. Shared USB identifiers
                    alone do not grant configuration access.
                  </p>
                  <button onClick={exit}>Open F75 controls</button>
                </section>
              ) : !draft ? (
                <section className="models-panel">
                  <h3>Research entry</h3>
                  <p>
                    No verified layout or protocol adapter is available for this
                    acquired package. No device controls are presented as
                    working.
                  </p>
                  {model.notes.map((n) => (
                    <p key={n}>{n}</p>
                  ))}
                </section>
              ) : (
                <>
                  <div className="models-profile-bar">
                    <label>
                      Profile name
                      <input
                        aria-label="Model profile name"
                        maxLength={120}
                        value={draft.name}
                        onChange={(e) => patch({ name: e.target.value })}
                      />
                    </label>
                    <span>
                      {dirty
                        ? "Unapplied local edits"
                        : stored
                          ? "Saved locally"
                          : "Unsaved local profile"}
                    </span>
                    <button
                      disabled={busy || !draft.name.trim()}
                      onClick={save}
                    >
                      Save local profile
                    </button>
                    <button disabled={busy} onClick={exportDraft}>
                      Export draft
                    </button>
                  </div>
                  <div
                    className="models-keyboard"
                    role="group"
                    aria-label={`${model.name} resource-derived key layout`}
                    style={{
                      aspectRatio: `${Math.max(...model.keys.map((k) => k.x + k.width)) + 16}/${Math.max(...model.keys.map((k) => k.y + k.height)) + 16}`,
                    }}
                  >
                    {model.keys.map((k) => {
                      const w =
                          Math.max(...model.keys.map((v) => v.x + v.width)) +
                          16,
                        h =
                          Math.max(...model.keys.map((v) => v.y + v.height)) +
                          16,
                        color = draft.per_key[k.id] ??
                          draft.lighting?.color ?? [95, 122, 144];
                      return (
                        <button
                          key={k.id}
                          className={selected.includes(k.id) ? "selected" : ""}
                          aria-label={`${k.label} · ${k.id}`}
                          aria-pressed={selected.includes(k.id)}
                          onClick={(e) =>
                            selectKey(k, e.shiftKey || e.ctrlKey || e.metaKey)
                          }
                          style={{
                            left: `${((k.x + 8) / w) * 100}%`,
                            top: `${((k.y + 8) / h) * 100}%`,
                            width: `${(k.width / w) * 100}%`,
                            height: `${(k.height / h) * 100}%`,
                            borderBottomColor: rgbHex(color as RGB),
                          }}
                        >
                          <span>{k.label}</span>
                          {(tab === "Assignments"
                            ? draft.assignments[k.id]
                            : tab === "Fn layer"
                              ? draft.fn_assignments[k.id]
                              : undefined) && <small>mapped</small>}
                        </button>
                      );
                    })}
                  </div>
                  <p className="models-layout-note">
                    Official resource geometry, independently rendered.
                    Shift-click selects multiple keys. No physical key/LED
                    correspondence is claimed verified.
                  </p>
                  <nav
                    className="models-tabs"
                    aria-label="Model workspace views"
                  >
                    {[
                      "Lighting",
                      "Assignments",
                      ...(model.fn_layer ? ["Fn layer"] : []),
                      "Local macros",
                      "Profiles",
                      "Evidence",
                    ].map((name) => (
                      <button
                        key={name}
                        aria-current={tab === name ? "page" : undefined}
                        onClick={() => setTab(name)}
                      >
                        {name}
                      </button>
                    ))}
                  </nav>
                  {tab === "Lighting" && (
                    <section className="models-panel">
                      <h3>Lighting draft</h3>
                      {draft.lighting && model.lighting ? (
                        <>
                          <div className="models-controls">
                            <label>
                              Documented mode
                              <select
                                aria-label="Model lighting mode"
                                value={draft.lighting.effect}
                                onChange={(e) =>
                                  patch({
                                    lighting: {
                                      ...draft.lighting!,
                                      effect: +e.target.value,
                                    },
                                  })
                                }
                              >
                                {model.lighting.effects.map((e) => (
                                  <option key={e.id} value={e.id}>
                                    {e.name} ({e.id})
                                  </option>
                                ))}
                              </select>
                            </label>
                            <label>
                              Brightness {draft.lighting.brightness}/
                              {model.lighting.brightness_max}
                              <input
                                aria-label="Model brightness"
                                type="range"
                                min="0"
                                max={model.lighting.brightness_max}
                                value={draft.lighting.brightness}
                                onChange={(e) =>
                                  patch({
                                    lighting: {
                                      ...draft.lighting!,
                                      brightness: +e.target.value,
                                    },
                                  })
                                }
                              />
                            </label>
                            <label>
                              Speed {draft.lighting.speed}/
                              {model.lighting.speed_max}
                              <input
                                aria-label="Model speed"
                                type="range"
                                min="0"
                                max={model.lighting.speed_max}
                                value={draft.lighting.speed}
                                onChange={(e) =>
                                  patch({
                                    lighting: {
                                      ...draft.lighting!,
                                      speed: +e.target.value,
                                    },
                                  })
                                }
                              />
                            </label>
                            <label>
                              Draft color
                              <input
                                aria-label="Model draft color"
                                type="color"
                                value={rgbHex(draft.lighting.color)}
                                onChange={(e) => {
                                  const s = e.target.value;
                                  patch({
                                    lighting: {
                                      ...draft.lighting!,
                                      color: [
                                        parseInt(s.slice(1, 3), 16),
                                        parseInt(s.slice(3, 5), 16),
                                        parseInt(s.slice(5, 7), 16),
                                      ],
                                    },
                                  });
                                }}
                              />
                            </label>
                          </div>
                          {model.lighting.per_key && (
                            <div className="models-actions">
                              <button
                                disabled={!selected.length}
                                onClick={() => {
                                  const colors = { ...draft.per_key };
                                  for (const id of selected)
                                    colors[id] = draft.lighting!.color;
                                  patch({ per_key: colors });
                                }}
                              >
                                Color selected keys locally
                              </button>
                              <button
                                disabled={!selected.length}
                                onClick={() => {
                                  const colors = { ...draft.per_key };
                                  for (const id of selected) delete colors[id];
                                  patch({ per_key: colors });
                                }}
                              >
                                Clear selected colors
                              </button>
                            </div>
                          )}
                          <p>
                            Only explicitly recovered modes/ranges are listed.
                            The keyboard shows draft colors, not an animation or
                            a physical-device read.
                          </p>
                        </>
                      ) : (
                        <p>
                          This package’s lighting definitions have not been
                          recovered. No lighting controls are enabled.
                        </p>
                      )}
                    </section>
                  )}
                  {(tab === "Assignments" || tab === "Fn layer") && (
                    <section className="models-panel">
                      <h3>
                        {tab === "Fn layer"
                          ? "Fn-layer draft"
                          : "Key assignment draft"}
                      </h3>
                      <p>
                        Selected:{" "}
                        {selected
                          .map(
                            (id) => model.keys.find((k) => k.id === id)?.label,
                          )
                          .join(", ") || "select keys above"}
                        . Protected controls are preserved.
                      </p>
                      <div className="models-controls">
                        <label>
                          Action type
                          <select
                            aria-label="Model action type"
                            value={assignmentKind}
                            onChange={(e) => {
                              setAssignmentKind(
                                e.target.value as "keyboard" | "consumer",
                              );
                              setAssignment(
                                e.target.value === "keyboard" ? 4 : 0xcd,
                              );
                            }}
                          >
                            <option value="keyboard">Keyboard usage</option>
                            <option value="consumer">Media usage</option>
                          </select>
                        </label>
                        <label>
                          Target
                          <select
                            aria-label="Model target action"
                            value={assignment}
                            onChange={(e) => setAssignment(+e.target.value)}
                          >
                            {assignmentKind === "keyboard"
                              ? model.keys
                                  .filter((k) => k.usage !== null)
                                  .map((k) => (
                                    <option key={k.id} value={k.usage!}>
                                      {k.label} · HID {k.usage}
                                    </option>
                                  ))
                              : mediaActions.map((m) => (
                                  <option key={m.usage} value={m.usage}>
                                    {m.label}
                                  </option>
                                ))}
                          </select>
                        </label>
                      </div>
                      <div className="models-actions">
                        <button
                          disabled={
                            !selected.some((id) =>
                              model.keys.some(
                                (k) =>
                                  k.id === id &&
                                  !k.protected &&
                                  k.usage !== null,
                              ),
                            )
                          }
                          onClick={() =>
                            setBinding(
                              tab === "Fn layer"
                                ? "fn_assignments"
                                : "assignments",
                            )
                          }
                        >
                          Assign locally
                        </button>
                        <button
                          disabled={!selected.length}
                          onClick={() =>
                            setBinding(
                              tab === "Fn layer"
                                ? "fn_assignments"
                                : "assignments",
                              true,
                            )
                          }
                        >
                          Restore selected draft defaults
                        </button>
                      </div>
                      <p>
                        Assignments are stored as local HID intent. Unverified
                        wire encodings and unknown matrix indices are never
                        guessed or transmitted.
                      </p>
                    </section>
                  )}
                  {tab === "Local macros" && (
                    <section className="models-panel models-macros">
                      <h3>Local macros</h3>
                      <p>
                        Recording stays in this focused editor. Preview never
                        injects keys into other apps. Onboard uploads are
                        unavailable.
                      </p>
                      <Macros
                        key={draft.id}
                        layout={macroLayout}
                        macros={draft.macros}
                        onChange={(macros) => patch({ macros })}
                        onExport={(name, text) => {
                          setJsonTitle("Export " + name);
                          setJsonText(text);
                        }}
                        onImport={() => {
                          setJsonTitle("Import macro JSON");
                          setJsonText("");
                        }}
                      />
                    </section>
                  )}
                  {tab === "Profiles" && (
                    <section className="models-panel">
                      <h3>Local profiles for {model.name}</h3>
                      <div className="models-actions">
                        <button
                          onClick={() => {
                            const p = {
                              ...structuredClone(draft),
                              id: crypto.randomUUID(),
                              name: (draft.name + " copy").slice(0, 120),
                            };
                            setDraft(p);
                            setBaseline(undefined);
                          }}
                        >
                          Duplicate draft
                        </button>
                        <button
                          onClick={() => {
                            setJsonTitle("Import local model profile");
                            setJsonText("");
                          }}
                        >
                          Import model profile
                        </button>
                        <button
                          disabled={!stored || busy}
                          onClick={() => {
                            if (
                              window.confirm(
                                "Delete this local profile? A history copy is retained.",
                              )
                            )
                              void perform(async () => {
                                await call("delete_model_profile", {
                                  id: draft.id,
                                });
                                await refreshProfiles();
                                const p = await call<ModelDraft>(
                                  "new_model_profile",
                                  {
                                    modelId: model.id,
                                    name: model.name + " profile",
                                  },
                                );
                                setDraft(p);
                                setBaseline(serialise(p));
                              });
                          }}
                        >
                          Delete saved profile
                        </button>
                      </div>
                      <ul>
                        {profiles
                          .filter((p) => p.model_id === model.id)
                          .map((p) => (
                            <li key={p.id}>
                              <button onClick={() => chooseProfile(p)}>
                                {p.name}
                              </button>
                            </li>
                          ))}
                      </ul>
                      <p>
                        Separate model-profile storage prevents these drafts
                        being loaded as a legacy F75 hardware profile. Save,
                        import and load never write a keyboard.
                      </p>
                    </section>
                  )}
                </>
              )}
              {(tab === "Evidence" || !draft) && (
                <section className="models-panel">
                  <h3>Evidence and limits</h3>
                  {model.notes.map((n) => (
                    <p key={n}>{n}</p>
                  ))}
                  <dl>
                    <dt>Model identity</dt>
                    <dd>
                      {model.model_identity
                        ? model.model_identity
                            .map((v) => v.toString(16).padStart(2, "0"))
                            .join(" ")
                        : "Not recovered"}
                    </dd>
                    <dt>USB candidates</dt>
                    <dd>
                      {model.usb.map((u, i) => (
                        <div key={i}>
                          {usb(u.vendor_id)}:{usb(u.product_id)} ·{" "}
                          {u.connection} · interface{" "}
                          {u.interface_number ?? "unknown"}
                        </div>
                      ))}
                    </dd>
                  </dl>
                  {model.evidence.map((e, i) => (
                    <details key={i}>
                      <summary>
                        Source {i + 1} · {e.level}
                      </summary>
                      <p>{e.source_url}</p>
                      <p>Installer SHA256: {e.installer_sha256}</p>
                      <p>
                        {e.configuration_path}
                        <br />
                        SHA256: {e.configuration_sha256}
                      </p>
                    </details>
                  ))}
                </section>
              )}
            </>
          )}
          <section className="models-panel models-discovery">
            <h3>Detected model candidates</h3>
            <button
              disabled={busy}
              onClick={() =>
                void perform(async () =>
                  setDevices(await call<ModelDevice[]>("model_devices")),
                )
              }
            >
              Refresh USB metadata
            </button>
            {devices.length ? (
              devices.map((d) => (
                <article key={d.path}>
                  <strong>{d.product_name || "HID interface"}</strong>
                  <p>
                    {d.path} · {usb(d.vendor_id)}:{usb(d.product_id)} ·
                    interface {d.interface_number}
                  </p>
                  <p>
                    Candidates:{" "}
                    {d.candidates
                      .map((id) => models.find((m) => m.id === id)?.name ?? id)
                      .join(", ")}
                  </p>
                  <small>{d.note}</small>
                </article>
              ))
            ) : (
              <p>
                No catalogued USB candidate is visible. Local editing still
                works.
              </p>
            )}
          </section>
        </main>
      </div>
      <input
        type="file"
        accept=".json,application/json"
        hidden
        ref={importFile}
        onChange={(e) => {
          const f = e.target.files?.[0];
          if (f)
            void perform(async () => {
              if (f.size > 512 * 1024)
                throw new Error("Profile exceeds 512 KiB");
              setJsonText(await f.text());
            });
          e.target.value = "";
        }}
      />
      {jsonTitle && (
        <div
          ref={modalRef}
          className="models-modal"
          role="dialog"
          aria-modal="true"
          aria-label={jsonTitle}
        >
          <div>
            <h2>{jsonTitle}</h2>
            {error && <p role="alert">{error}</p>}
            <textarea
              aria-label="Model JSON"
              value={jsonText}
              onChange={(e) => setJsonText(e.target.value)}
            />
            <div className="models-actions">
              {jsonTitle === "Import local model profile" ? (
                <>
                  <button onClick={() => importFile.current?.click()}>
                    Choose JSON file
                  </button>
                  <button
                    disabled={busy || !jsonText.trim()}
                    onClick={importText}
                  >
                    Import as new profile
                  </button>
                </>
              ) : jsonTitle === "Import macro JSON" ? (
                <button
                  disabled={busy || !jsonText.trim()}
                  onClick={() =>
                    void perform(async () => {
                      const parsed = JSON.parse(jsonText);
                      const m = parsed.macro;
                      if (
                        parsed.schema_version !== 1 ||
                        !m ||
                        typeof m.name !== "string" ||
                        !m.name.trim() ||
                        m.name.length > 80
                      )
                        throw new Error("Invalid macro format");
                      const invalid = validateMacro(m.events);
                      if (invalid) throw new Error(invalid);
                      if ((draft?.macros.length ?? 0) >= 64)
                        throw new Error("Maximum 64 macros");
                      patch({
                        macros: [
                          ...(draft?.macros ?? []),
                          {
                            id: crypto.randomUUID(),
                            name: m.name,
                            events: m.events,
                          },
                        ],
                      });
                      setJsonTitle("");
                    })
                  }
                >
                  Add local macro
                </button>
              ) : (
                <button
                  onClick={() =>
                    download(
                      jsonText,
                      "aula-" + (model?.id ?? "macro") + ".json",
                    )
                  }
                >
                  Download JSON
                </button>
              )}
              <button disabled={busy} onClick={() => setJsonTitle("")}>
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
