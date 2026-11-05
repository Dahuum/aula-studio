import { validateMacro } from "./model";
import type { MacroEvent } from "./model";

export type PreviewStep = {
  atMs: number;
  indices: number[];
  held: number[];
};

/** Local editor timing only. Never converts to a firmware macro or injects keys. */
export function macroTimeline(events: MacroEvent[]): {
  steps: PreviewStep[];
  durationMs: number;
} {
  const error = validateMacro(events);
  if (error) throw new Error(error);
  const steps: PreviewStep[] = [];
  const held = new Set<number>();
  let atMs = 0;
  events.forEach((event, index) => {
    if (event.kind === "delay") {
      atMs += event.ms!;
      return;
    }
    if (event.kind === "down") held.add(event.key!);
    else held.delete(event.key!);
    let step = steps[steps.length - 1];
    if (!step || step.atMs !== atMs) {
      step = { atMs, indices: [], held: [] };
      steps.push(step);
    }
    step.indices.push(index);
    step.held = [...held];
  });
  return { steps, durationMs: atMs };
}
