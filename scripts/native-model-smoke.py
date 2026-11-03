#!/usr/bin/env python3
"""Exercise multi-model workspaces in the real native GUI/backend with isolated storage and blocked writes.

Requires WebKitWebDriver and xvfb-run. Does not require a keyboard and never
calls commit commands. All generated files stay in analysis/native-model-smoke.
"""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import time
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', type=Path, help='Validate this packaged native executable instead of target/release')
parser.add_argument('--xdg', action='store_true', help='Exercise standard XDG config/data paths under the isolated test directory')
parser.add_argument('--dev', action='store_true', help='Use the locally built debug binary with the loopback Vite development server')
parser.add_argument('--fallback', action='store_true', help='Disable private original artwork for standalone rendering validation')
options = parser.parse_args()
BINARY = options.binary.resolve() if options.binary else ROOT / ('target/debug/aula-studio' if options.dev else 'target/release/aula-studio')
inputs = list((ROOT / 'crates').rglob('*.rs')) + list((ROOT / 'app/src-tauri').rglob('*.rs')) + ([] if options.dev else list((ROOT / 'app/dist').rglob('*')))
if not BINARY.exists() or (not options.binary and BINARY.stat().st_mtime < max(p.stat().st_mtime for p in inputs if p.is_file())):
    raise SystemExit('Build the current frontend and release binary first')
if options.binary and not BINARY.read_bytes().startswith(b'\x7fELF'):
    raise SystemExit('Expected a trusted native Linux package executable')
OUT = ROOT / 'analysis/native-model-smoke' / str(time.time_ns())
OUT.mkdir(parents=True)
DATA = OUT / 'data'
env = dict(os.environ, AULA_DATA_DIR=str(DATA), AULA_DISABLE_HARDWARE_WRITES='1',
           TAURI_WEBVIEW_AUTOMATION='true', GDK_BACKEND='x11', WEBKIT_DISABLE_COMPOSITING_MODE='1')
if options.fallback:
    env['AULA_NO_ORIGINAL_SKIN'] = '1'
# Keep browser exports and desktop preferences inside this test directory.
DOWNLOADS=OUT/'downloads';DOWNLOADS.mkdir()
EXPORT_CONFIG=OUT/'export-config';EXPORT_CONFIG.mkdir()
(EXPORT_CONFIG/'user-dirs.dirs').write_text('XDG_DOWNLOAD_DIR="'+str(DOWNLOADS)+'"\n')
env['XDG_CONFIG_HOME']=str(EXPORT_CONFIG)
env['XDG_DOWNLOAD_DIR']=str(DOWNLOADS)

CONFIG = DATA
if options.xdg:
    env.pop('AULA_DATA_DIR', None)
    env['XDG_DATA_HOME'] = str(OUT / 'xdg-data')
    env['XDG_CONFIG_HOME'] = str(OUT / 'xdg-config')
    DATA = OUT / 'xdg-data/aula-studio'
    CONFIG = OUT / 'xdg-config/aula-studio'
log = (OUT / 'driver.log').open('w')
process = subprocess.Popen(['xvfb-run', '-a', 'WebKitWebDriver', '--host=127.0.0.1', '--port=4449'],
                           env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
sid = None


def request(method, path, data=None):
    raw = None if data is None else json.dumps(data).encode()
    req = urllib.request.Request('http://127.0.0.1:4449' + path, data=raw, method=method,
                                 headers={'Content-Type': 'application/json'})
    try:
        with urllib.request.urlopen(req, timeout=30) as response:
            value = json.load(response)['value']
    except urllib.error.HTTPError as error:
        raise RuntimeError(error.read().decode()) from error
    if isinstance(value, dict) and value.get('error') and 'ok' not in value:
        raise RuntimeError(str(value))
    return value


def execute(script, args=None):
    return request('POST', f'/session/{sid}/execute/sync', {'script': script, 'args': args or []})


def invoke(command, args=None):
    assert not command.startswith('commit_'), 'Physical commits are forbidden in this test'
    return request('POST', f'/session/{sid}/execute/async', {
        'script': 'const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke(arguments[0],arguments[1]).then(value=>done({ok:true,value}),error=>done({ok:false,error:String(error)}));',
        'args': [command, args or {}]})


def ok(command, args=None):
    result = invoke(command, args)
    assert result['ok'], result
    return result['value']


def wait(check):
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        try:
            value = check()
            if value:
                return value
        except Exception:
            pass
        time.sleep(.2)
    raise AssertionError('Timed out waiting for native UI state')


def click(label):
    button = request('POST', f'/session/{sid}/element', {
        'using': 'xpath', 'value': f'//button[@aria-label="{label}" or normalize-space()="{label}"] | //summary[normalize-space()="{label}"]'})['element-6066-11e4-a52e-4f735466cecf']
    execute('arguments[0].scrollIntoView({block: "center", inline: "nearest"})', [{'element-6066-11e4-a52e-4f735466cecf':button}])
    request('POST', f'/session/{sid}/element/{button}/click', {})


def text_input(xpath, text):
    field = request('POST', f'/session/{sid}/element', {'using': 'xpath', 'value': xpath})['element-6066-11e4-a52e-4f735466cecf']
    request('POST', f'/session/{sid}/element/{field}/clear', {})
    request('POST', f'/session/{sid}/element/{field}/value', {'text': text, 'value': list(text)})


def arrow(label, key):
    field = request('POST', f'/session/{sid}/element', {'using': 'xpath', 'value': f'//input[@aria-label="{label}"]'})['element-6066-11e4-a52e-4f735466cecf']
    request('POST', f'/session/{sid}/element/{field}/value', {'text': key, 'value': [key]})


def screenshot(name):
    request('POST', f'/session/{sid}/execute/async', {
        'script': 'const done=arguments[arguments.length-1];requestAnimationFrame(()=>requestAnimationFrame(()=>done(true)));', 'args': []})
    (OUT / name).write_bytes(base64.b64decode(request('GET', f'/session/{sid}/screenshot')))


def select_option(label, value):
    option = request('POST', f'/session/{sid}/element', {
        'using': 'xpath', 'value': f'//select[@aria-label="{label}"]/option[@value="{value}"]'})['element-6066-11e4-a52e-4f735466cecf']
    request('POST', f'/session/{sid}/element/{option}/click', {})


def navigate(page):
    if page == 'Fn Layers':
        click('Key Remapping')
    if page in ['Diagnostics', 'About']:
        click('More views')
    click(page)
    wait(lambda: execute('return document.querySelector("h1").textContent') == page)


try:
    wait(lambda: request('GET', '/status'))
    session = request('POST', '/session', {'capabilities': {'alwaysMatch': {
        'browserName': 'wry', 'webkitgtk:browserOptions': {'binary': str(BINARY), 'args': []}}}})
    sid = session['sessionId']
    wait(lambda: execute('return !!document.querySelector(".original-app")'))
    assert ok('diagnostics')['safety']['hardware_writes_enabled'] is False
    models=ok('list_models')
    assert len(models)==14
    assert all(d['hardware_access_enabled'] is False for d in ok('model_devices'))
    click('More views');click('Device library')
    wait(lambda: execute('return document.querySelectorAll(".models-keyboard button").length')==87)
    assert execute('const a=document.querySelector(".models-sidebar").getBoundingClientRect(),m=document.querySelector(".models-main").getBoundingClientRect(),h=document.querySelector(".models-top").getBoundingClientRect();return a.top>=h.bottom-1&&m.left>=a.right-1'), 'Workspace overlaps the legacy shell'
    checked=[]
    for model in models:
        if model['id']=='f75-ansi': continue
        click(model['name'])
        expected=len(model['keys'])
        wait(lambda: execute('return document.querySelector(".models-heading h2").textContent')==model['name'])
        if not expected:
            wait(lambda: execute('return !document.querySelector(".models-keyboard")'))
            assert not execute('return !!document.querySelector(arguments[0])', ['input[aria-label="Model brightness"]'])
            continue
        wait(lambda: execute('return document.querySelectorAll(".models-keyboard button").length')==expected)
        wait(lambda: execute('return ![...document.querySelectorAll("button")].find(b=>b.textContent.trim()==="Save local profile").disabled'))
        text_input('//input[@aria-label="Model profile name"]',model['name']+' native QA')
        click('Save local profile')
        wait(lambda: any(p['name']==model['name']+' native QA' for p in ok('list_model_profiles')))
        assert execute('return !document.body.innerText.includes("Unapplied local edits")')
        checked.append({'model':model['id'],'resource_keys':expected,'local_profile_saved':True})
    click(next(m['name'] for m in models if m['id']=='s75-pro'))
    wait(lambda: execute('return document.querySelectorAll(".models-keyboard button").length')==80)
    s75=next(m for m in models if m['id']=='s75-pro')
    key=next(k for k in s75['keys'] if k['usage']==4 and not k['protected'])
    click(key['label']+' · '+key['id']);click('Color selected keys locally')
    click('Fn layer');select_option('Model target action','5');click('Assign locally')
    click('Local macros');click('Create macro');click('Start recording')
    area=request('POST',f'/session/{sid}/element',{'using':'xpath','value':'//*[@aria-label="Macro recording area"]'})['element-6066-11e4-a52e-4f735466cecf']
    request('POST',f'/session/{sid}/element/{area}/value',{'text':'a','value':['a']});click('Stop recording')
    click('Save local profile')
    wait(lambda: any(p['model_id']=='s75-pro' and len(p['macros'])==1 for p in ok('list_model_profiles')))
    saved=next(p for p in ok('list_model_profiles') if p['model_id']=='s75-pro')
    assert saved['fn_assignments'][key['id']]=={'kind':'keyboard','usage':5}
    assert saved['per_key'][key['id']]==saved['lighting']['color']
    assert [e for e in saved['macros'][0]['events'] if e['kind']!='delay']==[{'kind':'down','key':4},{'kind':'up','key':4}]
    screenshot('s75-macros.png')
    click('Export draft');wait(lambda: execute('return !!document.querySelector(".models-modal")'))
    exported=json.loads(execute('return document.querySelector(".models-modal textarea").value'))
    assert exported==saved
    click('Download JSON')
    wait(lambda: (DOWNLOADS/'aula-s75-pro.json').is_file())
    assert json.loads((DOWNLOADS/'aula-s75-pro.json').read_text())==saved
    click('Close')
    malformed=dict(saved,lighting=dict(saved['lighting'],brightness=6))
    assert not invoke('import_model_profile',{'json':json.dumps(malformed)})['ok']
    assert not invoke('import_profile',{'json':json.dumps(saved)})['ok'], 'Model draft entered F75 namespace'
    imported=ok('import_model_profile',{'json':json.dumps(saved)})
    assert imported['id']!=saved['id']
    ok('delete_model_profile',{'id':imported['id']})
    assert list((DATA/'model-profiles/history').glob('*.json'))
    assert not ok('list_profiles'), 'Model profiles leaked to F75 storage'
    assert not list((DATA/'operations').glob('*.json'))
    assert not list((DATA/'backups').glob('*.json'))
    for path in (DATA/'model-profiles').glob('*.json'):
        assert path.stat().st_mode&0o777==0o600
    click('Lighting');screenshot('s75-lighting.png')
    click('← F75 controls');wait(lambda: execute('return !!document.querySelector(".original-app")'))
    result={'status':'passed','binary_sha256':hashlib.sha256(BINARY.read_bytes()).hexdigest(),'physical_writes':False,
            'models':checked,'checks':['all recovered layouts in native WebKit','research-only entries have no controls',
            'native model profile save and clean state','per-key and Fn draft serialization','focused native macro recording',
            'draft export/import/delete/history and native file download','malformed imports rejected','legacy F75 namespace isolation','private file modes',
            'no physical journal or backup','return to existing F75 controller']}
    (OUT/'result.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2))

except Exception:
    if sid:
        try:
            screenshot('failure.png')
            (OUT / 'failure-state.json').write_text(json.dumps(execute('return {text:document.body.innerText,inputs:[...document.querySelectorAll("input")].map(e=>({label:e.getAttribute("aria-label"),value:e.value,disabled:e.disabled}))}'), indent=2))
            print('Failure evidence:', OUT, flush=True)
        except Exception:
            pass
    raise
finally:
    if sid:
        try:
            request('DELETE', f'/session/{sid}')
        except Exception:
            pass
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    process.wait(timeout=15)
    log.close()
