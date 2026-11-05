import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import type { Profile, PreparedLighting, Status } from "./model";
const bridge = vi.hoisted(() => ({ call: vi.fn() }));
vi.mock("./api", () => ({ native: true, call: bridge.call }));
import App from "./App";
import { createProfile } from "./model";
let draft: Profile;
const status: Status = {
  mode: "hardware",
  connected: true,
  devices: [
    {
      path: "/dev/hidraw-test",
      vendor_id: 0x258a,
      product_id: 0x010c,
      product: "Test F75",
      supported: true,
    },
  ],
  capabilities: {
    read_config: true,
    write_lighting: true,
    write_per_key: false,
    write_keymap: false,
    write_fn: false,
    onboard_macros: false,
  },
};
const preparation: PreparedLighting = {
  token: "single-use-test-token",
  device: "Test F75 · /dev/hidraw-test",
  summary: "Change supported lighting settings",
  backup_id: "backup-before-write",
  changes: [
    { field: "brightness", before: 1, after: 6 },
    { field: "effect", before: 15, after: 1 },
  ],
  warnings: ["Persistence after reconnecting has not been verified."],
};
const defaultCall = async (command: string) => {
  switch (command) {
    case "get_preferences":
      return { demo_mode: false, reduce_motion: false };
    case "get_status":
      return structuredClone(status);
    case "list_profiles":
      return [structuredClone(draft)];
    case "list_backups":
      return [];
    case "prepare_lighting":
      return structuredClone(preparation);
    case "commit_lighting":
      return {
        message: "Lighting applied.",
        readback_verified: true,
        persistence: "not_tested",
      };
    case "read_configuration":
      return {
        kind: "hardware",
        lighting: {
          effect: 15,
          brightness: 1,
          speed: 3,
          color: [12, 34, 56],
          colorful: false,
        },
      };
    default:
      throw new Error(`Unexpected command ${command}`);
  }
};
beforeEach(() => {
  draft = createProfile("Test profile");
  bridge.call.mockReset();
  bridge.call.mockImplementation(defaultCall);
});
afterEach(() => cleanup());
async function openReview() {
  render(<App />);
  const button = screen.getByRole("button", { name: "Review lighting write" });
  await waitFor(() =>
    expect((button as HTMLButtonElement).disabled).toBe(false),
  );
  fireEvent.click(button);
  return screen.findByRole("dialog", { name: "Review lighting changes" });
}
describe("physical lighting confirmation boundary", () => {
  it("only prepares after review action and displays the exact backup and change list", async () => {
    const dialog = await openReview();
    expect(bridge.call).toHaveBeenCalledWith("prepare_lighting", {
      profile: draft,
    });
    expect(bridge.call.mock.calls.some(([c]) => c === "commit_lighting")).toBe(
      false,
    );
    expect(within(dialog).getByText("backup-before-write")).toBeTruthy();
    expect(within(dialog).getByText("brightness")).toBeTruthy();
    expect(
      within(dialog).getByText(/other lighting controls require physical validation/),
    ).toBeTruthy();
    fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(bridge.call.mock.calls.some(([c]) => c === "commit_lighting")).toBe(
      false,
    );
  });
  it("commits the prepared token only after explicit write and reports readback separately from persistence", async () => {
    const dialog = await openReview();
    fireEvent.click(
      within(dialog).getByRole("button", {
        name: "Write lighting to keyboard",
      }),
    );
    await screen.findByRole("status");
    expect(
      bridge.call.mock.calls.filter(([c]) => c === "commit_lighting"),
    ).toEqual([["commit_lighting", { token: "single-use-test-token" }]]);
    expect(screen.getByRole("status").textContent).toContain(
      "Readback verified",
    );
    expect(screen.getByRole("status").textContent).toContain(
      "Persistence after reconnection has not been tested",
    );
    expect(screen.queryByRole("dialog")).toBeNull();
  });
  it("propagates preparation failure without presenting a write action", async () => {
    bridge.call.mockImplementation(async (command: string) => {
      if (command === "prepare_lighting")
        throw new Error(
          "Device config shape does not match supported firmware",
        );
      return defaultCall(command);
    });
    render(<App />);
    const button = screen.getByRole("button", {
      name: "Review lighting write",
    });
    await waitFor(() =>
      expect((button as HTMLButtonElement).disabled).toBe(false),
    );
    fireEvent.click(button);
    expect((await screen.findByRole("alert")).textContent).toContain(
      "config shape",
    );
    expect(
      screen.queryByRole("button", { name: "Write lighting to keyboard" }),
    ).toBeNull();
    expect(bridge.call.mock.calls.some(([c]) => c === "commit_lighting")).toBe(
      false,
    );
  });
  it("discards a failed transaction and requires a new preparation", async () => {
    bridge.call.mockImplementation(async (command: string) => {
      if (command === "commit_lighting")
        throw new Error(
          "Device disconnected before write; nothing transmitted",
        );
      return defaultCall(command);
    });
    const dialog = await openReview();
    fireEvent.click(
      within(dialog).getByRole("button", {
        name: "Write lighting to keyboard",
      }),
    );
    expect((await screen.findByRole("alert")).textContent).toContain(
      "Device disconnected",
    );
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(
      bridge.call.mock.calls.filter(([c]) => c === "commit_lighting"),
    ).toHaveLength(1);
    fireEvent.click(
      screen.getByRole("button", { name: "Review lighting write" }),
    );
    await screen.findByRole("dialog");
    expect(
      bridge.call.mock.calls.filter(([c]) => c === "prepare_lighting"),
    ).toHaveLength(2);
    expect(
      bridge.call.mock.calls.filter(([c]) => c === "commit_lighting"),
    ).toHaveLength(1);
  });
  it("does not write an empty change set", async () => {
    bridge.call.mockImplementation(async (command: string) =>
      command === "prepare_lighting"
        ? { ...preparation, changes: [] }
        : defaultCall(command),
    );
    const dialog = await openReview();
    expect(
      (
        within(dialog).getByRole("button", {
          name: "Write lighting to keyboard",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(true);
    expect(bridge.call.mock.calls.some(([c]) => c === "commit_lighting")).toBe(
      false,
    );
  });
});
it("loads read lighting only after the explicit local-draft action", async () => {
  render(<App />);
  fireEvent.click(screen.getByText("Read / recover"));
  const button = screen.getByRole("button", { name: "Read configuration" });
  await waitFor(() =>
    expect((button as HTMLButtonElement).disabled).toBe(false),
  );
  fireEvent.click(button);
  const load = await screen.findByRole("button", {
    name: "Load read lighting into draft",
  });
  expect(
    bridge.call.mock.calls.some(
      ([c]) => c === "prepare_lighting" || c === "commit_lighting",
    ),
  ).toBe(false);
  fireEvent.click(load);
  expect(
    screen.getByRole("heading", { level: 1, name: "Lighting" }),
  ).toBeTruthy();
  expect(
    (screen.getByLabelText("Lighting color") as HTMLInputElement).value,
  ).toBe("#0c2238");
  expect((screen.getByRole("slider", {name:"Brightness"}) as HTMLInputElement).value).toBe("1");
  expect((screen.getByRole("slider", {name:"Speed"}) as HTMLInputElement).value).toBe("3");
  expect(
    bridge.call.mock.calls.some(
      ([c]) =>
        c === "save_profile" ||
        c === "prepare_lighting" ||
        c === "commit_lighting",
    ),
  ).toBe(false);
});

describe("scoped mapping hardware operations", () => {
  for (const [page,kind,mouseField,keyboardField,otherField] of [
    ["Key Remapping","base","mouse_keymap","keymap","mouse_fn_layer"],
    ["Fn Layers","fn","mouse_fn_layer","fn_layer","mouse_keymap"],
  ] as const) {
    it(`replaces an assignment with a mouse button in ${kind} only on explicit review`,async()=>{
      draft.keymap={"4":5};draft.fn_layer={"4":6};
      draft.consumer_keymap={"5":0xcd};draft.consumer_fn_layer={"5":0xe2};
      draft.mouse_keymap={"6":3};draft.mouse_fn_layer={"6":4};
      bridge.call.mockImplementation(async(command:string)=>{
        if(command==="get_status")return {...status,capabilities:{...status.capabilities,write_keymap:true,write_fn:true}};
        if(command==="prepare_mapping")return preparation;
        return defaultCall(command);
      });
      render(<App/>);await waitFor(()=>expect(bridge.call).toHaveBeenCalledWith("list_profiles"));
      fireEvent.click(screen.getByRole("button",{name:"Key Remapping"}));
      if(page==="Fn Layers")fireEvent.click(screen.getByRole("button",{name:"Fn Layers"}));
      fireEvent.click(screen.getByRole("button",{name:"A"}));
      fireEvent.change(screen.getByLabelText("Action type"),{target:{value:"mouse"}});
      fireEvent.change(screen.getByLabelText("Mouse action"),{target:{value:"2"}});
      fireEvent.click(screen.getByRole("button",{name:"Assign locally"}));
      expect(bridge.call.mock.calls.some(([c])=>c==="prepare_mapping"||c.startsWith("commit_"))).toBe(false);
      const review=screen.getByRole("button",{name:`Review ${kind==="base"?"base":"Fn"} layer write`});
      await waitFor(()=>expect((review as HTMLButtonElement).disabled).toBe(false));fireEvent.click(review);
      await screen.findByRole("dialog");
      const request=bridge.call.mock.calls.find(([c])=>c==="prepare_mapping")?.[1] as {profile:Profile;kind:string};
      expect(request.kind).toBe(kind);expect(request.profile[mouseField]).toEqual({"4":2,"6":draft[mouseField]!["6"]});
      expect(request.profile[keyboardField]).toEqual({});expect(request.profile[otherField]).toEqual(draft[otherField]);
      expect(request.profile.consumer_keymap).toEqual(draft.consumer_keymap);expect(request.profile.consumer_fn_layer).toEqual(draft.consumer_fn_layer);
      expect(bridge.call.mock.calls.some(([c])=>c.startsWith("commit_"))).toBe(false);
    });
  }
  for (const [page, kind, field, other] of [
    ["Key Remapping", "base", "consumer_keymap", "fn_layer"],
    ["Fn Layers", "fn", "consumer_fn_layer", "keymap"],
  ] as const) {
    for (const usage of [0xe2, 0x223]) {
      it(`edits media ${usage} in ${kind} and sends the typed draft only on review`, async () => {
        draft.keymap = { "4": 5 };
        draft.fn_layer = { "4": 6 };
        bridge.call.mockImplementation(async (command: string) => {
          if (command === "get_status") return { ...status, capabilities: { ...status.capabilities, write_keymap: true, write_fn: true } };
          if (command === "prepare_mapping") return preparation;
          return defaultCall(command);
        });
        render(<App />);
        await waitFor(() => expect(bridge.call).toHaveBeenCalledWith("list_profiles"));
        if(page==="Fn Layers") fireEvent.click(screen.getByRole("button",{name:"Key Remapping"}));
        fireEvent.click(screen.getByRole("button", { name: new RegExp(`${page}$`) }));
        fireEvent.click(screen.getByRole("button", { name: "A" }));
        fireEvent.change(screen.getByLabelText("Action type"), { target: { value: "media" } });
        fireEvent.change(screen.getByLabelText("Media action"), { target: { value: String(usage) } });
        fireEvent.click(screen.getByRole("button", { name: "Assign locally" }));
        expect(bridge.call.mock.calls.some(([c]) => c === "prepare_mapping" || c.startsWith("commit_"))).toBe(false);
        const review = screen.getByRole("button", { name: `Review ${kind === "base" ? "base" : "Fn"} layer write` });
        await waitFor(() => expect((review as HTMLButtonElement).disabled).toBe(false));
        fireEvent.click(review);
        await screen.findByRole("dialog");
        const request = bridge.call.mock.calls.find(([c]) => c === "prepare_mapping")?.[1] as { profile: Profile; kind: string };
        expect(request.kind).toBe(kind);
        expect(request.profile[field]).toEqual({ "4": usage });
        expect(request.profile[kind === "base" ? "keymap" : "fn_layer"]).toEqual({});
        expect(request.profile[other]).toEqual(draft[other]);
        expect(bridge.call.mock.calls.some(([c]) => c.startsWith("commit_"))).toBe(false);
      });
    }
  }
  for (const [page, kind, label] of [
    ["Per-Key RGB", "per_key", "per-key colors"],
    ["Key Remapping", "base", "base layer"],
    ["Fn Layers", "fn", "Fn layer"],
  ]) {
    it(`prepares ${kind} without writing, then commits only its token`, async () => {
      bridge.call.mockImplementation(async (command: string) => {
        if (command === "get_status")
          return {
            ...status,
            capabilities: {
              ...status.capabilities,
              write_per_key: true,
              write_keymap: true,
              write_fn: true,
            },
          };
        if (command === "prepare_mapping")
          return { ...preparation, summary: `Change ${kind}` };
        if (command === "commit_mapping")
          return {
            message: "Readback matched",
            readback_verified: true,
            persistence: "not_tested",
          };
        return defaultCall(command);
      });
      render(<App />);
      if(page==="Fn Layers") fireEvent.click(screen.getByRole("button",{name:"Key Remapping"}));
      fireEvent.click(
        screen.getByRole("button", { name: new RegExp(`${page}$`) }),
      );
      const button = screen.getByRole("button", {
        name: `Review ${label} write`,
      });
      await waitFor(() =>
        expect((button as HTMLButtonElement).disabled).toBe(false),
      );
      fireEvent.click(button);
      const dialog = await screen.findByRole("dialog");
      expect(bridge.call.mock.calls.some(([c]) => c === "commit_mapping")).toBe(
        false,
      );
      expect(
        bridge.call.mock.calls.find(([c]) => c === "prepare_mapping")?.[1],
      ).toMatchObject({ kind });
      fireEvent.click(
        within(dialog).getByRole("button", {
          name: `Write ${label} to keyboard`,
        }),
      );
      await waitFor(() =>
        expect(
          bridge.call.mock.calls.filter(([c]) => c === "commit_mapping"),
        ).toHaveLength(1),
      );
      expect(
        bridge.call.mock.calls.some(([c]) => c === "commit_lighting"),
      ).toBe(false);
    });
  }
});

it("reviews sleep settings as a separate bounded configuration write", async () => {
  bridge.call.mockImplementation(async (command: string) => {
    if (command === "prepare_settings")
      return {
        ...preparation,
        summary: "Change sleep timer",
        changes: [{ field: "Sleep seconds", before: 300, after: 600 }],
      };
    return defaultCall(command);
  });
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: /Device Settings$/ }));
  await waitFor(() =>
    expect(
      (
        screen.getByRole("button", {
          name: "Review device settings write",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(false),
  );
  fireEvent.change(screen.getByLabelText("Hardware sleep timer"), {
    target: { value: "600" },
  });
  fireEvent.click(
    screen.getByRole("button", { name: "Review device settings write" }),
  );
  const dialog = await screen.findByRole("dialog");
  expect(
    bridge.call.mock.calls.find(([c]) => c === "prepare_settings")?.[1].profile
      .sleep_seconds,
  ).toBe(600);
  expect(bridge.call.mock.calls.some(([c]) => c === "commit_lighting")).toBe(
    false,
  );
  fireEvent.click(
    within(dialog).getByRole("button", {
      name: "Write device settings to keyboard",
    }),
  );
  await waitFor(() =>
    expect(
      bridge.call.mock.calls.filter(([c]) => c === "commit_lighting"),
    ).toHaveLength(1),
  );
});

it("reviews a physical snapshot restoration without automatically applying it", async () => {
  bridge.call.mockImplementation(async (command: string) => {
    if (command === "list_backups")
      return [
        {
          id: "old-snapshot",
          created_at: "local",
          device: "Test F75",
          kind: "hardware",
        },
      ];
    if (command === "prepare_restore")
      return {
        ...preparation,
        summary: "Restore supported lighting and sleep only",
        backup_id: "before-restore-backup",
      };
    return defaultCall(command);
  });
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: /Device Settings$/ }));
  const button = await screen.findByRole("button", {
    name: "Review lighting restore",
  });
  await waitFor(() =>
    expect((button as HTMLButtonElement).disabled).toBe(false),
  );
  fireEvent.click(button);
  const dialog = await screen.findByRole("dialog");
  expect(within(dialog).getByText("before-restore-backup")).toBeTruthy();
  expect(
    bridge.call.mock.calls.find(([c]) => c === "prepare_restore")?.[1],
  ).toEqual({ id: "old-snapshot" });
  expect(bridge.call.mock.calls.some(([c]) => c.startsWith("commit_"))).toBe(
    false,
  );
  fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
  expect(bridge.call.mock.calls.some(([c]) => c.startsWith("commit_"))).toBe(
    false,
  );
});

it("opens long hardware reviews at the heading and keeps keyboard focus inside", async () => {
  const dialog = await openReview();
  await waitFor(() => expect(document.activeElement).toBe(dialog));
  expect(dialog.scrollTop).toBe(0);
  fireEvent.keyDown(document, { key: "Tab" });
  expect(document.activeElement).toBe(
    within(dialog).getByRole("button", { name: "Cancel" }),
  );
});

it("allows Fn RGB without enabling Fn source remapping", async () => {
  bridge.call.mockImplementation(async (command: string) => {
    if (command === "get_status")
      return {
        ...status,
        capabilities: { ...status.capabilities, write_per_key: true },
      };
    if (command === "prepare_mapping")
      return { ...preparation, summary: "Per-key colors" };
    return defaultCall(command);
  });
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: /Per-Key RGB$/ }));
  await waitFor(() =>
    expect(
      (
        screen.getByRole("button", {
          name: "Review per-key colors write",
        }) as HTMLButtonElement
      ).disabled,
    ).toBe(false),
  );
  fireEvent.click(screen.getByRole("button", { name: "Fn" }));
  fireEvent.click(screen.getByRole("button", { name: "Assign color" }));
  fireEvent.click(
    screen.getByRole("button", { name: "Review per-key colors write" }),
  );
  await screen.findByRole("dialog");
  expect(
    bridge.call.mock.calls.find(([c]) => c === "prepare_mapping")?.[1].profile
      .per_key.fn,
  ).toEqual(draft.lighting.color);
  expect(bridge.call.mock.calls.some(([c]) => c.startsWith("commit_"))).toBe(
    false,
  );
});

it("routes a bank restore review to the mapping commit endpoint only after confirmation", async () => {
  bridge.call.mockImplementation(async (command: string) => {
    if (command === "list_backups")
      return [
        {
          id: "base-backup",
          created_at: "local",
          device: "Test F75",
          kind: "hardware",
          mapping_kind: "base",
        },
      ];
    if (command === "prepare_mapping_restore")
      return { ...preparation, summary: "Restore known base source records" };
    if (command === "commit_mapping")
      return {
        message: "Readback matched",
        readback_verified: true,
        persistence: "not_tested",
      };
    return defaultCall(command);
  });
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: /Device Settings$/ }));
  const button = await screen.findByRole("button", {
    name: "Review bank restore",
  });
  await waitFor(() =>
    expect((button as HTMLButtonElement).disabled).toBe(false),
  );
  fireEvent.click(button);
  const dialog = await screen.findByRole("dialog");
  expect(bridge.call.mock.calls.some(([c]) => c.startsWith("commit_"))).toBe(
    false,
  );
  fireEvent.click(
    within(dialog).getByRole("button", {
      name: "Write bank restore to keyboard",
    }),
  );
  await waitFor(() =>
    expect(
      bridge.call.mock.calls.filter(([c]) => c === "commit_mapping"),
    ).toHaveLength(1),
  );
  expect(bridge.call.mock.calls.some(([c]) => c === "commit_lighting")).toBe(
    false,
  );
});

it("prepares brightness-only changes without selecting a new effect or committing", async () => {
  bridge.call.mockImplementation(async (command: string) => {
    if (command === "prepare_brightness")
      return {
        ...preparation,
        summary: "Change only the reported preset brightness",
        changes: [
          {
            field: "Preset brightness",
            before: 8,
            after: draft.lighting.brightness,
          },
        ],
      };
    return defaultCall(command);
  });
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: /Lighting$/ }));
  fireEvent.click(screen.getByText("Read / recover"));
  const button = screen.getByRole("button", { name: "Review brightness only" });
  await waitFor(() =>
    expect((button as HTMLButtonElement).disabled).toBe(false),
  );
  fireEvent.click(button);
  const dialog = await screen.findByRole("dialog");
  expect(
    bridge.call.mock.calls.find(([c]) => c === "prepare_brightness")?.[1],
  ).toEqual({ value: draft.lighting.brightness });
  expect(bridge.call.mock.calls.some(([c]) => c.startsWith("commit_"))).toBe(
    false,
  );
  fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
  expect(bridge.call.mock.calls.some(([c]) => c.startsWith("commit_"))).toBe(
    false,
  );
});
