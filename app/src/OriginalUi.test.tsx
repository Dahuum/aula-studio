import { afterEach, expect, it } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import App from "./App";
import { originalEffectNames } from "./OriginalChrome";
import { wheelColor } from "./OriginalColors";
import { effects } from "./model";

afterEach(()=>{cleanup();localStorage.clear();});
async function open() { render(<App/>);await screen.findByText(/Browser preview · simulation only/); }

it("retains all twenty official lighting choices with working mode selection",async()=>{
  await open();expect(Object.keys(originalEffectNames)).toHaveLength(20);
  for(const effect of effects){fireEvent.click(screen.getByRole("button",{name:effect.name}));
    expect(document.querySelector('.original-app')?.getAttribute('data-page')).toBe(effect.id===21?"Per-Key RGB":"Lighting");
    expect(screen.getByRole("button",{name:effect.name}).getAttribute("aria-pressed")).toBe("true");
  }
});
it("keeps RGB, hex and preset controls synchronized and saves the real local draft",async()=>{
  await open();fireEvent.click(screen.getByRole("button",{name:"Preset #ff0000"}));
  fireEvent.change(screen.getByLabelText("G value"),{target:{value:"128"}});
  expect((screen.getByLabelText("Hex color") as HTMLInputElement).value).toBe("#ff8000");
  const hex=screen.getByLabelText("Hex color");fireEvent.change(hex,{target:{value:"#123456"}});fireEvent.blur(hex);
  expect((screen.getByLabelText("R value") as HTMLInputElement).value).toBe("18");
  fireEvent.change(hex,{target:{value:"#xyz"}});fireEvent.blur(hex);
  expect((hex as HTMLInputElement).value).toBe("#123456");
  fireEvent.click(screen.getByRole("button",{name:"Save profile"}));await screen.findByRole("status");
  const profile=JSON.parse(localStorage.getItem("aula-studio-preview-profiles")!)[0];expect(profile.lighting.color).toEqual([18,52,86]);
});
it("persists custom colors independently and enforces the ten-slot limit",async()=>{
  await open();const add=screen.getByRole("button",{name:"Add"});
  for(let i=0;i<10;i++){await waitFor(()=>expect((add as HTMLButtonElement).disabled).toBe(false));fireEvent.click(add);
    await waitFor(()=>expect(JSON.parse(localStorage.getItem("aula-studio-preview-preferences")!).custom_colors).toHaveLength(i+1));}
  expect((add as HTMLButtonElement).disabled).toBe(true);
});
it("exposes original Default/FN1 navigation and protects the actual Fn source",async()=>{
  await open();fireEvent.click(screen.getByRole("button",{name:"Key Remapping"}));
  const layers=screen.getByRole("group",{name:"Keyboard layer"});
  expect(layers.closest(".original-keyboard-panel")).toBeTruthy();
  expect(screen.getByRole("button",{name:"Default layer"}).getAttribute("aria-pressed")).toBe("true");
  fireEvent.click(screen.getByRole("button",{name:"Fn"}));
  expect((screen.getByRole("button",{name:"Assign locally"}) as HTMLButtonElement).disabled).toBe(true);
  fireEvent.click(screen.getByRole("button",{name:"Ctrl right"}));
  expect((screen.getByRole("button",{name:"Assign locally"}) as HTMLButtonElement).disabled).toBe(false);
  fireEvent.click(screen.getByRole("button",{name:"Fn Layers"}));
  expect(document.querySelector('.original-app')?.getAttribute('data-page')).toBe("Fn Layers");
  expect(screen.getByRole("button",{name:"Fn Layers"}).getAttribute("aria-pressed")).toBe("true");
  fireEvent.click(screen.getByRole("button",{name:"Default layer"}));
  expect(document.querySelector('.original-app')?.getAttribute('data-page')).toBe("Key Remapping");
});
it("keeps Linux diagnostics accessible from the additional-views menu",async()=>{
  await open();fireEvent.click(screen.getByRole("button",{name:"More views"}));fireEvent.click(screen.getByRole("button",{name:"Diagnostics"}));
  expect(screen.getByText("Device & protocol diagnostics")).toBeTruthy();
});
it("maps wheel coordinates to bounded RGB with the displayed HSV orientation",()=>{
  expect(wheelColor(0,0)).toEqual([255,255,255]);expect(wheelColor(1,0)).toEqual([255,0,0]);
  expect(wheelColor(-1,0)).toEqual([0,255,255]);
  for(const rgb of [wheelColor(5,5),wheelColor(-5,1)])expect(rgb.every(v=>Number.isInteger(v)&&v>=0&&v<=255)).toBe(true);
});
