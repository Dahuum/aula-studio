import { afterEach, expect, it, vi } from "vitest";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { ModelWorkbench } from "./ModelWorkbench";
import {
  modelRegistry,
  newModelDraft,
  parseModelDraft,
  modelCode,
} from "./modelLibrary";
import { call } from "./api";
afterEach(() => {
  cleanup();
  localStorage.clear();
  vi.restoreAllMocks();
});
const open = async () => {
  render(<ModelWorkbench onBack={() => {}} />);
  await screen.findByRole("textbox", { name: "Model profile name" });
  await waitFor(() =>
    expect(
      (
        screen.getByRole("button", {
          name: "Save local profile",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(false),
  );
};
const modelButton = (id: string) =>
  screen.getByRole("button", {
    name: modelRegistry.find((m) => m.id === id)!.name,
  });
it("keeps the editor when the selected model is clicked again", async () => {
  await open();
  fireEvent.click(modelButton("f87"));
  expect(document.querySelectorAll(".models-keyboard button")).toHaveLength(87);
  expect(screen.getByLabelText("Model profile name")).toBeTruthy();
});
it("opens recovered layouts and leaves unrecovered devices without invented controls", async () => {
  await open();
  expect(document.querySelectorAll(".models-keyboard button")).toHaveLength(87);
  fireEvent.click(modelButton("f108"));
  await waitFor(() =>
    expect(document.querySelectorAll(".models-keyboard button")).toHaveLength(
      108,
    ),
  );
  await waitFor(() =>
    expect(
      (
        screen.getByRole("button", {
          name: "Save local profile",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(false),
  );
  fireEvent.click(modelButton("f2088"));
  await screen.findByText("Research entry");
  expect(screen.queryByLabelText("Model brightness")).toBeNull();
  expect(
    screen.queryByRole("button", { name: /apply|upload|write/i }),
  ).toBeNull();
});
it("saves colors and key assignments through separate model storage and exports the draft", async () => {
  await open();
  const model = modelRegistry.find((m) => m.id === "f87")!,
    key = model.keys.find((k) => k.usage === 4)!;
  fireEvent.click(
    screen.getByRole("button", { name: `${key.label} · ${key.id}` }),
  );
  fireEvent.change(screen.getByLabelText("Model draft color"), {
    target: { value: "#123456" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Color selected keys locally" }),
  );
  fireEvent.click(screen.getByRole("button", { name: "Assignments" }));
  fireEvent.change(screen.getByLabelText("Model target action"), {
    target: { value: "5" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Assign locally" }));
  fireEvent.click(screen.getByRole("button", { name: "Save local profile" }));
  await screen.findByText(/Saved to this model/);
  const saved = (await call<any[]>("list_model_profiles"))[0];
  expect(saved.per_key[key.id]).toEqual([18, 52, 86]);
  expect(saved.assignments[key.id]).toEqual({ kind: "keyboard", usage: 5 });
  expect(await call("list_profiles")).toEqual([]);
  fireEvent.click(screen.getByRole("button", { name: "Export draft" }));
  const dialog = await screen.findByRole("dialog");
  expect(
    JSON.parse(
      (within(dialog).getByLabelText("Model JSON") as HTMLTextAreaElement)
        .value,
    ),
  ).toEqual(saved);
  fireEvent.keyDown(document, { key: "Escape" });
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
});
it("imports a selected profile across models without replacing it with another stored profile", async () => {
  const first = newModelDraft("s75-pro", "First profile"),
    wanted = newModelDraft("s75-pro", "Imported chosen profile");
  await call("save_model_profile", { profile: first });
  await open();
  fireEvent.click(screen.getByRole("button", { name: "Profiles" }));
  fireEvent.click(screen.getByRole("button", { name: "Import model profile" }));
  fireEvent.change(screen.getByLabelText("Model JSON"), {
    target: { value: JSON.stringify(wanted) },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Import as new profile" }),
  );
  await waitFor(() =>
    expect(
      (screen.getByLabelText("Model profile name") as HTMLInputElement).value,
    ).toBe(wanted.name),
  );
  expect(document.querySelectorAll(".models-keyboard button")).toHaveLength(80);
  expect(
    (await call<any[]>("list_model_profiles")).filter(
      (p) => p.model_id === "s75-pro",
    ),
  ).toHaveLength(2);
});
it("rejects malformed imports without altering storage and blocks protected source keys", async () => {
  const p = newModelDraft("s75-pro", "Safe"),
    protectedKey = modelRegistry
      .find((m) => m.id === "s75-pro")!
      .keys.find((k) => k.protected)!;
  p.assignments[protectedKey.id] = { kind: "keyboard", usage: 4 };
  expect(() => parseModelDraft(JSON.stringify(p))).toThrow(/protected/);
  await expect(
    call("import_model_profile", { json: JSON.stringify(p) }),
  ).rejects.toThrow();
  expect(await call("list_model_profiles")).toEqual([]);
  delete p.assignments[protectedKey.id];
  p.lighting!.brightness = 6;
  expect(() => parseModelDraft(JSON.stringify(p))).toThrow(/bounds/);
  expect(() =>
    parseModelDraft(
      JSON.stringify({ ...newModelDraft("f87", "Local"), unexpected: true }),
    ),
  ).toThrow(/schema/);
});
it("records numpad keys using the chosen model and roundtrips the exported macro wrapper", async () => {
  await open();
  fireEvent.click(modelButton("f108"));
  await waitFor(() =>
    expect(document.querySelectorAll(".models-keyboard button")).toHaveLength(
      108,
    ),
  );
  fireEvent.click(screen.getByRole("button", { name: "Local macros" }));
  fireEvent.click(screen.getByRole("button", { name: "Create macro" }));
  fireEvent.click(screen.getByRole("button", { name: "Start recording" }));
  const area = screen.getByRole("group", { name: "Macro recording area" });
  fireEvent.keyDown(area, { code: "Numpad1" });
  fireEvent.keyUp(area, { code: "Numpad1" });
  fireEvent.click(screen.getByRole("button", { name: "Stop recording" }));
  fireEvent.click(screen.getByRole("button", { name: "Export JSON" }));
  let dialog = await screen.findByRole("dialog");
  const text = (
    within(dialog).getByLabelText("Model JSON") as HTMLTextAreaElement
  ).value;
  expect(
    JSON.parse(text).macro.events.filter((e: any) => e.kind !== "delay"),
  ).toEqual([
    { kind: "down", key: 89 },
    { kind: "up", key: 89 },
  ]);
  fireEvent.click(within(dialog).getByRole("button", { name: "Close" }));
  fireEvent.click(screen.getByRole("button", { name: "Import macro JSON" }));
  dialog = await screen.findByRole("dialog");
  fireEvent.change(within(dialog).getByLabelText("Model JSON"), {
    target: { value: text },
  });
  fireEvent.click(
    within(dialog).getByRole("button", { name: "Add local macro" }),
  );
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  fireEvent.click(screen.getByRole("button", { name: "Save local profile" }));
  await waitFor(async () =>
    expect((await call<any[]>("list_model_profiles"))[0].macros).toHaveLength(
      2,
    ),
  );
  expect(modelCode(88)).toBe("NumpadEnter");
});
it("preserves shared identifiers as ambiguous rather than guessing a connected model", () => {
  const shared = modelRegistry.filter((m) =>
    m.usb.some((u) => u.vendor_id === 0x0c45 && u.product_id === 0x800a),
  );
  expect(shared.length).toBeGreaterThan(2);
  const s99 = modelRegistry.find((m) => m.id === "s99-pro")!;
  expect(s99.keys.some((k) => k.matrix_index === null)).toBe(true);
  const wrong = { ...newModelDraft("f87", "Local"), model_id: "sc620" };
  expect(() => parseModelDraft(JSON.stringify(wrong))).toThrow(/layout/);
});
