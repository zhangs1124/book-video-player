# YT Float 專案開發規則 (Project Rules)

## 🚀 GitHub 推送與自動編譯規範
當您需要將修改後的程式碼推送至 GitHub 並發佈新版本時，為了能自動觸發 GitHub Actions 編譯，請務必使用「推送 Tag」的方式進行：

### 執行步驟：
1. 在本地建立下一版號的版本標籤（Tag，例如 `v1.0.4`）：
   ```powershell
   git tag v1.0.4
   ```
2. 將該標籤推送至遠端 GitHub：
   ```powershell
   git push origin v1.0.4
   ```

### 💡 說明：
這樣做可以全自動啟動 GitHub Actions 的 Windows 平台優化編譯，並在 GitHub 專案的 Releases 頁面中，自動產生對應版本的 `.exe` 便攜版與安裝檔，無需再手動前往網頁點選執行。
