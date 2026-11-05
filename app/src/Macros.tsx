import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { KeyboardEvent } from "react";
import { KEYBOARD } from "./layout";
import type { Key } from "./layout";
import { validateMacro } from "./model";
import type { Macro, MacroEvent } from "./model";
import { macroTimeline } from "./macroTimeline";
import { copyName } from "./copyName";
export function Macros({
  macros,
  onChange,
  onExport,
  onImport,
  layout = KEYBOARD,
  onWirePreview,
  uiAssets = {},
}: {
  macros: Macro[];
  layout?: Key[];
  onChange: (value: Macro[]) => void;
  onExport: (name: string, text: string) => void;
  onImport: () => void;
  onWirePreview?: (macro: Macro) => void;
  uiAssets?: Record<string, string>;
}) {
  const codeUsage = Object.fromEntries(
    layout
      .filter((k) => k.usage !== undefined && k.code)
      .map((k) => [k.code, k.usage!]),
  );
  const usageLabel = (usage: number) =>
    layout.find((k) => k.usage === usage)?.label ?? `HID ${usage}`;
  const [selected, setSelected] = useState(macros[0]?.id ?? "");
  const [recording, setRecording] = useState(false);
  const [listRoot, setListRoot] = useState<HTMLElement | null>(null);
  useEffect(
    () => setListRoot(document.getElementById("original-macro-list")),
    [],
  );
  const [playing, setPlaying] = useState(-1);
  const [previewHeld, setPreviewHeld] = useState<number[]>([]);
  const [previewElapsed, setPreviewElapsed] = useState(0);
  const previewGeneration = useRef(0);
  const [cursor, setCursor] = useState<number | null>(null);
  const [insertAt, setInsertAt] = useState("append");
  const [eventPage, setEventPage] = useState(0);
  const pageSize = 100;
  const [key, setKey] = useState(4);
  const held = useRef(new Set<number>());
  const last = useRef(0);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const zone = useRef<HTMLDivElement>(null);
  const nameInput = useRef<HTMLInputElement>(null);
  const macro = macros.find((m) => m.id === selected);
  const update = (next: Macro) =>
    onChange(macros.map((m) => (m.id === next.id ? next : m)));
  const cancelPreview = () => {
    previewGeneration.current++;
    if (timer.current) clearTimeout(timer.current);
    timer.current = null;
    setPlaying(-1);
    setPreviewHeld([]);
    setPreviewElapsed(0);
  };
  useEffect(
    () => () => {
      previewGeneration.current++;
      if (timer.current) clearTimeout(timer.current);
    },
    [],
  );
  // A parent can load another profile while this editor remains mounted.
  // Cancel callbacks tied to the old sequence before displaying its replacement.
  useEffect(() => {
    cancelPreview();
  }, [macro?.id, macro?.events]);
  useEffect(() => {
    held.current.clear();
    last.current = 0;
    setRecording(false);
    setCursor(null);
    setInsertAt("append");
    setEventPage(0);
  }, [macro?.id]);
  useEffect(() => {
    if (recording)
      setEventPage(
        Math.max(0, Math.ceil((macro?.events.length ?? 0) / pageSize) - 1),
      );
    else
      setEventPage((p) =>
        Math.min(
          p,
          Math.max(0, Math.ceil((macro?.events.length ?? 0) / pageSize) - 1),
        ),
      );
  }, [macro?.events.length, recording]);
  useEffect(() => {
    if (cursor !== null) setEventPage(Math.floor(cursor / pageSize));
  }, [cursor]);
  useEffect(() => {
    if (cursor !== null && cursor >= (macro?.events.length ?? 0))
      setCursor(null);
  }, [cursor, macro?.events.length]);
  useEffect(() => {
    if (!macros.some((m) => m.id === selected))
      setSelected(macros[0]?.id ?? "");
  }, [macros, selected]);
  const insertEvents = (events: MacroEvent[]) => {
    if (!macro) return;
    const index =
      cursor === null || insertAt === "append"
        ? macro.events.length
        : Math.min(
            macro.events.length,
            cursor + (insertAt === "after" ? 1 : 0),
          );
    if (macro.events.length + events.length > 2048) return;
    update({
      ...macro,
      events: [
        ...macro.events.slice(0, index),
        ...events,
        ...macro.events.slice(index),
      ],
    });
    setCursor(index);
  };
  const moveEvent = (index: number, delta: number) => {
    if (!macro) return;
    const next = index + delta;
    if (next < 0 || next >= macro.events.length) return;
    const events = [...macro.events];
    [events[index], events[next]] = [events[next], events[index]];
    update({ ...macro, events });
    setCursor(next);
  };
  const stop = () => {
    if (macro && held.current.size) {
      update({
        ...macro,
        events: [
          ...macro.events,
          ...[...held.current].map((key) => ({ kind: "up" as const, key })),
        ],
      });
    }
    held.current.clear();
    setRecording(false);
  };
  const record = (e: KeyboardEvent<HTMLDivElement>, kind: "down" | "up") => {
    if (!recording || !macro) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.code === "Escape") {
      stop();
      return;
    }
    const usage = codeUsage[e.code];
    if (
      usage === undefined ||
      e.repeat ||
      (kind === "down" && held.current.has(usage)) ||
      (kind === "up" && !held.current.has(usage))
    )
      return;
    const now = performance.now();
    const delay = last.current
      ? Math.min(60000, Math.round(now - last.current))
      : 0;
    const nextHeld = new Set(held.current);
    if (kind === "down") nextHeld.add(usage);
    else nextHeld.delete(usage);
    const events = [
      ...macro.events,
      ...(delay ? [{ kind: "delay" as const, ms: delay }] : []),
      { kind, key: usage },
    ];
    // Reserve an up event for every held key before accepting this event.
    // An over-limit press must not be added to held before stop() runs.
    if (events.length + nextHeld.size > 2048) {
      stop();
      return;
    }
    last.current = now;
    held.current = nextHeld;
    update({ ...macro, events });
  };
  const play = () => {
    if (!macro) return;
    const timeline = macroTimeline(macro.events);
    cancelPreview();
    const generation = previewGeneration.current;
    let index = 0,
      elapsed = 0;
    setPlaying(0);
    const next = () => {
      if (generation !== previewGeneration.current) return;
      while (
        index < timeline.steps.length &&
        timeline.steps[index].atMs === elapsed
      ) {
        const step = timeline.steps[index++];
        setPlaying(step.indices[step.indices.length - 1]);
        setPreviewHeld(step.held);
        setPreviewElapsed(elapsed);
      }
      if (index === timeline.steps.length && elapsed === timeline.durationMs) {
        timer.current = setTimeout(() => {
          if (generation === previewGeneration.current) cancelPreview();
        }, 0);
        return;
      }
      const target =
        index < timeline.steps.length
          ? timeline.steps[index].atMs
          : timeline.durationMs;
      timer.current = setTimeout(() => {
        elapsed = target;
        next();
      }, target - elapsed);
    };
    next();
  };
  const add = () => {
    const m = {
      id: crypto.randomUUID(),
      name: `Macro ${macros.length + 1}`,
      events: [],
    };
    onChange([...macros, m]);
    setSelected(m.id);
  };
  const duplicate = () => {
    if (!macro) return;
    const copy = {
      ...macro,
      id: crypto.randomUUID(),
      name: copyName(macro.name, " (copy)"),
      events: macro.events.map((e) => ({ ...e })),
    };
    onChange([...macros, copy]);
    setSelected(copy.id);
  };
  const remove = () => {
    if (macro) onChange(macros.filter((m) => m.id !== macro.id));
  };
  const exportMacro = () => {
    if (macro)
      onExport(
        macro.name,
        JSON.stringify({ schema_version: 1, macro }, null, 2),
      );
  };
  const locked = recording || playing >= 0;
  const sidebarActions = [
    {
      icon: "export",
      label: "Export JSON",
      run: exportMacro,
      disabled: !macro || !!validateMacro(macro.events),
    },
    {
      icon: "import",
      label: "Import macro JSON",
      run: onImport,
      disabled: false,
    },
    {
      icon: "rename",
      label: "Rename macro",
      run: () => {
        nameInput.current?.focus();
        nameInput.current?.select();
      },
      disabled: !macro,
    },
    {
      icon: "copy",
      label: "Duplicate macro",
      run: duplicate,
      disabled: !macro || macros.length >= 64 || !!validateMacro(macro.events),
    },
    { icon: "del", label: "Delete", run: remove, disabled: !macro },
    { icon: "new", label: "+ New", run: add, disabled: macros.length >= 64 },
  ];
  const listPanel = (
    <section className="panel macro-list-panel">
      <div className="panel-heading">
        <h2>Your macros</h2>
        {listRoot ? (
          <div className="original-macro-toolbar">
            {sidebarActions.map((a) => (
              <button
                key={a.icon}
                disabled={locked || a.disabled}
                title={a.label}
                aria-label={a.label}
                onClick={a.run}
              >
                <MacroToolIcon
                  kind={a.icon}
                  image={uiAssets[`macro_${a.icon}_nr`]}
                />
                <span className="sr-only">{a.label}</span>
              </button>
            ))}
          </div>
        ) : (
          <button
            className="small"
            onClick={add}
            disabled={macros.length >= 64 || locked}
          >
            + New
          </button>
        )}
      </div>
      {macros.length === 0 ? (
        <div className="empty">
          <span>⌘</span>
          <p>Build your first sequence.</p>
          <button onClick={add}>Create macro</button>
        </div>
      ) : (
        <div className="macro-list">
          {macros.map((m) => (
            <button
              key={m.id}
              className={selected === m.id ? "active" : ""}
              disabled={recording || playing >= 0}
              onClick={() => setSelected(m.id)}
            >
              <span>{m.name}</span>
              <small>{m.events.length} events</small>
            </button>
          ))}
        </div>
      )}
      {!listRoot && (
        <button className="text-button" disabled={locked} onClick={onImport}>
          Import macro JSON
        </button>
      )}
    </section>
  );
  return (
    <>
      <div className="notice">
        <span className="notice-icon">i</span>
        <div>
          <strong>Local macros, with safe playback</strong>
          <p>
            Record and edit sequences here. Playback highlights events inside
            this editor; it never types into other applications. Onboard upload
            is disabled while its protocol is unresolved.
          </p>
        </div>
      </div>
      <div className={`macro-grid ${listRoot ? "portaled" : ""}`}>
        {listRoot ? createPortal(listPanel, listRoot) : listPanel}
        <section className="panel macro-editor">
          {macro ? (
            <>
              <div className="panel-heading">
                <input
                  aria-label="Macro name"
                  ref={nameInput}
                  value={macro.name}
                  maxLength={80}
                  disabled={recording || playing >= 0}
                  onChange={(e) => update({ ...macro, name: e.target.value })}
                />
                {!listRoot && (
                  <>
                    <button
                      disabled={
                        locked ||
                        macros.length >= 64 ||
                        !!validateMacro(macro.events)
                      }
                      onClick={duplicate}
                    >
                      Duplicate macro
                    </button>
                    <button
                      className="danger small"
                      disabled={locked}
                      onClick={remove}
                    >
                      Delete
                    </button>
                  </>
                )}
              </div>
              <div
                className={`record-zone ${recording ? "recording" : ""}`}
                ref={zone}
                role="group"
                aria-label="Macro recording area"
                tabIndex={0}
                onKeyDown={(e) => record(e, "down")}
                onKeyUp={(e) => record(e, "up")}
                onBlur={(e) => {
                  if (recording && !e.currentTarget.contains(e.relatedTarget))
                    stop();
                }}
              >
                <span
                  className={recording ? "record-dot pulse" : "record-dot"}
                />
                <strong>
                  {recording
                    ? "Recording in this area…"
                    : "Record a keyboard sequence"}
                </strong>
                <p>
                  {recording
                    ? "Press Escape or click outside to finish. Held keys are released automatically."
                    : "Only keys typed in this focused area are recorded."}
                </p>
                <button
                  disabled={
                    playing >= 0 ||
                    (!recording &&
                      (!!validateMacro(macro.events) ||
                        macro.events.length >= 2048))
                  }
                  onClick={() => {
                    if (recording) {
                      stop();
                      return;
                    }
                    last.current = 0;
                    held.current.clear();
                    setRecording(true);
                    zone.current?.focus();
                  }}
                >
                  {recording ? "Stop recording" : "Start recording"}
                </button>
              </div>
              <div className="toolbar macro-edit-tools">
                <select
                  aria-label="Insert position"
                  disabled={recording || playing >= 0}
                  value={insertAt}
                  onChange={(e) => setInsertAt(e.target.value)}
                >
                  <option value="append">At end</option>
                  <option value="before" disabled={cursor === null}>
                    Before selected event
                  </option>
                  <option value="after" disabled={cursor === null}>
                    After selected event
                  </option>
                </select>
                <select
                  aria-label="Key to insert"
                  disabled={recording || playing >= 0}
                  value={key}
                  onChange={(e) => setKey(+e.target.value)}
                >
                  {layout
                    .filter((k) => k.usage !== undefined)
                    .map((k) => (
                      <option key={k.id} value={k.usage}>
                        {k.label} ({k.code})
                      </option>
                    ))}
                </select>
                <button
                  disabled={
                    recording || playing >= 0 || macro.events.length > 2046
                  }
                  onClick={() =>
                    insertEvents([
                      { kind: "down", key },
                      { kind: "up", key },
                    ])
                  }
                >
                  Add key press
                </button>
                <button
                  disabled={
                    recording || playing >= 0 || macro.events.length >= 2048
                  }
                  onClick={() => insertEvents([{ kind: "delay", ms: 100 }])}
                >
                  + Delay
                </button>
                <button
                  disabled={recording || playing >= 0 || !macro.events.length}
                  onClick={() => {
                    if (
                      window.confirm("Clear all events in this local macro?")
                    ) {
                      update({ ...macro, events: [] });
                      setCursor(null);
                    }
                  }}
                >
                  Clear events
                </button>
              </div>
              <p className="macro-summary">
                {macro.events.length} events ·{" "}
                {macro.events.reduce(
                  (sum, e) => sum + (e.kind === "delay" ? (e.ms ?? 0) : 0),
                  0,
                )}{" "}
                ms of local delays
              </p>
              {playing >= 0 && (
                <p className="macro-preview-status" role="status">
                  Offline preview · {previewElapsed} ms · Held keys:{" "}
                  {previewHeld.map(usageLabel).join(" + ") || "none"}
                </p>
              )}
              <div className="events">
                {macro.events.length === 0 ? (
                  <p className="muted">
                    No events yet. Record a sequence or add a key press.
                  </p>
                ) : (
                  macro.events
                    .slice(eventPage * pageSize, (eventPage + 1) * pageSize)
                    .map((event, pageIndex) => {
                      const index = eventPage * pageSize + pageIndex;
                      return (
                        <div
                          className={`event ${playing === index ? "active" : ""} ${cursor === index ? "selected-event" : ""}`}
                          key={index}
                        >
                          <button
                            className="event-number"
                            aria-label={`Select event ${index + 1}`}
                            aria-pressed={cursor === index}
                            disabled={recording || playing >= 0}
                            onClick={() => setCursor(index)}
                          >
                            {String(index + 1).padStart(2, "0")}
                          </button>
                          <span className={`event-kind ${event.kind}`}>
                            {event.kind === "down"
                              ? "↓ Press"
                              : event.kind === "up"
                                ? "↑ Release"
                                : "◷ Delay"}
                          </span>
                          {event.kind === "delay" ? (
                            <label>
                              <input
                                aria-label={`Delay ${index + 1} in milliseconds`}
                                type="number"
                                disabled={recording || playing >= 0}
                                min={0}
                                max={60000}
                                value={event.ms}
                                onChange={(e) =>
                                  update({
                                    ...macro,
                                    events: macro.events.map((v, i) =>
                                      i === index
                                        ? {
                                            kind: "delay",
                                            ms: Math.max(
                                              0,
                                              Math.min(60000, +e.target.value),
                                            ),
                                          }
                                        : v,
                                    ),
                                  })
                                }
                              />{" "}
                              ms
                            </label>
                          ) : cursor === index ? (
                            <select
                              aria-label={`Key for event ${index + 1}`}
                              disabled={recording || playing >= 0}
                              value={event.key}
                              onChange={(e) =>
                                update({
                                  ...macro,
                                  events: macro.events.map((v, i) =>
                                    i === index
                                      ? {
                                          kind: event.kind,
                                          key: +e.target.value,
                                        }
                                      : v,
                                  ),
                                })
                              }
                            >
                              {!layout.some((k) => k.usage === event.key) && (
                                <option value={event.key}>
                                  {usageLabel(event.key!)}
                                </option>
                              )}
                              {layout
                                .filter((k) => k.usage !== undefined)
                                .map((k) => (
                                  <option key={k.id} value={k.usage}>
                                    {k.label}
                                  </option>
                                ))}
                            </select>
                          ) : (
                            <button
                              className="event-key-label"
                              aria-label={`Edit key for event ${index + 1}`}
                              disabled={recording || playing >= 0}
                              onClick={() => setCursor(index)}
                            >
                              {usageLabel(event.key!)}
                            </button>
                          )}
                          <button
                            className="icon-button"
                            aria-label={`Move event ${index + 1} up`}
                            disabled={recording || playing >= 0 || index === 0}
                            onClick={() => moveEvent(index, -1)}
                          >
                            ↑
                          </button>
                          <button
                            className="icon-button"
                            aria-label={`Move event ${index + 1} down`}
                            disabled={
                              recording ||
                              playing >= 0 ||
                              index === macro.events.length - 1
                            }
                            onClick={() => moveEvent(index, 1)}
                          >
                            ↓
                          </button>
                          <button
                            className="icon-button"
                            aria-label={`Remove event ${index + 1}`}
                            disabled={recording || playing >= 0}
                            onClick={() =>
                              update({
                                ...macro,
                                events: macro.events.filter(
                                  (_, i) => i !== index,
                                ),
                              })
                            }
                          >
                            ×
                          </button>
                        </div>
                      );
                    })
                )}
              </div>
              {macro.events.length > pageSize && (
                <div className="toolbar macro-pagination">
                  <button
                    disabled={recording || playing >= 0 || eventPage === 0}
                    onClick={() => setEventPage((p) => p - 1)}
                  >
                    Previous events
                  </button>
                  <span>
                    Events {eventPage * pageSize + 1}–
                    {Math.min((eventPage + 1) * pageSize, macro.events.length)}{" "}
                    of {macro.events.length}
                  </span>
                  <button
                    disabled={
                      recording ||
                      playing >= 0 ||
                      (eventPage + 1) * pageSize >= macro.events.length
                    }
                    onClick={() => setEventPage((p) => p + 1)}
                  >
                    Next events
                  </button>
                </div>
              )}
              {validateMacro(macro.events) && (
                <p className="error-inline">{validateMacro(macro.events)}</p>
              )}
              <div className="toolbar end">
                {onWirePreview && (
                  <button
                    disabled={
                      recording ||
                      playing >= 0 ||
                      !!validateMacro(macro.events) ||
                      !macro.events.length ||
                      !macro.name.trim()
                    }
                    onClick={() => onWirePreview(macro)}
                  >
                    Inspect F75 wire format (offline)
                  </button>
                )}
                {!listRoot && (
                  <button
                    disabled={
                      locked ||
                      !!validateMacro(macro.events) ||
                      !macro.name.trim()
                    }
                    onClick={exportMacro}
                  >
                    Export JSON
                  </button>
                )}
                <button
                  className="primary"
                  disabled={
                    recording ||
                    !!validateMacro(macro.events) ||
                    macro.events.length === 0
                  }
                  onClick={() => {
                    if (playing >= 0) {
                      cancelPreview();
                    } else play();
                  }}
                >
                  {playing >= 0 ? "Stop preview" : "▶ Preview sequence"}
                </button>
              </div>
            </>
          ) : (
            <div className="empty large">
              <span>⌘</span>
              <h2>A little less repetition.</h2>
              <p>Create a macro to start editing.</p>
            </div>
          )}
        </section>
      </div>
    </>
  );
}

function MacroToolIcon({ kind, image }: { kind: string; image?: string }) {
  const paths: Record<string, string> = {
    new: "M3 2h9l3 3v11H3zM9 6v7M5.5 9.5h7",
    del: "M3 5h12M6 5V3h6v2M5 5l1 11h6l1-11M8 8v5m2-5v5",
    copy: "M6 5h9v11H6zM3 13V2h9",
    rename: "m3 12 8-9 3 3-8 9-4 1zM10 4l3 3",
    import: "M3 11v5h12v-5M9 2v10M5 8l4 4 4-4",
    export: "M3 11v5h12v-5M9 12V2M5 6l4-4 4 4",
  };
  return image ? (
    <img src={image} alt="" />
  ) : (
    <svg viewBox="0 0 18 18" aria-hidden="true">
      <path d={paths[kind]} />
    </svg>
  );
}
