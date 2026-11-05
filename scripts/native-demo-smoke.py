#!/usr/bin/env python3
"""Exercise the real native GUI/backend with isolated storage and blocked writes.

Requires WebKitWebDriver and xvfb-run. Does not require a keyboard and never
calls commit commands. All generated files stay in analysis/native-demo-smoke.
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
OUT = ROOT / 'analysis/native-demo-smoke' / str(time.time_ns())
OUT.mkdir(parents=True)
DATA = OUT / 'data'
env = dict(os.environ, AULA_DATA_DIR=str(DATA), AULA_DISABLE_HARDWARE_WRITES='1',
           TAURI_WEBVIEW_AUTOMATION='true', GDK_BACKEND='x11', WEBKIT_DISABLE_COMPOSITING_MODE='1')
if options.fallback:
    env['AULA_NO_ORIGINAL_SKIN'] = '1'
CONFIG = DATA
if options.xdg:
    env.pop('AULA_DATA_DIR', None)
    env['XDG_DATA_HOME'] = str(OUT / 'xdg-data')
    env['XDG_CONFIG_HOME'] = str(OUT / 'xdg-config')
    DATA = OUT / 'xdg-data/aula-studio'
    CONFIG = OUT / 'xdg-config/aula-studio'
log = (OUT / 'driver.log').open('w')
process = subprocess.Popen(['xvfb-run', '-a', 'WebKitWebDriver', '--host=127.0.0.1', '--port=4446'],
                           env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
sid = None


def request(method, path, data=None):
    raw = None if data is None else json.dumps(data).encode()
    req = urllib.request.Request('http://127.0.0.1:4446' + path, data=raw, method=method,
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
    request('POST', f'/session/{sid}/element/{button}/click', {})


def text_input(xpath, text):
    field = request('POST', f'/session/{sid}/element', {'using': 'xpath', 'value': xpath})['element-6066-11e4-a52e-4f735466cecf']
    request('POST', f'/session/{sid}/element/{field}/clear', {})
    request('POST', f'/session/{sid}/element/{field}/value', {'text': text, 'value': list(text)})


def paste_input(xpath, text):
    # WebKitWebDriver drops non-BMP send-keys characters on this build. Exercise
    # the real controlled input handler as pasted text; never replace native IPC.
    field = request('POST', f'/session/{sid}/element', {'using': 'xpath', 'value': xpath})
    execute('const e=arguments[0];Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,"value").set.call(e,arguments[1]);e.dispatchEvent(new InputEvent("input",{bubbles:true,inputType:"insertFromPaste",data:arguments[1]}));', [field, text])
    wait(lambda: execute('return arguments[0].value', [field]) == text)


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
    assets = ok('local_ui_assets')
    assert assets['redistributed'] is False and assets['local_only'] is True
    if options.fallback:
        assert not assets['assets'], 'Standalone check accidentally loaded proprietary artwork'
    elif (ROOT / 'analysis/extracted/app/skins/main_nr.png').exists():
        assert assets['assets'].get('background', '').startswith('data:image/png;base64,')
        wait(lambda: execute('return !!document.querySelector(".with-original-art")'))
    initial = ok('get_status')
    if not initial['connected']:
        assert execute('return [...document.querySelectorAll("button")].find(b=>b.textContent.trim()==="Review lighting write").disabled')
        failed = invoke('prepare_brightness', {'value': 1})
        assert not failed['ok'], failed
    click('Enter demo mode')
    wait(lambda: ok('get_status')['mode'] == 'demo')
    wait(lambda: execute('return document.body.innerText.includes("Exit demo mode")'))
    preferences = CONFIG / 'preferences.json'
    assert json.loads(preferences.read_text())['demo_mode'] is True
    assert preferences.stat().st_mode & 0o777 == 0o600
    if options.xdg:
        assert not (DATA / 'preferences.json').exists(), 'XDG config must be separate from profile data'
    navigate('Device Settings')
    reduce_motion = request('POST', f'/session/{sid}/element', {'using':'xpath', 'value':'//label[contains(.,"Reduce motion")]/input'})['element-6066-11e4-a52e-4f735466cecf']
    request('POST', f'/session/{sid}/element/{reduce_motion}/click', {})
    wait(lambda: ok('get_preferences')['reduce_motion'])
    select_option('Hardware sleep timer', '600')
    click('Apply to demo')
    wait(lambda: ok('read_configuration')['profile']['sleep_seconds'] == 600)
    navigate('Lighting')
    click('Apply to demo')
    wait(lambda: ok('read_configuration').get('profile') is not None)
    demo = ok('read_configuration')
    click('Preset #ff0000')
    click('Add')
    wait(lambda: ok('get_preferences').get('custom_colors') == [[255, 0, 0]])
    click('Apply to demo')
    wait(lambda: ok('read_configuration')['profile']['lighting']['color'] == [255, 0, 0])
    text_input('//input[@aria-label="Hex color"]', '#123456')
    wait(lambda: execute('return document.querySelector("input[aria-label=\\"Hex color\\"]").value') == '#123456')
    arrow('Hex color', '\ue007')
    wait(lambda: execute('return document.querySelector("input[aria-label=\\"R value\\"]").value') == '18')
    text_input('//input[@aria-label="G value"]', '128')
    arrow('Brightness', '\ue012')
    click('Apply to demo')
    wait(lambda: ok('read_configuration')['profile']['lighting']['color'] == [18, 128, 86])
    assert ok('read_configuration')['profile']['lighting']['brightness'] == demo['profile']['lighting']['brightness'] - 1
    click('Choose hue and saturation')
    # Center of the displayed color wheel is white for both shipped and CSS versions.
    click('Apply to demo')
    wait(lambda: min(ok('read_configuration')['profile']['lighting']['color']) > 240)
    choices = execute('return [...document.querySelectorAll(".original-effects button")].map(b=>b.getAttribute("aria-label"))')
    assert len(choices) == 20 and len(set(choices)) == 20
    assert execute('return document.querySelector(".original-effects button:last-child").textContent') == 'OFF'
    for choice in choices:
        click(choice)
        assert execute('return [...document.querySelectorAll(".original-effects button")].find(b=>b.getAttribute("aria-label")===arguments[0]).getAttribute("aria-pressed")', [choice]) == 'true'
    click('Lights off')
    assert execute('return document.querySelector("input[aria-label=\\"Brightness\\"]").disabled')
    click('Breathing')
    speed = int(execute('return document.querySelector("input[aria-label=\\"Speed\\"]").value'))
    arrow('Speed', '\ue012' if speed else '\ue014')
    click('Apply to demo')
    wait(lambda: ok('read_configuration')['profile']['lighting']['speed'] == (speed-1 if speed else 1))
    click('Solid color')
    wait(lambda: execute('return document.querySelector(".original-effects button.active").textContent') == 'Fixed_on')
    screenshot('lighting.png')
    navigate('Per-Key RGB')
    click('Clear selection')
    click('Fn')
    click('Preset #00ff00')
    click('Assign color')
    click('Apply to demo')
    wait(lambda: ok('read_configuration')['profile']['per_key'].get('fn') == [0, 255, 0])
    click('Clear colors')
    click('Apply to demo')
    wait(lambda: 'fn' not in ok('read_configuration')['profile']['per_key'])
    screenshot('per-key.png')
    navigate('Macros')
    click('+ New')
    click('Start recording')
    area = request('POST', f'/session/{sid}/element', {
        'using': 'xpath', 'value': '//*[@aria-label="Macro recording area"]'})['element-6066-11e4-a52e-4f735466cecf']
    request('POST', f'/session/{sid}/element/{area}/value', {'text': 'a', 'value': ['a']})
    click('Stop recording')
    wait(lambda: execute('return [...document.querySelectorAll("button")].some(b=>b.textContent.trim()==="Start recording")'))
    # Exercise the event editor before persisting it through the real backend.
    click('Select event 1')
    select_option('Insert position', 'after')
    click('+ Delay')
    text_input('//input[@aria-label="Delay 2 in milliseconds"]', '25')
    click('Move event 2 down')
    click('Move event 3 up')
    click('Edit key for event 1')
    select_option('Key for event 1', '5')
    release_label=execute('return [...document.querySelectorAll(".event")].find(r=>r.querySelector(".event-kind.up")).querySelector(".event-key-label").getAttribute("aria-label")')
    release_index=int(release_label.split()[-1])
    click(release_label)
    select_option('Key for event '+str(release_index), '5')
    paste_input('//input[@aria-label="Macro name"]', '🎹' * 30)
    click('Duplicate macro')
    assert execute('return document.querySelector(arguments[0]).value', ['input[aria-label="Macro name"]']) == '🎹' * 28 + ' (copy)'
    click('Apply to demo')
    wait(lambda: len(ok('read_configuration').get('profile', {}).get('macros', [])) == 2)
    click('Delete')
    wait(lambda: execute('return document.querySelector(arguments[0]).value', ['input[aria-label="Macro name"]']) == '🎹' * 30)
    text_input('//input[@aria-label="Macro name"]', 'Macro 1')
    click('Apply to demo')
    wait(lambda: len(ok('read_configuration').get('profile', {}).get('macros', [])) == 1)
    demo = ok('read_configuration')
    key_events = [event for event in demo['profile']['macros'][0]['events'] if event['kind'] != 'delay']
    assert key_events == [{'kind': 'down', 'key': 5}, {'kind': 'up', 'key': 5}], key_events
    assert demo['source'] == 'demo' and demo['profile']['schema_version'] == 1
    click('Inspect F75 wire format (offline)')
    wait(lambda: execute('return !!document.querySelector(".modal")'))
    wire=json.loads(execute('return document.querySelector(".modal textarea").value'))
    delay=sum(e.get('ms',0) for e in demo['profile']['macros'][0]['events'])
    assert wire['protocol']=='original_f75_g5' and not wire['hardware_validated'] and not wire['upload_available']
    assert wire['nominal_duration_ms']==delay and wire['key_events']==2
    assert wire['record_bytes'][-8:]==[(delay>>16)&15,(delay>>8)&255,delay&255,5,0x80,0,0,5]
    assert wire['single_record_pool_bytes'][4:]==wire['record_bytes']
    click('Close')
    screenshot('macros.png')
    # Render every panel through actual React navigation in the native WebView.
    for page in ['Lighting', 'Per-Key RGB', 'Key Remapping', 'Fn Layers', 'Macros',
                 'Profiles', 'Device Settings', 'Diagnostics', 'About', 'Dashboard']:
        navigate(page)
    for page, field, source, source_usage, usage in [
        ('Key Remapping', 'consumer_keymap', 'A', '4', 0xcd),
        ('Fn Layers', 'consumer_fn_layer', 'A', '4', 0xe2),
        ('Key Remapping', 'consumer_keymap', 'B', '5', 0x223),
        ('Fn Layers', 'consumer_fn_layer', 'B', '5', 0x192),
    ]:
        navigate(page)
        geometry = execute('const k=document.querySelector(".original-keyboard-panel .keyboard").getBoundingClientRect(),tabs=[...document.querySelectorAll(".original-layer-tabs button")],keys=[...document.querySelectorAll(".original-keyboard-panel .key")];return {scale:k.width/679,keyTop:k.top,keyRight:Math.max(...keys.map(e=>e.getBoundingClientRect().right)),tabRight:tabs.at(-1).getBoundingClientRect().right,bottom:tabs[0].getBoundingClientRect().bottom,buttons:tabs.map(b=>{const r=b.getBoundingClientRect();return {width:r.width,height:r.height}})}')
        assert len(geometry['buttons']) == 2
        assert abs(geometry['keyRight'] - geometry['tabRight']) < 1
        assert abs(geometry['keyTop'] - geometry['bottom'] - 10*geometry['scale']) < 1
        for button in geometry['buttons']:
            assert abs(button['width'] - 85*geometry['scale']) < 1
            assert abs(button['height'] - 45*geometry['scale']) < 1
        click(source)
        select_option('Action type', 'media')
        assert execute('return [...document.querySelector("select[aria-label=\\"Media action\\"]").options].map(o=>Number(o.value))') == [0x183, 0xcd, 0xe2, 0xb7, 0xb6, 0xb5, 0xe9, 0xea, 0x223, 0x192, 0x18a, 0x194, 0x6f, 0x70]
        select_option('Media action', str(usage))
        click('Assign locally')
        click('Apply to demo')
        wait(lambda f=field, s=source_usage, u=usage: ok('read_configuration').get('profile', {}).get(f, {}).get(s) == u)
        assert execute('return [...document.querySelectorAll(".original-layer-tabs button")].every(b=>{const r=b.getBoundingClientRect();return document.elementFromPoint(r.left+r.width/2,r.top+r.height/2)?.closest("button")===b})'), 'A notification obscures a layer tab'
        if page == 'Key Remapping':
            screenshot('key-remapping.png')
    for page, field, button in [('Key Remapping', 'mouse_keymap', 1), ('Fn Layers', 'mouse_fn_layer', 5)]:
        navigate(page)
        click('C')
        select_option('Action type', 'mouse')
        assert execute('return [...document.querySelector("select[aria-label=\\"Mouse action\\"]").options].map(o=>Number(o.value))') == [1, 2, 3, 4, 5]
        select_option('Mouse action', str(button))
        click('Assign locally')
        click('Apply to demo')
        wait(lambda f=field, b=button: ok('read_configuration').get('profile', {}).get(f, {}).get('6') == b)
    screenshot('mouse-bindings.png')
    demo = ok('read_configuration')
    click('Save profile')
    wait(lambda: len(ok('list_profiles')) == 1)
    wait(lambda: execute('return document.body.innerText.includes("Profile saved locally.")'))
    assert not execute('return document.body.innerText.includes("Unsaved changes")'), 'Persisted native profile still marked dirty'
    saved = ok('list_profiles')[0]
    exported = json.loads(ok('export_profile', {'id': saved['id']}))
    assert exported['lighting'] == demo['profile']['lighting']
    assert exported['consumer_keymap'] == {'4': 0xcd, '5': 0x223} and exported['consumer_fn_layer'] == {'4': 0xe2, '5': 0x192}
    assert exported['mouse_keymap'] == {'6': 1} and exported['mouse_fn_layer'] == {'6': 5}
    assert exported['macros'] == demo['profile']['macros']
    navigate('Profiles')
    paste_input('//label[contains(.,"Current profile name")]/input', '🎹' * 30)
    click('Duplicate')
    assert request('GET', f'/session/{sid}/alert/text') == 'Discard unsaved changes to the current profile?'
    request('POST', f'/session/{sid}/alert/accept', {})
    click('Save profile')
    wait(lambda: len(ok('list_profiles')) == 2)
    duplicates = [p for p in ok('list_profiles') if p['id'] != saved['id']]
    assert len(duplicates) == 1 and duplicates[0]['name'] == '🎹' * 28 + ' copy'
    text_input('//label[contains(.,"Current profile name")]/input', 'Native duplicate')
    click('Save profile')
    wait(lambda: any(p['id'] == duplicates[0]['id'] and p['name'] == 'Native duplicate' for p in ok('list_profiles')))
    # Load the original profile through the visible row action.
    load = request('POST', f'/session/{sid}/element', {'using': 'xpath', 'value': f'//div[@class="profile-row"][.//b[normalize-space()="{saved["name"]}"]]//button[normalize-space()="Load"]'})['element-6066-11e4-a52e-4f735466cecf']
    request('POST', f'/session/{sid}/element/{load}/click', {})
    wait(lambda: execute('return document.querySelector(".field input").value') == saved['name'])
    ok('delete_profile', {'id': duplicates[0]['id']})
    assert len(ok('list_profiles')) == 1
    imported = ok('import_profile', {'json': json.dumps(exported)})
    assert imported['id'] != saved['id']
    assert len(ok('list_profiles')) == 2
    # The backend must reject a malformed imported macro even without UI checks.
    malformed = dict(exported, macros=[{'id': 'bad', 'name': 'Invalid', 'events': [{'kind': 'up', 'key': 4}]}])
    assert not invoke('import_profile', {'json': json.dumps(malformed)})['ok']
    assert len(ok('list_profiles')) == 2
    assert not list((DATA / 'operations').glob('*.json'))
    assert not list((DATA / 'backups').glob('*.json'))
    screenshot('profiles.png')
    # Verify that the measured canvas remains fully inside a small tiled window.
    original_rect = request('GET', f'/session/{sid}/window/rect')
    request('POST', f'/session/{sid}/window/rect', {'width': 940, 'height': 580})
    wait(lambda: execute('const r=document.querySelector(".original-app").getBoundingClientRect();return r.right<=innerWidth+1&&r.bottom<=innerHeight+1&&r.width>800'))
    screenshot('small-window.png')
    request('POST', f'/session/{sid}/window/rect', original_rect)
    request('POST', f'/session/{sid}/execute/async', {
        'script': 'const done=arguments[arguments.length-1];window.scrollTo(0,0);document.body.scrollTop=0;document.documentElement.scrollTop=0;document.querySelector("main").scrollTop=0;requestAnimationFrame(()=>requestAnimationFrame(()=>done(true)));', 'args': []})
    (OUT / 'native-demo.png').write_bytes(base64.b64decode(request('GET', f'/session/{sid}/screenshot')))
    result = {'status': 'passed', 'binary_sha256': hashlib.sha256(BINARY.read_bytes()).hexdigest(), 'physical_writes': False, 'storage': str(DATA), 'preferences': str(preferences),
              'storage_mode': 'XDG' if options.xdg else 'AULA_DATA_DIR',
              'artwork': 'private original assets' if assets['assets'] else 'independent standalone graphics',
              'headless_rendering': 'Xvfb; WebKit compositing disabled to avoid software-renderer snapshot hangs',
              'checks': ['native write guard', 'disconnected controls when applicable', 'demo toggle/apply/read',
                         'reduced-motion preference and sleep setting in demo',
                         'all navigation panels and twenty effects', 'hex/RGB/wheel/brightness/speed and palette persistence', 'Fn per-key assign/clear to demo', 'base/Fn media editor to native demo/profile storage', 'focused native macro recording and Stop action', 'native event insertion/reorder/key editing and independent duplicate', 'actual Rust offline G5 wire inspection with no HID commands', 'native profile save/duplicate/rename/load/export/import and clean saved state', 'backend profile deletion', 'malformed macro rejected', 'small-window canvas bounds',
                         'no physical operation journal or backup']}
    (OUT / 'result.json').write_text(json.dumps(result, indent=2))
    print(json.dumps(result, indent=2))
except Exception:
    if sid:
        try:
            screenshot('failure.png')
            (OUT / 'failure-state.json').write_text(json.dumps(execute('return {text:document.body.innerText,inputs:[...document.querySelectorAll("input")].map(e=>({label:e.getAttribute("aria-label"),value:e.value,disabled:e.disabled}))}'), indent=2))
            (OUT / 'failure-buttons.json').write_text(json.dumps(execute('return [...document.querySelectorAll("button")].map(b=>{const r=b.getBoundingClientRect();return {label:b.getAttribute("aria-label")||b.textContent.trim(),rect:{x:r.x,y:r.y,width:r.width,height:r.height},disabled:b.disabled,pointer:getComputedStyle(b).pointerEvents,hit:document.elementFromPoint(r.x+r.width/2,r.y+r.height/2)?.outerHTML.slice(0,160)}})'),indent=2))
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
