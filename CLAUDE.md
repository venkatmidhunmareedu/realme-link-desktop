# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is
A Linux-first (Windows/macOS later) Tauri desktop replacement for the Realme Link phone app, for realme/OPPO TWS earbuds. The wire protocol was reverse-engineered from the Realme Link APK; `PROTOCOL.md` is the source of truth (framing, command table, enums, confidence markers, open questions). **Model-agnostic by design**: nothing is defaulted to a specific bud.

## Commands
- Protocol tests (no system libs needed): `cargo test -p buds-protocol` (single: `cargo test -p buds-protocol frame::tests::roundtrip_multi_and_byte_by_byte`)
- Frontend typecheck/build: `./node_modules/.bin/tsc --noEmit && ./node_modules/.bin/vite build` (pnpm 11's `pnpm build` aborts if esbuild's build script isn't approved; `pnpm-workspace.yaml` allows it)
- Screenshot the UI (real-time, dark/light, any size) with headless Chrome: `node tools/shot.mjs "http://localhost:1420/#demo" out.png dark 6000 1280 900`
- UI in a plain browser with a faked backend: `./node_modules/.bin/vite --port 1420`, open `http://localhost:1420/#demo` (dev-only shim in `src/preview.ts`)
- Full app: `pnpm tauri dev` / `pnpm tauri build`. Needs Debian/Ubuntu packages: `libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev librsvg2-dev libayatana-appindicator3-dev libjavascriptcoregtk-4.1-dev` (plus `libdbus-1-dev`, `libssl-dev`, already present). Without them `cargo check -p realme-link-desktop` fails in `atk-sys`.

## Architecture
- `crates/buds-protocol` — pure Rust, no I/O. `frame` (0xAA varint-length frames + `Deframer` stream reassembly) → `packet` (cmd u16 LE, tid, len; responses = cmd|0x8000, first data byte is status 0=OK) → `cmd` builders / `parse` reply parsers → `caps` (0x0100 bitmap → allowed commands; table generated from the app's `Protocol.b2` into `caps_table.rs`) → `model` (per-model config catalog from `models.json`).
- `models.json` = the 34 bundled `headsetRusConfig` files decoded by `tools/decode_rus.py` (filenames are XOR+base64 obfuscated; contents are plain JSON). Each model's `viewItems` tree declares which features exist; **the UI renders only what the model declares and the device's capability bitmap allows**. `sppUUIDType` picks the RFCOMM UUID (`UUID_DEFAULT` vs `UUID_SELF_PROTOCOL`).
- `src-tauri` — `transport.rs` yields a `Link` (byte channels): real BlueZ RFCOMM via `bluer` Profile API, or `mock.rs` (an in-process fake bud speaking the real protocol — use it to develop without hardware). `session.rs` owns a Link, negotiates caps, polls state, handles notifications (0x0204) and the device's 0x0500 time-sync request, and emits a full `DeviceState` as the `state` event after every change. Setters send a command then re-read it (the official app does the same). Commands sent only if the capability bitmap allows.
- `src/` (React 19 + Tailwind v4 + shadcn/ui "base-nova", Base UI primitives) — `api.ts` holds typed invoke wrappers **and the wire-value tables** (noise, key functions, feature-switch ids); `App.tsx` routes between `Connect.tsx` and `Dashboard.tsx` (config-driven sections; shared `Segmented` control with a Framer Motion sliding pill); `Buds3D.tsx` draws a procedural bud pair with a GSAP assemble intro and loads `public/models/<slug>.glb` if present (slug = kebab-case of the model name). Design tokens live in `src/index.css` (Ink/Bone themes, `.glass`, `.stage`); the system is recorded in `DESIGN.md` (+ `.impeccable/design.json`) — read it before changing UI. shadcn components are in `src/components/ui/` (`@/` alias → `src/`).

## Gotchas
- `re/` (decompiled APK + jadx output, ignored by git) is regenerated with `~/tools/jadx/bin/jadx --no-res -d re/src re/com.realme.link/base.apk`; the APK itself comes from `adb` (`com.realme.link`). The headset code lives in `com/realme/iot/headset/tl/**`; obfuscated field names resolve via `Protocol.java`.
- Only the TL vendor stack is implemented. The app also has `VENDOR_LD` (Airoha) and `VENDOR_WM` stacks that aren't bundled in `models.json`.
- `PROTOCOL.md` lists what is verified on a real T310 (framing, packet layout, caps bitmap, all GETs) and what is not (every SET, notifications, multi-frame). Test against hardware with `cargo run -p realme-link-desktop --example probe -- <MAC>` (read-only GETs, prints raw bytes). Anything marked `?`/"verify live" is still static analysis.
- Under Xvfb, force X11 (`unset WAYLAND_DISPLAY; GDK_BACKEND=x11`) or GTK opens the window on the real Wayland desktop.
- Vite must ignore `re/` (32k files) — see `server.watch.ignored` in `vite.config.ts`.
- Official 3D models are downloaded by the phone app at runtime (not in the APK); we don't have them.
