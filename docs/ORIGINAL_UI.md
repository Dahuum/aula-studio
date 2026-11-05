# Original UI reconstruction — 0.4.1

The user requested the original Windows UI exactly. Version0.4 replaces the earlier mint dashboard with the original app's measured layout and connects it to the existing native configuration service. This is a static-evidence reconstruction; exact pixel identity across all screens and Windows font rendering remains unverified.

## Evidence and geometry

| Element | Supplied evidence | Implementation |
|---|---|---|
| Canvas | skins/main_nr.png,1200×720; main constructor459390 |1200×720 native window with original-position custom titlebar; scales to tiled windows |
| Header / rail / context | Background boundaries y68, x66, x285; CFG ptNavi120,12 |68px header,66px icon rail,219px context |
| Palette | CFG txt180/180/180, active10/147/255, button83, list33, active0/93/145 |Gray controls and blue active state |
| Effects | KB.ini effect definitions and localized labels |18 named presets, Self-define21, OFF0, same order |
| Keyboard | Supplied keyimg679×304, native keyboard panel constructor431580 |679×304 independently drawn ANSI keyboard; supplied image is French ISO and is not used as ANSI |
| Color wheel | skins/colorwheel.png116×117 |Local image pixel sampling when available; independent HSV wheel otherwise |
| Navigation | skins/navi50x50 normal/active sprites |Optional local artwork; independent SVG otherwise |
| Minimize/close | Main constructor1115,15 and1153,15 |Actions only on the invoking native window |

REA/Ghidra UI evidence is exported in ignored analysis/ui-static/evidence.json (9 records). Main constructor Evidence ev_19a9a056f2b465632347998f206c5b0bd9035a309b13a26c8df1247301f5a008, skin loader ev_d3989e6e183261befbe0f0b7c0cfef95c1ffb9d3f04832e7a7b5dc531279506a, keyboard panel ev_59a26c98a9aaefa083d9ff6538bd37b4418e708cb59f61340b4fcde427c8d220. The unresolved lighting-panel query remains recorded, not converted to invented coordinates. A public older lighting screenshot corroborates the general layout, not this version's exact runtime: [VRKAA software guide](https://vrkaa.com/blogs/before-you-buy/aula-f75-software-download-setup).

## Private original artwork

No supplied image, EXE, DLL, pseudocode or logo is in Git or release packages. When launched within this project, the native app can read the already-existing analysis/extracted/app tree near the executable. Each allowlisted PNG has an expected dimension and1MiB limit. No frontend-controlled arbitrary path, Windows execution or network fetch is involved.

For another local extraction, use:

```sh
AULA_ORIGINAL_SKIN_DIR=/absolute/path/to/extracted/app ./target/release/aula-studio
```

The directory must contain skins/main_nr.png and the other optional skins. To validate standalone rendering:

```sh
AULA_NO_ORIGINAL_SKIN=1 ./target/release/aula-studio
python3 scripts/native-demo-smoke.py --fallback
```

The shipped CSS/SVG fallback and ANSI key drawing are original project code. Public screenshots in docs/screenshots use only that fallback, explicit demo state and generated test data.

## Connected controls and differences

Effect list, RGB wheel, RGB sliders/numeric fields, hex entry, brightness/speed, Colourful, preset/custom colors, per-key selections and base/FN1 navigation change real local profiles. Apply in hardware mode still requires its scoped native review and separate Write. Custom colors persist through Rust XDG preferences, maximum10. Demo is explicitly labeled and never configures hardware.

Config opens profiles. Main rail opens Global, Key setting, Lighting, Self-define and Macro. Ellipsis opens Linux diagnostics/About. Save/Apply have distinct accessible names/tooltips. Read/recover tools and physical-write warnings remain available. Focused macro recording/editing, native reviews, profiles and Linux-specific diagnostics use the original palette but cannot yet be claimed as exact copies of every original dialog.

Unsupported macro upload, debounce/polling/OS settings and wireless configuration are unavailable rather than simulated as hardware features. Regional ANSI/French key roles deliberately follow the actual saved ANSI bank, not incompatible supplied ISO artwork. Animation previews are illustrative.


##0.4.1 visual refinement and remaining mismatch

The user correctly reports that the UI is still different. Initial0.4 used WebKit's native range handles and shallow CSS keycaps. Comparing the actual supplied slider.png12×12, slider2.png14×7 and check_nr/check_dn13×13 identifies these differences.0.4.1 allowlists those optional local sprites, implements3px tracks with correctly sized round/triangular handles and checkbox faces, and adds independent keycap bevels plus the visible USB connector. No proprietary file is copied into Git or release assets. ANSI geometry remains independently drawn instead of incorrectly presenting French ISO artwork.

Remaining: exact keyboard art/shading, font metrics, detailed remapping action picker, original macro editor layout, original global/settings and Config dialogs. The public screenshot is only a corroborating older build, not an exact screenshot of every panel in this installer. These differences must be resolved with available static UI construction/layout evidence and comparisons; a claim of complete original-UI identity is premature.


Additional verified icon correction: the earlier UI wrongly assigned navi7 (cloud) to Per-Key RGB and navi5 (RGB/ring) to Macros. Direct comparison of supplied sprites with the public original lighting screen identifies navi5 for RGB and navi4 for Macros; these are corrected, and unused account/cloud sprites6/7 are no longer read. CFG EnableYun=0 separately supports excluding the cloud entry from the supplied original UI.

## Macro sidebar follow-up — unreleased0.6

REA native constructor0043c420 establishes local tree width218, tree top70,28-pixel list rows and six18-pixel icons at8-pixel gaps. The toolbar now follows its recovered left-to-right Export/Import/Rename/Copy/Delete/New order and performs the real local operations. Normal original icon PNGs may be read only from the existing bounded private extraction; independent SVGs ship instead. Key edits, reorder/insertion/duplicate and100-event paging are functional. A native click-area regression was caught and fixed before handoff. Parent-coordinate/font/dialog fidelity and the separate46-pixel original event-tree presentation remain incomplete; this is not a claim of full visual parity. See [macro evidence](MACRO_FORMAT.md).

## Layer navigation follow-up — unreleased0.6

The native CKeyDlg constructor is0041ebd0 (template103), initialized by0041f420 and laid out by0041f730. GetRuntimeClass0041ebc0 was initially a misleading factory lead. Static construction establishes85×45 layer buttons with1px gaps, ending10px above the keyboard. FnAlign defaults to1 in parser00411830 (assembly004123cd–004123ee), placing the last button at the rightmost visible key. Default/FN1 now use this geometry with actual existing layer navigation; unsupported FN2/Tap are not offered. Keyboard rectangles are centralized so alignment follows the independently drawn ANSI layout. The conflicting inherited10px keyboard margin is removed.

Native WebKit tests verify scaled dimensions, right alignment, the10px gap, protected source controls, working layers and unobstructed hit targets while a success notification is open. Notices now occupy spare header space. F75 regression, all ten model workspaces and eight startup cases pass for this change; no physical write occurred. MS Shell Dlg is the native default font alias (00457e90); runtime Windows substitution and exact pixel matching remain unverified. Detailed action-picker reconstruction remains unfinished.

Primary evidence: constructor ev_6bcb12574142649e21a78c411e5e00a6c81bc0f55211c392fa7ae9cab3aa7b15; layout ev_6e96c8c337234ccafce5f1ef377491eb00f5cd7c38bea5b0d7f73369fcebc14d; assembly ev_c1b783ea9c6931a110e3252b81827c4c0d7a872e4db74888d3833aa46d8b72bc; alignment parser assembly ev_0ec019e25d467d73d67df30b78e30fa90269947c4f5ed7442e7963a65d24cca9. Original decompiler output and artwork remain private; shipping code is independently implemented.
