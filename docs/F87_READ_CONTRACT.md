# F87 read adapter — unreleased0.6 development

Status: **IMPLEMENTED / OFFLINE TESTED**. No F87 physical validation has occurred. This adapter exposes explicit, bounded read-only captures for the three variants in the acquired F87 installer. There are no configuration setters, macro queries, wireless queries or restoration operations in this adapter.

## Evidence and device gate

Target `OemDrv.exe` SHA256 `c46e6cad5ad85addbaef4975d27721719aa567bcb27582312b797a7108ec5a38`. The original package resources declare Fw24, VID258a/PID010c and MatrixLen128. The native selector checks usagepageFF00/usage1, feature report520B, and compares a six-byte model identity. Linux access is additionally restricted to interface1. The interface-number restriction is conservative; the Windows selector's collection/report checks are the direct native evidence.

| Resource model | Required identity |
| --- | --- |
| F87 |03 00 00 00 00 8f |
| F87 Pro010b |03 00 00 00 01 0b |
| F87 Pro Wired |03 00 00 00 00 a9 |

A shared USB ID does not select a variant. The backend compares the live identity with the explicitly selected model before any configuration-bank query. F75, unknown identities, receivers, other models, changed device epochs and invalid descriptors are rejected. Existing F75 write access keeps its independent CD identity gate.

## Bounded read set

All requests are520B including reportID06, with eight-byte headers and zero padding. Each uses SetFeature to submit a documented read selector, then GetFeature. No arbitrary packet constructor is exposed by the service/CLI. No automatic reads occur on model selection or startup.

| Scope | Header | Payload | Native function |
| --- | --- | ---: | --- |
| Identity |06 82 01 00 01 00 06 00 |6 |00489a20, independently analyzed earlier |
| Configuration |06 84 00 00 01 00 80 00 |128 |00489150 |
| Effect palette |06 8a 00 00 01 00 00 02 |512 |00489810 |
| Base matrix, target0 |06 83 00 00 01 00 00 02 |512 |00488ca0 |
| Custom RGB group0 |06 86 00 00 01 00 80 01 |384 |004893c0; transfer-size reconstruction below |

The keymap512B follows MatrixLen128×4. The old F87 ApplySetting initializes its transfer count from that same resource and passes count×3 to SetGame, producing384B. The paired GetGame function accepts the caller's bounded transfer length. The384B read size is reconstructed from the native write transfer and paired read framing; it has not been confirmed by a physical F87 read or a recovered live GetGame caller.

**Keymap capacity and RGB geometry are distinct.** Despite transferring384B, this F87 ApplySetting packs colors using126-byte plane strides (ushort offsets0x3f/0x7e) and378-byte group strides. Decode only126 red +126 green +126 blue positions. Preserve the remaining six bytes without assigning LED meanings. A first implementation inferred128-byte planes from MatrixLen; it was corrected before physical use after tracing the parser and transfer call separately. Synthetic tests now anchor channel offsets at126/252 and verify preservation of a distinct six-byte tail. The supplied newer F75 program explicitly transfers378B; its behavior must not be substituted for the old F87 program's384B transfer.

## Responses and accepted captures

Accept only a complete520B feature report or exactly header+payload bytes, with matching report ID, command/read counterpart, target, chunk count/index and declared payload length. Never pad partial data. Fail on transport errors, malformed headers or disconnection, and do not query remaining banks after a rejected response.

The config sentinelFF FF at payload56/57 comes from FillCfg0048abf0. Palette5A A5 at506/507 is explicit in ApplySetting0048afa0. The matrix5A A5 at510/511 is placed by FillMatrix0048a6b0 with MatrixLen128/CRC1 and corroborated by the licensed public F87 templates. These checks reject untouched zero-filled request echoes. The F75-specific config-tail marker is not required for F87. A firmware lacking these observed format markers is conservatively rejected; there is no fallback that invents settings.

The four accepted bank payloads are retained unchanged under private `model-captures/`, with model ID, exact identity and source. Files are immutable by capture ID, atomic0600; new directories0700. They are not legacy F75 backups or model profiles and cannot be imported through either profile schema. Capture export contains bounded bank data; review it before sharing.

A server-created local lighting draft projects only supported effect/scalar values and known resource LED indices. Unsupported brightness/speed/effect values or nonzero palette selectors cause projection to fail while preserving the raw capture. Key assignments, Fn assignments and macros are excluded rather than inferred. Off/custom draft scalar defaults describe local intent; captures remain the raw evidence. Hardware persistence is not established by a read.

## Transport and application behavior

Native reads share the existing operation mutex, process advisory lock, device epoch checks and five-second outer deadline. Kernel GetFeature itself may remain blocked after the UI timeout; its lease remains locked until the worker finishes. No automatic retry, write or reset is used. A UI “ignore result” action only suppresses late UI callbacks; it does not cancel kernel IO or unlock the device.

GUI demo mode independently blocks physical captures before device access. Explicit synthetic packet fixtures exercise the same typed read/decoder/storage pipeline with source `simulated`, and are labeled accordingly. A simulated capture is never accepted as the result of the physical-read action. Browser preview cannot invoke native captures.

The native F87 panel provides capture selection, explicit read, synthetic fixtures, raw JSON export and lighting-only local-draft creation. CLI: `aula read-model /dev/hidrawN f87` (or the exact Pro resource ID). Additional-model configuration writes remain disabled.

## REA provenance

All new evidence is from the same target hash, REA6.7/Ghidra12.1.4, static only. Private full records are under analysis/f87-read-contract and are excluded from publication.

| Observation | REA Evidence |
| --- | --- |
| Get configuration00489150 |ev_fe1a874064e95a8583972eb424e4c4bbfd3a076cf0c33c46aa8c43b79a02b595 |
| Get RGB004893c0 |ev_d5c5277bfa2397d1555c2b495bbe01b56bf5ba55f611f034d9aef667b8bf4356 |
| Get matrix00488ca0 |ev_16f4bf5edd89971e93c6996b180083f320d6c1de3589e8ffcd376b8aa650d794 |
| Get palette00489810 |ev_990b41b8318d05d0f28f609fc5b4f86beeeaebd9d07cfbf7d906ff76f5a539ce |
| MatrixLen parser0041124d→config3024 |ev_4379047de7e42d380151d50b072b0915d8575852bb23c72b05e871613fd8c1f0 |
| Matrix construction/CRC0048a6b0 |ev_7beebff500bfd2c9d42a960162ce3e905cf4aaa8a66ba8a75f5a2748fce2dcd2 |
| Vtable at0060c81c |ev_90708a068cf0f239fdc63655c3552651ed3e50991682b9d45a5712fe0ebbf462 |
| ApplySetting0048afa0, earlier session |ev_2ffeb4eddef2bf45ee6f41b691ad37116369711f4289d9f6bc3f78d9f7ee21b6 |

The nearby SyncCfg0048c600 routine reads400B and contains mouse sensor logic; it was excluded from this keyboard contract. Shared function names are insufficient evidence of a keyboard operation. Physical response acceptance, revision behavior, visible RGB correspondence and any future write/persistence operation still need separate validation.
