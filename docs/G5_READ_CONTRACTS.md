# F108 and H108 read contracts — unreleased 0.6

Status: **IMPLEMENTED / OFFLINE TESTED**. Neither model has been physically queried or configured here. These adapters permit an explicit, bounded wired read; they provide no configuration setter, restore, firmware, macro or wireless operation. [F87](F87_READ_CONTRACT.md) has a separately established, older transfer format.

## Independent targets and selection

| Model | Native executable SHA256 | Required six-byte identity |
| --- | --- | --- |
| F108 | `92821d43a829b3c44e9120f67f20bacd44763fdfd1d01c948161323a477ee279` | `03 00 00 00 01 ba` |
| H108 | `05bb069a9f0a24ec42a9ad85a6bf5371409304d92652753703e0e8ea59c99f37` | `03 00 00 00 03 7c` |

Each package independently supplies Fw24, CRC1 and RGB/GameRGBIndex 0,1,2. H108 explicitly sets MatrixLen128; F108 omits it and its native parser defaults to128. Both omit SubFw; their parsers default to0 and select standard FillCfg. Alternate low-delay configuration branches are excluded. Different identities in other resources are not automatically accepted.

Both native selectors check VID258a/PID010c, usage pageFF00/usage1, feature report520B and the exact six-byte identity. Linux additionally restricts access to interface1 and checks the report descriptor, fresh connection epoch and operation locks. A shared USB ID or product name never grants configuration access. The existing original F75 callback retains its separate CD identity gate.

## Exact read requests

All reports contain520 bytes including report ID06. The following eight-byte headers are followed by zero padding. SetFeature submits the read selector; GetFeature retrieves its bounded response. No arbitrary packet API or automatic query on startup/model selection is exposed.

| Scope | Header | Payload |
| --- | --- | ---: |
| Identity | `06 82 01 00 01 00 06 00` |6 |
| Configuration | `06 84 00 00 01 00 80 00` |128 |
| Effect palette | `06 8a 00 00 01 00 00 02` |512 |
| Base matrix, target0 | `06 83 00 00 01 00 00 02` |512 |
| Custom RGB, group0 | `06 86 00 00 01 00 7a 01` |378 |

The keymap is128 four-byte records. Independently, each ApplySetting passes the explicit literal0x17a for the RGB transfer and uses126-byte plane strides. The RGB read size is reconstructed from this transfer and the paired generic GetGame method, rather than a recovered live GetGame caller or a hardware trace. Do not substitute the old F87's384-byte transfer. No checksum is invented.

Responses must contain a complete520-byte report or exactly header+declared payload, with matching ID, command/read counterpart, target and chunk/length fields. Config must have FF FF at56/57 and5A A5 at126/127. Palette must have5A A5 at506/507; base matrix at510/511. Native standard FillCfg and ApplySetting establish these markers; they are conservative acceptance rules, not cryptographic authentication. A missing config-tail marker stops the transaction before requesting the palette. Any transport error, partial/incorrect reply or foreign identity aborts without retries or accepted partial captures. Unknown firmware is rejected rather than normalized.

Captures retain raw config/palette/base/RGB payloads and exact model/source under private immutable `model-captures/` files (0600, new directories0700). They are separate from profiles and F75 backups. Export contains bounded keyboard configuration; review before sharing. Local draft projection includes only known lighting fields/resource LED indices. Assignments, Fn layers and macros remain raw, unprojected data. Unsupported scalar/effect/palette selections reject projection without discarding the capture. No persistence or hardware-success claim follows from a generated draft.

## REA evidence

REA6.7.0 with Ghidra12.1.4 analyzed each executable independently, statically. Full records remain private in `analysis/f108-read-contract/` and `analysis/h108-read-contract/`; both sessions were exported and closed. No supplied executable or JavaScript bootstrap was executed.

| Operation | F108 function / REA Evidence | H108 function / REA Evidence |
| --- | --- | --- |
| Device selector |0048e730 / `ev_3afb695eba856cb869fe6f653fcfdb5de7e8fc91fa708f973f128050cea6d46d` |00498e30 / `ev_ddbf094b7cc13f8bf46ed1731034cc9ca882b916b2c213514555748d17845373` |
| Identity |0048fb50 / `ev_e619bc9de03448e5129ff6da5b05c9db95ff2ee8c0be774efeacd63ffaccee7f` |0049a2b0 / `ev_89c81e0a9eb8c7934254f89dd86ff491859e91847910500bea632dda6ce51a3d` |
| Config |0048f210 / `ev_91610ee546aced15f7b48850007f3c9e2295863b4546bafe1bc725c2ffde72e6` |00499970 / `ev_3c6472636896bae15b9d2a260a8fad6b4942c77e434ba46953bd35b4f00714bc` |
| Palette |0048f8b0 / `ev_9bf0cda3e1eb2d86a923ce9503479a835073c201a41cf19e089c3404eca2b8da` |0049a010 / `ev_3f194be721c1e47faccd2a4683b88da748d0b4071f34ede475bd7e76d67fc9d5` |
| Base matrix |0048ed60 / `ev_8b324accc52d8e30273311e1c1ee31abd458271fc027f912fdaea526c12a39d5` |004994c0 / `ev_8c493b62ac09f99b822de86f698060410a2df93be63e54b90cf4b1c253f04d08` |
| RGB |0048f470 / `ev_2a243f025279bb7c7e9e218eb0cbb080b8a882b34c5b1b9239d480e368d48f78` |00499bd0 / `ev_6bc2aea210f501b46f50fe032f3da555fd149a6c07522573650a4f5a1166987c` |
| Apply / transfer lengths / palette marker |00491250 / `ev_49364dd26b6fa7b51b66beed6a70d336afba60a1a221be8c7ebf826d1d4e0bb4` |0049bea0 / `ev_f06357ed47a1f4fcc5e18957417a61443b11979c2d6184d7a21b276fdb0b07e8` |
| Standard FillCfg |00490ed0 / `ev_598cdb35d76488e77f0a0effa66437d35669b1e48b693e523dd6f9ebb97386d7` |0049ba70 / `ev_645b19556bc769ccdca07987ba36bac1db2cb074601313ddace482c43c6e6f94` |
| MatrixLen / SubFw parser |00411809 / `ev_63f2bcdb20032bab52a2afb5cd4eaaa1de54e89d69914c4f5cd626a5deb32dd4` |004122ff / `ev_1ed25f3066082203f5b4f758384745d5c51dead1176b0ac48283cf44a69e394b` |
| FillMatrix / CRC |00490830 / `ev_a411241c06d60ac50561cba7b469203be899e918d478e38819cca64dcbabb68e` |0049b250 / `ev_e3dac40248271bab8db8933fc46a8961fe2f0668cba04cb7c067d9b5b2d9a281` |

Confidence: native packet constructors, selectors and transfer literals are **confirmed by official binary analysis**; the requested RGB read sizes are **inferred from paired native operations**; typed adapters/fixtures are **IMPLEMENTED / OFFLINE TESTED**. Physical response acceptance, visible geometry/channel correspondence, revision coverage, writes, macros and persistence are **BLOCKED ON HARDWARE VALIDATION** or remain unknown.

## Use and recovery

In the development native application: More views → Device library → the exact model → Experimental device captures. Physical reads require leaving F75 demo mode and an eligible wired interface; synthetic fixtures are explicitly labeled. The CLI supports `./target/release/aula read-model /dev/hidrawN f108` or `h108` when independently requested. Do not run these commands to probe an unknown device. No actual F108/H108 query occurred during development.

On a failed read, retain the error, wait for any blocked kernel operation to finish and inspect Diagnostics. Do not retry automatically or issue recovery/reset commands. No configuration restoration is necessary because these adapters contain no configuration setters. Further hardware validation should start with descriptors and the known bounded identity read on the exact revision.
