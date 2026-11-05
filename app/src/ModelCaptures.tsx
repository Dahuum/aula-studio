import { useEffect, useId, useRef, useState } from "react";
import { call, native } from "./api";
import type { ModelDefinition, ModelDevice, ModelDraft } from "./modelLibrary";

export type ModelCapture = {
  schema_version: 1;
  id: string;
  model_id: string;
  identity: number[];
  source: "physical_read" | "simulated";
  config: number[];
  palette: number[];
  base: number[];
  per_key: number[];
};
type Props = {
  model: ModelDefinition;
  devices: ModelDevice[];
  demoMode?: boolean;
  onDraft: (draft: ModelDraft) => void;
  onExport: (title: string, text: string) => void;
};
type Operation = { modelId: string; revision: number; ignored: boolean };
type ReadCandidate = ModelDevice & { read_candidate?: boolean };
const legacy = new Set(["f87", "f87-pro-010b", "f87-pro-wired"]);
const supported = new Set([...legacy, "f108", "h108"]);
const sourceName = (source: ModelCapture["source"]) =>
  source === "physical_read"
    ? "Physical read · experimental"
    : "Simulated · synthetic packet fixtures";
const hexIdentity = (identity: number[]) =>
  identity
    .map((n) => n.toString(16).padStart(2, "0"))
    .join(" ")
    .toUpperCase();
const byteArray = (value: unknown, length: number): value is number[] =>
  Array.isArray(value) &&
  value.length === length &&
  value.every((n) => Number.isInteger(n) && n >= 0 && n <= 255);

// Native code validates protocol data. These shape checks prevent incomplete or
// mis-scoped IPC results from being presented as a saved capture in the UI.
function captureResult(value: ModelCapture, modelId: string): ModelCapture {
  if (
    !value ||
    value.schema_version !== 1 ||
    typeof value.id !== "string" ||
    !value.id ||
    value.model_id !== modelId ||
    !["physical_read", "simulated"].includes(value.source) ||
    !byteArray(value.identity, 6) ||
    !byteArray(value.config, 128) ||
    !byteArray(value.palette, 512) ||
    !byteArray(value.base, 512) ||
    !byteArray(value.per_key, legacy.has(modelId) ? 384 : 378)
  )
    throw new Error(
      "The native backend returned an incomplete or mismatched capture.",
    );
  return value;
}

export function ModelCaptures({
  model,
  devices,
  demoMode = false,
  onDraft,
  onExport,
}: Props) {
  const headingId = useId();
  const isSupported = supported.has(model.id);
  const [stored, setStored] = useState<{ scope: string; rows: ModelCapture[] }>(
    {
      scope: model.id,
      rows: [],
    },
  );
  const [selectedId, setSelectedId] = useState("");
  const [selectedPath, setSelectedPath] = useState("");
  const [draftName, setDraftName] = useState(`${model.name} captured lighting`);
  const [busy, setBusy] = useState("");
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  const [reload, setReload] = useState(0);
  const mounted = useRef(true);
  const operation = useRef<Operation | null>(null);
  const scope = useRef({ modelId: model.id, revision: 0 });
  // Invalidate immediately on a changed selection, including A -> B -> A while
  // an old promise is pending. Keep the operation lock until that promise ends.
  if (scope.current.modelId !== model.id) {
    scope.current = {
      modelId: model.id,
      revision: scope.current.revision + 1,
    };
  }
  const current = (job: Operation) =>
    mounted.current &&
    !job.ignored &&
    scope.current.modelId === job.modelId &&
    scope.current.revision === job.revision;
  const perform = async (
    label: string,
    action: (job: Operation) => Promise<void>,
  ) => {
    if (!native || !isSupported || operation.current) return;
    const job: Operation = { ...scope.current, ignored: false };
    operation.current = job;
    setBusy(label);
    setError("");
    setMessage("");
    try {
      await action(job);
    } catch (cause) {
      if (current(job))
        setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      if (operation.current === job) operation.current = null;
      if (mounted.current) {
        setBusy("");
        if (
          scope.current.modelId !== job.modelId ||
          scope.current.revision !== job.revision
        )
          setReload((n) => n + 1);
      }
    }
  };
  const load = () =>
    perform("Loading stored captures…", async (job) => {
      const rows = await call<ModelCapture[]>("list_model_captures", {
        modelId: job.modelId,
      });
      if (!current(job)) return;
      if (!Array.isArray(rows)) throw new Error("Invalid stored capture list.");
      const valid = rows.map((row) => captureResult(row, job.modelId));
      if (new Set(valid.map((row) => row.id)).size !== valid.length)
        throw new Error(
          "The stored capture list contains duplicate identifiers.",
        );
      setStored({ scope: job.modelId, rows: valid });
      setSelectedId((id) =>
        valid.some((row) => row.id === id) ? id : (valid[0]?.id ?? ""),
      );
    });
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      scope.current.revision += 1;
    };
  }, []);
  useEffect(() => {
    setSelectedId("");
    setSelectedPath("");
    setDraftName(`${model.name} captured lighting`);
    setError("");
    setMessage("");
  }, [model.id]);
  useEffect(() => {
    if (native && isSupported && !operation.current) void load();
  }, [model.id, reload, isSupported]);

  if (!isSupported) return null;
  const candidates = (devices as ReadCandidate[]).filter(
    (device) =>
      device.read_candidate === true &&
      device.path.trim().length > 0 &&
      device.vendor_id === 0x258a &&
      device.product_id === 0x010c &&
      device.interface_number === 1 &&
      device.candidates.includes(model.id),
  );
  const path = candidates.some((device) => device.path === selectedPath)
    ? selectedPath
    : (candidates[0]?.path ?? "");
  const captures = stored.scope === model.id ? stored.rows : [];
  const selected = captures.find((capture) => capture.id === selectedId);
  const nameValid =
    draftName.trim().length > 0 &&
    new TextEncoder().encode(draftName.trim()).length <= 120;
  const addCapture = (value: ModelCapture, job: Operation) => {
    const capture = captureResult(value, job.modelId);
    if (!current(job)) return;
    setStored((old) => ({
      scope: job.modelId,
      rows: [
        capture,
        ...(old.scope === job.modelId ? old.rows : []).filter(
          (row) => row.id !== capture.id,
        ),
      ],
    }));
    setSelectedId(capture.id);
    setMessage(
      capture.source === "physical_read"
        ? "Stored the experimental physical read locally. Review its raw banks before using the lighting draft."
        : "Stored synthetic packet fixtures locally. This capture contains simulated data.",
    );
  };
  const readDevice = () => {
    if (demoMode || !path) return;
    void perform("Reading selected device…", async (job) => {
      const capture = await call<ModelCapture>("read_model_capture", {
        modelId: job.modelId,
        path,
      });
      if (!current(job)) return;
      if (capture.source !== "physical_read")
        throw new Error(
          "The device read returned a simulated capture; it was not accepted as a physical read.",
        );
      addCapture(capture, job);
    });
  };
  const simulate = () =>
    perform("Generating synthetic packet fixtures…", async (job) => {
      const capture = await call<ModelCapture>("simulate_model_capture", {
        modelId: job.modelId,
      });
      if (!current(job)) return;
      if (capture.source !== "simulated")
        throw new Error(
          "The fixture generator returned an unexpected capture source.",
        );
      addCapture(capture, job);
    });
  const exportCapture = () => {
    if (!selected) return;
    void perform("Exporting capture…", async (job) => {
      const text = await call<string>("export_model_capture", {
        id: selected.id,
      });
      if (current(job))
        onExport(
          `${model.name} · ${sourceName(selected.source)} · raw capture`,
          text,
        );
    });
  };
  const makeDraft = () => {
    if (!selected || !nameValid) return;
    void perform("Creating a local lighting draft…", async (job) => {
      const draft = await call<ModelDraft>("model_capture_draft", {
        id: selected.id,
        name: draftName.trim(),
      });
      if (!current(job)) return;
      if (
        draft.model_id !== job.modelId ||
        !draft.assignments ||
        !draft.fn_assignments ||
        !Array.isArray(draft.macros) ||
        Object.keys(draft.assignments).length ||
        Object.keys(draft.fn_assignments).length ||
        draft.macros.length
      )
        throw new Error(
          "The backend returned a draft outside the lighting-only scope.",
        );
      onDraft(draft);
      setMessage(
        "Created a local lighting-only draft. Key assignments, Fn assignments and macros are excluded.",
      );
    });
  };
  return (
    <section
      className="models-panel"
      aria-labelledby={headingId}
      aria-busy={!!busy}
    >
      <h3 id={headingId}>Experimental device captures</h3>
      <p>
        Read bounded configuration banks through the native backend and retain
        their source locally. Captures can seed a local lighting-only draft;
        restoration remains unavailable.
      </p>
      {!native && (
        <p>
          Open the native desktop application to read devices or generate
          synthetic packet fixtures.
        </p>
      )}
      {demoMode && (
        <p>
          Exit F75 demo mode before reading a physical device. Synthetic
          fixtures remain available in the native application.
        </p>
      )}
      {native && !demoMode && candidates.length === 0 && (
        <p>
          No eligible wired read candidate for this model. Connect its USB
          interface and refresh device discovery.
        </p>
      )}
      <div className="models-controls">
        <label>
          Device for experimental read
          <select
            aria-label="Device for experimental read"
            value={path}
            disabled={!native || !!busy || demoMode || !candidates.length}
            onChange={(event) => setSelectedPath(event.target.value)}
          >
            {!candidates.length && <option value="">No eligible device</option>}
            {candidates.map((device) => (
              <option key={device.path} value={device.path}>
                {device.product_name || model.name} · {device.path}
              </option>
            ))}
          </select>
        </label>
      </div>
      <div className="models-actions">
        <button
          type="button"
          disabled={!native || !!busy || demoMode || !path}
          onClick={readDevice}
        >
          Read device (experimental)
        </button>
        <button
          type="button"
          disabled={!native || !!busy}
          onClick={() => void simulate()}
        >
          Generate synthetic packet fixtures
        </button>
        <button
          type="button"
          disabled={!native || !!busy}
          onClick={() => void load()}
        >
          Refresh stored captures
        </button>
        {busy && (
          <button
            type="button"
            onClick={() => {
              if (operation.current) operation.current.ignored = true;
              setMessage(
                "The pending result will be ignored here. The native operation continues until it finishes; a capture may still be stored locally.",
              );
            }}
          >
            Ignore pending result
          </button>
        )}
      </div>
      {busy && <p role="status">{busy}</p>}
      {error && (
        <p role="alert" className="models-alert">
          {error}
        </p>
      )}
      {message && (
        <p role="status" className="models-message">
          {message}
        </p>
      )}
      <label>
        Stored capture
        <select
          aria-label="Stored capture"
          value={selected?.id ?? ""}
          disabled={!native || !!busy || !captures.length}
          onChange={(event) => setSelectedId(event.target.value)}
        >
          {!captures.length && (
            <option value="">No stored capture selected</option>
          )}
          {captures.map((capture) => (
            <option key={capture.id} value={capture.id}>
              {sourceName(capture.source)} · {capture.id}
            </option>
          ))}
        </select>
      </label>
      {native && !busy && !error && !captures.length && (
        <p>No stored captures for this model.</p>
      )}
      {selected && (
        <>
          <dl>
            <dt>Capture source</dt>
            <dd>{sourceName(selected.source)}</dd>
            <dt>Device identity</dt>
            <dd>
              <code>{hexIdentity(selected.identity)}</code>
            </dd>
            <dt>Stored bank sizes</dt>
            <dd>
              Config: {selected.config.length} bytes · Palette:{" "}
              {selected.palette.length} bytes · Base: {selected.base.length}{" "}
              bytes · Per-key: {selected.per_key.length} bytes
            </dd>
          </dl>
          {selected.source === "simulated" && (
            <p>
              Synthetic fixtures exercise the packet and storage workflow. They
              provide no physical-device validation.
            </p>
          )}
          <label>
            Capture lighting draft name
            <input
              aria-label="Capture lighting draft name"
              value={draftName}
              disabled={!!busy}
              maxLength={120}
              onChange={(event) => setDraftName(event.target.value)}
            />
          </label>
          <div className="models-actions">
            <button
              type="button"
              disabled={!native || !!busy}
              onClick={exportCapture}
            >
              Export raw capture JSON
            </button>
            <button
              type="button"
              disabled={!native || !!busy || !nameValid}
              onClick={makeDraft}
            >
              Create local lighting-only draft
            </button>
          </div>
        </>
      )}
    </section>
  );
}
