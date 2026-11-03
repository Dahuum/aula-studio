# Publish a clean release

The working project contains local reverse-engineering material and agent notes. Publish the prepared snapshot, not an unreviewed copy of the whole working directory or private Git history.

## Prepare locally

```sh
python3 scripts/build-site.py --include-binaries
python3 scripts/check-publication.py
```

The binary option uses the existing trusted dist/aula-studio-0.4.2-linux-x86_64 binaries and cached dependency license texts. It does not rebuild, install or configure hardware. Without that option, the page includes a source download and does not need native build tools or binary artifacts.

Outputs are under dist/publication/ and site/dist/. The exporter allowlists source/docs, normalizes archive ownership/timestamps, rejects likely credentials, redacts home paths in documentation, retains attribution and excludes the installer, original artwork, analysis, raw backups, internal progress/continuation notes and Git history. It never pushes or uploads.

Inspect manifest.json, SHA256SUMS and the generated source tree before publishing. The portable archive preserves the tested executable bytes and adds public documentation/third-party notices. Its checksum differs from a private development archive containing internal notes; the exported PKGBUILD is updated to match the public archive.

## Repository

Create an empty public repository without a generated README/license/gitignore when connecting the existing project. For the sanitized snapshot, copy it to a separate publication checkout such as dist/public-checkout/ and initialize a new main branch there. Preserve the private working repository and history. Do not initialize Git inside the generated dist/publication/ cache; the exporter refuses to replace an existing Git repository.

Review the resulting commit before any push. Enable issues and private vulnerability reporting if desired. Attach the source archive, portable archive and SHA256SUMS to an early-access release rather than claiming stable or complete compatibility.

No remote is configured by the preparation scripts. Repository creation, pushing, releases and website publication are separate explicit actions.

## Landing page

The static site is buildless HTML/CSS/JavaScript with real standalone-artwork screenshots. Its working local downloads require no GitHub URL. After a repository exists, pass its real URL:

```sh
python3 scripts/build-site.py --repository-url https://github.com/OWNER/REPOSITORY
```

Pass --site-url only once the actual public address is known. It adds canonical/social metadata. Never claim a preview URL is public.

The Landing page GitHub Actions workflow is manual-only. Configure Pages to use GitHub Actions, then dispatch it. It builds a source-only landing page with a working source archive; binaries can be attached to a separately reviewed GitHub release. Actions versions are pinned to the upstream commit IDs checked during preparation. Remote CI/Pages jobs have not run merely because local preparation passed.

A Sites-hosted deployment is also possible from the static output. The user's original restriction on remote pushes still applies; no source synchronization or deployment occurs during local preparation.
