# Third-party notices

AULA Studio's original code is MIT licensed. Dependencies retain their own licenses. Cargo.lock and app/package-lock.json pin source dependencies; no proprietary Windows software or artwork is included.

The public binary export generates THIRD_PARTY_NOTICES.txt from the cached dependency graph for the actual native target and frontend package licenses. It conservatively includes build-time dependencies, unmodified upstream license texts and versioned source URLs. System GTK/WebKitGTK libraries are dynamically linked and are not bundled.

Some published crates omit their upstream license file. Exact upstream texts are retained under third_party/licenses/ with source commit URLs in index.json. The selectors crate specifies MPL 2.0 in its source headers; the retained full license comes from Mozilla. The plist crate spells its license filename LICENCE, which the exporter also recognizes.

The binary exporter stops if a resolved dependency has no available license text. It never fetches licenses automatically. Maintainers should review changes when updating dependencies. This inventory does not relicense any dependency or assert that every dependency uses MIT.

Two MIT F87-family fixture buffers retain fixtures/keymap/LICENSE and attribution in SOURCES.md. They are developer fixtures and never F75 factory defaults.

## Landing-page fonts

The website self-hosts unmodified Space Grotesk variable WOFF2, Instrument Sans variable WOFF2 and IBM Plex Mono Regular WOFF2. Both are SIL Open Font License 1.1; full upstream copyright/license files ship alongside them under site/fonts/ and in the deployed assets/fonts/. These fonts are website assets, not proprietary keyboard artwork. No external font CDN or font API is contacted by visitors.

[Instrument Sans upstream](https://github.com/Instrument/instrument-sans) at commit7fa22308a3d0c94ee2b3cd537a1196b65db34a3e; [IBM Plex upstream](https://github.com/IBM/plex) at commit71f558992315a12ca1f06a2215d57962a1992162. Exact download URLs, SHA256 digests and sizes are recorded in site/fonts/sources.json. Font files were copied unchanged; licenses are retained in full.

The Space Grotesk source commit and exact font/license hashes are recorded alongside the other fonts in `site/fonts/sources.json`. Full upstream OFL bytes are preserved, including their original line endings.

## Landing-page animation

Motion 14.9.1 is self-hosted from its published npm distribution, under MIT. The unmodified JavaScript, complete license and source URL/SHA256 are in `site/vendor/`. The visitor does not need npm or a CDN. Reduced-motion preferences disable animation and pointer tilt.

## macOS notices

The objc2 workspace publishes several crates without their license declaration. The exact upstream `LICENSE.md` at each crate's `.cargo_vcs_info.json` commit is retained under `third_party/licenses/`. The index records those pinned URLs, hashes and crate author metadata. The export retains the upstream SDK-derived-code explanation without rewriting its licensing claims.

The Windows WebView2 wrapper is MIT licensed; its bundled Microsoft WebView2 loader has a separate BSD license. Both the SDK1.0.3800.47 license and notices are retained verbatim and included in Windows THIRD_PARTY_NOTICES. The exact NuGet source/package hash and byte-identical static-loader evidence are recorded in the license index. This does not relicense the SDK as original project code.
