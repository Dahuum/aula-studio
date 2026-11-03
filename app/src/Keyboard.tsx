import type { CSSProperties } from "react";
import { KEYBOARD } from "./layout";
import { effects, rgbHex, mediaActions } from "./model";
import type { Profile } from "./model";
export function Keyboard({
  profile,
  selected = [],
  onSelect,
  highlight,
  showMapping = false,
  fnLayer = false,
}: {
  profile: Profile;
  selected?: string[];
  onSelect?: (id: string, multi: boolean) => void;
  highlight?: number;
  showMapping?: boolean;
  fnLayer?: boolean;
}) {
  const mapping = fnLayer ? profile.fn_layer : profile.keymap;
  const mediaMapping = (fnLayer ? profile.consumer_fn_layer : profile.consumer_keymap) ?? {};
  const pattern =
    effects.find((effect) => effect.id === profile.lighting.effect)?.pattern ??
    "solid";
  return (
    <div className="keyboard-scroll">
      <div
        className={`keyboard pattern-${pattern}`}
        style={
          {
            "--preview-brightness": profile.lighting.brightness / 9,
            "--preview-duration": `${6 - profile.lighting.speed}s`,
          } as CSSProperties
        }
        role="group"
        aria-label="AULA F75 ANSI 80-key layout"
      >
        <div className="keyboard-cable" aria-hidden="true" />
        <div className="keyboard-surface" />
        <div
          className="knob"
          title="Physical rotary knob · software configuration unverified"
        >
          <i />
        </div>
        {KEYBOARD.map((key, index) => {
          const color =
            profile.lighting.effect === 21
              ? rgbHex(profile.per_key[key.id] ?? profile.lighting.color)
              : profile.lighting.colorful
                ? `hsl(${Math.round(key.x * 22 + key.y * 15)} 85% 70%)`
                : rgbHex(profile.lighting.color);
          const style = {
            "--key-color": color,
            left: `${(/^F\d+$/.test(key.label) ? [99,139,179,219,272,312,352,392,445,485,525,565][+key.label.slice(1)-1] : 21+key.x*39.5)/679*100}%`,
            top: `${(30+key.y*41)/304*100}%`,
            width: `${(key.width*39.5-2)/679*100}%`,
            height: `${40/304*100}%`,
            "--key-index": index,
          } as CSSProperties;
          const mapped = mapping[key.id] !== undefined || mediaMapping[key.id] !== undefined;
          const className = `key ${selected.includes(key.id) ? "selected" : ""} ${highlight === key.usage ? "pressed" : ""} ${profile.lighting.effect === 0 ? "unlit" : ""} ${showMapping && mapped ? "mapped" : ""}`;
          const content = (
            <>
              <span>{key.label}</span>
              {showMapping && mapped && (
                <small>
                  →{" "}
                  {mediaMapping[key.id] !== undefined ? mediaActions.find(a => a.usage === mediaMapping[key.id])?.short ?? "Media" :
                    KEYBOARD.find((k) => k.usage === mapping[key.id])?.label ?? mapping[key.id]}
                </small>
              )}
              <i />
            </>
          );
          return onSelect ? (
            <button
              type="button"
              key={key.id}
              style={style}
              className={className}
              aria-label={`${key.label}${key.code.includes("Right") ? " right" : ""}`}
              aria-pressed={selected.includes(key.id)}
              onClick={(e) =>
                onSelect(key.id, e.shiftKey || e.ctrlKey || e.metaKey)
              }
            >
              {content}
            </button>
          ) : (
            <div key={key.id} style={style} className={className}>
              {content}
            </div>
          );
        })}
      </div>
      <div className="keyboard-caption">
        <span>F75 · ANSI · 80 keys + rotary knob</span>
        <span>
          {onSelect
            ? "Shift-click to select multiple keys"
            : "Local configuration preview"}
        </span>
      </div>
    </div>
  );
}
