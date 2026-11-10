# AULA Studio landing page

Buildless, self-hosted HTML/CSS/JavaScript for the existing GitHub Pages address. The project builder replaces the version/download placeholders, copies the independently drawn application screenshots and supplies `project.json`. All runtime scripts, fonts and licenses are local; the page requires no CDN, analytics or account.

Visual direction: charcoal product stage, warm paper workspaces, restrained citrus accents, Space Grotesk display typography, Instrument Sans body text and IBM Plex Mono utility labels. The F75 stage is an independently drawn, explicitly illustrative ANSI keyboard. Lighting controls change that illustration only. Actual app screenshots are labeled demo mode.

Motion uses the real Motion JavaScript library14.9.1 (MIT), retained verbatim at `vendor/motion.js`. Its upstream license and exact source/hash inventory are alongside it. The builder should copy `site/vendor/` to `assets/vendor/`. Animation adds finite entrance/scroll/interaction transitions; all content remains visible without the library. Reduced-motion preferences disable animation and pointer tilt. The native package download controls are accessible tabs with keyboard navigation; all platform panels remain visible if JavaScript is unavailable.

Fonts are retained unmodified from their pinned upstream distributions, with OFL texts and SHA256/source metadata in `fonts/sources.json`. The builder copies that directory to `assets/fonts/`. Space Grotesk's pinned upstream revision is03507d024a01282884232081fc6011c09ff4e849. No proprietary AULA artwork or AI-generated bitmap is included.

Template download variables:

- `@@VERSION@@`: current cross-platform release/source version.
- `@@WINDOWS_URL@@`: actual published Windows NSIS package URL.
- `@@MAC_ARM_URL@@` / `@@MAC_INTEL_URL@@`: actual published macOS package URLs, architecture-specific or a verified universal artifact.
- `@@LINUX_VERSION@@`: independently selected existing Linux package version; avoids linking nonexistent Linux artifacts when native preview builds advance separately.

Do not enable links for artifacts that have not been generated and published. Unsigned native-preview and unverified physical-USB labels must match the release's actual capabilities. Build and visual QA are owned by the project scripts; this source directory does not publish itself.
