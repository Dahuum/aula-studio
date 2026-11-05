import { useState } from "react";
import { afterEach, expect, it, vi } from "vitest";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
} from "@testing-library/react";
import { Macros } from "./Macros";
import { validateMacro } from "./model";
import type { Macro, MacroEvent } from "./model";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
  vi.useRealTimers();
});

function editor(events: MacroEvent[] = []) {
  function Harness() {
    const [macros, setMacros] = useState<Macro[]>([
      { id: "test", name: "Test", events },
    ]);
    return (
      <>
        <Macros
          macros={macros}
          onChange={setMacros}
          onImport={() => {}}
          onExport={() => {}}
        />
        <output data-testid="events">{JSON.stringify(macros[0].events)}</output>
      </>
    );
  }
  render(<Harness />);
  return () =>
    JSON.parse(screen.getByTestId("events").textContent!) as MacroEvent[];
}

it("releases recorded held keys on focus loss and ignores repeated downs", () => {
  const events = editor();
  fireEvent.click(screen.getByRole("button", { name: "Start recording" }));
  const area = screen.getByRole("group", { name: "Macro recording area" });
  fireEvent.keyDown(area, { code: "KeyA", key: "a" });
  fireEvent.keyDown(area, { code: "KeyA", key: "a", repeat: true });
  fireEvent.blur(area);
  expect(events().filter((e) => e.kind !== "delay")).toEqual([
    { kind: "down", key: 4 },
    { kind: "up", key: 4 },
  ]);
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
  const events = editor(
    Array.from({ length: 2046 }, () => ({ kind: "delay", ms: 0 })),
  );
  fireEvent.click(screen.getByRole("button", { name: "Start recording" }));
  const area = screen.getByRole("group", { name: "Macro recording area" });
  fireEvent.keyDown(area, { code: "KeyA", key: "a" });
  fireEvent.keyDown(area, { code: "KeyB", key: "b" });
  expect(events()).toHaveLength(2048);
  expect(events().slice(-2)).toEqual([
    { kind: "down", key: 4 },
    { kind: "up", key: 4 },
  ]);
  expect(validateMacro(events())).toBeNull();
  expect(screen.getByRole("button", { name: "Start recording" })).toBeTruthy();
  // Rendering and querying the real 2048-event editor can exceed five seconds on
  // shared CI CPUs. Keep the full boundary/assertions and budget this case only.
}, 15_000);

it("keeps recording and preview isolated from macro switching and delay edits", () => {
  vi.useFakeTimers();
  editor([
    { kind: "down", key: 4 },
    { kind: "delay", ms: 100 },
    { kind: "up", key: 4 },
  ]);
  fireEvent.click(screen.getByRole("button", { name: /Preview sequence/ }));
  for (const label of ["+ New", "Import macro JSON", "Delete", "Export JSON"]) {
    expect(
      (screen.getByRole("button", { name: label }) as HTMLButtonElement)
        .disabled,
    ).toBe(true);
  }
  expect(
    (screen.getByLabelText("Delay 2 in milliseconds") as HTMLInputElement)
      .disabled,
  ).toBe(true);
  fireEvent.click(screen.getByRole("button", { name: "Stop preview" }));
  expect(
    (screen.getByLabelText("Delay 2 in milliseconds") as HTMLInputElement)
      .disabled,
  ).toBe(false);
});

it("edits key actions, reorders events and inserts at the selected position", () => {
  const events = editor([
    { kind: "down", key: 4 },
    { kind: "delay", ms: 100 },
    { kind: "up", key: 4 },
  ]);
  fireEvent.click(screen.getByRole("button", { name: "Edit key for event 1" }));
  fireEvent.change(screen.getByLabelText("Key for event 1"), {
    target: { value: "5" },
  });
  expect(validateMacro(events())).not.toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Edit key for event 3" }));
  fireEvent.change(screen.getByLabelText("Key for event 3"), {
    target: { value: "5" },
  });
  expect(validateMacro(events())).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Move event 2 up" }));
  expect(events()[0]).toEqual({ kind: "delay", ms: 100 });
  fireEvent.change(screen.getByLabelText("Insert position"), {
    target: { value: "after" },
  });
  fireEvent.click(screen.getByRole("button", { name: "+ Delay" }));
  expect(events().slice(0, 2)).toEqual([
    { kind: "delay", ms: 100 },
    { kind: "delay", ms: 100 },
  ]);
  expect(validateMacro(events())).toBeNull();
});
it("previews the actual local timeline and cancels on a profile replacement", () => {
  vi.useFakeTimers();
  const macro: Macro = {
    id: "a",
    name: "A",
    events: [
      { kind: "down", key: 4 },
      { kind: "delay", ms: 1000 },
      { kind: "up", key: 4 },
      { kind: "delay", ms: 100 },
    ],
  };
  const view = render(
    <Macros
      macros={[macro]}
      onChange={() => {}}
      onImport={() => {}}
      onExport={() => {}}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: /Preview sequence/ }));
  expect(screen.getByRole("status").textContent).toContain("Held keys: A");
  act(() => vi.advanceTimersByTime(999));
  expect(screen.getByRole("status").textContent).toContain("Held keys: A");
  act(() => vi.advanceTimersByTime(1));
  expect(screen.getByRole("status").textContent).toContain(
    "1000 ms · Held keys: none",
  );
  act(() => vi.advanceTimersByTime(101));
  expect(screen.queryByRole("status")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: /Preview sequence/ }));
  view.rerender(
    <Macros
      macros={[{ id: "b", name: "Replacement", events: [] }]}
      onChange={() => {}}
      onImport={() => {}}
      onExport={() => {}}
    />,
  );
  act(() => vi.runAllTimers());
  expect(screen.queryByRole("status")).toBeNull();
  expect((screen.getByLabelText("Macro name") as HTMLInputElement).value).toBe(
    "Replacement",
  );
});
it("pages long sequences and keeps boundary moves attached to the correct event", () => {
  const events = editor(
    Array.from({ length: 125 }, (_, ms) => ({ kind: "delay", ms })),
  );
  expect(document.querySelectorAll(".events .event")).toHaveLength(100);
  fireEvent.click(screen.getByRole("button", { name: "Next events" }));
  expect(document.querySelectorAll(".events .event")).toHaveLength(25);
  fireEvent.click(screen.getByRole("button", { name: "Move event 101 up" }));
  expect(events()[99]).toEqual({ kind: "delay", ms: 100 });
  expect(events()[100]).toEqual({ kind: "delay", ms: 99 });
  expect(
    screen
      .getByRole("button", { name: "Select event 100" })
      .getAttribute("aria-pressed"),
  ).toBe("true");
});
it("duplicates a macro independently and clears only after explicit confirmation", () => {
  const events = editor([
    { kind: "down", key: 4 },
    { kind: "up", key: 4 },
  ]);
  fireEvent.click(screen.getByRole("button", { name: "Duplicate macro" }));
  expect((screen.getByLabelText("Macro name") as HTMLInputElement).value).toBe(
    "Test (copy)",
  );
  const confirm = vi.spyOn(window, "confirm").mockReturnValue(false);
  fireEvent.click(screen.getByRole("button", { name: "Clear events" }));
  expect(
    screen.queryByText("No events yet. Record a sequence or add a key press."),
  ).toBeNull();
  confirm.mockReturnValue(true);
  fireEvent.click(screen.getByRole("button", { name: "Clear events" }));
  expect(
    screen.getByText("No events yet. Record a sequence or add a key press."),
  ).toBeTruthy();
  expect(events()).toEqual([
    { kind: "down", key: 4 },
    { kind: "up", key: 4 },
  ]);
});

it("duplicates a maximum-byte Unicode macro into a saveable independent copy", () => {
  const original: Macro = {
    id: "unicode",
    name: "🎹".repeat(30),
    events: [{ kind: "down", key: 4 }, { kind: "up", key: 4 }],
  };
  const onChange = vi.fn();
  render(<Macros macros={[original]} onChange={onChange} onImport={() => {}} onExport={() => {}} />);
  fireEvent.click(screen.getByRole("button", { name: "Duplicate macro" }));
  const copied = onChange.mock.calls[0][0][1] as Macro;
  expect(copied.id).not.toBe(original.id);
  expect(copied.name).toBe("🎹".repeat(28) + " (copy)");
  expect(copied.events).toEqual(original.events);
  expect(copied.events[0]).not.toBe(original.events[0]);
  expect(original.name).toBe("🎹".repeat(30));
});

it("connects the original six-button sidebar toolbar and locks it during recording", async () => {
  const exported = vi.fn();
  function Harness() {
    const [macros, setMacros] = useState<Macro[]>([
      {
        id: "toolbar",
        name: "Toolbar",
        events: [
          { kind: "down", key: 4 },
          { kind: "up", key: 4 },
        ],
      },
    ]);
    return (
      <>
        <div id="original-macro-list" />
        <Macros
          macros={macros}
          onChange={setMacros}
          onImport={() => {}}
          onExport={exported}
        />
      </>
    );
  }
  render(<Harness />);
  expect(
    [...document.querySelectorAll(".original-macro-toolbar button")].map((b) =>
      b.getAttribute("aria-label"),
    ),
  ).toEqual([
    "Export JSON",
    "Import macro JSON",
    "Rename macro",
    "Duplicate macro",
    "Delete",
    "+ New",
  ]);
  fireEvent.click(screen.getByRole("button", { name: "Rename macro" }));
  expect(document.activeElement).toBe(screen.getByLabelText("Macro name"));
  fireEvent.click(screen.getByRole("button", { name: "Export JSON" }));
  expect(exported).toHaveBeenCalledWith(
    "Toolbar",
    expect.stringContaining('"schema_version": 1'),
  );
  fireEvent.click(screen.getByRole("button", { name: "Start recording" }));
  expect(
    [
      ...document.querySelectorAll<HTMLButtonElement>(
        ".original-macro-toolbar button",
      ),
    ].every((b) => b.disabled),
  ).toBe(true);
  fireEvent.click(screen.getByRole("button", { name: "Stop recording" }));
});
