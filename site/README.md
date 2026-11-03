# AULA Studio landing page

Buildless HTML/CSS/JavaScript, real standalone-artwork app screenshots and local downloads. Instrument Sans and IBM Plex Mono are self-hosted with their OFL licenses and recorded source hashes. No font CDN, analytics, account system, WebHID or keyboard access.

```sh
python3 scripts/build-site.py --include-binaries
python3 scripts/check-publication.py
python3 -m http.server --bind 127.0.0.1 8780 --directory site/dist
```

Omit --include-binaries for source-only previews or CI without native artifacts. Open http://127.0.0.1:8780. Stop the server when finished. Optional actual WebKit QA: python3 scripts/site-smoke.py (requires installed WebKitWebDriver, MiniBrowser and Xvfb).

Edit index.html, style.css and app.js. Metadata defaults to no public repository URL. Pass only a real published repository/site URL to the builder. Working local source/binary downloads remain available before publication. The site ZIP under dist/publication/ is ready for static hosting.

See [docs/PUBLISHING.md](../docs/PUBLISHING.md) for repository and deployment steps. The builder never pushes or publishes. Current screenshots demonstrate the desktop app's demo mode and do not imply that every hardware feature is verified.

Typography and motion preserve the neutral editorial layout. Initial content uses a short staggered entrance; sections reveal once through IntersectionObserver/Web Animations, with link/button/tab/FAQ feedback. Main sections and download links are visible by default if JavaScript/motion APIs are unavailable; screenshot tabs, zoom and clipboard buttons require JavaScript. No perpetual animation or scroll event loop is used. Actual prefers-reduced-motion disables CSS and JavaScript animation, including changes while the page is open.

Browser checks:

```sh
python3 scripts/site-smoke.py
python3 scripts/site-smoke.py --reduced-motion
python3 scripts/site-smoke.py --live
python3 scripts/site-smoke.py --live --reduced-motion
```

The QA process uses project-local GTK settings to test the real browser preference without changing host preferences. Tests check actual loaded fonts, animation behavior, section navigation, responsive layouts and existing controls.
