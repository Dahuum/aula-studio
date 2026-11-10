#!/usr/bin/env python3
"""Smoke our built Windows/macOS app in isolated demo storage; never writes HID."""
import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import time

def visible_window(pid):
    if os.name == 'nt':
        from ctypes import wintypes
        user = ctypes.windll.user32
        callback_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
        user.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
        user.IsWindowVisible.argtypes = [wintypes.HWND]
        user.IsWindowVisible.restype = wintypes.BOOL
        user.EnumWindows.argtypes = [callback_type, wintypes.LPARAM]
        found = []
        def visit(window, _):
            owner = wintypes.DWORD()
            user.GetWindowThreadProcessId(window, ctypes.byref(owner))
            if owner.value == pid and user.IsWindowVisible(window):
                found.append(True)
            return True
        callback = callback_type(visit)
        user.EnumWindows(callback, 0)
        return bool(found)
    # Query only owner PID; do not retain titles or other window metadata.
    cg = ctypes.CDLL('/System/Library/Frameworks/CoreGraphics.framework/CoreGraphics')
    cf = ctypes.CDLL('/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation')
    cg.CGWindowListCopyWindowInfo.argtypes = [ctypes.c_uint32, ctypes.c_uint32]
    cg.CGWindowListCopyWindowInfo.restype = ctypes.c_void_p
    cf.CFArrayGetCount.argtypes = [ctypes.c_void_p]; cf.CFArrayGetCount.restype = ctypes.c_long
    cf.CFArrayGetValueAtIndex.argtypes = [ctypes.c_void_p, ctypes.c_long]; cf.CFArrayGetValueAtIndex.restype = ctypes.c_void_p
    cf.CFDictionaryGetValue.argtypes = [ctypes.c_void_p, ctypes.c_void_p]; cf.CFDictionaryGetValue.restype = ctypes.c_void_p
    cf.CFNumberGetValue.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.c_void_p]
    cf.CFRelease.argtypes = [ctypes.c_void_p]
    key = ctypes.c_void_p.in_dll(cg, 'kCGWindowOwnerPID')
    windows = cg.CGWindowListCopyWindowInfo(1, 0)
    if not windows:
        return False
    try:
        for i in range(cf.CFArrayGetCount(windows)):
            record = cf.CFArrayGetValueAtIndex(windows, i)
            value = cf.CFDictionaryGetValue(record, key)
            owner = ctypes.c_int64()
            if value and cf.CFNumberGetValue(value, 4, ctypes.byref(owner)) and owner.value == pid:
                return True
        return False
    finally:
        cf.CFRelease(windows)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--cli', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert platform.system() in {'Windows', 'Darwin'}, 'Use the Linux native smoke suite on Linux'
    binary, cli = args.binary.resolve(), args.cli.resolve()
    out = args.output.resolve(); out.mkdir(parents=True, exist_ok=False)
    data = out / 'data'; data.mkdir()
    (data / 'preferences.json').write_text(json.dumps({'demo_mode': True, 'reduce_motion': True}))
    env = dict(os.environ, AULA_DATA_DIR=str(data), AULA_DISABLE_HARDWARE_WRITES='1', AULA_NO_ORIGINAL_SKIN='1')
    version = subprocess.check_output([str(cli), '--version'], env=env, text=True, timeout=30).strip()
    models = json.loads(subprocess.check_output([str(cli), 'models'], env=env, text=True, timeout=30))
    devices = json.loads(subprocess.check_output([str(cli), 'devices'], env=env, text=True, timeout=30))
    assert len(models) >= 10 and isinstance(devices, list)
    with (out / 'app.log').open('w') as log:
        process = subprocess.Popen([str(binary)], env=env, stdout=log, stderr=subprocess.STDOUT)
        try:
            deadline = time.monotonic() + 60
            mapped = False
            while time.monotonic() < deadline:
                assert process.poll() is None, 'Native app exited before showing its window'
                if visible_window(process.pid):
                    mapped = True; break
                time.sleep(.5)
            assert mapped, 'No native app window observed'
            assert not list((data / 'operations').glob('*.json'))
            assert not list((data / 'backups').glob('*.json'))
        finally:
            process.terminate()
            try: process.wait(timeout=15)
            except subprocess.TimeoutExpired: process.kill(); process.wait(timeout=10)
    result = {'status':'passed', 'platform':platform.system(), 'architecture':platform.machine(),
              'version':version, 'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),
              'native_window_observed':mapped, 'models':len(models), 'physical_writes':False,
              'physical_validation':False, 'checks':['native CLI/model registry/device enumeration',
              'actual native GUI window in demo mode', 'isolated storage and write guard', 'no hardware operation journals']}
    (out / 'result.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result, indent=2))

if __name__ == '__main__': main()
