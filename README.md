# Realme Link Desktop

A Linux-first desktop replacement for the Realme Link phone app, for realme/OPPO TWS earbuds. Built with Tauri (Rust backend, React frontend). Windows and macOS are planned.

The wire protocol was reverse-engineered from the Realme Link APK. It is documented in [`PROTOCOL.md`](PROTOCOL.md), including framing, the command table, enums, and which parts are verified on real hardware.

The app is model-agnostic. Nothing is defaulted to a specific bud. The UI shows only the features a model declares and the device's capability bitmap allows.

## Status

Verified on a real T310: framing, packet layout, the capability bitmap, and all read-only GETs. Not yet verified on hardware: every SET, notifications, and multi-frame replies. Only the TL vendor stack is implemented.

## Layout

| Path | What |
|------|------|
| `crates/buds-protocol` | Pure-Rust protocol: framing, packets, commands, parsers, capabilities, per-model catalog (`models.json`) |
| `src-tauri` | Tauri backend: BlueZ RFCOMM transport, in-process mock bud, session/state management |
| `src` | React 19 + Tailwind v4 + shadcn/ui frontend |
| `tools` | Helper scripts (model config decoder, screenshot tool) |

See [`CLAUDE.md`](CLAUDE.md) for architecture detail and [`DESIGN.md`](DESIGN.md) for the design system.

## Development

Protocol tests (no system libraries needed):

```sh
cargo test -p buds-protocol
```

UI in a plain browser with a faked backend:

```sh
pnpm install
./node_modules/.bin/vite --port 1420   # open http://localhost:1420/#demo
```

Full app (`pnpm tauri dev` / `pnpm tauri build`) needs these Debian/Ubuntu packages:

```sh
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev librsvg2-dev \
  libayatana-appindicator3-dev libjavascriptcoregtk-4.1-dev libdbus-1-dev libssl-dev
```

Probe a real device (read-only GETs, prints raw bytes):

```sh
cargo run -p realme-link-desktop --example probe -- <MAC>
```

## Disclaimer

Unofficial, not affiliated with realme or OPPO. Protocol details come from analysing the publicly distributed Realme Link app for interoperability.
