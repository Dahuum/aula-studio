import { useEffect, useRef, useState } from "react";
import type { CSSProperties, ReactNode } from "react";
import type { Profile, RGB } from "./model";
import { hexRgb, lightingCapabilities, rgbHex } from "./model";

const presets:RGB[]=[[255,0,0],[255,128,0],[255,255,0],[0,255,0],[0,0,255],[0,255,255],[255,0,255],[255,255,255]];
export function wheelColor(x:number,y:number):RGB {
  const hue=(Math.atan2(y,x)*180/Math.PI+360)%360;
  const saturation=Math.min(1,Math.sqrt(x*x+y*y));
  const c=saturation,h=hue/60,z=c*(1-Math.abs(h%2-1)),m=1-c;
  const channels=h<1?[c,z,0]:h<2?[z,c,0]:h<3?[0,c,z]:h<4?[0,z,c]:h<5?[z,0,c]:[c,0,z];
  return channels.map(v=>Math.round((v+m)*255)) as RGB;
}
export function OriginalColors({profile,onChange,perKey,customColors,onAddColor,wheel,children}:{profile:Profile;onChange:(value:Profile["lighting"])=>void;perKey:boolean;customColors:RGB[];onAddColor:()=>void;wheel?:string;children?:ReactNode}) {
  const light=profile.lighting, capabilities=lightingCapabilities(light.effect),colorEnabled=perKey||(capabilities.color&&!light.colorful);
  const [hex,setHex]=useState(rgbHex(light.color));
  const spectrum=useRef<HTMLCanvasElement|null>(null);
  useEffect(()=>setHex(rgbHex(light.color)),[light.color]);
  useEffect(()=>{
    spectrum.current=null;
    if(!wheel)return;
    let active=true;const image=new Image();
    image.onload=()=>{if(!active)return;const canvas=document.createElement("canvas");canvas.width=image.naturalWidth;canvas.height=image.naturalHeight;canvas.getContext("2d")?.drawImage(image,0,0);spectrum.current=canvas;};
    image.src=wheel;return()=>{active=false;};
  },[wheel]);
  const color=(value:RGB)=>onChange({...light,color:value});
  const applyHex=(value:string)=>{if(/^#[0-9a-f]{6}$/i.test(value))color(hexRgb(value));else setHex(rgbHex(light.color));};
  return <section className={`original-color-panel ${perKey?"paint":""}`} aria-label="Lighting parameters">
    <div className="original-levels">
      <label>Brightness <span className="level-value">{light.brightness}</span><input aria-label="Brightness" type="range" min={0} max={9} value={light.brightness} style={{"--range-fill":`${light.brightness/9*100}%`} as CSSProperties} disabled={perKey||!capabilities.brightness} onChange={e=>onChange({...light,brightness:+e.target.value})}/></label>
      <label>Speed <span className="level-value">{light.speed}</span><input aria-label="Speed" type="range" min={0} max={4} value={light.speed} style={{"--range-fill":`${light.speed/4*100}%`} as CSSProperties} disabled={perKey||!capabilities.speed} onChange={e=>onChange({...light,speed:+e.target.value})}/></label>
      <div className="original-color-line"><label>Color <input aria-label="Lighting color" type="color" disabled={!colorEnabled} value={rgbHex(light.color)} onChange={e=>color(hexRgb(e.target.value))}/></label><label><input aria-label="Multicolor — use this effect’s built-in palette" type="checkbox" disabled={perKey||!capabilities.color} checked={light.colorful??false} onChange={e=>onChange({...light,colorful:e.target.checked})}/> Colourful</label></div>
    </div>
    <button className="original-wheel" aria-label="Choose hue and saturation" disabled={!colorEnabled} style={wheel?{backgroundImage:`url("${wheel}")`}:undefined} onClick={e=>{
      const b=e.currentTarget.getBoundingClientRect();if(!b.width||!b.height)return;
      const x=(e.clientX-b.left)/b.width,y=(e.clientY-b.top)/b.height,canvas=spectrum.current;
      if(canvas){const pixel=canvas.getContext("2d")?.getImageData(Math.min(canvas.width-1,Math.max(0,Math.floor(x*canvas.width))),Math.min(canvas.height-1,Math.max(0,Math.floor(y*canvas.height))),1,1).data;if(pixel&&pixel[3]>100)color([pixel[0],pixel[1],pixel[2]]);return;}
      color(wheelColor(2*x-1,2*y-1));
    }}/>
    <div className="original-rgb">
      <input className="original-hex" aria-label="Hex color" disabled={!colorEnabled} value={hex} maxLength={7} onChange={e=>setHex(e.target.value)} onBlur={e=>applyHex(e.currentTarget.value)} onKeyDown={e=>{if(e.key==="Enter")applyHex(e.currentTarget.value);}}/>
      {(["R","G","B"] as const).map((name,i)=><label key={name}>{name}<input aria-label={`${name} color channel`} type="range" min={0} max={255} disabled={!colorEnabled} value={light.color[i]} style={{"--channel-color":["#ff0000","#00ff00","#0000ff"][i]} as CSSProperties} onChange={e=>{const rgb=[...light.color] as RGB;rgb[i]=+e.target.value;color(rgb);}}/><input aria-label={`${name} value`} type="number" min={0} max={255} disabled={!colorEnabled} value={light.color[i]} onChange={e=>{const rgb=[...light.color] as RGB;rgb[i]=Math.min(255,Math.max(0,Math.round(+e.target.value)));color(rgb);}}/></label>)}
      <div className="original-preset-colors">{presets.map(rgb=><button key={rgb.join()} disabled={!colorEnabled} style={{background:rgbHex(rgb)}} aria-label={`Preset ${rgbHex(rgb)}`} onClick={()=>color(rgb)}/>)}</div>
    </div>
    <div className="original-custom-colors"><span>Custom color</span><div>{Array.from({length:10},(_,i)=>customColors[i]??([255,255,255] as RGB)).map((rgb,i)=><button key={i} style={{background:rgbHex(rgb)}} aria-label={`Custom color ${i+1}`} disabled={!colorEnabled} onClick={()=>color(rgb)}/>)}</div><button onClick={onAddColor} disabled={!colorEnabled||customColors.length>=10}>Add</button></div>
    {children}
  </section>;
}
