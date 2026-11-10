/* AULA Studio landing page. All keyboard lighting below is an illustration.
   Motion 14.9.1 is self-hosted with its upstream MIT license in assets/vendor. */
const motionPreference = window.matchMedia('(prefers-reduced-motion: reduce)');
const animationRuns = new Set();
let stopInView;
function motionAnimate(target, frames, options = {}) {
  if (motionPreference.matches || typeof window.Motion?.animate !== 'function') return;
  const control = window.Motion.animate(target, frames, options);
  animationRuns.add(control);
  control.then?.(() => animationRuns.delete(control));
  return control;
}
function setupTabs(selector, onActivate) {
  const tabs = [...document.querySelectorAll(selector)];
  function activate(tab, focus = false) {
    for (const candidate of tabs) {
      const selected = candidate === tab;
      candidate.setAttribute('aria-selected', String(selected));
      candidate.tabIndex = selected ? 0 : -1;
      document.getElementById(candidate.getAttribute('aria-controls')).hidden = !selected;
    }
    onActivate?.(tab);
    if (focus) tab.focus();
  }
  for (const tab of tabs) {
    tab.addEventListener('click', () => activate(tab));
    tab.addEventListener('keydown', event => {
      let next;
      if (event.key === 'ArrowRight') next = tabs[(tabs.indexOf(tab) + 1) % tabs.length];
      if (event.key === 'ArrowLeft') next = tabs[(tabs.indexOf(tab) + tabs.length - 1) % tabs.length];
      if (event.key === 'Home') next = tabs[0];
      if (event.key === 'End') next = tabs[tabs.length - 1];
      if (next) { event.preventDefault(); activate(next, true); }
    });
  }
  return { tabs, activate };
}
const screenshots = setupTabs('[data-tab]', tab => {
  motionAnimate(document.getElementById(tab.getAttribute('aria-controls')), { opacity: [0.55, 1], y: [6, 0] }, { duration: .3 });
});
const platforms = setupTabs('[data-os]', tab => {
  const panel = document.getElementById(tab.getAttribute('aria-controls'));
  motionAnimate(panel, { opacity: [0.5, 1], y: [8, 0] }, { duration: .35 });
});
const platform = navigator.userAgentData?.platform || navigator.platform || '';
const detected = /win/i.test(platform) ? 'windows' : /mac/i.test(platform) ? 'macos' : /linux/i.test(platform) ? 'linux' : 'windows';
platforms.activate(platforms.tabs.find(tab => tab.dataset.os === detected) || platforms.tabs[0]);

const dialog = document.getElementById('image-dialog');
for (const button of document.querySelectorAll('[data-zoom]')) {
  button.addEventListener('click', () => {
    document.getElementById('large-image').src = button.dataset.zoom;
    dialog.showModal();
  });
}
document.getElementById('close-image').addEventListener('click', () => dialog.close());
dialog.addEventListener('click', event => { if (event.target === dialog) dialog.close(); });
for (const button of document.querySelectorAll('[data-copy]')) {
  button.addEventListener('click', async () => {
    const content = document.getElementById(button.dataset.copy);
    try {
      await navigator.clipboard.writeText(content.textContent);
      button.textContent = 'Copied';
      document.getElementById('copy-status').textContent = 'Commands copied.';
    } catch {
      const range = document.createRange();
      range.selectNodeContents(content);
      const selection = window.getSelection();
      selection.removeAllRanges(); selection.addRange(range);
      document.getElementById('copy-status').textContent = 'Select and copy the highlighted commands.';
      button.textContent = 'Select & copy';
    }
  });
}
fetch('project.json').then(response => {
  if (!response.ok) throw new Error('Metadata unavailable');
  return response.json();
}).then(project => {
  if (project.repository_url) {
    const url = new URL(project.repository_url);
    if (url.protocol === 'https:' && url.hostname === 'github.com') {
      for (const anchor of document.querySelectorAll('[data-source]')) anchor.href = url.href;
    }
  }
}).catch(() => {}); // Packaged source links still work when metadata is unavailable.

// Independently drawn ANSI F75. The board is visual only, never a USB device.
const keybed = document.getElementById('keybed');
const key = (label, width = '', extra = '') => ({ label, width, extra });
const rows = [
  [key('Esc', '', 'esc'), ...Array.from({ length: 12 }, (_, i) => key(`F${i + 1}`)), key('', '', 'key-knob')],
  ['`', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '−', '='].map(label => key(label)).concat([key('Backspace', 'w2'), key('Del')]),
  [key('Tab', 'w15'), ...'QWERTYUIOP'.split('').map(label => key(label)), key('['), key(']'), key('\\', 'w15'), key('PgUp')],
  [key('Caps', 'w175'), ...'ASDFGHJKL'.split('').map(label => key(label)), key(';'), key("'"), key('Enter', 'w225'), key('PgDn')],
  [key('Shift', 'w225'), ...'ZXCVBNM'.split('').map(label => key(label)), key(','), key('.'), key('/'), key('Shift', 'w175'), key('↑'), key('End')],
  [key('Ctrl', 'w125'), key('Win', 'w125'), key('Alt', 'w125'), key('Space', 'w625'), key('Fn', 'w125'), key('Ctrl', 'w125'), key('', '', 'keyspace'), key('←'), key('↓'), key('→')],
];
for (const [rowIndex, data] of rows.entries()) {
  const row = document.createElement('div'); row.className = 'key-row' + (rowIndex === 0 ? ' function' : '');
  for (const item of data) {
    const cap = document.createElement('span'); cap.className = ['keycap', item.width, item.extra].filter(Boolean).join(' '); cap.textContent = item.label;
    row.append(cap);
  }
  keybed.append(row);
}
const keyboard = document.getElementById('keyboard-object');
const keycaps = [...keybed.querySelectorAll('.keycap:not(.keyspace):not(.key-knob)')];
const palettes = { ice: 174, citrus: 82, ember: 26 };
function paintKeyboard() {
  const base = palettes[keyboard.dataset.palette] ?? 174;
  keycaps.forEach((cap, index) => {
    const hue = keyboard.dataset.effect === 'spectrum' ? (base + index * 9) % 360 : base;
    cap.style.setProperty('--key-hue', String(hue));
  });
}
function lightingPulse(center = 30) {
  if (motionPreference.matches) return;
  motionAnimate(keycaps, { opacity: [1, .35, 1] }, {
    duration: .75, delay: index => Math.abs(index - center) * .009, ease: 'easeInOut',
  });
}
for (const button of document.querySelectorAll('[data-effect]')) {
  if (button.tagName !== 'BUTTON') continue;
  button.addEventListener('click', () => {
    keyboard.dataset.effect = button.dataset.effect;
    for (const option of document.querySelectorAll('button[data-effect]')) option.setAttribute('aria-pressed', String(option === button));
    paintKeyboard();
    if (keyboard.dataset.effect === 'ripple') lightingPulse();
    else motionAnimate(keycaps, { opacity: [.5, 1] }, { duration: .35, delay: window.Motion?.stagger?.(.003) });
  });
}
for (const button of document.querySelectorAll('button[data-palette]')) {
  button.addEventListener('click', () => {
    keyboard.dataset.palette = button.dataset.palette;
    for (const option of document.querySelectorAll('button[data-palette]')) option.setAttribute('aria-pressed', String(option === button));
    paintKeyboard();
    if (keyboard.dataset.effect === 'ripple') lightingPulse(25);
  });
}
keyboard.addEventListener('click', event => {
  const index = keycaps.indexOf(event.target.closest('.keycap'));
  lightingPulse(index >= 0 ? index : 30);
});
paintKeyboard();

function configureMotion() {
  stopInView?.();
  for (const control of animationRuns) { control.complete?.(); control.stop?.(); }
  animationRuns.clear();
  if (motionPreference.matches || !window.Motion?.animate) {
    document.documentElement.classList.remove('motion-enabled');
    document.querySelectorAll('[data-reveal]').forEach(element => { element.style.opacity = ''; element.style.transform = ''; });
    return;
  }
  document.documentElement.classList.add('motion-enabled');
  const targets = document.querySelectorAll('.section-top, .platform-selector, .download-row, .hardware-summary, .support-table, .steps>li, .faq-list>details');
  targets.forEach(element => element.setAttribute('data-reveal', ''));
  stopInView = window.Motion.inView(targets, element => {
    if (element.contains(document.activeElement)) return;
    motionAnimate(element, { opacity: [0, 1], y: [20, 0] }, { duration: .65, ease: [.22, 1, .36, 1] });
  }, { amount: .12, margin: '0px 0px -25px 0px' });
}
configureMotion();
if (!motionPreference.matches && window.Motion?.animate) {
  motionAnimate('.hero-heading .eyebrow, .hero h1, .hero-intro', { opacity: [0, 1], y: [22, 0] }, { duration: .9, delay: window.Motion.stagger(.1), ease: [.22, 1, .36, 1] });
  motionAnimate('.keyboard-perspective', { opacity: [0, 1], y: [32, 0] }, { duration: 1.1, delay: .15, ease: [.22, 1, .36, 1] });
}
motionPreference.addEventListener('change', configureMotion);
const stage = document.querySelector('.keyboard-perspective');
let pointerFrame;
stage.addEventListener('pointermove', event => {
  if (motionPreference.matches || event.pointerType === 'touch') return;
  cancelAnimationFrame(pointerFrame);
  pointerFrame = requestAnimationFrame(() => {
    const rect = stage.getBoundingClientRect(), x = (event.clientX - rect.left) / rect.width - .5, y = (event.clientY - rect.top) / rect.height - .5;
    keyboard.style.transform = `rotateX(${12 - y * 5}deg) rotateZ(${-9 + x * 3}deg)`;
  });
});
stage.addEventListener('pointerleave', event => {
  if (event.pointerType === 'touch') return;
  cancelAnimationFrame(pointerFrame);
  if (!motionPreference.matches) motionAnimate(keyboard, { rotateX: 12, rotateZ: -9 }, { duration: .5, ease: [.22, 1, .36, 1] });
});

const navLinks = [...document.querySelectorAll('.header nav a[href^="#"]')];
let navFrame;
function updateNavigation() {
  const current = [...navLinks].reverse().find(link => {
    const section = document.querySelector(link.getAttribute('href'));
    return section && section.getBoundingClientRect().top <= 135;
  });
  for (const link of navLinks) {
    if (link === current) link.setAttribute('aria-current', 'location');
    else link.removeAttribute('aria-current');
  }
}
window.addEventListener('scroll', () => {
  if (navFrame) return;
  navFrame = requestAnimationFrame(() => { navFrame = 0; updateNavigation(); });
}, { passive: true });
updateNavigation();
