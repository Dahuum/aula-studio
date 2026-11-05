# S75 Pro: identity discrimination and native read evidence

Status: **INVESTIGATED / NOT IMPLEMENTED** for a live read adapter. Static evidence establishes a useful product-string gate and the shape of one per-key lighting read. It does **not** yet establish a reliable complete-settings read, backup/restore contract, revision identity, or a safe live implementation. No S75 device was contacted during this work.

Analysis used REA 6.7.0 with Ghidra 12.1.4 against `DeviceDriver.exe`, SHA256 `e92e83b8466fa35bf370f8bc1092f2a851ec303c41893e62d96073ba4d620e3e`. Addresses below refer to that image only. Original binaries, assembly, decompiler output and evidence bundles remain private in ignored `analysis/s75-read-contract/`. This document is an independently written factual summary.

## Product matching distinguishes the configured models

The shipped configurations declare:

| Configured model / mode | VID:PID | Product string | Interface |
| --- | --- | --- | --- |
| S75 Pro wired | `0C45:800A` | `AULA-S75Pro` | `MI_00` |
| S75 Pro receiver | `0C45:FEFE` | `2.4G Dongle` | `MI_03` |
| F108 Pro wired | `0C45:800A` | `AULA F108Pro` | `MI_00` |
| F108 Pro receiver | `05AC:024F` | `F108Pro Dongle` | `MI_03` |

Configuration SHA256 values are `12d8d3286e2e182b70d993e32101a559daa267005b9e5a9835ce454a7e3e31be` for S75 and `13090512554e959b0b9066c1f8fdf77581123b4f1e51ca5ca2733e49dc7a57fe` for F108 Pro. These are expected descriptor strings from software, not readings from physical keyboards.

The S75 executable proves the matching chain:

1. `0041BFD0` reads the `product_name` attribute and passes it into `0044E820`, which stores the string alongside configured VID/PID and connection metadata.
2. `00450A00` resolves `HidD_GetProductString` into function-pointer global `005950B8`.
3. `00450B10` enumerates HID collections. At `00450E4A` it invokes that product-string API; at `00450E6F` it stores a duplicated wide string at enumeration-item offset `14h`.
4. `0044F980` passes that item field to `0044FF40`. For a nonempty configured product string, `0044FF40` requires VID/PID equality **and exact, case-sensitive UTF-16 code-unit equality**. There is no prefix match, whitespace trimming or case folding in the observed comparison. The empty-configured-string branch permits VID/PID-only matching; S75's supplied string is nonempty.

Microsoft documents `HidD_GetProductString` as returning a product-identifying, null-terminated wide string for the top-level collection. This supports the API interpretation, independently of the binary's string labels. [HidD_GetProductString documentation](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/hidsdi/nf-hidsdi-hidd_getproductstring)

Therefore a Linux metadata candidate must not collapse S75 and F108 Pro merely because both use `0C45:800A`. The configured exact product strings distinguish those entries. Missing/mismatched product metadata must not silently fall back to that shared VID/PID. A matching string remains a metadata discriminator; it does not authenticate a device or identify every firmware/hardware revision.

## Interface and report-size boundaries

The native enumeration records usage, usage page and interface information. `00450B10` parses an `&mi_` path suffix as a hexadecimal interface number. The supplied S75 configuration names interface zero for wired mode. However, the inspected `0044FF40` match itself checks VID/PID and product, and the feature-interface branch in `0044F980` checks **FeatureReportByteLength = 65**. A particular usage page/value or an explicit interface-zero test was not established as a requirement of that feature branch. Do not invent a vendor usage constant from a different keyboard.

`00450F90` obtains preparsed HID capabilities with `HidP_GetCaps`. Its capability-to-object copies are corroborated by assembly. Windows capability lengths include a report-ID byte, including zero for an unnumbered report. [HIDP_CAPS documentation](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/hidpi/ns-hidpi-_hidp_caps)

| Representation | Observed / expected size |
| --- | ---: |
| Windows `FeatureReportByteLength` | 65 bytes, including leading report ID |
| Full hidapi request/response buffer for this unnumbered report | 65 bytes, leading byte `00` |
| Feature payload | 64 bytes |
| Existing Linux descriptor parser's length for ID zero | 64 payload bytes; do not compare this directly with Windows's 65 |

No complete raw S75 HID report descriptor was recovered or physically inspected here. The report-ID-zero conclusion comes from native buffer construction. A future adapter must verify actual descriptor compatibility separately and refuse ambiguous interfaces; a 65-byte capability alone is insufficient model identification. Other branches handle 33-byte input/output collections and a 4097-byte screen channel. They must not be treated as this feature protocol.

## The one established feature-bank read request

`0044F280` has exactly one resolved caller in this image: `0042ADB0`, the per-key lighting reader. Unresolved indirect calls and other modules remain outside that call-reference claim.

For the RGB branch, `0042ADB0` initializes a buffer with this payload:

```text
payload[0] = 04
payload[1] = F5
payload[8] = 09
all other request payload bytes = 00
```

`0044F280` prepends report ID zero and sends a 65-byte feature request. The full report therefore begins `00 04 F5 00 00 00 00 00 00 09`, followed by zero padding. This is an observed read selector, not a live probe recommendation.

The caller passes the numeric value **nine** as the helper's second argument. Assembly proves the helper initially passes that value directly as the GET_FEATURE **buffer length**, rather than multiplying it into a page count or substituting 65. A mono branch supplies three; that branch is not a validated contract either.

## Native response sequence and validation gaps

The observed RGB path is:

1. Send the 65-byte request, then sleep 5 ms.
2. Attempt GET_FEATURE with the caller-supplied length **9**.
3. If that first call returns a nonnegative result, exit without collecting bank pages. Only the first-call failure branch enters the retry/data path.
4. Retry GET_FEATURE with a 65-byte buffer, at most ten times, sleeping 5 ms between attempts.
5. Check **full-report byte 4 = `01`**, equivalently payload byte 3. No echoed `04 F5`, report-ID, revision, transaction-ID or page-number validation was observed.
6. Read up to nine further 65-byte feature buffers. Copy each buffer's bytes `1..64` into successive 64-byte bank pages. Each Windows IOCTL return count must equal **64**.
7. The helper resets its success flag before each page. Its success return is reached only after **all nine full pages**, totaling **576 payload bytes**. A short/error/late read does not constitute a complete bank.
8. On reported success, `0042ADB0` maps returned color bytes into UI keys, sends payload `04 02` through the generic send/response wrapper, then refreshes the UI.

The apparent mismatch between a requested nine-byte initial receive and later 65-byte receives is real in the inspected instructions. Its rationale is unknown. Do not silently normalize it and call the resulting sequence verified. The RGB read's **576 bytes** also differs from the separately observed RGB write's **512 bytes**. The extra read page's meaning and preservation requirements remain unknown.

No response-page header or checksum contract was established. In particular, the native read loop skips each report's first byte but does not verify that byte, a page counter, command echo or a bank trailer. The only identified pre-bank check is the single status byte above. A Linux implementation must not turn these omissions into invented validation rules or accept arbitrary same-length reports as trusted configuration.

### Counts and timeouts are API-specific

`004513F0` uses `DeviceIoControl(0x000B0192)` for GET_FEATURE, then `GetOverlappedResult(..., TRUE)`. It returns the raw Windows `bytes_returned` unchanged; it does **not** subtract a report-ID byte itself. The native page loop expects that raw count to be 64. A Linux hidapi return count and its 65-byte caller buffer are separate API conventions and require an explicit adaptation, not blind reuse of the native count comparison.

The read loop measures elapsed time around each call and rejects a call taking more than **360 ms**. This check happens **after** the blocking receive returns. It is not a hard timeout or cancellation mechanism, and it does not bound the entire transaction. The helper sleeps 3 ms between pages. A future live adapter needs genuine deadlines and safe disconnect/cancellation behavior; the native blocking design should not be copied unchanged.

The concluding `04 02` also appears in configuration-write sequences. Its exact state/persistence effect and recovery behavior have not been independently established. Consequently this read workflow is not yet a proven non-disruptive backup operation, even though its main selector is used for reading by the official application.

## Settings reads and passive input are separate

All resolved calls to native feature-receive wrapper `004513F0` come from `0044EDB0`, `0044EFA0` and `0044F280`. The first two are generic send/response wrappers; the sole resolved caller of the bank helper is the lighting reader above. A dedicated wired read selector for complete scalar settings, base/Fn keymaps, macro banks or a revision identity was not established by this investigation. That is an explicit research limit, not proof that firmware lacks those capabilities.

`00435160` is a separate passive input-report consumer. It periodically calls `00451210` with a 65-byte destination and 10 ms wait, and branches on input data beginning with `05` and sometimes `A6`. `00451210` uses overlapped **ReadFile**, not GET_FEATURE; when the incoming first byte is zero, it removes that byte before copying. This stream is associated with notifications/control handling in the inspected application. Its existence is not evidence of complete configuration readback and does not justify treating input reports as feature-bank responses.

## Evidence index

| Observation / function | REA evidence ID |
| --- | --- |
| XML configuration reader `0041BFD0` | `ev_b4df34640e8904cc4ce38cdde30dc9fa1f55eae5576e9c70e004d150ea2c2998` |
| Device registration `0044E820` | `ev_2a04a85d5303a64df1ec51dd4ef3e0c8b4b8cd63882192ffec6f9cfef88d8b6d` |
| HID API resolver `00450A00` | `ev_a3e1ac466a23afbaaf9e099cff8fe4031c98f625c0922bc5fbc04cd1dc6a96a0` |
| Product enumeration assembly `00450B10` | `ev_3a5e2fc329ee340e15ba858ae513b7efafae77267b449f26eb9fb2bb366edc39` |
| Exact product comparison assembly `0044FF40` | `ev_6a17599c301be4031a01f8b455d95bc49040098eee75d20dac7390e49b66bd5d` |
| Interface feature-size selection `0044F980` | `ev_779b43c3fb74890df6b26709ea22a9542b3e31bf7c1d86b40d8d0908c49b8203` |
| Capability retrieval assembly `00450F90` | `ev_9c26f8d3747d51ac968a05d1059925ff29a4fed6b4fba6767f77be93cd0b8931` |
| RGB request assembly `0042ADB0` | `ev_9d3c79df8263873aff206dbf35819a5c8fd522d885e082647b8b280caa56c9ff` |
| Read helper assembly `0044F280` | `ev_c2e59d75f0336356c8ae1a0393a25368afa64f0a4a8105ae5a4351ed2a070635` |
| Raw GET_FEATURE wrapper `004513F0` | `ev_7cd69f6a4af814270a0e030bc95fdf75aef9b9e34a120b60ee2e6aad9428d6db` |
| Read-helper caller inventory | `ev_620d3e3b517bf236f6921a0c5fe275cd251acae85fbf9b4194d963a200e01f02` |
| Feature-receiver caller inventory | `ev_ac2c94714dd385bebacafa6d6a90712d91eff5466d167390c2c8dc95d8740691` |
| Passive input reader `00435160` | `ev_1e8bd2dda88ba81825ba56e4ea271092fe0bd47ab3fcf1db66d2cd6decd833a5` |
| ReadFile wrapper `00451210` | `ev_887e25bf26fc8e3b5ef698da15943d5df116383e47ffc6ebdba508868cd79e31` |

## Implementation boundary

Exact product matching is useful for conservative **metadata-only identification**. It is not sufficient to enable new-model configuration writes or call a read physically verified. A live S75 read adapter remains disabled pending descriptor confirmation, a reliable acknowledgment/page contract, resolution of the initial-length branch and extra page, verified cleanup/recovery, and separately documented physical validation. No existing Rust/UI code or release artifact was changed for this investigation.

## Passive application matching

The development registry now stores the exact wired product strings from the S75/F108 Pro/F98 Pro resources. USB metadata discovery prioritizes an exact configured product match; case changes, trailing spaces or unknown names preserve ambiguous USB candidates. This performs no HID feature query, does not attest a hardware revision and does not enable S75/Sonix reads or writes. The F87 read adapter independently checks its live six-byte model identity.
