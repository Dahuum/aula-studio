// Independently drawn ANSI F75 geometry; HID usages follow the USB keyboard usage table.
// Source facts: AULA F75 manual and public F75 key-name inventory (see docs/SOURCES.md).
export type Key = {
  id: string;
  label: string;
  code: string;
  x: number;
  y: number;
  width: number;
  usage?: number;
};
const keys: Key[] = [];
function add(
  label: string,
  usage: number | undefined,
  code: string,
  x: number,
  y: number,
  width = 1,
) {
  keys.push({
    id: usage === undefined ? "fn" : String(usage),
    label,
    usage,
    code,
    x,
    y,
    width,
  });
}
add("Esc", 41, "Escape", 0, 0);
for (let i = 1; i <= 12; i++)
  add(
    "F" + i,
    57 + i,
    "F" + i,
    1.5 + i - 1 + Math.floor((i - 1) / 4) * 0.25,
    0,
  );
let x = 0;
[
  ["`", 53, "Backquote"],
  ["1", 30, "Digit1"],
  ["2", 31, "Digit2"],
  ["3", 32, "Digit3"],
  ["4", 33, "Digit4"],
  ["5", 34, "Digit5"],
  ["6", 35, "Digit6"],
  ["7", 36, "Digit7"],
  ["8", 37, "Digit8"],
  ["9", 38, "Digit9"],
  ["0", 39, "Digit0"],
  ["−", 45, "Minus"],
  ["=", 46, "Equal"],
].forEach(([l, u, c]) => add(String(l), Number(u), String(c), x++, 1.3));
add("Backspace", 42, "Backspace", 13, 1.3, 2);
add("Del", 76, "Delete", 15.25, 1.3);
add("Tab", 43, "Tab", 0, 2.3, 1.5);
"QWERTYUIOP"
  .split("")
  .forEach((l, i) => add(l, l.charCodeAt(0) - 61, "Key" + l, 1.5 + i, 2.3));
add("[", 47, "BracketLeft", 11.5, 2.3);
add("]", 48, "BracketRight", 12.5, 2.3);
add("\\", 49, "Backslash", 13.5, 2.3, 1.5);
add("PgUp", 75, "PageUp", 15.25, 2.3);
add("Caps", 57, "CapsLock", 0, 3.3, 1.75);
"ASDFGHJKL"
  .split("")
  .forEach((l, i) => add(l, l.charCodeAt(0) - 61, "Key" + l, 1.75 + i, 3.3));
add(";", 51, "Semicolon", 10.75, 3.3);
add("'", 52, "Quote", 11.75, 3.3);
add("Enter", 40, "Enter", 12.75, 3.3, 2.25);
add("PgDn", 78, "PageDown", 15.25, 3.3);
add("Shift", 225, "ShiftLeft", 0, 4.3, 2.25);
"ZXCVBNM"
  .split("")
  .forEach((l, i) => add(l, l.charCodeAt(0) - 61, "Key" + l, 2.25 + i, 4.3));
add(",", 54, "Comma", 9.25, 4.3);
add(".", 55, "Period", 10.25, 4.3);
add("/", 56, "Slash", 11.25, 4.3);
add("Shift", 229, "ShiftRight", 12.25, 4.3, 1.75);
add("↑", 82, "ArrowUp", 14.25, 4.3);
add("End", 77, "End", 15.25, 4.3);
add("Ctrl", 224, "ControlLeft", 0, 5.3, 1.25);
add("Win", 227, "MetaLeft", 1.25, 5.3, 1.25);
add("Alt", 226, "AltLeft", 2.5, 5.3, 1.25);
add("Space", 44, "Space", 3.75, 5.3, 6.25);
add("Fn", undefined, "Fn", 10, 5.3, 1.25);
add("Ctrl", 228, "ControlRight", 11.25, 5.3, 1.25);
add("←", 80, "ArrowLeft", 13.25, 5.3);
add("↓", 81, "ArrowDown", 14.25, 5.3);
add("→", 79, "ArrowRight", 15.25, 5.3);
export const KEYBOARD = keys;
export const CODE_USAGE = Object.fromEntries(
  keys.filter((k) => k.usage !== undefined).map((k) => [k.code, k.usage!]),
);
export const usageLabel = (usage: number) =>
  KEYBOARD.find((k) => k.usage === usage)?.label ?? `HID ${usage}`;
