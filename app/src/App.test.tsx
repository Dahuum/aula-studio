import { afterEach, it, expect } from "vitest";
import {
  render,
  screen,
  fireEvent,
  waitFor,
  cleanup,
} from "@testing-library/react";
import App from "./App";
afterEach(() => {
  cleanup();
  localStorage.clear();
});
it("opens explicit browser demo and saves a local profile", async () => {
  render(<App />);
  await screen.findByText(/Browser preview · simulation only/);
  fireEvent.click(screen.getByRole("button", { name: "Save profile" }));
  await screen.findByRole("status");
  expect(localStorage.getItem("aula-studio-preview-profiles")).toContain(
    "Untitled profile",
  );
  fireEvent.click(screen.getByRole("button", { name: /Profiles$/ }));
  expect(screen.getByText("Local profile library")).toBeTruthy();
});
it("disables physical writes outside demo", async () => {
  render(<App />);
  await screen.findByText(/Browser preview · simulation only/);
  fireEvent.click(screen.getByRole("button", { name: "Exit demo mode" }));
  await waitFor(() =>
    expect(
      (
        screen.getByRole("button", {
          name: "Review lighting write",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(true),
  );
});
it("only enables lighting parameters exposed by the selected official effect", async () => {
  render(<App />);
  await screen.findByText(/Browser preview · simulation only/);
  fireEvent.click(screen.getByRole("button", { name: /Lighting$/ }));
  const brightness = screen.getByRole("slider", {
    name: /Brightness/,
  }) as HTMLInputElement;
  const speed = screen.getByRole("slider", {
    name: /Speed/,
  }) as HTMLInputElement;
  const color = screen.getByLabelText("Lighting color") as HTMLInputElement;
  const multicolor = screen.getByRole("checkbox", {
    name: /Multicolor/,
  }) as HTMLInputElement;
  expect(brightness.disabled).toBe(false);
  expect(speed.disabled).toBe(true); // Solid color has no animation speed.
  fireEvent.click(screen.getByRole("button", { name: /^Windmill/ }));
  expect(brightness.disabled).toBe(false);
  expect(speed.disabled).toBe(false);
  expect(color.disabled).toBe(true);
  expect(multicolor.disabled).toBe(true);
  fireEvent.click(screen.getByRole("button", { name: /^Lights off/ }));
  expect(brightness.disabled).toBe(true);
  expect(speed.disabled).toBe(true);
  fireEvent.click(screen.getByRole("button", { name: /Per-Key RGB$/ }));
  expect(
    (screen.getByLabelText("Lighting color") as HTMLInputElement).disabled,
  ).toBe(false);
});
