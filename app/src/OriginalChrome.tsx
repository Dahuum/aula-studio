import { useEffect, useState } from "react";
import type { CSSProperties } from "react";
import { call, native } from "./api";
import { effects } from "./model";

export type OriginalAssets = Record<string, string>;
export const originalEffectNames: Record<number, string> = {
  1:"Fixed_on",2:"Respire",3:"Rainbow",4:"Flash_away",5:"Raindrops",
  6:"Rainbow_wheel",7:"Ripples_shining",8:"Stars_twinkle",9:"Shadow_disappear",
  10:"Retro_snake",11:"Neon_stream",12:"Reaction",13:"Sine_wave",14:"Retinue scanning",
  15:"Rotating windmill",16:"Colorful waterfall",17:"Blossoming",18:"Rotating storm",21:"Self-define",0:"OFF",
};
export function useOriginalFrame() {
  const [assets,setAssets]=useState<OriginalAssets>({});
  const [size,setSize]=useState({width:window.innerWidth,height:window.innerHeight});
  useEffect(()=>{
    if(native) void call<{assets:OriginalAssets}>("local_ui_assets").then(v=>setAssets(v.assets ?? {})).catch(()=>{});
    const resize=()=>setSize({width:window.innerWidth,height:window.innerHeight});
    window.addEventListener("resize",resize);return()=>window.removeEventListener("resize",resize);
  },[]);
  const scale=Math.min(size.width/1200,size.height/720);
  const style={transform:`scale(${scale})`,left:Math.max(0,(size.width-1200*scale)/2),top:Math.max(0,(size.height-720*scale)/2),
    ...(assets.background?{"--original-background":`url("${assets.background}")`}:{}),
    ...Object.fromEntries([['slider','--original-slider'],['rgbSlider','--original-rgb-slider'],['checkOff','--original-check-off'],['checkOn','--original-check-on']].filter(([key])=>assets[key]).map(([key,property])=>[property,`url("${assets[key]}")`]))} as CSSProperties;
  return {assets,style};
}
export function UiIcon({kind}:{kind:string}) {
  const shapes:Record<string,string>={
    settings:"M4 7h16M4 17h16M8 4v6m8 4v6",
    keyboard:"M3 6h18v12H3zM6 9h1m3 0h1m3 0h1m3 0h1M6 12h1m3 0h1m3 0h1m3 0h1M7 15h10",
    light:"M9 18h6m-5 3h4M8 12a6 6 0 1 1 8 0l-1 3H9zM12 1v1M2 9h2m16 0h2M4 3l2 2m12 0 2-2",
    game:"M18 6a8 8 0 1 0 2 5M16 8a5 5 0 1 0 1 4M12 12l8-8m-4 0h4v4",
    macro:"M3 7h7l2-2h9v14H3zM10 10l-3 3 3 3m4-6 3 3-3 3",
    save:"M4 3h13l3 3v15H4zM8 3v6h8V3M8 21v-8h8v8",
    home:"m3 11 9-8 9 8M6 10v11h12V10M10 21v-6h4v6",
    close:"m6 6 12 12M18 6 6 18",minimize:"M5 16h14",more:"M4 12h2m5 0h2m5 0h2",
  };
  return <svg viewBox="0 0 24 24" aria-hidden="true"><path d={shapes[kind]??shapes.keyboard}/></svg>;
}
export function OriginalTop({assets,dirty,busy,mode,connected,onNavigate,onClose}:{assets:OriginalAssets;dirty:boolean;busy:boolean;mode:string;connected:boolean;onNavigate:(page:string)=>void;onClose:()=>void}) {
  return <div className="original-top">
    <div className={`original-brand ${assets.background?"with-original-art":""}`} aria-label="AULA F75">{!assets.background&&<span>AULA</span>}</div>
    <div className="original-drag" onMouseDown={e=>{if(native&&e.button===0)void call("window_action",{action:"drag"}).catch(()=>{});}}/>
    <button className="original-device" title="Device overview" aria-label="Dashboard" onClick={()=>onNavigate("Dashboard")}><UiIcon kind="keyboard"/><span>F75</span></button>
    <span className="original-state">{mode === "demo" ? "DEMO" : connected?"USB":"Disconnected"} · {dirty?"Unsaved changes":"Saved locally"}</span>
    <button className="original-config" aria-label="Profiles" onClick={()=>onNavigate("Profiles")}>Config</button>
    <button className="original-minimize" aria-label="Minimize" disabled={!native} onClick={()=>void call("window_action",{action:"minimize"}).catch(()=>{})}><UiIcon kind="minimize"/></button>
    <button className="original-close" aria-label="Close application" disabled={!native||busy} onClick={onClose}><UiIcon kind="close"/></button>
  </div>;
}
export function OriginalNavigation({page,effect,assets,onNavigate,onEffect,demo,connected,busy,toggleDemo,inert}:{page:string;effect:number;assets:OriginalAssets;onNavigate:(page:string)=>void;onEffect:(id:number)=>void;demo:boolean;connected:boolean;busy:boolean;toggleDemo:()=>void;inert:boolean}) {
  const [more,setMore]=useState(false);
  const lighting=page==="Lighting"||page==="Per-Key RGB";
  const mapping=page==="Key Remapping"||page==="Fn Layers";
  const entries=[{page:"Device Settings",icon:"settings",id:1},{page:"Key Remapping",icon:"keyboard",id:2},
    {page:"Lighting",icon:"light",id:3},{page:"Per-Key RGB",icon:"game",id:5},{page:"Macros",icon:"macro",id:4}];
  return <>
    <aside className="original-rail" inert={inert}>
      <nav aria-label="Main sections">{entries.map(item=>{
        const active=page===item.page||(item.page==="Key Remapping"&&mapping);
        const image=assets[`nav${item.id}_${active?"dn":"nr"}`];
        return <button key={item.page} className={active?"active":""} aria-label={item.page} aria-current={active?"page":undefined} title={item.page} onClick={()=>onNavigate(item.page)}>
          {image?<img src={image} alt=""/>:<UiIcon kind={item.icon}/>}<span className="sr-only">{item.page}</span>
        </button>;
      })}</nav>
      <button className="original-more" title="Linux tools" aria-label="More views" aria-expanded={more} onClick={()=>setMore(!more)}><UiIcon kind="more"/></button>
      {more&&<div className="original-extra-views">{["Models","Diagnostics","About"].map(p=><button key={p} aria-label={p==="Models"?"Device library":p} onClick={()=>{onNavigate(p);setMore(false);}}>{p==="Models"?"Device library":p}</button>)}</div>}
    </aside>
    <section className="original-context" inert={inert} aria-label="Section options">
      <h2>{lighting?"Light effect":mapping?"Key setting":page==="Macros"?"Macro":page==="Profiles"?"Config":page==="Device Settings"?"Global":page}<span aria-hidden="true">»</span></h2>
      {lighting&&<div className="original-effects">{[...effects.filter(e=>e.id!==0),...effects.filter(e=>e.id===0)].map(e=><button key={e.id} className={effect===e.id?"active":""} aria-pressed={effect===e.id} aria-label={e.name} onClick={()=>onEffect(e.id)}>{originalEffectNames[e.id]}</button>)}</div>}
      {mapping&&<p className="original-context-hint">Select a key to edit its assignment.<br/><small>Changes stay local until you confirm a hardware write.</small></p>}
      {page==="Macros"&&<div id="original-macro-list"/>}
      {!["Lighting","Per-Key RGB","Key Remapping","Fn Layers","Macros"].includes(page)&&<p className="original-context-hint">AULA F75<br/><small>Native Linux configuration</small></p>}
      <div className="original-context-bottom"><span>{demo?"Demo device":connected?"Wired USB":"Disconnected"}</span><button disabled={busy} onClick={toggleDemo}>{demo?"Exit demo mode":"Enter demo mode"}</button></div>
    </section>
  </>;
}
