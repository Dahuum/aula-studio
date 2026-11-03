# Third-party notices

AULA Studio's original code is MIT licensed. Dependencies retain their own licenses. Cargo.lock and app/package-lock.json pin source dependencies; no proprietary Windows software or artwork is included.

The public binary export generates THIRD_PARTY_NOTICES.txt from the cached Linux dependency graph and frontend package licenses. It conservatively includes build-time dependencies, unmodified upstream license texts and versioned source URLs. System GTK/WebKitGTK libraries are dynamically linked and are not bundled.

Some published crates omit their upstream license file. Exact upstream texts are retained under third_party/licenses/ with source commit URLs in index.json. The selectors crate specifies MPL 2.0 in its source headers; the retained full license comes from Mozilla. The plist crate spells its license filename LICENCE, which the exporter also recognizes.

The binary exporter stops if a resolved dependency has no available license text. It never fetches licenses automatically. Maintainers should review changes when updating dependencies. This inventory does not relicense any dependency or assert that every dependency uses MIT.

Two MIT F87-family fixture buffers retain fixtures/keymap/LICENSE and attribution in SOURCES.md. They are developer fixtures and never F75 factory defaults.

## Landing-page fonts

The website self-hosts unmodified Instrument Sans variable WOFF2 and IBM Plex Mono Regular WOFF2. Both are SIL Open Font License 1.1; full upstream copyright/license files ship alongside them under site/fonts/ and in the deployed assets/fonts/. These fonts are website assets, not proprietary keyboard artwork. No external font CDN or font API is contacted by visitors.

[Instrument Sans upstream](https://github.com/Instrument/instrument-sans) at commit7fa22308a3d0c94ee2b3cd537a1196b65db34a3e; [IBM Plex upstream](https://github.com/IBM/plex) at commit71f558992315a12ca1f06a2215d57962a1992162. Exact download URLs, SHA256 digests and sizes are recorded in site/fonts/sources.json. Font files were copied unchanged; licenses are retained in full.
