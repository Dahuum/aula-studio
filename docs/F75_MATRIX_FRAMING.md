# F75 matrix framing cross-check

Status: native static evidence and previously verified physical reads. No new hardware operations or keymap writes were performed for this audit.

The supplied original package's `Dev/kb/fr/KB.ini` declares **MatrixLen128**, CRC1. Its126-position RGB planes are a separate size. A preliminary concern based on a126-slot keymap premise was rejected after checking the actual resource.

Retained REA/Ghidra functions for original OemDrv SHA256 `1dfe4803efdbe14f66680f970a6d6f26108342fd49754234fa448dd0ebae44a7` establish:

- ApplySetting0049c020 reads the resource matrix count into object slotA5. The matrix setter receives this count×4, yielding512B for the actual128-slot resource.
- FillMatrix0049b3d0 writes little-endian A55A0000 at buffer+(matrix count×4)−4 when CRC is enabled. With128 slots, the signature is payload510/511, full report518/519. The final slot overlaps the marker and is protected by the current protocol library.
- SetMatrix004993c0 accepts the caller length and sends command03 through the common520B report framing. GetMatrix00499640 likewise accepts a caller length; a comment calling it inherently “128 slots” would be too broad, but the acquired resource does select128 here.
- The same ApplySetting's RGB call explicitly passes0x17a=378, with126-byte plane strides. Thus512B keymaps and378B per-key colors coexist and must not be inferred from each other.

The existing bounded512B keymap reader/write encoder and marker offsets agree with this supplied package's native construction. Earlier physical reads accepted512B and matched the known ANSI control records. Physical keymap/Fn writes remain unverified; this static cross-check does not promote them to hardware-verified status. Their existing review/backup/layout/identity gates remain necessary.

Hub's different SDK contains a504B advertised matrix area with512B padding, an unresolved version/model discrepancy documented in MULTI_MODEL_RESEARCH.md. It is not evidence to replace the supplied original installer's resource-driven512B framing. Different F87 software also combines a128-slot keymap with126-position RGB packing and a384B RGB transfer; see F87_READ_CONTRACT.md.

Sources: unchanged original resource, retained original REA evidence and function exports reviewed under ignored analysis/f75-matrix-contract, existing docs/REVERSE_ENGINEERING.md and docs/PROTOCOL.md. Full native source/decompiler output is not redistributed. No installer/Windows application was executed.
