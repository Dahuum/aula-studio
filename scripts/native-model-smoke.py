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
parser.add_argument('--no-screenshots', action='store_true', help='Skip image capture when the container WebKit snapshot endpoint hangs; retain every workflow assertion')
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
    # Native actions briefly disable controls. WebDriver silently ignores
    # disabled clicks; wait for the real enabled state first.
    wait(lambda: request('GET', f'/session/{sid}/element/{button}/enabled'))
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
    if options.no_screenshots:
        return
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
    click('Enter demo mode')
    wait(lambda: ok('get_status')['mode']=='demo')
    models=ok('list_models')
    assert len(models)==14
    assert all(d['hardware_writes_enabled'] is False for d in ok('model_devices'))
    click('More views');click('Device library')
    wait(lambda: execute('return document.querySelectorAll(".models-keyboard button").length')==87)
    assert execute('const a=document.querySelector(".models-sidebar").getBoundingClientRect(),m=document.querySelector(".models-main").getBoundingClientRect(),h=document.querySelector(".models-top").getBoundingClientRect();return a.top>=h.bottom-1&&m.left>=a.right-1'), 'Workspace overlaps the legacy shell'
    # The actual Rust fixture pipeline exercises decoding/storage/projection;
    # GUI demo mode rejects physical capture before device enumeration/open.
    wait(lambda: execute('return ![...document.querySelectorAll("button")].find(b=>b.textContent.trim()==="Generate synthetic packet fixtures").disabled'))
    assert execute('return [...document.querySelectorAll("button")].find(b=>b.textContent.trim()==="Read device (experimental)").disabled')
    blocked=invoke('read_model_capture',{'modelId':'f87','path':'/dev/hidraw-no-such-device'})
    assert not blocked['ok'] and 'demo' in blocked['error'].lower(),blocked
    click('Generate synthetic packet fixtures')
    wait(lambda: len(ok('list_model_captures',{'modelId':'f87'}))==1)
    captured=ok('list_model_captures',{'modelId':'f87'})[0]
    assert captured['source']=='simulated' and len(captured['per_key'])==384
    assert captured['per_key'][126+9]==246 and captured['per_key'][252+9]==27
    assert captured['per_key'][378:]==[222,173,190,239,18,52]
    wait(lambda: execute('return !!document.querySelector(arguments[0])', ['input[aria-label="Capture lighting draft name"]']))
    click('Export raw capture JSON');wait(lambda: execute('return !!document.querySelector(".models-modal")'))
    assert json.loads(execute('return document.querySelector(".models-modal textarea").value'))==captured
    click('Download JSON');wait(lambda: (DOWNLOADS/'aula-f87.json').is_file())
    assert json.loads((DOWNLOADS/'aula-f87.json').read_text())==captured
    click('Close');click('Create local lighting-only draft')
    wait(lambda: execute('return document.body.innerText.includes("Created a local lighting-only draft")'))
    click('Save local profile');wait(lambda: len(ok('list_model_profiles'))==1)
    projected=ok('list_model_profiles')[0]
    assert projected['lighting']['color']==[32,64,128]
    assert projected['per_key']['key-50']==[18,246,27]
    assert not projected['assignments'] and not projected['fn_assignments'] and not projected['macros']
    assert not invoke('import_model_profile',{'json':json.dumps(captured)})['ok']
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
        if model['id'] in ['f108', 'h108']:
            # Actual native Rust fixture/decoder/storage pipeline, no mocked IPC.
            wait(lambda: execute('return ![...document.querySelectorAll("button")].find(b=>b.textContent.trim()==="Generate synthetic packet fixtures").disabled'))
            click('Generate synthetic packet fixtures')
            wait(lambda: len(ok('list_model_captures', {'modelId': model['id']})) == 1)
            standard = ok('list_model_captures', {'modelId': model['id']})[0]
            assert standard['source'] == 'simulated' and len(standard['per_key']) == 378
            assert standard['config'][126:] == [0x5a, 0xa5]
            assert standard['identity'] == model['model_identity']
            assert execute('return [...document.querySelectorAll("button")].find(b=>b.textContent.trim()==="Read device (experimental)").disabled')
            wait(lambda: execute('return !!document.querySelector(arguments[0])', ['input[aria-label="Capture lighting draft name"]']))
            click('Create local lighting-only draft')
            wait(lambda: execute('return document.body.innerText.includes("Created a local lighting-only draft")'))
            projected_id = execute('return document.querySelector(arguments[0]).value', ['input[aria-label="Model profile name"]'])
            assert projected_id and not any(p['model_id'] == model['id'] for p in ok('list_model_profiles'))
        if model['id'] == 'f98-pro':
            assert len(model['lighting']['effects']) == 20
            assert execute('return document.querySelector(arguments[0]).max', ['input[aria-label="Model brightness"]']) == '5'
            assert execute('return document.querySelector(arguments[0]).disabled', ['input[aria-label="Model speed"]'])
            for effect in ['6', '8']:
                select_option('Model lighting mode', effect)
                assert execute('return document.querySelector(arguments[0]).disabled', ['input[aria-label="Model draft color"]'])
                assert not execute('return document.querySelector(arguments[0]).disabled', ['input[aria-label="Model speed"]'])
            select_option('Model lighting mode', '0')
            for label in ['Model brightness', 'Model speed', 'Model draft color']:
                assert execute('return document.querySelector(arguments[0]).disabled', ['input[aria-label="'+label+'"]'])
            select_option('Model lighting mode', '7')
            key = next(k for k in model['keys'] if k['usage'] == 4)
            click(key['label'] + ' · ' + key['id'])
            click('Color selected keys locally')
            assert not execute('return [...document.querySelectorAll("button")].some(b=>b.textContent.trim()==="Read device (experimental)")')
        text_input('//input[@aria-label="Model profile name"]',model['name']+' native QA')
        click('Save local profile')
        wait(lambda: any(p['name']==model['name']+' native QA' for p in ok('list_model_profiles')))
        assert execute('return !document.body.innerText.includes("Unapplied local edits")')
        if model['id'] in ['f108', 'h108']:
            profile = next(p for p in ok('list_model_profiles') if p['model_id'] == model['id'])
            a_key = next(k for k in model['keys'] if k['usage'] == 4)
            i = a_key['light_index']
            assert profile['per_key'][a_key['id']] == [(2*i)&255, 255-i, (3*i)&255]
            assert profile['lighting']['color'] == [32,64,128]
            assert not profile['assignments'] and not profile['fn_assignments'] and not profile['macros']
        if model['id'] == 'f98-pro':
            profile = next(p for p in ok('list_model_profiles') if p['model_id'] == model['id'])
            assert profile['lighting']['effect'] == 7
            assert profile['per_key'][key['id']] == profile['lighting']['color']
            legacy = dict(profile, id='f98-native-legacy', name='Legacy F98 native', lighting=None,
                          per_key={}, assignments={key['id']: {'kind': 'keyboard', 'usage': 5}},
                          macros=[{'id': 'keep-me', 'name': 'Retained macro', 'events': [
                              {'kind': 'down', 'key': 4}, {'kind': 'up', 'key': 4}]}])
            legacy_file = DATA / 'model-profiles/f98-native-legacy.json'
            assert not legacy_file.exists()
            original = json.dumps(legacy).encode()
            legacy_file.write_bytes(original);legacy_file.chmod(0o600)
            migrated = next(p for p in ok('list_model_profiles') if p['id'] == legacy['id'])
            assert migrated['lighting']['effect'] == 1
            assert migrated['assignments'] == legacy['assignments'] and migrated['macros'] == legacy['macros']
            assert legacy_file.read_bytes() == original, 'Reading silently rewrote the legacy profile'
            # Refresh the visible list by saving the current, separate draft.
            click('Save local profile')
            wait(lambda: execute('return ![...document.querySelectorAll("button")].find(b=>b.textContent.trim()==="Save local profile").disabled'))
            click('Profiles')
            wait(lambda: execute('return [...document.querySelectorAll("button")].some(b=>b.textContent.trim()==="Legacy F98 native")'))
            click('Legacy F98 native');click('Lighting')
            assert execute('return document.querySelector(arguments[0]).value', ['input[aria-label="Model brightness"]']) == '5'
            assert legacy_file.read_bytes() == original
            click('Save local profile')
            wait(lambda: json.loads(legacy_file.read_text())['lighting'] is not None)
            assert any(p.read_bytes() == original for p in (DATA / 'model-profiles/history').glob('*.json'))
            click('Profiles');click(profile['name']);click('Lighting')
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
    result={'status':'passed','binary_sha256':hashlib.sha256(BINARY.read_bytes()).hexdigest(),'physical_writes':False,'screenshots_captured':not options.no_screenshots,
            'models':checked,'checks':['all recovered layouts in native WebKit','research-only entries have no controls',
            'native model profile save and clean state','F87 actual Rust synthetic read/384B transfer/126-plane projection','F108/H108 actual Rust 378B capture projection and config signatures','demo physical-read gate before IO','raw capture download/schema isolation','per-key and Fn draft serialization','focused native macro recording',
            'F98 native effect controls/per-key persistence and non-destructive legacy migration through visible save/history',
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
