# Release 流程

## 標準發版（release.bat 選項 1）

```
release.bat → [1] New release
  → 輸入新版本號（如 0.2.0）
  → 自動從 git log 擷取 commit 產生 changelog 草稿
  → 確認後寫入 src/data/changelog.ts
  → 更新 package.json / tauri.conf.json / Cargo.toml 版本號
  → git commit "chore: bump version to X.X.X"
  → git tag vX.X.X
  → git push origin main + vX.X.X
  → GitHub Actions 自動建置三平台（Windows / macOS ARM / macOS x86）
  → 自動上傳 latest.json（需 TAURI_SIGNING_PRIVATE_KEY secret）
```

## 重推 Tag（release.bat 選項 2）

Actions 失敗需重跑時使用，不變更版本號，只刪除並重建 tag。

## GAS 部署（release.bat 選項 3）

```
clasp push → clasp deploy --deploymentId AKfycbxmRRqtmd9lCVG8qBQ2gZ_22zAT_rvtATjQem8Fi5a-CWO-sDvs6giEvT8hqODt-Rp2
```

## GitHub Actions 必要 Secrets

| Secret | 說明 |
|--------|------|
| `TAURI_SIGNING_PRIVATE_KEY` | `.key` 檔案的**原始文字內容**（以 `untrusted comment:` 開頭），非 base64 |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 金鑰密碼（產生時設定的） |

## 重要注意事項（見 BF-009）

- `tauri.conf.json` bundle 必須包含 `"createUpdaterArtifacts": true`，否則不生成 `.sig` 與 `latest.json`
- Updater endpoint：`https://github.com/stevedaydream/ORtable/releases/latest/download/latest.json`
- 版本號需同時存在於三個檔案：`package.json`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml`

## release.bat 注意事項

- 檔案必須使用 CRLF 行尾（Windows CMD 要求），Write tool 寫出為 LF 需轉換（見 BF 記錄）
- 若顯示 `/d 不是內部命令` 等錯誤，代表行尾為 LF，需重新轉換
