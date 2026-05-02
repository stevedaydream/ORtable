// Smart OR Triage — Google Apps Script 後端
// 部署步驟：
//   1. 開啟 Google Sheets，建立新試算表
//   2. 將此檔案貼到 [擴充功能 → Apps Script]
//   3. 填入下方 SPREADSHEET_ID（試算表 URL 中的長 ID）
//   4. [部署 → 新增部署] → 類型：Web 應用程式
//      執行身分：我、存取權限：所有人（含匿名）
//   5. 複製部署 URL，貼入 OR Triage 設定頁面的 GAS URL 欄位

const SPREADSHEET_ID = '1cZB0Z7bwspie_3D_uN78WY9fNBa14C1HUFBzenyTLP8';  // ← 填入你的試算表 ID

// ── Sheet 欄位定義 ─────────────────────────────────────────────────────────────

const TASK_COLS = [
  'id','patient_name','chart_no','procedure','diagnosis','vs_note',
  'dept','expected_room','urgency','scheduled_at','created_at','est_time_mins','status'
];

const STAFF_COLS = [
  'id','name','role','type','is_on_call','is_volunteer_extra',
  'today_shift_start','next_day_shift_start'
];

const RULE_COLS = ['dept','priority_bonus','preferred_rooms'];

// ── HTTP 處理 ─────────────────────────────────────────────────────────────────

function doGet(e) {
  try {
    const action = (e.parameter || {}).action || '';
    let data;
    if      (action === 'ping')         data = { ok: true, ts: Date.now() };
    else if (action === 'getTasks')     data = { tasks:      readSheet('Tasks',     TASK_COLS)  };
    else if (action === 'getStaff')     data = { staff:      readSheet('Staff',     STAFF_COLS) };
    else if (action === 'getDeptRules') data = { dept_rules: readSheet('DeptRules', RULE_COLS)  };
    else data = { error: 'unknown action: ' + action };
    return json(data);
  } catch(err) {
    return json({ error: err.message });
  }
}

function doPost(e) {
  try {
    const body = JSON.parse(e.postData.contents);
    const action = body.action || '';
    if (action === 'syncAll') {
      writeSheet('Tasks',     TASK_COLS,  body.tasks       || []);
      writeSheet('Staff',     STAFF_COLS, body.staff       || []);
      writeSheet('DeptRules', RULE_COLS,  body.dept_rules  || []);
      appendSyncLog('push', (body.tasks||[]).length, (body.staff||[]).length);
      return json({ ok: true, synced_at: Date.now() });
    }
    return json({ error: 'unknown action: ' + action });
  } catch(err) {
    return json({ error: err.message });
  }
}

// ── Sheet 讀寫 ────────────────────────────────────────────────────────────────

function getOrCreateSheet(name) {
  const ss = SpreadsheetApp.openById(SPREADSHEET_ID);
  return ss.getSheetByName(name) || ss.insertSheet(name);
}

function readSheet(sheetName, cols) {
  const sheet = getOrCreateSheet(sheetName);
  const data  = sheet.getDataRange().getValues();
  if (data.length < 2) return [];          // header only or empty

  const header = data[0];
  return data.slice(1).map(row => {
    const obj = {};
    cols.forEach(col => {
      const idx = header.indexOf(col);
      if (idx === -1) { obj[col] = null; return; }
      let val = row[idx];
      // Restore booleans stored as 0/1
      if (col === 'is_on_call' || col === 'is_volunteer_extra') val = val === 1 || val === true;
      // Restore JSON arrays (preferred_rooms)
      if (col === 'preferred_rooms' && typeof val === 'string') {
        try { val = JSON.parse(val); } catch(_) { val = []; }
      }
      // Restore null for empty scheduled_at
      if (col === 'scheduled_at' && val === '') val = null;
      obj[col] = val === '' ? null : val;
    });
    return obj;
  }).filter(r => r.id !== null && r.id !== '');
}

function writeSheet(sheetName, cols, rows) {
  const sheet = getOrCreateSheet(sheetName);
  sheet.clearContents();

  // Write header
  sheet.getRange(1, 1, 1, cols.length).setValues([cols]);

  if (rows.length === 0) return;

  // Write data rows
  const values = rows.map(row =>
    cols.map(col => {
      const val = row[col];
      if (val === null || val === undefined) return '';
      if (Array.isArray(val))   return JSON.stringify(val);
      if (typeof val === 'boolean') return val ? 1 : 0;
      return val;
    })
  );
  sheet.getRange(2, 1, values.length, cols.length).setValues(values);
}

function appendSyncLog(direction, taskCount, staffCount) {
  const sheet = getOrCreateSheet('SyncLog');
  if (sheet.getLastRow() === 0) {
    sheet.appendRow(['direction','tasks','staff','timestamp']);
  }
  sheet.appendRow([direction, taskCount, staffCount, new Date().toISOString()]);
}

function json(obj) {
  return ContentService.createTextOutput(JSON.stringify(obj))
    .setMimeType(ContentService.MimeType.JSON);
}
