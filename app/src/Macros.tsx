import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { KeyboardEvent } from "react";
import { CODE_USAGE, KEYBOARD, usageLabel } from "./layout";
import { validateMacro } from "./model";
import type { Macro } from "./model";
export function Macros({
  macros,
  onChange,
  onExport,
  onImport,
}: {
  macros: Macro[];
  onChange: (value: Macro[]) => void;
  onExport: (name: string, text: string) => void;
  onImport: () => void;
}) {
  const [selected, setSelected] = useState(macros[0]?.id ?? "");
  const [recording, setRecording] = useState(false);
  const [listRoot,setListRoot]=useState<HTMLElement|null>(null);
  useEffect(()=>setListRoot(document.getElementById("original-macro-list")),[]);
  const [playing, setPlaying] = useState(-1);
  const [key, setKey] = useState(4);
  const held = useRef(new Set<number>());
  const last = useRef(0);
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const zone = useRef<HTMLDivElement>(null);
  const macro = macros.find((m) => m.id === selected);
  const update = (next: Macro) =>
    onChange(macros.map((m) => (m.id === next.id ? next : m)));
  useEffect(
    () => () => {
      if (timer.current) clearTimeout(timer.current);
    },
    [],
  );
  useEffect(() => {
    if (!macros.some((m) => m.id === selected))
      setSelected(macros[0]?.id ?? "");
  }, [macros, selected]);
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
    const usage = CODE_USAGE[e.code];
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
    let i = 0;
    setPlaying(0);
    const next = () => {
      if (i >= macro.events.length) {
        setPlaying(-1);
        return;
      }
      const event = macro.events[i];
      setPlaying(i++);
      timer.current = setTimeout(
        next,
        event.kind === "delay" ? (event.ms ?? 0) : 180,
      );
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
  const listPanel=(
<section className="panel macro-list-panel">
          <div className="panel-heading">
            <h2>Your macros</h2>
            <button
              className="small"
              onClick={add}
              disabled={macros.length >= 64 || recording || playing >= 0}
            >
              + New
            </button>
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
          <button className="text-button" disabled={recording || playing >= 0} onClick={onImport}>
            Import macro JSON
          </button>
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
      <div className={`macro-grid ${listRoot?"portaled":""}`}>
        {listRoot?createPortal(listPanel,listRoot):listPanel}
        <section className="panel macro-editor">
          {macro ? (
            <>
              <div className="panel-heading">
                <input
                  aria-label="Macro name"
                  value={macro.name}
                  maxLength={80}
                  disabled={recording}
                  onChange={(e) => update({ ...macro, name: e.target.value })}
                />
                <button
                  className="danger small"
                  disabled={recording || playing >= 0}
                  onClick={() =>
                    onChange(macros.filter((m) => m.id !== macro.id))
                  }
                >
                  Delete
                </button>
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
                  if (recording && !e.currentTarget.contains(e.relatedTarget)) stop();
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
                  disabled={playing >= 0 || (!recording && (!!validateMacro(macro.events) || macro.events.length >= 2048))}
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
              <div className="toolbar">
                <select
                  aria-label="Key to insert"
                  disabled={recording || playing >= 0}
                  value={key}
                  onChange={(e) => setKey(+e.target.value)}
                >
                  {KEYBOARD.filter((k) => k.usage !== undefined).map((k) => (
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
                    update({
                      ...macro,
                      events: [
                        ...macro.events,
                        { kind: "down", key },
                        { kind: "up", key },
                      ],
                    })
                  }
                >
                  Add key press
                </button>
                <button
                  disabled={
                    recording || playing >= 0 || macro.events.length >= 2048
                  }
                  onClick={() =>
                    update({
                      ...macro,
                      events: [...macro.events, { kind: "delay", ms: 100 }],
                    })
                  }
                >
                  + Delay
                </button>
              </div>
              <div className="events">
                {macro.events.length === 0 ? (
                  <p className="muted">
                    No events yet. Record a sequence or add a key press.
                  </p>
                ) : (
                  macro.events.map((event, index) => (
                    <div
                      className={`event ${playing === index ? "active" : ""}`}
                      key={index}
                    >
                      <span className="event-number">
                        {String(index + 1).padStart(2, "0")}
                      </span>
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
                      ) : (
                        <span>{usageLabel(event.key!)}</span>
                      )}
                      <button
                        className="icon-button"
                        aria-label={`Remove event ${index + 1}`}
                        disabled={recording || playing >= 0}
                        onClick={() =>
                          update({
                            ...macro,
                            events: macro.events.filter((_, i) => i !== index),
                          })
                        }
                      >
                        ×
                      </button>
                    </div>
                  ))
                )}
              </div>
              {validateMacro(macro.events) && (
                <p className="error-inline">{validateMacro(macro.events)}</p>
              )}
              <div className="toolbar end">
                <button
                  disabled={recording || playing >= 0 || !!validateMacro(macro.events) || !macro.name.trim()}
                  onClick={() =>
                    onExport(
                      macro.name,
                      JSON.stringify({ schema_version: 1, macro }, null, 2),
                    )
                  }
                >
                  Export JSON
                </button>
                <button
                  className="primary"
                  disabled={
                    recording ||
                    !!validateMacro(macro.events) ||
                    macro.events.length === 0
                  }
                  onClick={() => {
                    if (playing >= 0) {
                      if (timer.current) clearTimeout(timer.current);
                      setPlaying(-1);
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
