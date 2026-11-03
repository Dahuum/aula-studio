# Third-party notices

AULA Studio's original code is MIT licensed. Dependencies retain their own licenses. Cargo.lock and app/package-lock.json pin source dependencies; no proprietary Windows software or artwork is included.

The public binary export generates THIRD_PARTY_NOTICES.txt from the cached Linux dependency graph and frontend package licenses. It conservatively includes build-time dependencies, unmodified upstream license texts and versioned source URLs. System GTK/WebKitGTK libraries are dynamically linked and are not bundled.

Some published crates omit their upstream license file. Exact upstream texts are retained under third_party/licenses/ with source commit URLs in index.json. The selectors crate specifies MPL 2.0 in its source headers; the retained full license comes from Mozilla. The plist crate spells its license filename LICENCE, which the exporter also recognizes.

The binary exporter stops if a resolved dependency has no available license text. It never fetches licenses automatically. Maintainers should review changes when updating dependencies. This inventory does not relicense any dependency or assert that every dependency uses MIT.

Two MIT F87-family fixture buffers retain fixtures/keymap/LICENSE and attribution in SOURCES.md. They are developer fixtures and never F75 factory defaults.
