const tabs = [...document.querySelectorAll('[data-tab]')];
function activate(tab, focus = false) {
  for (const button of tabs) {
    const selected = button === tab;
    button.setAttribute('aria-selected', String(selected));
    button.tabIndex = selected ? 0 : -1;
    document.getElementById(button.getAttribute('aria-controls')).hidden = !selected;
  }
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
    const text = document.getElementById(button.dataset.copy).textContent;
    try {
      await navigator.clipboard.writeText(text);
      button.textContent = 'Copied';
      document.getElementById('copy-status').textContent = 'Commands copied.';
    } catch {
      const range = document.createRange();
      range.selectNodeContents(document.getElementById(button.dataset.copy));
      const selection = window.getSelection();
      selection.removeAllRanges(); selection.addRange(range);
      document.getElementById('copy-status').textContent = 'Select and copy the highlighted commands.';
      button.textContent = 'Select & copy';
    }
  });
}
fetch('project.json').then(response => {
  if (!response.ok) throw new Error('Project metadata unavailable');
  return response.json();
}).then(project => {
  if (project.repository_url) {
    const url = new URL(project.repository_url);
    if (url.protocol === 'https:' && url.hostname === 'github.com') {
      for (const anchor of document.querySelectorAll('[data-source]')) {
        anchor.href = url.href;
        if (anchor.classList.contains('header-source')) anchor.textContent = 'GitHub ↗';
      }
    }
  }
}).catch(() => {}); // Local source downloads remain functional without metadata.


// Every section is visible by default. If motion APIs or JS are unavailable,
// content and controls remain usable, with no hidden reveal classes to clear.
const motionPreference = window.matchMedia('(prefers-reduced-motion: reduce)');
const revealed = new WeakSet();
let revealObserver;
function configureMotion() {
  revealObserver?.disconnect();
  if (motionPreference.matches) {
    document.documentElement.classList.remove('motion-enabled');
    document.getAnimations?.().forEach(animation => animation.cancel());
    return;
  }
  document.documentElement.classList.add('motion-enabled');
  if (!('IntersectionObserver' in window) || !Element.prototype.animate) return;
  revealObserver = new IntersectionObserver(entries => {
    let order = 0;
    for (const entry of entries) {
      if (!entry.isIntersecting) continue;
      const target = entry.target;
      revealObserver.unobserve(target);
      if (revealed.has(target) || motionPreference.matches) continue;
      revealed.add(target);
      // Keyboard users should never have a focused control move away from them.
      if (target.contains(document.activeElement)) continue;
      target.animate([
        { opacity: 0, transform: 'translateY(18px)' },
        { opacity: 1, transform: 'translateY(0)' }
      ], { duration: 680, delay: Math.min(order++ * 65, 195),
           easing: 'cubic-bezier(.22,1,.36,1)', fill: 'backwards' });
    }
  }, { threshold: 0.12, rootMargin: '0px 0px -32px 0px' });
  for (const target of document.querySelectorAll('.section-intro, .download-row, .support-table, .workspace-content, .steps>li, .faq-list>details, .footer')) {
    if (!revealed.has(target)) revealObserver.observe(target);
  }
}
configureMotion();
motionPreference.addEventListener('change', configureMotion);

// The navigation follows the section in view without a per-frame scroll loop.
const sectionLinks = [...document.querySelectorAll('.header nav a[href^="#"]')];
if ('IntersectionObserver' in window) {
  let currentSection;
  let navigationFrame;
  function watchNavigation() {
    currentSection?.disconnect();
    const visible = new Set();
    for (const link of sectionLinks) link.removeAttribute('aria-current');
    currentSection = new IntersectionObserver(entries => {
      for (const entry of entries) {
        if (entry.isIntersecting) visible.add(entry.target);
        else visible.delete(entry.target);
      }
      const current = [...visible].sort((a,b) => a.getBoundingClientRect().top-b.getBoundingClientRect().top)[0];
      for (const link of sectionLinks) {
        if (current && link.hash === '#'+current.id) link.setAttribute('aria-current', 'location');
        else link.removeAttribute('aria-current');
      }
    // Pixel margins follow viewport height, including on wide, short windows.
    }, { rootMargin: `-${Math.round(innerHeight*.12)}px 0px -${Math.round(innerHeight*.55)}px 0px` });
    for (const link of sectionLinks) {
      const section = document.querySelector(link.hash);
      if (section) currentSection.observe(section);
    }
  }
  watchNavigation();
  window.addEventListener('resize', () => {
    cancelAnimationFrame(navigationFrame);
    navigationFrame = requestAnimationFrame(watchNavigation);
  }, { passive: true });
}
