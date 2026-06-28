# 🚀 GitHub Actions 自動編譯學習指引 (Tauri 專案)

這份文件旨在幫助你學習與理解如何利用 GitHub Actions 為 Tauri 專案（YT Float）建立自動化編譯與發佈（CI/CD）流程。

---

## 1. 什麼是 GitHub Actions？

GitHub Actions 是 GitHub 提供的持續整合與持續部署（CI/CD）服務。它可以讓你在特定的 GitHub 事件發生時（例如：推送程式碼、建立 Pull Request、或者手動點擊按鈕），自動在雲端虛擬機（稱為 Runner）上執行指定的指令。

### 核心概念：
*   **Workflow（工作流）**：一個完整的自動化流程，定義在 `.github/workflows/` 目錄下的 YAML 檔案中（例如我們的 `release.yml`）。
*   **Event（事件）**：觸發工作流執行的條件（如 `push` 程式碼或手動 `workflow_dispatch`）。
*   **Jobs（任務）**：工作流中的一個執行單元，每個 Job 會在獨立的虛擬機中運行。多個 Job 預設會平行（同時）執行。
*   **Steps（步驟）**：Job 內部的具體操作指令，例如：複製程式碼、安裝 Node、執行編譯等。
*   **Actions（動作）**：GitHub 社群或官方封裝好的重複使用套件（例如：`actions/checkout` 用於複製程式碼）。

---

## 2. 深入解析 `release.yml` 設定檔

我們的自動編譯設定檔位於 [.github/workflows/release.yml](file:///d:/project/YTPlayer/.github/workflows/release.yml)，以下是逐段的詳細說明：

### A. 觸發條件 (`on`)
```yaml
on:
  push:
    tags:
      - 'v*' # 當你推送符合 v 開頭的標籤（例如 v1.0.2）時觸發
  workflow_dispatch:
    inputs:
      tag:
        description: 'Release tag (e.g. v1.0.0)'
        required: true
        default: 'v1.0.0' # 允許在 GitHub 網頁手動點擊執行，並輸入自訂的標籤
```

### B. 權限設定 (`permissions`)
```yaml
permissions:
  contents: write # 必須授予寫入權限，GitHub Actions 才能自動建立 Release 並上傳檔案
```

### C. 跨平台編譯矩陣 (`matrix`)
```yaml
strategy:
  fail-fast: false
  matrix:
    include:
      - platform: 'macos-latest'      # Apple Silicon (M1/M2/M3)
        args: '--target aarch64-apple-darwin'
      - platform: 'macos-latest'      # Intel 晶片 Mac
        args: '--target x86_64-apple-darwin'
      - platform: 'ubuntu-22.04'      # Linux 系統
        args: ''
      - platform: 'windows-latest'    # Windows 系統
        args: ''
```
*   `strategy.matrix` 允許我們只寫一次步驟，GitHub 就會自動開起 **4 台不同的虛擬主機** 同時幫我們編譯各平台的安裝檔。

### D. 編譯步驟 (`steps`)
1.  **複製專案代碼** (`actions/checkout@v4`)：將你在 GitHub 上的原始碼下載到虛擬主機中。
2.  **設定 Node.js** (`actions/setup-node@v4`)：安裝 Node 環境以運行 npm 指令。
3.  **設定 Rust 環境** (`dtolnay/rust-toolchain@stable`)：Tauri 的核心是 Rust，因此需要安裝 Rust 編譯器。
4.  **快取 Rust 編譯產物** (`swatinem/rust-cache@v2`)：快取編譯中間產物，下次編譯時可節省 5~10 分鐘！
5.  **安裝 Linux 依賴**：Linux (Ubuntu) 下編譯 Tauri 需要額外安裝 `libwebkit2gtk` 等圖形庫。
6.  **安裝前端依賴** (`npm install`)：下載前端相關的依賴套件。
7.  **Tauri 官方編譯 Action** (`tauri-apps/tauri-action@v0`)：
    *   自動執行 `npm run tauri build`。
    *   收集編譯出來的 `.exe`、`.dmg`、`.AppImage` 等檔案。
    *   自動在 GitHub 上建立一個新的 Release，並把這些檔案作為附件上傳。
8.  **上傳 Windows 免安裝版**：針對 Windows，將原本需要安裝的 exe 複製並命名為 `YT-Float-portable-x64.exe`，手動上傳到 Release 中，提供使用者免安裝單檔下載。

---

## 3. 實作教學：如何一步步看見編譯成果？

### 第一步：確認程式碼已推送到 GitHub
1.  在 GitHub 上建立一個新的 **Private** 或 **Public** 儲存庫。
2.  將本機的專案關聯到該 GitHub 遠端儲存庫：
    ```bash
    git remote add origin <你的GitHub專案網址>
    git branch -M main
    git push -u origin main
    ```

### 第二步：設定 GitHub Token 權限（若編譯失敗必看）
預設情況下，GitHub Actions 的 Token 可能沒有寫入權限，會導致最後建立 Release 時失敗。
1.  在你的 GitHub 專案頁面，點選右上角的 **Settings**。
2.  在左側選單找到 **Actions** -> **General**。
3.  拉到最下方找到 **Workflow permissions**。
4.  將選項改為 **Read and write permissions**，並按下 **Save**。

### 第三步：手動啟動編譯
1.  點選 GitHub 網頁上方的 **Actions** 頁籤。
2.  在左側選擇 **Release** 工作流。
3.  點選右邊的 **Run workflow** 下拉選單。
4.  直接點選綠色的 **Run workflow** 按鈕。
5.  此時會出現一個正在運行的工作，點進去可以看到 4 個平台的編譯進度與詳細日誌！

### 第四步：查看成果
1.  編譯完成後（大約需要 10~20 分鐘），回到專案的首頁。
2.  在右側欄位會看到 **Releases**，點進去即可看到各平台的安裝包。

---

## 4. 練習挑戰（你可以試著修改看看）
當你熟悉流程後，可以嘗試修改 [.github/workflows/release.yml](file:///d:/project/YTPlayer/.github/workflows/release.yml)：
*   **修改 Release 說明文字**：修改 `releaseBody:` 後方的說明文案，換成你專屬的產品介紹。
*   **只在 Windows 上編譯**：如果你只需要 Windows 版本，可以將 `strategy.matrix` 底下的 macos 與 ubuntu 項目刪除，這樣編譯速度會快很多，也能節省 GitHub 的免費額度（每個月有 2,000 分鐘）。
