import { useState } from "react";
import { afterEach, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { Macros } from "./Macros";
import { validateMacro } from "./model";
import type { Macro, MacroEvent } from "./model";

afterEach(() => { cleanup(); vi.restoreAllMocks(); vi.useRealTimers(); });

function editor(events: MacroEvent[] = []) {
  function Harness() {
    const [macros, setMacros] = useState<Macro[]>([{ id: "test", name: "Test", events }]);
    return <><Macros macros={macros} onChange={setMacros} onImport={() => {}} onExport={() => {}} />
      <output data-testid="events">{JSON.stringify(macros[0].events)}</output></>;
  }
  render(<Harness />);
  return () => JSON.parse(screen.getByTestId("events").textContent!) as MacroEvent[];
}

it("releases recorded held keys on focus loss and ignores repeated downs", () => {
  const events = editor();
  fireEvent.click(screen.getByRole("button", { name: "Start recording" }));
  const area = screen.getByRole("group", { name: "Macro recording area" });
  fireEvent.keyDown(area, { code: "KeyA", key: "a" });
  fireEvent.keyDown(area, { code: "KeyA", key: "a", repeat: true });
  fireEvent.blur(area);
  expect(events().filter(e => e.kind !== "delay")).toEqual([{ kind: "down", key: 4 }, { kind: "up", key: 4 }]);
  expect(validateMacro(events())).toBeNull();
  expect(screen.getByRole("button", { name: "Start recording" })).toBeTruthy();
});

it("stops once when focus moves to the recording area's own Stop button", () => {
  editor();
  fireEvent.click(screen.getByRole("button", { name: "Start recording" }));
  const area = screen.getByRole("group", { name: "Macro recording area" });
  const stop = screen.getByRole("button", { name: "Stop recording" });
  fireEvent.blur(area, { relatedTarget: stop });
  expect(screen.queryByRole("button", { name: "Start recording" })).toBeNull();
  fireEvent.click(stop);
  expect(screen.getByRole("button", { name: "Start recording" })).toBeTruthy();
});

it("reserves release slots at the event limit without releasing an unrecorded key", () => {
  vi.spyOn(performance, "now").mockReturnValue(100);
  const events = editor(Array.from({ length: 2046 }, () => ({ kind: "delay", ms: 0 })));
  fireEvent.click(screen.getByRole("button", { name: "Start recording" }));
  const area = screen.getByRole("group", { name: "Macro recording area" });
  fireEvent.keyDown(area, { code: "KeyA", key: "a" });
  fireEvent.keyDown(area, { code: "KeyB", key: "b" });
  expect(events()).toHaveLength(2048);
  expect(events().slice(-2)).toEqual([{ kind: "down", key: 4 }, { kind: "up", key: 4 }]);
  expect(validateMacro(events())).toBeNull();
  expect(screen.getByRole("button", { name: "Start recording" })).toBeTruthy();
// Rendering and querying the real 2048-event editor can exceed five seconds on
// shared CI CPUs. Keep the full boundary/assertions and budget this case only.
}, 15_000);

it("keeps recording and preview isolated from macro switching and delay edits", () => {
  vi.useFakeTimers();
  editor([{ kind: "down", key: 4 }, { kind: "delay", ms: 100 }, { kind: "up", key: 4 }]);
  fireEvent.click(screen.getByRole("button", { name: /Preview sequence/ }));
  for (const label of ["+ New", "Import macro JSON", "Delete", "Export JSON"]) {
    expect((screen.getByRole("button", { name: label }) as HTMLButtonElement).disabled).toBe(true);
  }
  expect((screen.getByLabelText("Delay 2 in milliseconds") as HTMLInputElement).disabled).toBe(true);
  fireEvent.click(screen.getByRole("button", { name: "Stop preview" }));
  expect((screen.getByLabelText("Delay 2 in milliseconds") as HTMLInputElement).disabled).toBe(false);
});
