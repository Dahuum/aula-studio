#!/usr/bin/env python3
"""Native WebKit smoke test: read/prepare/cancel only, physical writes blocked.
Requires existing WebKitWebDriver and xvfb-run. Artifacts stay under analysis/.
Never call commit_* here. Do not count this test as physical write validation.
"""
import base64,json,os,signal,subprocess,time,urllib.request,urllib.error
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
binary=ROOT/'target/release/aula-studio'
inputs=list((ROOT/'crates').rglob('*.rs'))+list((ROOT/'app/src-tauri').rglob('*.rs'))+list((ROOT/'app/dist').rglob('*'))
latest=max(p.stat().st_mtime for p in inputs if p.is_file())
if not binary.exists() or binary.stat().st_mtime<latest:raise SystemExit('Native binary is stale; complete the release build before running this test')
OUT=ROOT/'analysis/native-review-smoke';OUT.mkdir(parents=True,exist_ok=True)
DATA=OUT/'data';(DATA/'profiles').mkdir(parents=True,exist_ok=True)
profile={'schema_version':1,'id':'native-qa','name':'Native review QA','lighting':{'effect':1,'brightness':7,'speed':2,'color':[11,22,33],'direction':0,'colorful':False},'per_key':{'4':[123,50,70],'fn':[47,85,123]},'keymap':{'4':5},'fn_layer':{'20':104},'macros':[],'updated_at':'QA local draft only','sleep_seconds':600}
devices=json.loads(subprocess.check_output([str(ROOT/'target/release/aula'),'devices'],text=True))
paths=sorted({d['path'] for d in devices if d['read_supported']})
if len(paths)!=1:raise SystemExit('Connect exactly one supported wired F75 for native QA')
initial=json.loads(subprocess.check_output([str(ROOT/'target/release/aula'),'read',paths[0]],text=True))
profile['lighting']['effect']=2 if initial['config'][10]==1 else 1
profile['sleep_seconds']=300 if initial['config'][24]==20 else 600
base=json.loads(subprocess.check_output([str(ROOT/'target/release/aula'),'read-layer',paths[0],'base'],text=True))['keymap']
fn=json.loads(subprocess.check_output([str(ROOT/'target/release/aula'),'read-layer',paths[0],'fn'],text=True))['keymap']
leds=json.loads(subprocess.check_output([str(ROOT/'target/release/aula'),'read-per-key',paths[0]],text=True))['per_key']
profile['keymap']['4']=6 if base[47]==5 else 5
profile['fn_layer']['20']=105 if fn[43]==104 else 104
profile['per_key']['4']=[leds[9]^255,leds[126+9],leds[252+9]]
profile['per_key']['fn']=[leds[53]^255,leds[126+53],leds[252+53]]
(DATA/'profiles/native-qa.json').write_text(json.dumps(profile));(DATA/'profiles/native-qa.json').chmod(0o600)
# Guard is inherited by the native application launched by WebKitWebDriver.
env=dict(os.environ,AULA_DATA_DIR=str(DATA),AULA_DISABLE_HARDWARE_WRITES='1',TAURI_WEBVIEW_AUTOMATION='true',GDK_BACKEND='x11', WEBKIT_DISABLE_COMPOSITING_MODE='1')
log=(OUT/'driver.log').open('w')
process=subprocess.Popen(['xvfb-run','-a','WebKitWebDriver','--host=127.0.0.1','--port=4445'],env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
sid=None

def request(method,path,data=None):
 raw=None if data is None else json.dumps(data).encode();req=urllib.request.Request('http://127.0.0.1:4445'+path,data=raw,method=method,headers={'Content-Type':'application/json'})
 try:
  with urllib.request.urlopen(req,timeout=30)as r:response=json.load(r)
 except urllib.error.HTTPError as e:raise RuntimeError(e.read().decode()) from e
 v=response.get('value',response)
 if isinstance(v,dict)and v.get('error') and 'ok' not in v:raise RuntimeError(str(v))
 return v

def execute(script,args=None):return request('POST',f'/session/{sid}/execute/sync',{'script':script,'args':args or []})
def invoke(command,args=None):
 assert not command.startswith('commit_'),'Smoke test must never commit physical writes'
 return request('POST',f'/session/{sid}/execute/async',{'script':"const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke(arguments[0],arguments[1]).then(v=>done({ok:true,value:v}),e=>done({ok:false,error:String(e)}));",'args':[command,args or {}]})
def ok(command,args=None):
 r=invoke(command,args)
 if not r['ok']:raise RuntimeError(f'{command}: {r["error"]}')
 return r['value']
def element(text):
 return request('POST',f'/session/{sid}/element',{'using':'xpath','value':f'//button[@aria-label="{text}" or normalize-space()="{text}"] | //summary[normalize-space()="{text}"]'})['element-6066-11e4-a52e-4f735466cecf']
def click(text):request('POST',f'/session/{sid}/element/{element(text)}/click',{})
try:
 for _ in range(80):
  try:request('GET','/status');break
  except Exception:time.sleep(.2)
 session=request('POST','/session',{'capabilities':{'alwaysMatch':{'browserName':'wry','webkitgtk:browserOptions':{'binary':str(ROOT/'target/release/aula-studio'),'args':[]}}}});sid=session['sessionId']
 request('POST',f'/session/{sid}/timeouts',{'script':15000})
 for _ in range(80):
  try:
   if execute('return !!document.querySelector(".original-app")'):break
  except Exception:pass
  time.sleep(.2)
 assert ok('diagnostics')['safety']['hardware_writes_enabled'] is False,'Native write guard is not reflected in diagnostics'
 bounds=execute('const r=document.querySelector(".original-app").getBoundingClientRect();return r.right<=innerWidth+1&&r.bottom<=innerHeight+1')
 assert bounds,'Scaled original canvas must fit the native window'
 status=ok('get_status');assert status['mode']=='hardware' and status['connected'],'No physical wired keyboard for this test'
 before=ok('read_configuration');assert len(before['config'])==128 and len(before['colors'])==512
 banks_before={kind:ok('read_mapping',{'kind':kind}) for kind in ['base','fn','per_key']}
 # Exercise actual React -> Tauri prepare and then cancellation, never commit.
 for _ in range(60):
  try:click('Review lighting write');break
  except Exception:time.sleep(.2)
 for _ in range(60):
  text=execute('return document.body.innerText')
  if 'Review lighting changes' in text:break
  time.sleep(.2)
 assert 'Review lighting changes' in text,text
 screenshot=request('GET',f'/session/{sid}/screenshot');(OUT/'review.png').write_bytes(base64.b64decode(screenshot))
 click('Cancel')
 reviews={}
 # Separate native Rust command paths, all read/prepare only.
 for kind in ['base','fn','per_key']:
  review=ok('prepare_mapping',{'profile':profile,'kind':kind});assert review['token']and review['backup_id']and review['changes'];reviews[kind]=review
 settings=ok('prepare_settings',{'profile':profile});assert settings['changes'];reviews['settings']=settings
 for kind in ['base','fn','per_key']:
  recovery=invoke('prepare_mapping_restore',{'id':reviews[kind]['backup_id']});assert not recovery['ok'] and 'already match' in recovery['error'],recovery
 lighting_recovery=invoke('prepare_restore',{'id':reviews['settings']['backup_id']});assert not lighting_recovery['ok'] and ('custom per-key mode' if before['config'][10]==21 else 'already match') in lighting_recovery['error'],lighting_recovery
 after=ok('read_configuration');assert before['config']==after['config']and before['colors']==after['colors'],'Unexpected settings change during read-only QA'
 for kind,bank in banks_before.items():
  after_bank=ok('read_mapping',{'kind':kind});key='per_key' if kind=='per_key' else 'keymap';assert bank[key]==after_bank[key],f'{kind} bank changed during read-only QA'
 assert not list((DATA/'operations').glob('*.json')),'A commit journal was unexpectedly created'
 result={'status':'passed','checks':['native React review/cancel','scaled original canvas bounds','actual wired config read','native base/Fn/per-key prepare','native sleep prepare','durable backups','native bank/lighting restore no-op validation','config/palette/base/Fn/per-key unchanged','no commit journal'],'physical_writes':False,'write_guard':'AULA_DISABLE_HARDWARE_WRITES=1','reviews':reviews}
 (OUT/'result.json').write_text(json.dumps(result,indent=2));print(json.dumps({k:v for k,v in result.items()if k!='reviews'},indent=2))
finally:
 if sid:
  try:request('DELETE',f'/session/{sid}')
  except Exception:pass
 try:os.killpg(process.pid,signal.SIGTERM)
 except ProcessLookupError:pass
 process.wait(timeout=15);log.close()
