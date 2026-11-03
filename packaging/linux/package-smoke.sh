#!/usr/bin/env bash
# Install, map a real native window as a regular user, and uninstall IN A CONTAINER.
set -euo pipefail
if [[ ${AULA_CONTAINER_PACKAGE_TEST:-} != 1 || ! -f /.dockerenv ]]; then
  echo 'Refusing package installation outside an explicit disposable test container.' >&2
  exit 1
fi
trap 'cat /tmp/aula-smoke.log 2>/dev/null || true' EXIT
case "$1" in
 deb)
  export DEBIAN_FRONTEND=noninteractive
  apt-get update
  apt-get install -y --no-install-recommends /packages/*.deb xvfb xauth dbus-x11 x11-utils python3 fonts-dejavu-core
  ;;
 rpm)
  dnf install -y /packages/*.rpm xorg-x11-server-Xvfb xorg-x11-xauth dbus-daemon xwininfo python3 dejavu-sans-fonts shadow-utils util-linux
  ;;
 *) echo 'Expected deb or rpm' >&2; exit 1;;
esac
if getent passwd 1000 >/dev/null; then
  aula_test_user=$(getent passwd 1000 | cut -d: -f1)
else
  useradd -m -u 1000 aula-test
  aula_test_user=aula-test
fi
runuser -u "$aula_test_user" -- env AULA_DISABLE_HARDWARE_WRITES=1 aula diagnostics > /tmp/diagnostics.json
python3 - <<'PY'
import json
v=json.load(open('/tmp/diagnostics.json'))
assert v['hardware_writes_enabled'] is False
assert v['version']=='0.5.0'
PY
# This container has no USB devices, host display/home mounts or external page.
# WebKit's extra namespace sandbox cannot start under standard Docker seccomp;
# this override applies only to this disposable, device-free package smoke test.
runuser -u "$aula_test_user" -- env AULA_DISABLE_HARDWARE_WRITES=1 AULA_DATA_DIR=/tmp/aula-smoke-data AULA_NO_ORIGINAL_SKIN=1 WEBKIT_DISABLE_COMPOSITING_MODE=1 WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1 GDK_BACKEND=x11 dbus-run-session -- xvfb-run -a python3 - <<'PY'
import subprocess,time,json,os
p=subprocess.Popen(['/usr/bin/aula-studio'],stdout=open('/tmp/aula-smoke.log','w'),stderr=subprocess.STDOUT)
try:
    mapped=False
    for _ in range(30):
        assert p.poll() is None, 'Application exited before window mapped'
        tree=subprocess.check_output(['xwininfo','-root','-tree'],text=True)
        if '"AULA F75"' in tree:
            mapped=True
            break
        time.sleep(.5)
    assert mapped, 'No native GTK window mapped'
    assert os.getuid()==1000
    print(json.dumps({'status':'passed','native_window_mapped':True,'app_uid':os.getuid(),'hardware_writes':0}))
finally:
    p.terminate()
    try:p.wait(timeout=10)
    except subprocess.TimeoutExpired:p.kill();p.wait()
PY
case "$1" in
 deb) apt-get remove -y aula-studio;;
 rpm) dnf remove -y aula-studio;;
esac
[[ ! -e /usr/bin/aula-studio && ! -e /usr/bin/aula ]]
echo 'PACKAGE_INSTALL_LAUNCH_UNINSTALL_PASSED'
