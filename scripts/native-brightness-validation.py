#!/usr/bin/env python3
"""Bounded native GUI brightness test; default is read/prepare/cancel only.

--execute-authorized requires separately recorded human authorization for one
one-step brightness change and its restoration. It does not grant permission.
Never run against a receiver or a different revision. See HARDWARE_VALIDATION.md.
"""
import argparse
import base64
import json
import os
from pathlib import Path
import signal
import subprocess
import time
import urllib.error
import urllib.request

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--execute-authorized', action='store_true')
args = parser.parse_args()
ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / 'target/release/aula-studio'
CLI = ROOT / 'target/release/aula'
inputs = list((ROOT / 'crates').rglob('*.rs')) + list((ROOT / 'app/src-tauri').rglob('*.rs')) + list((ROOT / 'app/dist').rglob('*'))
if not BINARY.exists() or BINARY.stat().st_mtime < max(p.stat().st_mtime for p in inputs if p.is_file()):
    raise SystemExit('Build the current frontend and release binary first')
if args.execute_authorized and 'AULA_DISABLE_HARDWARE_WRITES' in os.environ:
    raise SystemExit('An inherited hardware write guard is set; it will not be overridden')


def cli(*words):
    return json.loads(subprocess.check_output([str(CLI), *words], text=True, timeout=25))


def unique_device():
    devices = {d['path']: d for d in cli('devices') if d['read_supported']}
    if len(devices) != 1:
        raise SystemExit('Connect exactly one supported wired F75; no write attempted')
    device = next(iter(devices.values()))
    assert (device['vendor_id'], device['product_id'], device['interface_number'], device['feature_report_bytes']) == (0x258a, 0x010c, 1, 520)
    return device


DEVICE = unique_device()


def read_all():
    fresh = unique_device()
    assert fresh['path'] == DEVICE['path'] and fresh['connection_id'] == DEVICE['connection_id'], 'Connection changed; stop'
    path = fresh['path']
    snapshot = cli('read', path)
    base = cli('read-layer', path, 'base')
    fn = cli('read-layer', path, 'fn')
    leds = cli('read-per-key', path)
    for s in [snapshot, base['snapshot'], fn['snapshot'], leds['snapshot']]:
        assert s['device']['connection_id'] == DEVICE['connection_id'], 'Snapshot connection changed'
    verify = cli('read', path)
    assert snapshot['config'] == verify['config'] and snapshot['colors'] == verify['colors'], 'Settings changed during backup'
    banks = {'config': snapshot['config'], 'colors': snapshot['colors'], 'base': base['keymap'],
             'fn': fn['keymap'], 'per_key': leds['per_key']}
    assert [len(banks[k]) for k in ['config', 'colors', 'base', 'fn', 'per_key']] == [128, 512, 520, 520, 378]
    return banks


BEFORE = read_all()
effect = BEFORE['config'][10]
if not 1 <= effect <= 18:
    raise SystemExit('Current off/custom/unknown effect excluded; no write attempted')
OFFSET = 56 + 2 * effect
OLD = BEFORE['config'][OFFSET]
assert 0 <= OLD <= 9
NEW = OLD - 1 if OLD > 0 else 1
EXPECTED = {**BEFORE, 'config': list(BEFORE['config'])}
EXPECTED['config'][OFFSET] = NEW
OUT = ROOT / 'analysis/brightness-validation' / str(time.time_ns())
OUT.mkdir(parents=True, mode=0o700)
DATA = OUT / 'data'
(DATA / 'profiles').mkdir(parents=True, mode=0o700)


def save(name, value):
    target = OUT / name
    fd = os.open(target, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, 'w') as stream:
        json.dump(value, stream, indent=2)
        stream.flush()
        os.fsync(stream.fileno())
    directory = os.open(OUT, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(directory)
    finally:
        os.close(directory)


save('baseline.json', {'device': DEVICE, 'banks': BEFORE})
save('recovery.json', {'offset': OFFSET, 'original_brightness': OLD, 'test_brightness': NEW,
                      'instructions': 'Only restore this byte through Review brightness only if every bank exactly matches expected-after.json and the same connection remains. On timeout, disconnect or unrelated changes, stop and retain evidence. Never blindly replay raw banks.'})
save('expected-after.json', EXPECTED)
profile = {'schema_version': 1, 'id': 'brightness-validation', 'name': 'Brightness validation',
           'lighting': {'effect': effect, 'brightness': NEW, 'speed': 2, 'color': [0, 0, 0], 'direction': 0},
           'per_key': {}, 'keymap': {}, 'fn_layer': {}, 'macros': [], 'updated_at': 'Validation draft only'}
profile_path = DATA / 'profiles/brightness-validation.json'
profile_path.write_text(json.dumps(profile))
profile_path.chmod(0o600)
env = dict(os.environ, AULA_DATA_DIR=str(DATA), TAURI_WEBVIEW_AUTOMATION='true', GDK_BACKEND='x11', WEBKIT_DISABLE_COMPOSITING_MODE='1')
if not args.execute_authorized:
    env['AULA_DISABLE_HARDWARE_WRITES'] = '1'
log = (OUT / 'driver.log').open('w')
process = subprocess.Popen(['xvfb-run', '-a', 'WebKitWebDriver', '--host=127.0.0.1', '--port=4447'],
                           env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
sid = None
write_actions = 0


def request(method, path, data=None):
    raw = None if data is None else json.dumps(data).encode()
    req = urllib.request.Request('http://127.0.0.1:4447' + path, data=raw, method=method,
                                 headers={'Content-Type': 'application/json'})
    try:
        with urllib.request.urlopen(req, timeout=30) as response:
            value = json.load(response)['value']
    except urllib.error.HTTPError as error:
        raise RuntimeError(error.read().decode()) from error
    if isinstance(value, dict) and value.get('error') and 'ok' not in value:
        raise RuntimeError(str(value))
    return value


def execute(script, values=None):
    return request('POST', f'/session/{sid}/execute/sync', {'script': script, 'args': values or []})


def invoke(command, values=None):
    assert not command.startswith('commit_'), 'Writes must pass through the actual reviewed GUI action'
    result = request('POST', f'/session/{sid}/execute/async', {
        'script': 'const done=arguments[arguments.length-1];window.__TAURI_INTERNALS__.invoke(arguments[0],arguments[1]).then(value=>done({ok:true,value}),error=>done({ok:false,error:String(error)}));',
        'args': [command, values or {}]})
    assert result['ok'], result
    return result['value']


def wait(check):
    deadline = time.monotonic() + 20
    while time.monotonic() < deadline:
        try:
            value = check()
            if value:
                return value
        except Exception:
            pass
        time.sleep(.2)
    raise AssertionError('Native UI timed out; no automatic retry or rollback')


def element(xpath):
    return request('POST', f'/session/{sid}/element', {'using': 'xpath', 'value': xpath})['element-6066-11e4-a52e-4f735466cecf']


def click(label):
    global write_actions
    if label.startswith('Write '):
        assert args.execute_authorized and label == 'Write brightness to keyboard', 'Only the authorized brightness action is permitted'
        assert write_actions < 2, 'Two-action limit reached; no retry'
        write_actions += 1
    button = element(f'//button[@aria-label="{label}" or normalize-space()="{label}"] | //summary[normalize-space()="{label}"]')
    wait(lambda: not execute('return arguments[0].disabled', [
        {'element-6066-11e4-a52e-4f735466cecf': button}]))
    request('POST', f'/session/{sid}/element/{button}/click', {})


def prepare(number, old, new, banks):
    if not execute('return document.querySelector(".original-safe-tools").open'):
        click('Read / recover')
    click('Review brightness only')
    # Tauri deliberately makes its invoke function immutable. Observe the actual
    # review DOM and durable journal instead of trying to monkeypatch that API.
    review = wait(lambda: execute('''const dialog=document.querySelector('[role="dialog"][aria-label="Review brightness changes"]');if(!dialog)return null;
const labels=[...dialog.querySelectorAll('dt')];const value=name=>labels.find(e=>e.textContent===name)?.nextElementSibling?.textContent;
return {backup_id:value('Backup'),device:value('Device'),changes:[...dialog.querySelectorAll('tbody tr')].map(row=>{const c=[...row.querySelectorAll('td')].map(e=>e.textContent);return {field:c[0],before:JSON.parse(c[1]),after:JSON.parse(c[2])};})};'''))
    assert review['changes'] == [{'field': 'Preset brightness', 'before': old, 'after': new}], review
    backup = json.loads((DATA / 'backups' / (review['backup_id'] + '.json')).read_text())['snapshot']
    assert backup['config'] == banks['config'] and backup['colors'] == banks['colors']
    assert read_all() == banks, 'State changed before write; abort'
    save(f'review-{number}.json', review)
    (OUT / f'review-{number}.png').write_bytes(base64.b64decode(request('GET', f'/session/{sid}/screenshot')))
    return review


def completion(review):
    def observed():
        for path in (DATA / 'operations').glob('*.json'):
            record = json.loads(path.read_text())
            if record['backup_id'] == review['backup_id'] and record['state'] != 'started':
                assert record['operation'] == 'brightness', 'Unexpected operation; stop'
                return record
        return None
    record = wait(observed)
    return {'ok': record['state'] == 'readback_verified', 'journal': record}


try:
    wait(lambda: request('GET', '/status'))
    session = request('POST', '/session', {'capabilities': {'alwaysMatch': {
        'browserName': 'wry', 'webkitgtk:browserOptions': {'binary': str(BINARY), 'args': []}}}})
    sid = session['sessionId']
    wait(lambda: execute('return !!document.querySelector(".original-app")'))
    assert invoke('get_status')['mode'] == 'hardware'
    assert invoke('diagnostics')['safety']['hardware_writes_enabled'] is args.execute_authorized
    click('Lighting')
    wait(lambda: execute('return document.querySelector("h1").textContent') == 'Lighting')
    wait(lambda: execute('return document.querySelector("input[type=range][max=\\"9\\"]").value') == str(NEW))
    first_review = prepare(0, OLD, NEW, BEFORE)
    if not args.execute_authorized:
        click('Cancel')
        assert read_all() == BEFORE
        assert write_actions == 0 and not list((DATA / 'operations').glob('*.json'))
        save('result.json', {'status': 'dry-run passed', 'physical_writes': 0, 'all_banks_unchanged': True})
    else:
        click('Write brightness to keyboard')
        forward = completion(first_review)
        save('forward-result.json', forward)
        after = read_all()
        save('observed-after.json', after)
        assert after == EXPECTED, 'Unexpected state after first write; stop, no blind restoration'
        # Restore only the single byte after confirming every bank and connection.
        slider = element('//input[@type="range" and @max="9"]')
        arrow = '\ue014' if OLD > NEW else '\ue012'
        request('POST', f'/session/{sid}/element/{slider}/value', {'text': arrow, 'value': [arrow]})
        wait(lambda: execute('return document.querySelector("input[type=range][max=\\"9\\"]").value') == str(OLD))
        second_review = prepare(1, NEW, OLD, EXPECTED)
        click('Write brightness to keyboard')
        restoration = completion(second_review)
        save('restore-result.json', restoration)
        restored = read_all()
        save('observed-restored.json', restored)
        assert restored == BEFORE, 'Restoration unconfirmed; keep backups and stop'
        assert forward['ok'] and restoration['ok'], 'Bytes restored but a GUI operation reported failure; retain evidence'
        assert write_actions == 2 and len(list((DATA / 'operations').glob('*.json'))) == 2
        save('result.json', {'status': 'passed', 'physical_writes': 2, 'scope': 'one brightness byte',
                             'config_offset': OFFSET, 'original': OLD, 'test': NEW,
                             'all_banks_restored': True, 'visible_led_validation': False, 'power_cycle_persistence': 'not_tested'})
    print((OUT / 'result.json').read_text())
    print('Evidence:', OUT)
except Exception as error:
    save('failure.json', {'error': str(error), 'physical_write_actions': write_actions, 'automatic_retry': False})
    if sid:
        try:
            (OUT / 'failure.png').write_bytes(base64.b64decode(request('GET', f'/session/{sid}/screenshot')))
            save('failure-ui.json', {'text': execute('return document.body.innerText')})
        except Exception:
            pass
    print('Failure evidence:', OUT, flush=True)
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
