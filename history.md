# 專案開發歷史與還原記憶（history.md）

> **用途**：這份檔案記錄 YT Float 從零到現在的完整開發脈絡、決策與踩過的坑。
> 重灌電腦後，`git clone` 回來，把這份檔案（連同 `README_AI.md`）貼給 AI agent，即可還原專案記憶並接續開發。

---

## 0. 一句話總結

用 **Tauri v2 + 系統 WebView2** 做的**極省資源、永遠最上層的迷你浮動瀏覽器**，釘在螢幕右下角邊工作邊看 YouTube / Netflix / Bilibili，含老闆鍵。已發佈 v1.0.0、v1.0.1 多平台 Release，有 GitHub Pages 介紹頁與中英文 README。

- **GitHub repo**：https://github.com/craig7351/book-video-player
- **官網 / Demo（中）**：https://craig7351.github.io/book-video-player/
- **官網 / Demo（EN）**：https://craig7351.github.io/book-video-player/en.html
- **本機路徑**：`F:\0_CODE\youtube`
- **產品名 / 套件名**：YT Float / `yt-float`；identifier `com.ytfloat.app`

---

## 1. 需求與目標（使用者原始訴求）

1. 最省資源的瀏覽器，能看 YouTube
2. 可設定「永遠最上層」，拖到螢幕右下角邊工作邊看
3. 後續陸續加：網址列（可隱藏）、可縮放視窗、只看影片、上一頁/下一頁、三大服務捷徑、老闆鍵
4. 也要能看 Netflix、Bilibili（變成通用追劇浮動視窗）

## 2. 技術選型決策

- 比較 Electron vs Tauri vs pywebview，選 **Tauri v2**：共用系統 WebView2（Edge 引擎），不打包 Chromium → 殼層僅約 35–50 MB、exe 極小。
- **單一視窗直接載入遠端網站**（`WebviewUrl::External`），不寫自家前端。
- 所有 UI/互動邏輯集中在一支注入腳本 **`src-tauri/overlay.js`**，透過 `WebviewWindowBuilder::initialization_script` 注入每個頁面。

## 3. 檔案地圖

```
F:\0_CODE\youtube
├── package.json                 # Tauri CLI 指令、版本號
├── README.md / README.en.md     # 中 / 英文說明（頂部互相切換）
├── README_AI.md                 # 給 AI 的架構與踩坑筆記
├── history.md                   # （本檔）
├── app-icon.png                 # 圖示來源（藍紫漸層圓 + 白色播放鍵）
├── cdp_eval.py                  # 除錯工具：用 CDP 在頁面內跑 JS
├── dist/index.html              # 佔位頁（實際載遠端網站）
├── docs/                        # GitHub Pages（main 分支 /docs）
│   ├── index.html               # 中文 landing page
│   ├── en.html                  # 英文 landing page
│   ├── .nojekyll
│   └── screenshot-1..4-*.png    # 介紹截圖
├── .github/workflows/release.yml # CI：多平台 build + Release
└── src-tauri/
    ├── Cargo.toml               # 依賴、release profile、版本號
    ├── tauri.conf.json          # withGlobalTauri / csp:null / windows:[] / bundle 圖示
    ├── build.rs
    ├── src/main.rs              # 建視窗、系統匣、老闆鍵全域熱鍵
    ├── overlay.js               # ⭐ 核心注入腳本
    ├── capabilities/default.json # 視窗權限 + remote.urls 白名單
    └── icons/                   # 各尺寸圖示（由 `npx tauri icon app-icon.png` 產）
```

## 4. 已完成功能

- 無邊框、永遠最上層、預設開在螢幕右下角、可縮放（`min_inner_size`）
- 頂部**注入式工具列**（自動隱藏）：`◀ ▶ [網址列] YT NF B 🎬 📌 ✕`
- **紅色握把**（頂部置中）：按住拖曳移動視窗、hover 顯示工具列、雙擊聚焦網址列
- **縮放**：四邊/四角隱形感應區，呼叫 `startResizeDragging`
- **網址列**：自動隱藏、`Ctrl+L` 叫出、Enter 跳轉、Esc 收起；📌 釘住不隱藏
- **上一頁/下一頁** ◀ ▶（`history.back/forward`）
- **服務捷徑** YT / NF / B（YouTube / Netflix / Bilibili，品牌色）
- **新分頁攔截**：覆寫 `window.open` + 攔截 `a[target=_blank]` 改同視窗開啟（修 Bilibili 點影片沒反應）
- **🎬 只看影片（劇場模式）**：CSS 隱藏頁面其餘部分、播放器填滿視窗
  - YouTube 一般影片、YouTube Shorts、Bilibili 皆支援；Netflix 本就全畫面（按 🎬 顯示提示、不套用）
  - Shorts 額外支援：填滿無偏移、**滾輪上下切換**短影音
- **🕶️ 老闆鍵 `Ctrl+Shift+Z`**（全域熱鍵）：瞬間隱藏視窗 + 暫停所有影片（防聲音穿幫），再按還原
- **系統匣**：✕ 收到系統匣；托盤選單「顯示 / 結束」
- 應用圖示：藍紫漸層圓形 + 白色播放鍵

## 5. ⚠️ 關鍵踩坑（還原時務必記得）

1. **Tauri v2 預設禁止遠端網址呼叫視窗 API** → `capabilities/default.json` 必須有
   `"remote": { "urls": ["http://*","https://*"] }`，否則 startDragging/resize/hide 全被拒
   （錯誤訊息：`... not allowed on ... URL: https://...  allowed on: ... URL: local`）。
2. **SPA 會清掉注入的 DOM**（YouTube/Bilibili 重建 `<body>`）→ 元素掛在 `document.documentElement`，
   並用 `setInterval(mount, 1000)` keepalive，被移除就補回。
3. **改圖示不會自動重編** → 圖檔在編譯期由 `generate_context!`/build.rs 嵌入；
   只換 `icons/` 不動 Rust 碼，cargo 不會重編 → 用 `cargo clean -p yt-float` 強制重編。
   （改 `overlay.js` 因 `include_str!` 有追蹤，會自動重編。）
4. **Netflix/DRM + 除錯埠 = 黑畫面**：開 `--remote-debugging-port` 時 Widevine 內容會強制黑畫面（防側錄）。
   `video.readyState=4`、播放中但畫面黑屬正常。**正式版不可內建除錯埠**（目前只在 dev 用 env var 加）。
5. **劇場模式 video 規則必須限定播放器，不可全域** `video`：
   全域會套到 Netflix 的 `<video>` 害它播放器出錯跳首頁。
   只 scope：`#movie_player video`（YouTube）、`#shorts-player video`（Shorts）、`.bpx-player-container video`（Bilibili）。
6. **YouTube Shorts 有兩個坑**：
   - 頁面有隱藏的 decoy `#movie_player`（0×0），舊規則把它撐成全螢幕 → 蓋住短影音「切到別的影片」。
     用 `ytf-shorts` 情境 class，只鎖定 `#shorts-player`，並 `html.ytf-cinema:not(.ytf-shorts) #movie_player` 排除。
   - `.reel-video-in-sequence-new` 有 `contain: size layout` → CSS containment 讓 `position:fixed` 相對它定位，
     造成影片左/上空白偏移 → 加 `html.ytf-shorts .reel-video-in-sequence-new{contain:none}` 解除。
   - 劇場模式蓋住原生滾動 → 攔 `wheel` 改點 `#navigation-button-down/up button` 切換短影音。
7. **Bilibili**：影片用 target=_blank 開新分頁（靠攔截器）；劇場模式要隱藏 `#biliMainHeader`、
   `.bpx-player-sending-bar`、`[class*=dm-wrap]`（彈幕）。
8. **git push 在本機會卡住**：Windows 憑證管理員（GCM）彈窗會 hang。
   解法：`gh auth setup-git`，或 push 時 `git -c credential.helper='!gh auth git-credential' push`。
   若已卡死，`taskkill //F //IM git.exe` 清掉殘留程序再推。**不要同時觸發多個背景 push**（會互相卡）。

## 6. 各站「只看影片」選擇器速查

| 站 | 播放器容器 | 隱藏的 chrome |
|---|---|---|
| YouTube 影片 | `#movie_player` | `#masthead-container`、`ytd-watch-metadata`、`#secondary`、`#comments`、`#below` |
| YouTube Shorts | `#shorts-player`（class `ytf-shorts`） | `ytd-reel-player-overlay-renderer`、reel 容器 `contain:none`、滾輪轉導航鈕 |
| Bilibili | `.bpx-player-container` | `#biliMainHeader`/`.bili-header`、`.bpx-player-sending-bar`、`[class*=dm-wrap]` |
| Netflix | （不適用，本就全畫面） | 按 🎬 只顯示提示 |

## 7. 開發 / 除錯流程

- **跑起來（dev）**：`cargo run --manifest-path src-tauri/Cargo.toml`（或 `npm run tauri dev`）
- **除錯（CDP）**：用除錯埠啟動後可遠端檢查 DOM
  ```bash
  WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222" \
    cargo run --manifest-path src-tauri/Cargo.toml
  # 然後用 cdp_eval.py 在頁面內跑 JS（純 stdlib WebSocket）：
  PYTHONIOENCODING=utf-8 python cdp_eval.py "JSON.stringify({url:location.href})"
  ```
  ⚠️ 開除錯埠時 Netflix 會黑畫面（坑 4），屬正常。
- **出 release exe**：`cargo build --release --manifest-path src-tauri/Cargo.toml`
  → 產物 `src-tauri/target/release/yt-float.exe`（單檔 ~4.4 MB）
- **改圖示**：改 `app-icon.png` → `npx tauri icon app-icon.png` → `cargo clean -p yt-float` 再 build（坑 3）

## 8. CI/CD 與發佈

- `.github/workflows/release.yml`：推 `v*` 標籤觸發，matrix 在
  Windows / macOS(aarch64+x86_64) / Ubuntu 22.04 編譯，用 `tauri-apps/tauri-action` 建 Release。
- **Linux 修正**：runner 無 FUSE → AppImage 失敗 → env `APPIMAGE_EXTRACT_AND_RUN: 1`、`NO_STRIP: true`。
- **Windows portable exe**：Tauri 只產 NSIS/MSI 安裝版；額外步驟把 `target/release/yt-float.exe`
  以**固定檔名** `YT-Float-portable-x64.exe` 上傳，永久連結：
  `https://github.com/craig7351/book-video-player/releases/latest/download/YT-Float-portable-x64.exe`
- **發版步驟**：改 4 處版本號（package.json、Cargo.toml、tauri.conf.json、Cargo.lock 的 yt-float），
  commit → `git tag vX.Y.Z` → `git push origin vX.Y.Z` → CI 自動建置發佈。
- 偶發：Windows NSIS 下載 504（暫時性）→ `gh run rerun <id> --failed` 重跑即可。

## 9. GitHub Pages

- 來源：`main` 分支 `/docs`（已用 `gh api ... /pages` 啟用）。
- 中文 `docs/index.html`、英文 `docs/en.html`，導覽列互相切換；`.nojekyll` 避免 Jekyll 處理。
- repo About → Website 已設為 Pages 網址。

## 10. 目前狀態（截至最後一次工作）

- 最新版本：**v1.0.1**（含 Shorts 修正、滾輪切換、老闆鍵）。v1.0.0 也在。
- 兩個 Release 皆四平台齊全 + Windows portable exe。
- README 中英雙語 + 徽章（官網/版本/下載）+「🔒 安全與透明」說明。
- GitHub Pages 中英雙語上線。

## 11. 未來可做（尚未實作）

- 記住上次視窗位置/大小（目前每次開都回右下角預設值）
- 開機自動啟動
- 音量 / 視窗透明度調整
- 自訂老闆鍵、自訂服務捷徑
- Demo GIF
- 注意：`Ctrl+Shift+Z` 在某些軟體是「重做」，全域註冊會攔截 → 之後可考慮做成可設定

## 12. 還原步驟（重灌後）

1. 裝 Node 18+、Rust（cargo）、（Windows 11 內建 WebView2）。
2. `git clone https://github.com/craig7351/book-video-player.git && cd book-video-player`
3. `npm install`
4. 把這份 `history.md` + `README_AI.md` 貼給 AI agent 還原脈絡。
5. `cargo run --manifest-path src-tauri/Cargo.toml` 確認能跑。
6. （若要 push）先 `gh auth login` 再 `gh auth setup-git`（見坑 8）。
