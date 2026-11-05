import { expect, it } from "vitest";
import { macroTimeline } from "./macroTimeline";

it("keeps local delays, chords and trailing silence without artificial key pauses", () => {
  const t = macroTimeline([
    { kind: "down", key: 4 },
    { kind: "down", key: 5 },
    { kind: "delay", ms: 125 },
    { kind: "delay", ms: 75 },
    { kind: "up", key: 4 },
    { kind: "delay", ms: 100 },
    { kind: "up", key: 5 },
    { kind: "delay", ms: 300 },
  ]);
  expect(t).toEqual({
    durationMs: 600,
    steps: [
      { atMs: 0, indices: [0, 1], held: [4, 5] },
      { atMs: 200, indices: [4], held: [5] },
      { atMs: 300, indices: [6], held: [] },
    ],
  });
});
it("supports local leading silence and zero-time pairs but rejects invalid key state", () => {
  expect(
    macroTimeline([
      { kind: "delay", ms: 50 },
      { kind: "down", key: 4 },
      { kind: "up", key: 4 },
    ]),
  ).toEqual({
    durationMs: 50,
    steps: [{ atMs: 50, indices: [1, 2], held: [] }],
  });
  expect(() => macroTimeline([{ kind: "up", key: 4 }])).toThrow();
  expect(() => macroTimeline([{ kind: "delay", ms: 60001 }])).toThrow();
});
