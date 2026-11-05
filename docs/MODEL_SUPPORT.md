# Adding keyboard models

An installer is enough to begin reverse engineering. It is not enough to declare a keyboard supported.

## Without the keyboard

Static REA/Ghidra analysis can identify USB identifiers, interface checks, layouts, effect lists, capability flags, packet constructors, configuration addresses, serializers, macro formats, checksums and persistence sequences when those exist in recoverable code/resources. Compare related installers and public traces. Build original encoders, decoders, layout definitions and fixture tests with evidence references. Never execute an unknown Windows binary on the Linux host or redistribute its code/artwork.

Encryption, packing, downloaded components, runtime negotiation and proprietary helper drivers can leave important gaps. Different hardware revisions may share a name or VID/PID; protocol-family similarities are evidence to investigate, not permission to write.

## Required evidence states

| State | Meaning |
| --- | --- |
| Researched | Static evidence and remaining unknowns documented; no support promise |
| Implemented / offline tested | Typed protocol and fixtures pass; no physical-device claim |
| Read verified | A volunteer has verified identification and bounded known reads on the exact revision |
| Write scope verified | Backed-up, explicitly authorized reversible operation, visible result/readback and restoration verified for that scope |
| Persistence verified | Explicitly tested after reconnection or power cycle; separate from readback |

No new model may inherit F75 write access merely because its USB ID or packet size matches. Each adapter needs descriptor/revision gates, bounded commands, a distinct capability definition, backup/readback behavior and fixtures. Unverified model adapters should remain discovery/offline-only until their safe write path is validated.

## Expansion sequence

1. Record each installer hash/version and model/revision claims; keep installers private.
2. Recover and compare its device selection, protocol, layouts and macro/persistence logic using REA and a native static engine.
3. Group genuinely matching protocol families, retaining per-model interface/layout/capability differences.
4. Implement a separate adapter and software tests. Document unsupported or unknown commands; never invent them.
5. Have willing owners verify exact device descriptors and known bounded reads. Sanitize public reports.
6. Obtain authorization for one reversible write scope at a time, verify readback/visible result, restore and document results. Do not flash firmware or probe arbitrary commands.
7. Publish a per-model/per-revision/per-connection compatibility matrix. Expand other brands using the same process.

Version0.5 adds an evidence-bounded model registry, model-specific local profile storage, resource-derived editors and offline S75 packet codecs. Twelve direct official Windows packages have been acquired and statically inspected; see [catalog](DOWNLOAD_CATALOG.md), [evidence](MULTI_MODEL_RESEARCH.md) and [local workspaces](MODEL_WORKSPACES.md). There is still one enabled hardware configuration adapter: original wired ANSI F75. New-model hardware access remains disabled. Bluetooth/dongle support and onboard macros remain separate research tasks. Community testing can supply hardware coverage without every model being owned by the maintainer; absent that testing, label the model researched/offline-tested rather than working.

## Unreleased 0.6

Three F87 variants, F108 and H108 have independently documented experimental read-only adapters. Explicit physical reads verify exact model identity before bounded configuration queries; no physical validation has occurred locally. All new-model writes remain disabled. See [F87](F87_READ_CONTRACT.md) and [F108/H108](G5_READ_CONTRACTS.md). Passive exact USB product-name matching narrows S75/F108Pro/F98Pro discovery candidates only and cannot grant hardware access.
