# Smart OR Triage - 資料庫與 API Schema 設計文件

本文件依據醫院真實排程表 (PDF) 與房間分配表 (CSV) 梳理，做為 Rust (Tauri) 後端與 SQLite 資料庫建置的標準格式。

## 1. 每日手術排程表 (Surgery_Task)

此表對應 `20260430手術排程.pdf`，記錄每日進線的病患與手術詳細資訊。

| 欄位名稱 (Field) | 資料型態 (Type) | 來源 / 說明 | 範例資料 | 
| ----- | ----- | ----- | ----- | 
| **`id`** | String (UUID) | 系統生成，唯一識別碼 | `task_9f8a...` | 
| **`seq_no`** | Integer | PDF [序號] | `1`, `2`, `3` | 
| **`room_req`** | String | PDF [房號] (WT 代表待排) | `05`, `06`, `WT` | 
| **`patient_name`** | String | PDF [姓名] | `潘萬益` | 
| **`gender`** | String | PDF [性別] | `男`, `女` | 
| **`age`** | Integer | PDF [AGE] | `57`, `69` | 
| **`dept`** | String | PDF [科別] | `一般外科`, `整形外科` | 
| **`chart_no`** | String | PDF [病歷號碼] | `0006214852` | 
| **`bed_no`** | String | PDF [床號] | `91503`, `ICU02`, `門診` | 
| **`diagnosis`** | String | PDF [診斷] (Diagnosis) | `rectal cancer` | 
| **`body_part`** | String | PDF [部位] | `腹部(含骨盆腔)-不分` | 
| **`procedure`** | String | PDF [術式] (Procedure) | `根治性直腸切除術...` | 
| **`anesthesia`** | String | PDF [麻醉] | `G`, `E`, `IVG` | 
| **`surgeon`** | String | PDF [手術者] | `陳樞鴻` | 
| **`req_time`** | String (Time) | PDF [預起時間/申請時間] | `08:00` | 
| **`vs_note`** | String | PDF [備註] | `9A 0800到`, `需停藥72小時` | 
| --- | --- | **以下為 Triage 系統擴充欄位** | --- | 
| **`est_time_mins`** | Integer | 預估手術分鐘數 (由系統推估或手動輸入) | `180`, `90` | 
| **`urgency`** | Enum | 緊急程度 (Trauma, Level1, Level2, Level3, Normal) | `Level 1` | 
| **`created_at`** | Timestamp | 排入系統(或匯入)的準確時間，用於 FIFO 排序 | `1714435200` | 
| **`status`** | Enum | 狀態 (waiting, scheduled, completed) | `waiting` | 

## 2. 房間時段配置表 (Room_Block_Time)

此表對應 `2026手術時段 (7).xlsx.csv`。由於 CSV 屬於二維樞紐分析表（以星期一~日為欄位），系統匯入時需正規化為逐日、逐房間、逐時段的一維關聯資料。

| 欄位名稱 (Field) | 資料型態 (Type) | 來源 / 說明 | 範例資料 | 
| ----- | ----- | ----- | ----- | 
| **`id`** | String (UUID) | 系統生成，唯一識別碼 | `block_a1b2...` | 
| **`date`** | Date (YYYY-MM-DD) | CSV 對應日期 (如 2026-07-06) | `2026-07-06` | 
| **`room_name`** | String | CSV [房間列] | `OR5`, `OR6`, `門診`, `局麻` | 
| **`period`** | Enum | CSV [時段] (AM / PM / 值班) | `AM`, `PM` | 
| **`allocated_dept`** | String | CSV [交叉比對出的科別] | `GS`, `URO`, `ORTHO`, `NS` | 
| **`start_time_mins`** | Integer | 該時段當日起始分鐘 (基準0=07:30) | AM=`0`, PM=`300` | 
| **`end_time_mins`** | Integer | 該時段當日結束分鐘 | AM=`300`, PM=`510` | 
| --- | --- | **以下為 Triage 系統擴充欄位** | --- | 
| **`is_backup`** | Boolean | 是否為二線備用房 (觸發二線機制時啟用) | `false` | 
| **`is_extra_line`** | Boolean | 是否為 Extra 房 (觸發 Extra 機制時啟用) | `false` | 

## 3. 人員與排班池 (Staff_Pool)

用於支援拖拉派班、Extra 機制與勞基法防呆運算的資料結構。

| 欄位名稱 (Field) | 資料型態 (Type) | 說明 | 範例資料 | 
| ----- | ----- | ----- | ----- | 
| **`id`** | String | 員工代號 | `s001` | 
| **`name`** | String | 姓名 | `王醫師`, `陳護理師` | 
| **`role`** | Enum | 角色 (VS, R, Scrub, Circ) | `Scrub` | 
| **`type`** | Enum | 醫護分類 (doc: 醫師, nur: 護理師) | `nur` | 
| **`is_on_call`** | Boolean | 是否為今日二線待命人員 | `true` | 
| **`is_volunteer_extra`** | Boolean | 是否有意願接 Extra 刀 | `true` | 
| **`today_shift_start`** | Timestamp | 今日表定上班時間 (防呆: 連續工時計算) | `1714433400` (07:30) | 
| **`next_day_shift_start`** | Timestamp | 明日表定上班時間 (防呆: 班距間隔計算) | `1714519800` (07:30) | 

## 4. 各科自訂排序規則設定表 (Dept_Priority_Rules)

實作「各科自訂優先進刀規則」機制的設定檔。

| 欄位名稱 (Field) | 資料型態 (Type) | 說明 | 範例資料 | 
| ----- | ----- | ----- | ----- | 
| **`dept_code`** | String | 科別代碼 / 名稱 | `一般外科`, `GS` | 
| **`is_active`** | Boolean | 是否啟用自訂規則 (未啟用則預設 FIFO) | `true` | 
| **`rule_type`** | Enum | 優先條件 (e.g., AGE_OLDEST, SPECIFIC_PROCEDURE) | `AGE_OLDEST` (高齡優先) | 
| **`target_doctor`** | String (Optional) | 若規則是「特定醫師優先」則填入醫師名 | `陳樞鴻` |

PS=整形外科
ORTHO=骨科
GYN=婦產科
OPH=眼科
GS=一般外科
URO=泌尿科
CS=胸腔外科
CVS=心臟外科
ENT=耳鼻喉科
NS=腦神經外科
其餘無特定
AM=0730-1230
PM=1230-1550
值班時段=1550-隔日0730