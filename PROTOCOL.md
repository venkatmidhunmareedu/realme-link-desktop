# Realme Link buds protocol — findings from com.realme.link (decompiled with jadx 1.5.6)

Source of truth: `re/src/sources/com/realme/iot/headset/**` (regenerate: `~/tools/jadx/bin/jadx --no-res -d re/src re/com.realme.link/base.apk`).

## Three vendor stacks (`HeadsetAndroidConfig.sdkVendor`)
| Vendor | Package | Notes |
|---|---|---|
| `VENDOR_TL` (default) | `headset/tl` | OPPO-style framed protocol over RFCOMM; **documented below** |
| `VENDOR_LD` | `headset/ld` | Airoha SDK (`com.airoha`), SPP UUID `00001101-0000-1000-8000-00805F9B34FB` |
| `VENDOR_WM` | `headset/wm` | not yet analysed |

## Universal, config-driven design (this is how the official app works)
Each model has a JSON config in `assets/headsetRusConfig/` (only the *filename* is obfuscated: XOR key `#*8&@45***453$` + base64, twice; see `tools/decode_rus.py`). Decoded: `crates/buds-protocol/models.json` (34 bundled models, all `VENDOR_TL`; LD/WM models are not bundled — presumably served remotely).
Per model: `androidConfig` (sdkVendor, modelId, sppUUIDType, sppCmdInterval), `productAttr` (deviceType, noise-control protocol, customEqFreq), `viewItems` (declarative UI tree: GROUP_DEVICE_3D, GROUP_HEADER_BATTERY, GROUP_NOISE_CONTROL…, with min/max firmware gates), `headsetDetectConfig`, `headsetOtaConfig`, `statusDisplayConfig`.
**The desktop app must do the same: match the paired device name → config → render only the features that model declares. No model is a default.**
`sppUUIDType`: `TYPE_DEFAULT` = `00001101-0000-1000-8000-00805F9B34FB`; `TYPE_SELF_PROTOCOL` = `0000079A-D102-11E1-9B23-00025B00A5A5` (T110, T01, DIZO Z Power).

## Transport (TL)
- Classic Bluetooth RFCOMM (`createRfcommSocketToServiceRecord`), not BLE GATT.
- UUID chosen per model by `sppUUIDType` (see above).

## Outer frame (`OPPOv1Wrapper`)
```
0xAA | len (varint, 7-bit LE groups, MSB=continue) | ctrl | 0x00 | [fsn] | payload
```
- `len` = bytes after the length field. `ctrl & 3`: 0 single, 1 first, 2 middle, 3 last of a multi-frame message; multi-frame frames carry a 1-byte `fsn` (frame index). Bits 0x08/0x04 set by flags on send. No CRC seen at this layer.
- Default max frame 2000.

## Inner packet (`Packet`, `PacketFactory`)
```
cmd (u16 LE) | transferId (u8, per-device counter 0..255) | dataLen (u16 LE) | data
```
- Response = request cmd | 0x8000 (`Packet.c`); `cmd & 0x7FFF` is the request id.
- All multi-byte ints via `ProtocolUtil.c/e` with `littleEndian=true`.

## Verified on real hardware (realme Buds T310, read-only probe: `src-tauri/examples/probe.rs`)
Confirmed against the device, not just the decompiled code:
- RFCOMM on the standard SPP UUID (`sppUUIDType = TYPE_DEFAULT`) works through BlueZ's Profile API; the device also advertises the vendor UUID `0000079a-…`.
- Outer frame `AA | len | ctrl 00 | 00 | payload` and inner packet `cmd u16 LE | tid | len u16 LE | data` are exact; reply cmd = request | 0x8000 with a leading status byte (0 = OK, 1 = error/unsupported, with a 1-byte reason).
- `0x0100` capability bitmap (T310 returned `bf 1f 5a 20 45 04 40 00`) parses and the resulting command set is sane.
- `0x0106` battery, `0x0109` wear, `0x0108` key map (6 entries: left/right × actions 2,3,4 → functions 1,6,8), `0x010B` colour id, `0x010F` EQ id, `0x0110` high-volume all parse as documented.
- **`0x010C` noise must be queried with body `[1, 1]`** (an empty body returns status 1). Reply data = `[action, type, value]`; the T310 returned value 4 (light NC), consistent with the wire map below.
- Key `action` values 2 / 3 / 4 = double tap / triple tap / long press (the T310 model config declares exactly those three).
- Notification types the T310 offers (`0x0200` reply): `09 01 02 03 04 05 06 0d f1 f4`; registering them works and the buds then push `0x0204` (e.g. a battery push `01 03 02 64 01 64 03 00`).
- `0x0112` multi-connect list returned the buds' remembered devices with names and states (mac reversed, state, name) — useful for a future "connected devices" panel.
- `0x0122` EQ presets legitimately returned zero presets on this model: don't assume presets exist.

- **`0x0404` SET noise works**: body `[1, 1, value]` → reply `[00]`; reading `0x010C` back shows the new value (T310: 4 → 1 → 4, restored). Read back after ~600 ms. (`src-tauri/examples/noise_test.rs`)

- **Case, observed live on the T310** (`watch` example): with both buds in the ears the case battery entry reads `0%` and there is no case wear entry. Docking a bud made the case report its real level (`0x0a` = 10%, charging bit clear) and added a case wear entry `03 04` (flags 4 = lid open); both buds docked gave `03 03 04 01 00 02 00`, and closing the lid changed the case flags to `00`. Pushes: type 1 battery, type 2 wear, type 3 noise (`03 01 02`, `03 01 01`: the mode changes when a bud leaves the ear).
- The charging bit (bit 7 of a battery entry) is parsed but deliberately not shown in the UI (the official app doesn't surface it either).

Still unverified on hardware: other SET commands (EQ, feature switches, key map, high-volume), notifications (0x0204), multi-frame messages, and the remaining `?` items below.

## Command table (TL stack)
Response id = request | 0x8000. **Every response payload starts with a status byte (0 = OK)** followed by data, unless noted. Set commands reply `[status]`. Confidence: **S** = read straight from code, **I** = inferred from handler/caller, **?** = unresolved.

### Capability negotiation (do this first, drives everything)
| Cmd | Dir | Payload | Notes |
|---|---|---|---|
| 0x0100 | →dev | none | S. Reply 0x8100: `[status, bitmap...]`. Bit *i* (LSB-first per byte) set ⇒ the commands in `Protocol.b2[i]` are supported. Table below. |
| 0x0200 | →dev | none | S. Notification capabilities. Reply 0x8200 `[status, n, type*n]` |
| 0x0201 | →dev | `[type]` | S. Register for notification `type`. 0x0202/0x0203 = other register / cancel. |
| 0x0205 | →dev | `[n, type*n]` | S. Bulk register (used when `0x0205` is supported). |
| 0x0204 | dev→ | `[type, ...]` | S. Notification, see types below. |

Capability bit → commands enabled (generated from `Protocol.b2`, bit: cmds):
`0: 0x0105 · 1: 0x0106 · 2: 0x0107 · 3: 0x0108, 0x0401 · 4: 0x0109 · 5: 0x0400 · 6: 0x0402 · 7: 0x010D, 0x0403 · 8: 0x010C, 0x0404 · 9: 0x0405 · 10: 0x0406, 0x010F · 11: 0x0110, 0x0407 · 13: 0x0408 · 14: 0x0409 · 15: 0x040A, 0x0111 · 19: 0x040E, 0x040D, 0x0115, 0x0116 · 22: 0x0205 · 23: 0x0F00 · 25: 0x0118, 0x0411 · 26: 0x011A, 0x0412 · 29: 0x0112, 0x040B · 30: 0x011E, 0x011F, 0x0415 · 32: 0x0120 · 34: 0x0122, 0x0418 · 38: 0x0124, 0x041B · 43: 0x041E · 46: 0x0128, 0x050E · 54: 0x012C · 58: 0x0131, 0x0428 · 60: 0x042C`

Always allowed regardless of bitmap (`Protocol.c2`): 0x0100-0x0104, 0x010B, 0x0F00.

### Get (phone → device), parse of reply data after the status byte
| Cmd | Reply | Meaning | Data layout |
|---|---|---|---|
| 0x0106 | 0x8106 | Battery | `n, {deviceType u8, (level&0x7F)|charging<<7}*n`; deviceType 1=left 2=right 3=case |
| 0x0109 | 0x8109 | Wear/box status | `n, {deviceType, flags}*n`; flags: bit0=0 ⇒ in box, bit1 ⇒ in ear, bit2=0 ⇒ **lid closed** (the app's `coverClose`; meaningful on the case entry). **The case (device 3) only has an entry while a bud is docked.** |
| 0x0108 | 0x8108 | Key mappings | `n, {device, button, action, function}*n` |
| 0x010B | 0x810B | Colour id | `u8` (selects 3D model colour) |
| 0x010C | 0x810C | Noise control | `[action, type, value]` (see Noise) |
| 0x010F | 0x810F | Active EQ preset id | `u8` |
| 0x0110 | 0x8110 | High-volume protection | `u8` |
| 0x0111 | 0x8111 | Auto power-off time | `u8` |
| 0x0112 | 0x8112 | Multi-connect devices | `n, {mac[6] (reversed), …}`, see `CommandUtil.h` |
| 0x0120 | 0x8120 | Language | `u8` |
| 0x0122 | 0x8122 | All EQ presets | `n, {selected u8, min i8, max i8, id u8, nameLen u8, name, nPts u8, {freq u16 LE, db i8}*nPts}*n` |
| 0x0124 | 0x8124 | Bass engine | `min, max, current[, type]` (all i8) |
| 0x012C | 0x812C | Multi bass engine | `n, {min,max,cur,type}*n` |
| 0x0129 | 0x8129 | Feature switch states | `n, {featureId u8, value u32 LE}*n` |
| 0x012A | 0x812A | Feature detect | `[featureId]` → `[featureId, u32 LE]` |
| 0x0131 | 0x8131 | AI/translation app status | `[status, type]` |
| 0x011A | 0x811A | Personalised-NC result exists | see `PersonalizedNoiseReductionResultExist` |
| 0x0118 / 0x0115 / 0x0116 / 0x011E / 0x011F | | HearID restore data / detection / ear-scan / freq data | HearID feature; layouts in `PollCommandManager` (`c0`, `Z`, `d0`, `e0`) |
| 0x0103 | 0x8103 | **Model id** | 3 bytes LE; T310 returned `12 3c 06` = `063C12`, which equals `androidConfig.modelId` in `models.json`. So a model can be identified from the device itself, not only its Bluetooth name. |
| 0x0105 | 0x8105 | **Firmware versions** | `n`, then ASCII `"<dev>,<?>,<version>,..."`; T310: `1,2,1.1.0.47,2,2,1.1.0.47` (left and right both 1.1.0.47). |
| 0x0104 | 0x8104 | ASCII `"114"` on T310 (probably a hardware/protocol revision) | observed only |
| 0x0102 | 0x8102 | `9a 07` = 0x079A on T310 | observed only |
| 0x0107 | 0x8107 | `01 03` on T310 | meaning unknown |
| 0x0101, 0x010D, 0x0128 | | unresolved (0x0101 returned status 1 on T310) | |
| 0x0500-0x0516 | | device→phone requests, see `RequestCommandManager` (time sync 0x0500 reply `[0, u32 LE unix secs]`) |

### Set (phone → device); reply `[status]`
| Cmd | Reply | Meaning | Payload |
|---|---|---|---|
| 0x0400 | 0x8400 | Find-my-buds beep | `[bool]` |
| 0x0401 | 0x8401 | Key mapping | `[n, {device, button, action, function}*n]` |
| 0x0403 | 0x8403 | Feature switch | `[featureId, bool]` — ids below |
| 0x0404 | 0x8404 | Noise control | `[action, type, value]`; (SDK sends `[1|2, 1|2, value]`) |
| 0x0406 | 0x8406 | Select EQ preset | `[id]` (also bool form) |
| 0x0407 | 0x8407 | High-volume protection | `[bool]` |
| 0x040A | 0x840A | Auto power-off | `[minutes-code]` |
| 0x040B | 0x840B | Multi-connect action | `[1, mac[6] reversed, action]` |
| 0x0418 | 0x8418 | Custom EQ | `[mode, min, max, id, nameLen, name, nPts, {freq u16 LE, db i8}*]`; mode 5 = fetch/reset (empty body) |
| 0x041B | 0x841B | Bass engine | `[a, b, c[, type]]` |
| 0x041E | 0x841E | Spatial audio type | `[u8]` |
| 0x0428 | 0x8428 | AI status | `[status, type]` |
| 0x042C | 0x842C | Headset mic | `[u8]` |
| 0x040D,0x040E,0x0411,0x0412,0x0415 | | HearID / ear-scan / restore / misc | see `SetCommandManager` `g,i,w,v,h` |
| 0x0402 | dev→ | device asks phone to play music ("free music") — reply `[0]` |

### Notification types (cmd 0x0204 first data byte)
1 battery list · 2 wear/box status · 3 noise `[type, value]` (len 2) or 3 bytes · 4 compactness (fit) detect, 2×2 bytes · 5 ? `[?, value]` · 6 multi-connect list · 7 ? · 8 `[sub, val]` sub 2/4 · 11 personalised-NC result · 14 ear-scan result · 15 `flags u16 LE`: bit3 ⇒ high-volume `[0|1]`, bit1 ⇒ AI state `[status,type]` · 0xF1 user interaction event (5 bytes).

### Enumerations
- Key function **wire** values: 1 play/pause · 6 next · 5 previous · 4 (or 3 on some firmware; `FeatureOption.a()`) voice assistant · 8 noise-control cycle · 13 switch device · 17 game mode · 11 volume+ · 12 volume− · 26 AI live translate (conversation) · 27 AI live translate · 31 AI assist · 0 off.
- Key `device`: 1 left · 2 right · 4 other(neck-band/case). `button` 1 (4 for settingType 4). `action` (tap pattern): 1 single · 2 double · 3 triple · 4 long-press · 5,6 further long-press variants (from `settingKey`; **verify live**).
- Noise-control wire `value` (set `[1,1,value]`; resolved from `ItemFactory` + `O5`): **1 off · 2 transparency · 4 light NC · 8 deep NC (the plain "noise cancelling" mode) · 16 medium · 32 smart/adaptive**. Cycle config (action 2) uses 11/3/10/9 and type 1 when the model's `noiseControlProtocol` is `PROTOCOL_STANDARD`, else 2.
- Feature-switch ids (`0x0403`, from caller names): 4/6/8 game mode (model-dependent) · 5 ? · 7 party mode · 9 & 12 enhance-voice · 11 ? · 17 double-device connect · 24 high-quality sound · 26 ? · 27 3D sound effect · 29 dynamic low-sound · 38 ?. Which id applies to which model is gated by the model config `viewItems` + capability negotiation.

## Firmware update (OTA) and diagnostics: what exists, NOT implemented
Read from the decompiled app; nothing here has been sent to a device.
- **Update protocol** (`protocol/upgrade/vendor/UpgradeStateMachine`): phone→device `0x0300` SYNC, `0x0301` START, `0x0302` DATA (the device pulls blocks), `0x0303` BLOCK_VERIFY (phone sends a CRC32), `0x0304` IS_VALIDATION_DONE, `0x0305` TRANSFER_COMPLETE, `0x0306` COMMIT, `0x0307` ABORT, `0x0308` ERROR; replies +0x8000. The phone only checks CRC32; whether an image is *authentic* is decided on the device during "validation" (algorithm unknown to us).
- Gating in the app: both buds must be ≥30% battery; hardware version must match the image's `hardware_version` (or `all`); image `software_version` must be newer than the device's; resume points are supported (`headsetOtaConfig.supportBreakpointResume`).
- **Where images come from**: not downloaded by the app itself — it looks for files placed by the phone OS's updater (ColorOS "SAU" resources under `/data/oppo/common/sau_res/res/ota_<modelHex>_<n>/`). We have no official image source from a desktop.
- **Diagnostic log stream**: `0x0F00` request `[type, a, b]` makes the device stream typed log chunks on `0x0F01` (ack with an empty reply every 8192 bytes), reply `0x8F00` carries a total length. Read-only; the app uses it for "collect logs".
- Firmware versions are readable at any time via `0x0105` (T310: 1.1.0.47).

## 3D models
Downloaded at runtime to the phone (`files/headsetModel`, keyed by device name + colorId); **not in the APK**, and the app is not debuggable so they can't be pulled over adb.

## Open items
1. ~~Decode headsetRusConfig~~ done → `crates/buds-protocol/models.json`.
2. Read Set/Poll/Notification managers into a full command table.
3. LD (Airoha) and WM stacks.
4. Find the 3D model download endpoint (`RusManager`/`HeadsetUtil`).
