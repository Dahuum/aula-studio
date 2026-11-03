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
