import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { ModelCaptures } from "./ModelCaptures";
import type { ModelCapture } from "./ModelCaptures";
import { modelRegistry, newModelDraft } from "./modelLibrary";
import type { ModelDefinition, ModelDevice } from "./modelLibrary";

const mocked = vi.hoisted(() => ({ native: true, call: vi.fn() }));
vi.mock("./api", () => ({
  get native() {
    return mocked.native;
  },
  call: mocked.call,
}));
const model = (id = "f87") => modelRegistry.find((entry) => entry.id === id)!;
const device = (
  patch: Partial<ModelDevice> & { read_candidate?: boolean } = {},
): ModelDevice & { read_candidate?: boolean } => ({
  path: "/dev/hidraw-f87",
  vendor_id: 0x258a,
  product_id: 0x010c,
  interface_number: 1,
  product_name: "F87",
  candidates: ["f87", "f87-pro-010b", "f87-pro-wired"],
  hardware_writes_enabled: false,
  note: "Experimental bounded reader",
  read_candidate: true,
  ...patch,
});
const capture = (
  source: ModelCapture["source"] = "simulated",
  modelId = "f87",
): ModelCapture => ({
  schema_version: 1,
  id: `${modelId}-${source}`,
  model_id: modelId,
  identity: [3, 0, 0, 0, 0, 0x8f],
  source,
  config: Array(128).fill(0),
  palette: Array(512).fill(0),
  base: Array(512).fill(0),
  per_key: Array(["f108", "h108"].includes(modelId) ? 378 : 384).fill(0),
});
const onDraft = vi.fn(),
  onExport = vi.fn();
const props = (selected: ModelDefinition = model()) => ({
  model: selected,
  devices: [device()],
  onDraft,
  onExport,
});
const idle = async () =>
  waitFor(() =>
    expect(
      (
        screen.getByRole("button", {
          name: "Refresh stored captures",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(false),
  );
const button = (name: string) =>
  screen.getByRole("button", { name }) as HTMLButtonElement;
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}
beforeEach(() => {
  mocked.native = true;
  mocked.call.mockReset();
  mocked.call.mockImplementation(async (command: string) => {
    if (command === "list_model_captures") return [];
    throw new Error(`Unexpected command: ${command}`);
  });
  onDraft.mockReset();
  onExport.mockReset();
});
afterEach(cleanup);

it("renders nothing and calls no backend for unsupported models", () => {
  const view = render(<ModelCaptures {...props(model("s75-pro"))} />);
  expect(view.container.childElementCount).toBe(0);
  expect(mocked.call).not.toHaveBeenCalled();
});

it("requires the native application for reads, fixtures and stored captures", () => {
  mocked.native = false;
  render(<ModelCaptures {...props()} />);
  expect(screen.getByText(/Open the native desktop application/)).toBeTruthy();
  expect(button("Read device (experimental)").disabled).toBe(true);
  expect(button("Generate synthetic packet fixtures").disabled).toBe(true);
  expect(mocked.call).not.toHaveBeenCalled();
});

it("requires every eligibility gate and keeps physical reads disabled in demo mode", async () => {
  const view = render(
    <ModelCaptures
      {...props()}
      devices={[
        device({ path: "/dev/unapproved", read_candidate: undefined }),
        device({ path: "/dev/wrong-interface", interface_number: 0 }),
        device({ path: "/dev/wrong-model", candidates: ["f87-pro-wired"] }),
        device({ path: "/dev/wrong-vendor", vendor_id: 0 }),
        device({ path: "/dev/wrong-product", product_id: 0 }),
        device({ path: "" }),
      ]}
    />,
  );
  await idle();
  expect(button("Read device (experimental)").disabled).toBe(true);
  expect(screen.getByText(/No eligible wired read candidate/)).toBeTruthy();
  view.rerender(<ModelCaptures {...props()} demoMode />);
  expect(button("Read device (experimental)").disabled).toBe(true);
  expect(button("Generate synthetic packet fixtures").disabled).toBe(false);
  expect(screen.getByText(/Exit F75 demo mode/)).toBeTruthy();
  fireEvent.click(button("Read device (experimental)"));
  expect(
    mocked.call.mock.calls.some(([name]) => name === "read_model_capture"),
  ).toBe(false);
});

it("labels synthetic captures honestly and delegates export and lighting drafts to the backend", async () => {
  const synthetic = capture();
  const draft = newModelDraft("f87", "Captured lighting");
  mocked.call.mockImplementation(async (command: string) => {
    if (command === "list_model_captures") return [];
    if (command === "simulate_model_capture") return synthetic;
    if (command === "export_model_capture") return JSON.stringify(synthetic);
    if (command === "model_capture_draft") return draft;
    throw new Error(command);
  });
  render(<ModelCaptures {...props()} demoMode />);
  await idle();
  fireEvent.click(button("Generate synthetic packet fixtures"));
  await screen.findByText(/This capture contains simulated data/);
  expect(
    screen.getByText(/They provide no physical-device validation/),
  ).toBeTruthy();
  expect(
    screen.getByText(
      /Config: 128 bytes · Palette: 512 bytes · Base: 512 bytes · Per-key: 384 bytes/,
    ),
  ).toBeTruthy();
  expect(screen.getByText("03 00 00 00 00 8F")).toBeTruthy();
  fireEvent.click(button("Export raw capture JSON"));
  await waitFor(() =>
    expect(onExport).toHaveBeenCalledWith(
      expect.stringContaining("Simulated"),
      JSON.stringify(synthetic),
    ),
  );
  await idle();
  fireEvent.change(screen.getByLabelText("Capture lighting draft name"), {
    target: { value: "  Captured lighting  " },
  });
  fireEvent.click(button("Create local lighting-only draft"));
  await waitFor(() => expect(onDraft).toHaveBeenCalledWith(draft));
  expect(mocked.call).toHaveBeenCalledWith("model_capture_draft", {
    id: synthetic.id,
    name: "Captured lighting",
  });
  expect(
    screen.queryByRole("button", { name: /restore|write|upload/i }),
  ).toBeNull();
});

it("reads the selected eligible path and presents its physical source without a verification claim", async () => {
  mocked.call.mockImplementation(async (command: string) =>
    command === "read_model_capture" ? capture("physical_read") : [],
  );
  render(
    <ModelCaptures
      {...props()}
      devices={[device(), device({ path: "/dev/hidraw-other" })]}
    />,
  );
  await idle();
  fireEvent.change(screen.getByLabelText("Device for experimental read"), {
    target: { value: "/dev/hidraw-other" },
  });
  fireEvent.click(button("Read device (experimental)"));
  await screen.findByText(/Stored the experimental physical read locally/);
  expect(mocked.call).toHaveBeenCalledWith("read_model_capture", {
    modelId: "f87",
    path: "/dev/hidraw-other",
  });
  expect(
    screen.getByText("Physical read · experimental", { selector: "dd" }),
  ).toBeTruthy();
  expect(screen.queryByText(/VERIFIED ON HARDWARE/)).toBeNull();
});

it("shows read failures, clears its lock, and rejects a simulated response to a physical read", async () => {
  mocked.call.mockImplementation(async (command: string) => {
    if (command === "list_model_captures") return [];
    throw new Error("Device disconnected during bounded read");
  });
  render(<ModelCaptures {...props()} />);
  await idle();
  fireEvent.click(button("Read device (experimental)"));
  expect((await screen.findByRole("alert")).textContent).toContain(
    "Device disconnected",
  );
  expect(button("Read device (experimental)").disabled).toBe(false);
  mocked.call.mockResolvedValue(capture("simulated"));
  fireEvent.click(button("Read device (experimental)"));
  await waitFor(() =>
    expect(screen.getByRole("alert").textContent).toContain(
      "not accepted as a physical read",
    ),
  );
  expect(screen.queryByText("Capture source")).toBeNull();
});

it("ignores a pending result without releasing its lock or starting another native operation", async () => {
  const pending = deferred<ModelCapture>();
  mocked.call.mockImplementation((command: string) =>
    command === "read_model_capture" ? pending.promise : Promise.resolve([]),
  );
  render(<ModelCaptures {...props()} />);
  await idle();
  fireEvent.click(button("Read device (experimental)"));
  fireEvent.click(button("Read device (experimental)"));
  fireEvent.click(button("Ignore pending result"));
  expect(button("Read device (experimental)").disabled).toBe(true);
  expect(button("Generate synthetic packet fixtures").disabled).toBe(true);
  fireEvent.click(button("Generate synthetic packet fixtures"));
  expect(
    mocked.call.mock.calls.filter(([name]) => name === "read_model_capture"),
  ).toHaveLength(1);
  expect(
    mocked.call.mock.calls.some(([name]) => name === "simulate_model_capture"),
  ).toBe(false);
  await act(async () => pending.resolve(capture("physical_read")));
  await idle();
  expect(screen.queryByText("Capture source")).toBeNull();
  expect(
    screen.getByText(/native operation continues until it finishes/),
  ).toBeTruthy();
});

it("drops stale model results and loads the new model only after the native operation settles", async () => {
  const pending = deferred<ModelCapture>();
  mocked.call.mockImplementation((command: string) =>
    command === "read_model_capture" ? pending.promise : Promise.resolve([]),
  );
  const view = render(<ModelCaptures {...props()} />);
  await idle();
  fireEvent.click(button("Read device (experimental)"));
  view.rerender(<ModelCaptures {...props(model("f87-pro-010b"))} />);
  expect(button("Generate synthetic packet fixtures").disabled).toBe(true);
  expect(
    mocked.call.mock.calls.filter(([name]) => name === "list_model_captures"),
  ).toHaveLength(1);
  await act(async () => pending.resolve(capture("physical_read")));
  await idle();
  expect(mocked.call).toHaveBeenLastCalledWith("list_model_captures", {
    modelId: "f87-pro-010b",
  });
  expect(screen.queryByText("Capture source")).toBeNull();
  expect(onDraft).not.toHaveBeenCalled();
  expect(onExport).not.toHaveBeenCalled();
});

it("rejects incomplete stored banks and drafts containing assignments", async () => {
  mocked.call.mockResolvedValue([{ ...capture(), per_key: [0] }]);
  const view = render(<ModelCaptures {...props()} />);
  expect((await screen.findByRole("alert")).textContent).toContain(
    "incomplete or mismatched",
  );
  expect(screen.queryByText("Capture source")).toBeNull();
  const invalidDraft = newModelDraft("f87", "Wrong scope");
  invalidDraft.assignments[model().keys[0].id] = { kind: "keyboard", usage: 4 };
  mocked.call.mockImplementation(async (command: string) =>
    command === "model_capture_draft" ? invalidDraft : [capture()],
  );
  fireEvent.click(button("Refresh stored captures"));
  await screen.findByRole("button", {
    name: "Create local lighting-only draft",
  });
  await idle();
  fireEvent.click(button("Create local lighting-only draft"));
  await waitFor(() =>
    expect(screen.getByRole("alert").textContent).toContain(
      "lighting-only scope",
    ),
  );
  expect(onDraft).not.toHaveBeenCalled();
  view.unmount();
});

it.each(["f108", "h108"])(
  "accepts only the independently established 378-byte capture for %s",
  async (modelId) => {
    let c = capture("simulated", modelId);
    mocked.call.mockImplementation(async (command: string) => {
      if (command === "list_model_captures") return [];
      if (command === "simulate_model_capture") return c;
      throw new Error(command);
    });
    render(<ModelCaptures {...props(model(modelId))} demoMode />);
    await idle();
    fireEvent.click(button("Generate synthetic packet fixtures"));
    await screen.findByText(/Per-key: 378 bytes/);
    expect(button("Read device (experimental)").disabled).toBe(true);
    c = { ...c, per_key: Array(384).fill(0) };
    fireEvent.click(button("Generate synthetic packet fixtures"));
    await screen.findByText(/incomplete or mismatched capture/);
    expect(screen.getByText(/Per-key: 378 bytes/)).toBeTruthy();
  },
);
