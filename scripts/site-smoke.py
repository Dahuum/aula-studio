#!/usr/bin/env python3
"""Actual WebKit browser QA of the local landing page; no external publication."""
import base64
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import threading
import time
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'analysis/site-smoke' / str(time.time_ns())
OUT.mkdir(parents=True)
class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, *args):
        pass
server = ThreadingHTTPServer(('127.0.0.1', 0), partial(QuietHandler, directory=str(ROOT / 'site/dist')))
thread = threading.Thread(target=server.serve_forever, daemon=True)
thread.start()
with socket.socket() as probe:
    probe.bind(('127.0.0.1', 0)); port = probe.getsockname()[1]
env = dict(os.environ, GDK_BACKEND='x11', WEBKIT_DISABLE_COMPOSITING_MODE='1',
           XDG_DATA_HOME=str(OUT / 'data'), XDG_CONFIG_HOME=str(OUT / 'config'), XDG_CACHE_HOME=str(OUT / 'cache'))
log = (OUT / 'driver.log').open('w')
process = subprocess.Popen(['xvfb-run', '-a', 'WebKitWebDriver', '--host=127.0.0.1', f'--port={port}'],
                           env=env, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
sid = None


def request(method, path, data=None):
    raw = json.dumps(data).encode() if data is not None else None
    req = urllib.request.Request(f'http://127.0.0.1:{port}'+path, data=raw, method=method,
                                 headers={'Content-Type':'application/json'})
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            value = json.load(r)['value']
    except urllib.error.HTTPError as error:
        raise RuntimeError(error.read().decode()) from error
    if isinstance(value, dict) and value.get('error'):
        raise RuntimeError(str(value))
    return value


def execute(script, args=None):
    return request('POST', f'/session/{sid}/execute/sync', {'script':script, 'args':args or []})


def wait(check):
    deadline = time.monotonic()+15
    while time.monotonic()<deadline:
        try:
            value = check()
            if value:
                return value
        except Exception:
            pass
        time.sleep(.1)
    raise AssertionError('Landing-page state timed out')


def element(xpath):
    return request('POST', f'/session/{sid}/element', {'using':'xpath','value':xpath})['element-6066-11e4-a52e-4f735466cecf']


def click(xpath):
    target=element(xpath)
    execute('arguments[0].scrollIntoView({block:"center",behavior:"instant"})', [{'element-6066-11e4-a52e-4f735466cecf':target}])
    request('POST', f'/session/{sid}/element/{target}/click', {})


def screenshot(name):
    execute('document.activeElement?.blur()')
    execute('window.scrollTo({top:0,left:0,behavior:"instant"})')
    wait(lambda: execute('return window.scrollY===0'))
    request('POST', f'/session/{sid}/execute/async', {'script':'const done=arguments[arguments.length-1];requestAnimationFrame(()=>requestAnimationFrame(()=>done(true)));','args':[]})
    (OUT / name).write_bytes(base64.b64decode(request('GET', f'/session/{sid}/screenshot')))


try:
    wait(lambda: request('GET','/status'))
    session=request('POST','/session',{'capabilities':{'alwaysMatch':{'webkitgtk:browserOptions':{'binary':'/usr/lib/webkit2gtk-4.1/MiniBrowser','args':['--automation']}}}})
    sid=session['sessionId']
    request('POST',f'/session/{sid}/window/rect',{'width':1366,'height':1000})
    request('POST',f'/session/{sid}/url',{'url':f'http://127.0.0.1:{server.server_port}/'})
    wait(lambda: execute('return document.querySelector("h1")?.textContent.includes("Linux")'))
    wait(lambda: execute('return [...document.images].filter(i=>i.hasAttribute("src")&&i.getClientRects().length>0).every(i=>i.complete&&i.naturalWidth>0)'))
    assert execute('return document.documentElement.scrollWidth<=innerWidth+1')
    click('//*[@id="tab-macros"]')
    wait(lambda: execute('return !document.getElementById("panel-macros").hidden'))
    tab=element('//*[@id="tab-macros"]')
    request('POST',f'/session/{sid}/element/{tab}/value',{'text':'\ue012','value':['\ue012']})
    wait(lambda: execute('return document.getElementById("tab-lighting").getAttribute("aria-selected")==="true"'))
    click('//summary[normalize-space()="Does this fully replace the Windows software?"]')
    assert execute('return [...document.querySelectorAll("details")][0].open')
    click('//button[@aria-label="Copy build commands"]')
    wait(lambda: execute('return !!document.getElementById("copy-status").textContent'))
    click('//button[@aria-label="Enlarge the native lighting editor screenshot"]')
    assert execute('return document.getElementById("image-dialog").open')
    click('//*[@id="close-image"]')
    assert not execute('return document.getElementById("image-dialog").open')
    screenshot('desktop.png')
    request('POST',f'/session/{sid}/window/rect',{'width':390,'height':844})
    wait(lambda: execute('return innerWidth<=500'))
    assert execute('return document.documentElement.scrollWidth<=innerWidth+1'), 'Mobile horizontal overflow'
    assert execute('return getComputedStyle(document.querySelector(".header nav")).display==="none"')
    screenshot('mobile.png')
    href=execute('return document.getElementById("main-download").href')
    with urllib.request.urlopen(urllib.request.Request(href,method='HEAD'),timeout=10) as response:
        assert response.status==200 and int(response.headers['Content-Length'])>1000
    result={'status':'passed','browser':'actual WebKit MiniBrowser','checks':['desktop and mobile layout','screenshots loaded',
            'pointer and keyboard tabs','FAQ','copy or selection fallback','image modal open/close','real download HTTP 200'],
            'network_publication':False,'evidence':str(OUT)}
    (OUT/'result.json').write_text(json.dumps(result,indent=2));print(json.dumps(result,indent=2))
except Exception:
    if sid:
        try:screenshot('failure.png')
        except Exception:pass
    print('Failure evidence:',OUT,flush=True)
    raise
finally:
    if sid:
        try:request('DELETE',f'/session/{sid}')
        except Exception:pass
    if process.poll() is None:os.killpg(process.pid,signal.SIGTERM)
    try:process.wait(timeout=10)
    except subprocess.TimeoutExpired:os.killpg(process.pid,signal.SIGKILL);process.wait()
    log.close();server.shutdown();server.server_close()
