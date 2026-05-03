export interface ChangelogEntry {
  version: string;
  date: string;
  changes: string[];
}

export const CHANGELOG: ChangelogEntry[] = [
  {
    version: "0.1.1",
    date: "2026-05-03",
    changes: [
      "feat: add room recommendation functionality and related types",
      "feat: enable creation of updater artifacts in bundle configuration",
      "feat: implement update modal and changelog functionality, enhance updater logic",
      "feat: add release script for version management and GAS deployment",
      "feat: update build workflow to generate changelog and modify release body, update spreadsheet ID, enhance self-pay item functionality, and adjust CSS zoom level",
      "feat: add PatientListPanel, PendingQueuePanel, SelfPayPickerModal, StaffPoolPanel components",
      "base",
    ],
  },
  {
    version: "0.1.0",
    date: "2026-05-03",
    changes: [
      "初始版本發布",
      "手術排程看板（Timeline 時間軸，07:30–07:30，60px/hr）",
      "待排定病患佇列（緊急程度動態排序、逾時閃爍警示）",
      "PDF / CSV 病患清單批次匯入（支援 HIS 手術紀錄表）",
      "人員拖拉指派至房間（刷手 / 流動 / 助手 / VS）",
      "科別管理、人員管理、房間管理",
      "自費項目管理（CSV 範本下載與匯入）",
      "病人清單視圖（搜尋、狀態篩選、批量刪除）",
      "Google Sheets 雲端同步（push / pull）",
      "自動背景更新檢查",
    ],
  },
];
