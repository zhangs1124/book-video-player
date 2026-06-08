# README_AI · 給 AI Agent 的開發筆記

這份文件記錄本專案的架構、**非顯而易見的關鍵陷阱**，以及如何安全地修改。若你是接手此專案的 AI agent，請先讀完再動手 —— 以下每一條都是實際 debug 出來的血淚。

---

## 專案本質

- **Tauri v2 + Rust + 系統 WebView2** 的浮動瀏覽器（Windows）。
- 目標：極省資源、永遠最上層、可看 YouTube / Netflix / Bilibili。
- **單一視窗直接載入遠端網站**（`WebviewUrl::External`），沒有自家前端框架。
- 所有 UI/互動邏輯集中在一支注入腳本 `src-tauri/overlay.js`，透過 `WebviewWindowBuilder::initialization_script` 注入到**每個載入的頁面**。

## 檔案地圖

| 檔案 | 作用 |
|---|---|
| `src-tauri/src/main.rs` | 建視窗（無邊框/最上層/右下角）、系統匣；用 `include_str!("../overlay.js")` 嵌入注入腳本 |
| `src-tauri/overlay.js` | **核心**。注入工具列、拖曳、縮放、劇場模式(🎬)、服務捷徑、新分頁攔截 |
| `src-tauri/tauri.conf.json` | `withGlobalTauri:true`、`csp:null`、`windows:[]`（視窗在 `main.rs` 用 builder 建立）、bundle 圖示清單 |
| `src-tauri/capabilities/default.json` | 視窗權限 + **遠端網域授權**（見陷阱 1） |
| `dist/index.html` | 佔位頁，實際不用（視窗載的是遠端 URL） |
| `cdp_eval.py` | 除錯工具：透過 CDP 在頁面內執行 JS（見除錯流程） |

---

## ⚠️ 關鍵陷阱（務必先讀）

### 1. Tauri v2 預設禁止「遠端網址」呼叫視窗 API
`startDragging()` / `startResizeDragging()` / `window.hide()` 等，預設**只允許本地頁面**。在 YouTube 等遠端網址呼叫會被拒：
```
window.start_dragging not allowed on ... URL: https://...  allowed on: [windows: "main", URL: local]
```
**解法**：`capabilities/default.json` 必須有 `remote` 白名單：
```json
"remote": { "urls": ["http://*", "https://*"] }
```
拿掉這段，拖曳/縮放/關閉全部失效。改 capability 後要**重新編譯**才生效。

### 2. SPA 會清掉注入的 DOM → 必須 keepalive
YouTube / Bilibili 是 SPA，載入後會重建 `<body>`，把注入到 body 的元素一起清掉（症狀：腳本有跑、`window.__TAURI__` 也在，但工具列消失）。
**解法**（已實作於 `overlay.js`）：
- 元素一律掛在 `document.documentElement`（`<html>`）而非 `<body>`。
- `setInterval(mount, 1000)`：元素若 `!isConnected` 就重新 append。

### 3. 改圖示不會自動重編 → 看到「圖案沒變」
圖示在**編譯期**由 `generate_context!` + build.rs 嵌入。只換 `icons/` 裡的圖檔、不動 Rust 原始碼時，cargo **不會偵測到**而沿用舊 binary。
**解法**：
```bash
cargo clean -p yt-float --manifest-path src-tauri/Cargo.toml   # 或 touch src/main.rs
```
再重新 build。（反之，改 `overlay.js` 因為 `include_str!` 有追蹤，會自動重編。）

### 4. DRM 內容（Netflix）+ 除錯埠 = 黑畫面
開 `--remote-debugging-port` 時，Netflix 等 Widevine DRM 內容會**強制顯示黑畫面**（防側錄）。此時 `video.readyState` 是 4、`paused:false`、`videoWidth` 正常 —— **媒體管線是好的，只是受保護畫面不合成顯示**。
**重點**：
- 正式版**絕不能**內建除錯埠（目前只在開發時用 env var 加，沒寫進程式，正確）。
- 用 CDP 除錯時若遇到 Netflix 黑畫面，那是正常的，不是 bug；關掉除錯埠就有畫面。

### 5. 劇場模式(🎬)的 video 規則必須「限定播放器」，不可全域
通用的 `html.ytf-cinema video { position:absolute; ... }` 會套到 Netflix 的 `<video>`，導致其播放器出錯、跳回首頁。
**解法**：video 規則只 scope 在各站播放器底下：
```css
html.ytf-cinema #movie_player video,
html.ytf-cinema .html5-video-player video,
html.ytf-cinema .bpx-player-container video { ... }
```
Netflix 觀看頁本來就是全畫面，🎬 對它直接顯示提示、不套用（`/netflix\.com$/` 判斷）。

### 6. 新分頁連結要攔截成同視窗
Bilibili 點影片是 `target="_blank"` / `window.open` 開新分頁，單視窗 webview 會沒反應。
**解法**（已實作）：覆寫 `window.open` → `location.assign`，並在 capture 階段攔截 `a[target=_blank]` 點擊改成同視窗導航。

---

## 各站「只看影片(🎬)」的選擇器

劇場模式做法：在 `<html>` 加 class `ytf-cinema`，用 CSS 把播放器 fixed 填滿視窗、隱藏其餘 chrome，並 dispatch `resize` 讓播放器重算尺寸。

| 站 | 播放器容器 | 要隱藏的 chrome |
|---|---|---|
| YouTube | `#movie_player` / `.html5-video-player` | `#masthead-container`, `ytd-watch-metadata`, `#secondary`, `#comments`, `#below` … |
| Bilibili | `.bpx-player-container`（內層 `.bpx-player-video-wrap`） | `#biliMainHeader`/`.bili-header`、`.bpx-player-sending-bar`、`[class*=dm-wrap]`（彈幕） |
| Netflix | （不適用，本來就全畫面） | — |

**新增一個站的 🎬 支援**：用除錯埠 + `cdp_eval.py` 找出該站的 `video` 祖先鏈與要隱藏的頂部/側邊元素，照上表模式加 CSS 規則 + 更新 `hasPlayer()`。

---

## 除錯流程（CDP）

WebView2 支援 Chrome DevTools Protocol。開發時這樣啟動可遠端檢查 DOM：
```bash
WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222" \
  cargo run --manifest-path src-tauri/Cargo.toml
```
然後用內附的 `cdp_eval.py`（純 stdlib，自製 WebSocket）在頁面內跑 JS：
```bash
PYTHONIOENCODING=utf-8 python cdp_eval.py "JSON.stringify({url:location.href, bar:!!document.getElementById('ytf-bar')})"
```
常用檢查：注入是否成功、元素是否被 SPA 清掉、video 幾何/狀態、`startDragging().then().catch()` 看權限是否被拒。
⚠️ 記得陷阱 4：開除錯埠時 Netflix 會黑畫面，屬正常。

---

## 修改備忘

- **改 `overlay.js`** → 直接 `cargo run`，會自動重編（`include_str!` 有追蹤）。
- **改圖示 / capability / tauri.conf.json** → 可能需要 `cargo clean -p yt-float` 才會重嵌入。
- **新增工具列按鈕**：在 `overlay.js` 的 `ready()` 內建立 button，加進 `bar.appendChild(...)` 的順序中；服務捷徑用 `navBtn(label, url, bgColor)`。
- **release profile**（`Cargo.toml`）：`opt-level="s"` + `lto` + `strip` + `panic="abort"`，產物小。第一次 release 編譯較久。
- 視窗預設大小/位置、起始網址在 `main.rs` 頂部常數（`START_URL`, `WIN_W`, `WIN_H`）。

---

## 目前狀態

YouTube / Bilibili / Netflix 三站皆驗證可用（開影片、🎬、新分頁攔截、DRM 播放）。已產出 release exe。後續可能的擴充：記住上次視窗位置/大小、開機自啟、透明度調整、更多服務捷徑。
