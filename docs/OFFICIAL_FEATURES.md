# Official package feature inventory

Source: the user-supplied `AULA F75 Setup v2.0 20260514.exe`, extracted statically. Evidence is from resources and disassembly, **not execution** of the Windows application and **not physical hardware validation**. The package has a generic multi-device engine; strings for mice, screens and other keyboards are not automatically F75 capabilities.

## Package-specific capabilities

The installer supplies only `Dev/kb/fr/KB.ini`. It describes an ISO/French F75, with 81 visible key entries and a knob entry. The visible key labels use Windows virtual-key codes, not HID usage numbers. Matrix indices are separate fields. This regional definition must not replace the ANSI keyboard's geometry silently.

| Property | Official configuration evidence | Interpretation |
|---|---|---|
| Wired identity | VID `258a`, PID `010c` | Candidate F75 wired identity; interface descriptors still required |
| Wireless identity | VID `3554`, PID `fa09` | Candidate receiver; identity does not establish a usable Linux configuration transport |
| Protocol family | `Fw=24` | Application selector, **not** a measured firmware version |
| Matrix | `MatrixLen=128` | 128 four-byte slots in the wired layer representation |
| Integrity marker | `CRC=1` | Config/matrix signature feature; it is not evidence for inventing a CRC polynomial |
| Brightness | UI/hardware values 0 through 9 | Ten levels |
| Speed | UI/hardware values 0 through 4 | Five levels |
| RGB order | `RGBIndex=0,1,2`, `GameRGBIndex=0,1,2` | RGB triplets in selected tables |
| Fn | `[FN1]`, `FnNotToSet=0` | Default Fn assignments are present; runtime writes require independent protocol validation |
| Debounce UI | `ShowDebounce=0x04000101` | Capability/control descriptor; exact interpretation requires configuration-parser tracing |
| Sleep | `SleepTime=300`; UI 60,90,120,180,240,300,600,900,1200 | Hardware values 2,3,4,6,8,10,20,30,40; 30-second units in this resource mapping |
| Windows/Mac UI | `MacLayout=1` in Cfg.ini | Application exposes platform-layout support; does not alone prove a switch command |
| Channels | `ChannelMask=3`, `ShowPower=0` | Package flags; power display disabled by default |
| Automatic application | `ApplyNow=0` | Settings applied explicitly in this package |
| Online integration | `EnableYun=0`, `UpdateByLocal=0` | Disabled package options; not needed in Linux replacement |

## Lighting options in this package

Hardware IDs from `LedOptN`, names interpreted from the corresponding English localization entries. These names describe choices in the official UI, not measured animation behavior.

| Hardware ID | UI effect | Parameter flags |
|---|---|---|
| 1 | Fixed color | Brightness, random/color; no speed |
| 2 | Breathing | Speed, brightness, random/color |
| 3 | Rainbow | Speed, brightness |
| 4 | Flash away | Speed, brightness, random/color |
| 5 | Raindrops | Speed, brightness, random/color |
| 6 | Rainbow wheel | Speed, brightness, random/color |
| 7 | Ripples | Speed, brightness, random/color |
| 8 | Twinkling stars | Speed, brightness, random/color |
| 9 | Fading shadow | Speed, brightness, random/color |
| 10 | Snake | Speed, brightness, random/color |
| 11 | Neon stream | Speed, brightness, random/color |
| 12 | Reaction | Speed, brightness, random/color |
| 13 | Sine wave | Speed, brightness, random/color |
| 14 | Scanning | Speed, brightness, random/color |
| 15 | Windmill | Speed, brightness |
| 16 | Waterfall | Speed, brightness |
| 17 | Blossoming | Speed, brightness |
| 18 | Rotating storm | Speed, brightness, random/color |
| 21 | Per-key / self-defined | Separate color-table path |
| 0 | Off | No adjustable parameters |

All direction flags are zero in these package entries. Generic localization for direction is not a reason to expose direction as verified for this revision.

## Other official application surfaces

English localization and nearby code establish macro list/create/edit/import/export, key press/release recording, delays, repeat-count/repeat-until-release/repeat-until-any-key modes, and macro preview. Native code includes macro conversion, a slot directory, binding reconstruction, and command `05`/`85` transport. The transport is **not yet safe for hardware use** merely because these paths exist.

Localization and default Fn assignments establish ordinary key reassignment, media keys, application shortcuts, platform/system shortcuts, and vendor-defined lighting/connectivity actions. Some are Windows host actions; Linux equivalents are not necessarily onboard features.

Audio capture and music/RGB streaming code is present in the generic executable. An F75-specific supported stream and activation sequence have not been established. Generic mouse DPI/polling, screen/GIF, firmware update and factory-reset strings are **not proof of F75 support**. No firmware/reset operations are implemented or authorized by this investigation.
