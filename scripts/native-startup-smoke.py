#!/usr/bin/env python3
"""Verify TTY display recovery and clean startup errors without hardware writes.

Requires an existing Hyprland session for the positive Wayland cases. Unlike
the Xvfb WebKit tests, these deliberately omit DISPLAY and WAYLAND_DISPLAY.
Every app uses isolated project-local data, demo mode and the write guard.
"""
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / 'target/release/aula-studio'
inputs = list((ROOT / 'app/src-tauri/src').glob('*.rs')) + list((ROOT / 'app/dist').rglob('*'))
assert BINARY.exists() and BINARY.stat().st_mtime >= max(p.stat().st_mtime for p in inputs if p.is_file()), 'Build the current release first'
OUT = ROOT / 'analysis/native-startup-smoke' / str(time.time_ns())
OUT.mkdir(parents=True)
results = []


def environment(name):
    env = dict(os.environ, AULA_DATA_DIR=str(OUT / name / 'data'),
               AULA_DISABLE_HARDWARE_WRITES='1', XDG_SESSION_TYPE='tty')
    for key in ['DISPLAY', 'WAYLAND_DISPLAY', 'GDK_BACKEND']:
        env.pop(key, None)
    return env


def error_case(name, env, expected):
    directory = OUT / name
    directory.mkdir(exist_ok=True)
    result = subprocess.run([str(BINARY)], env=env, text=True, capture_output=True, timeout=15)
    (directory / 'stderr.log').write_text(result.stderr)
    assert result.returncode == 1, (name, result.returncode, result.stderr)
    assert expected in result.stderr and 'panicked at' not in result.stderr, (name, result.stderr)
    assert not Path(env['AULA_DATA_DIR']).exists(), 'Display failure must precede data initialization'
    results.append({'case': name, 'status': 'passed', 'exit': 1, 'no_panic': True, 'no_app_data_created': True})


def wayland_case(name, unset_runtime=False, desktop=False):
    env = environment(name)
    if desktop:
        env['XDG_SESSION_TYPE'] = 'wayland'
        for key in ['DISPLAY', 'WAYLAND_DISPLAY']:
            if key in os.environ:
                env[key] = os.environ[key]
    if unset_runtime:
        env.pop('XDG_RUNTIME_DIR', None)
    directory = OUT / name
    data = Path(env['AULA_DATA_DIR'])
    data.mkdir(parents=True)
    preferences = data / 'preferences.json'
    preferences.write_text(json.dumps({'demo_mode': True, 'reduce_motion': True}))
    preferences.chmod(0o600)
    with (directory / 'stderr.log').open('w') as log:
        process = subprocess.Popen([str(BINARY)], env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            window = None
            for _ in range(100):
                assert process.poll() is None, f'{name}: application exited before mapping'
                clients = json.loads(subprocess.check_output(['hyprctl', '-j', 'clients']))
                window = next((w for w in clients if w['pid'] == process.pid and w['mapped']), None)
                if window:
                    break
                time.sleep(.1)
            assert window and not window['xwayland'], f'{name}: no native Wayland window'
            assert not list((data / 'operations').glob('*.json'))
            results.append({'case': name, 'status': 'passed', 'mapped': True, 'xwayland': False, 'physical_writes': False})
        finally:
            os.killpg(process.pid, signal.SIGTERM) if process.poll() is None else None
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait()
    if not desktop:
        assert 'opening on your Wayland desktop' in (directory / 'stderr.log').read_text()


try:
    wayland_case('desktop-default', desktop=True)
    wayland_case('tty-with-runtime')
    wayland_case('tty-without-runtime', unset_runtime=True)
    empty = OUT / 'empty-runtime'
    empty.mkdir(mode=0o700)
    env = environment('no-display')
    env['XDG_RUNTIME_DIR'] = str(empty)
    error_case('no-display', env, 'cannot open a graphical display')
    env = environment('explicit-x11-no-display')
    env['GDK_BACKEND'] = 'x11'
    error_case('explicit-x11-no-display', env, 'cannot open a graphical display')
    env = environment('explicit-stale-wayland')
    env.update(GDK_BACKEND='wayland', WAYLAND_DISPLAY='wayland-aula-missing')
    error_case('explicit-stale-wayland', env, 'cannot open a graphical display')
    env = environment('relative-runtime')
    env['XDG_RUNTIME_DIR'] = 'relative-runtime'
    error_case('relative-runtime', env, 'must be an absolute')
    # Keep UNIX socket paths below sockaddr_un's small platform limit.
    ambiguous = OUT / 'r'
    ambiguous.mkdir(mode=0o700)
    listeners = []
    try:
        for name in ['wayland-1', 'wayland-2']:
            listener = socket.socket(socket.AF_UNIX)
            listener.bind(str(ambiguous / name))
            listeners.append(listener)
        env = environment('ambiguous-display')
        env['XDG_RUNTIME_DIR'] = str(ambiguous)
        error_case('ambiguous-display', env, 'Multiple Wayland')
    finally:
        for listener in listeners:
            listener.close()
        for path in ambiguous.iterdir():
            path.unlink()
    result = {'status': 'passed', 'physical_writes': False, 'checks': results}
    (OUT / 'result.json').write_text(json.dumps(result, indent=2))
    print(json.dumps(result, indent=2))
    print('Evidence:', OUT)
except Exception:
    print('Failure evidence:', OUT, flush=True)
    raise
