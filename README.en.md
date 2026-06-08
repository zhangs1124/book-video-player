# YT Float · An always-on-top mini video window

[繁體中文](README.md) ｜ **English**

<p align="center">
  <a href="https://craig7351.github.io/book-video-player/"><img src="https://img.shields.io/badge/🌐_Website-Demo-6c47ff?style=for-the-badge" alt="Website / Demo"></a>
  <a href="https://github.com/craig7351/book-video-player/releases/latest"><img src="https://img.shields.io/github/v/release/craig7351/book-video-player?style=for-the-badge&color=3b82f6" alt="Latest release"></a>
  <a href="https://github.com/craig7351/book-video-player/releases/latest"><img src="https://img.shields.io/badge/Download-Windows_·_macOS_·_Linux-22c55e?style=for-the-badge" alt="Download"></a>
</p>

> 🌐 **Landing page / Demo**: <https://craig7351.github.io/book-video-player/>

A super-lightweight floating browser you can pin to the bottom-right corner of your screen to watch YouTube / Netflix / Bilibili **while you work**.

Built on **Tauri v2 + the system's built-in WebView2**: instead of bundling a whole Chromium like Electron, it shares the system Edge engine, so the shell only uses about **35–50 MB** of RAM and the binary is tiny.

---

## 📸 Screenshots

| Normal YouTube video | 🎬 Video-only (theater mode) |
|:---:|:---:|
| ![YouTube](docs/screenshot-1-youtube.png) | ![Video only](docs/screenshot-2-cinema.png) |
| Full YouTube UI, floating in the corner | One click hides everything else; video fills the window |

| YouTube Shorts | Always on top · watch while you work |
|:---:|:---:|
| ![Shorts](docs/screenshot-3-shorts.png) | ![Always on top](docs/screenshot-4-always-on-top.png) |
| Vertical Shorts: fill + scroll-wheel switching | Pinned bottom-right, above your work windows |

---

## ✨ Features

- **Tiny & install-free**: a single executable — double-click and go, no runtime to install
- **Low resource usage**: shares the system browser engine (no bundled Chromium), low memory footprint
- **Always on top**: pin it to the bottom-right corner and watch while you work
- **Frameless mini window**: opens bottom-right by default, freely resizable
- **Drag & resize**: drag the red handle on top to move; drag edges/corners to resize
- **Auto-hiding address bar**: clean view by default, slides out only when you need it
- **Back / Forward**
- **Quick service shortcuts**: jump to YouTube / Netflix / Bilibili in one click
- **🎬 Video-only**: hide everything but the player, filling the whole window (YouTube + Bilibili; Netflix is already full-frame)
- **New-tab interception**: links that "open in a new tab" open in the same window instead (fixes Bilibili videos not opening)
- **🕶️ Boss key `Ctrl+Shift+Z`**: a global hotkey that **instantly hides the window and pauses the video** (no taskbar trace, no leaking audio); press again to restore
- **System tray**: closes to the tray; show or quit anytime

---

## 🎮 Usage

The window is frameless; all controls live in an auto-hiding toolbar at the top:

```
◀  ▶  [ address bar .......... ]  YT  NF  B  🎬  📌  ✕
```

| Action | How |
|---|---|
| Move window | Drag the **red handle** at the top |
| Resize window | Drag the **edges / corners** (invisible grips) |
| Show address bar | Move mouse to the top / click the red handle / press `Ctrl+L` |
| Change URL | Type in the address bar and press `Enter` (`Esc` to hide) |
| Back / Forward | ◀ / ▶ |
| Jump to service | **YT** / **NF** / **B** |
| Video-only | **🎬** (press again to restore) |
| Pin toolbar | **📌** (disable auto-hide) |
| Hide to tray | **✕**, or "Show / Quit" from the tray icon |
| 🕶️ Boss key | **`Ctrl+Shift+Z`** (global — works even when unfocused; hide + pause, press again to restore) |

> **About the boss key**: it uses an OS-level global hotkey, so it fires instantly even when you're in Excel / your IDE and the window isn't focused. It pauses all videos before hiding so audio won't give you away, and keeps them paused on restore (hit Space to resume) so bringing the window back doesn't suddenly blast sound.

---

## 🛠️ Development & build

### Requirements
- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) (with cargo)
- Windows 10/11 (WebView2 Runtime is built into Win11; on Win10 install it from Microsoft if missing)

### Install
```bash
npm install
```

### Dev mode (live run)
```bash
npm run tauri dev
```

### Build a standalone executable
```bash
npm run tauri build
# or just the single exe:
cargo build --release --manifest-path src-tauri/Cargo.toml
```
Output: `src-tauri/target/release/yt-float.exe` (single file, double-click to run).

---

## 📁 Project structure

```
.
├── package.json              # Tauri CLI scripts
├── dist/index.html           # placeholder page (the real content is the remote site)
└── src-tauri/
    ├── tauri.conf.json       # window config: frameless / always-on-top / icons
    ├── src/main.rs           # creates the window, tray, boss-key global shortcut
    ├── overlay.js            # ⭐ core: injected toolbar + drag/resize/theater/new-tab interception
    ├── capabilities/default.json  # window permissions (incl. remote-domain grant)
    └── icons/                # app icons
```

> Almost all logic lives in `overlay.js` — it's injected into every loaded page as a content script, builds the floating toolbar, and talks to the Rust-side window API.

---

## ⚙️ Technical notes

- **A single WebView loading the remote site directly** (no custom frontend) — minimal resource usage.
- The toolbar, drag bar, resize grips, theater mode and new-tab interception are all done by the single injected `overlay.js`.
- Window control (drag, resize) uses Tauri v2's `startDragging()` / `startResizeDragging()`.
- For architecture details and the gotchas we hit, see [`README_AI.md`](./README_AI.md).

---

## 🔒 Security & transparency

Every [Release](https://github.com/craig7351/book-video-player/releases) binary is **built automatically by GitHub Actions directly from the open-source code in this repo** — **nobody hand-packages and uploads builds from their own computer.**

- **Source is public**: every line of code is visible (core in `src-tauri/`)
- **Build process is public & auditable**: see the build script [`.github/workflows/release.yml`](.github/workflows/release.yml); full logs of every build are in [Actions](https://github.com/craig7351/book-video-player/actions)
- So the file you download corresponds to the public source code, meaning you can be **relatively assured it carries no bundled virus or backdoor**

> Note: automated builds greatly reduce the risk of "sneaked-in malware" (the process is public and auditable), but cannot 100% guarantee absolute safety. If you'd rather be sure, clone the source and build it yourself with `npm run tauri build`.

---

## 📝 License

Free for personal use. All streaming content is copyright of its respective owners; this tool is only a browser shell.
