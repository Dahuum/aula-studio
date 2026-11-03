# Official download catalog —2026-11-03

Source: [AULA download page](https://www.aulagaming.com/pages/download). This records the 31 entries in one captured page; it is not a claim to enumerate every device AULA has ever sold. The machine-readable [catalog](../data/download-catalog.json) includes source roles, URLs, acquisition decisions, byte sizes and SHA256 hashes.

## Scope and results

12 direct Windows configuration packages were downloaded privately: 8 keyboard installers, 3 mouse installers and the AULA Hub ZIP. 11 browser configurators, 6 hosted-link entries and 2 standalone firmware entries were excluded. The 6 hosted entries contain 5 unique URLs because F99 and F99/F99 Pro share a link. No hosted-link resolver, login, browser configurator or firmware installer was used.

The source HTML SHA256 is `fa39a4e15a0a6a73ef15c2eb5f240c6a873e068ac70659d4996d02720433e31e`. The binaries, extracted proprietary resources and full analysis evidence stay under ignored analysis/. None are distributed with source/releases. Downloads were HTTPS-only, size/time bounded, atomically saved and made non-executable. Windows code was never executed.

## Acquired direct packages

| Catalog entry | Category | Download | Bytes | SHA256 |
| --- | --- | --- | ---: | --- |
| AULA Hub Driver | software | [Direct source](https://www.aulastar.com/AULA%20HUB.zip) | 150455271 | `f18f4bfa2a6d66f24bed8d2c17b61eff8530ebc9ad0615344f72003f5bf9db08` |
| S99 Pro | keyboard | [Direct source](https://cdn.shopify.com/s/files/1/0695/6987/1965/files/AULA_S99_PRO_Driver.exe?v=1791883907) | 15839784 | `117d7cbded86c0ba63e190a1aee38c6ab9bb63aa5aba65a5cbc8f3269ffaf5c6` |
| F87/F87 Pro | keyboard | [Direct source](https://cdn.shopify.com/s/files/1/0695/6987/1965/files/AULA_F87_Software.exe?v=1756176168) | 3395537 | `a91db0e84dd6e677aaa1701a6c4851e71131b561815909aacfc6f8679d20fdee` |
| S75 Pro | keyboard | [Direct source](https://cdn.shopify.com/s/files/1/0695/6987/1965/files/AULA_S75_Pro_Software.exe?v=1756176170) | 8472376 | `b097008ff9aa4497ca36660e5c61574aca2847500c4263637d236cd92220686b` |
| F108 | keyboard | [Direct source](https://cdn.shopify.com/s/files/1/0695/6987/1965/files/AULA_F108_Software.exe?v=1756176166) | 2271344 | `423f3bab5b10daae865d69c95d58f7a8127f47360163a370fc966574631787aa` |
| F108 Pro | keyboard | [Direct source](https://cdn.shopify.com/s/files/1/0695/6987/1965/files/AULA_F108Pro_Software.exe?v=1756176171) | 10759872 | `566b634be9f065717164a76fd0bed24c8175fa587c350599a44648ad226ad36a` |
| F2088 | keyboard | [Direct source](https://cdn.shopify.com/s/files/1/0695/6987/1965/files/AULA_F2088_Driver_setup_v1.0.5.7_1_1.exe?v=1762134219) | 10074083 | `8731252ed943e177d659558aef0f0a091a1f630f474abfb35da9459b5b0ed3ee` |
| H108 | keyboard | [Direct source](https://cdn.shopify.com/s/files/1/0695/6987/1965/files/AULA_H108_Setup_v2.0_20251022_1.exe?v=1771334790) | 2411590 | `7d0367b01e40cd8fc101a66699cf9acb42cd5fdba04a2e2623d2ecbf283584c9` |
| F98 Pro | keyboard | [Direct source](https://cdn.shopify.com/s/files/1/0695/6987/1965/files/AULA_F98Pro_Software.exe?v=1756176508) | 6776456 | `d75ffce415aa10f291c7d26afb55e17f6a0308795b27efdabee6a38e510d5935` |
| SC620 | mouse | [Direct source](https://cdn.shopify.com/s/files/1/0695/6987/1965/files/AULA_SC620-V1.2.2.exe?v=1785234034) | 5064144 | `8c2537d687360a2922ec2b11e1710ba6b2612c8d8a3d4d60966fcad39cee9263` |
| SC580/580 X | mouse | [Direct source](https://cdn.shopify.com/s/files/1/0695/6987/1965/files/AULA_SC580_Software.exe?v=1756176167) | 2697653 | `a246a196be023714436d08430e337c03bf36f6c43d65838ae6819d2ab16c5b99` |
| S12 Pro | mouse | [Direct source](https://cdn.shopify.com/s/files/1/0695/6987/1965/files/AULA_S12PRO.exe?v=1762306665) | 5592630 | `2693eab2ec85723185f34c1be86dbd9b140fcf30876caee8c3bfda2e4c886fb0` |

## Static extraction coverage

- 10 installers were recognized and extracted using a project-local native innoextract utility with path/count/expansion preflight.
- AULA Hub ZIP contained a Windows NSIS installer. Static 7z extraction recovered its inner archive and Electron app.asar plus unpacked application resources. No Electron/Windows application was launched.
- F2088 was not recognized by innoextract or 7z in this pass. REA PE resource inspection succeeded, but recovery of the configuration application remains unresolved. This is not evidence that its protocol is understood.
- Bundled firmware updaters, filter drivers and helper executables were indexed only; they were never run or installed.
- Some vendor config XML has unescaped ampersands in HID identifiers. Inventory uses explicitly labeled permissive attribute tokenization; it does not claim those files are well-formed XML.

## Excluded entries

| Entry | Source role | Reason |
| --- | --- | --- |
| [AG60 Web Driver](https://hub.aulacn.com/) | web_driver | Browser-hosted configurator; user requested direct Windows downloads only. Not acquired. |
| [AG75 Web Driver](https://hub.aulacn.com/) | web_driver | Browser-hosted configurator; user requested direct Windows downloads only. Not acquired. |
| [NEOS 75](https://www.aulastar.com/aula-hub/) | web_driver | Browser-hosted configurator; user requested direct Windows downloads only. Not acquired. |
| [HERO75HE Web Driver](https://www.aulastar.com/aula-hub/) | web_driver | Browser-hosted configurator; user requested direct Windows downloads only. Not acquired. |
| [F75 Pro](https://www.mediafire.com/file/5whrw6wr1bdpst0/AULA+F75+Setup+v2.0+20260514.exe/file) | driver | Hosted file-service landing page, not a direct artifact URL. Not acquired; no landing-page scraping or bypass. |
| [MINI 60HE](https://www.aulastar.com/aula-hub/) | web_driver | Browser-hosted configurator; user requested direct Windows downloads only. Not acquired. |
| [F75HE](https://www.aulastar.com/aula-hub/) | web_driver | Browser-hosted configurator; user requested direct Windows downloads only. Not acquired. |
| [F99](https://www.mediafire.com/file/qayztlag770zpqg/AULA+F99+Setup+v2.0+20260514.exe/file) | driver | Hosted file-service landing page, not a direct artifact URL. Not acquired; no landing-page scraping or bypass. |
| [F99/F99 Pro](https://www.mediafire.com/file/qayztlag770zpqg/AULA+F99+Setup+v2.0+20260514.exe/file) | driver | Hosted file-service landing page, not a direct artifact URL. Not acquired; no landing-page scraping or bypass. |
| [S98 Pro](https://www.mediafire.com/file_premium/uvvgwhufd9le821/AULA_S98Pro_softwaredriver.exe/file) | driver | Hosted file-service landing page, not a direct artifact URL. Not acquired; no landing-page scraping or bypass. |
| [L99](https://www.mediafire.com/file_premium/w97f57zhqp4pn8o/AULA_L99-EN_Driver-1.0.1.4.exe/file) | driver | Hosted file-service landing page, not a direct artifact URL. Not acquired; no landing-page scraping or bypass. |
| [WIN60HE Web Driver](https://www.aulastar.com/aula-hub/) | web_driver | Browser-hosted configurator; user requested direct Windows downloads only. Not acquired. |
| [Win60HE Pro Firmware](https://cdn.shopify.com/s/files/1/0695/6987/1965/files/WIN60_HE_PRO_V1.1.1.exe?v=1793252507) | firmware | Firmware-only package; outside the requested configuration-software scope. Not downloaded. |
| [WIN68HE Web Driver](https://www.aulastar.com/aula-hub/) | web_driver | Browser-hosted configurator; user requested direct Windows downloads only. Not acquired. |
| [HERO68HE](https://hero.aulastar.com) | web_driver | Browser-hosted configurator; user requested direct Windows downloads only. Not acquired. |
| [HERO 84HE](https://hero.aulastar.com/) | web_driver | Browser-hosted configurator; user requested direct Windows downloads only. Not acquired. |
| [F98X](https://www.mediafire.com/file_premium/u2fzfclkyhwv9bo/AULA__F98X__RGB%25E5%258F%2591%25E5%2585%2589_%25E4%25B8%2589%25E6%25A8%25A1%25E6%259C%25BA%25E6%25A2%25B0%25E9%2594%25AE%25E7%259B%2598_Driver-1.0.0.5%25281%2529.exe/file) | driver | Hosted file-service landing page, not a direct artifact URL. Not acquired; no landing-page scraping or bypass. |
| [F98 pro v3 Firmware](https://cdn.shopify.com/s/files/1/0695/6987/1965/files/F98PRO_V3__SI-2838_10000mAh___RGB_1.14_HFD80CP100_V1.29_20260402__0x4172.exe?v=1778760849) | firmware | Firmware-only package; outside the requested configuration-software scope. Not downloaded. |
| [G7 Pro](https://hub.aulacn.com/home) | web_driver | Browser-hosted configurator; user requested direct Windows downloads only. Not acquired. |

## Evidence limits

A download, installer label, configuration identifier, matching file size or shared VID/PID is not working Linux support. Each model needs protocol evidence, independent implementation/tests, interface/revision gates and separately documented physical validation before hardware-write claims. Mouse packages remain research-only here.

The F75 Pro catalog entry points to a hosted filename resembling the supplied original F75 installer. This naming ambiguity does not establish Pro compatibility. Several models share USB identifiers; names and IDs alone must not enable an F75 write path.

No additional model is marked hardware-verified by this catalog. See [the model-support policy](MODEL_SUPPORT.md).

Reproduction tools: scripts/research_catalog.py parses the archived page and acquires direct artifacts only with --download. scripts/research_extract.py verifies recorded hashes and performs bounded static extraction into analysis/. No proprietary payload is copied to data/ or docs/.
