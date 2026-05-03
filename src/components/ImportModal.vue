<template>
  <div class="fixed inset-0 bg-black/70 z-50 flex items-center justify-center p-4" @mousedown.self="$emit('close')">
    <div class="bg-gray-800 rounded-xl shadow-2xl w-full max-w-3xl flex flex-col max-h-[90vh]">

      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-4 border-b border-gray-700 shrink-0">
        <h2 class="text-base font-bold text-white">
          <i class="fa-solid fa-file-import text-green-400 mr-2"></i>匯入病患清單
        </h2>
        <button class="text-gray-400 hover:text-white" @click="$emit('close')">
          <i class="fa-solid fa-xmark"></i>
        </button>
      </div>

      <!-- Tabs -->
      <div class="flex gap-0 px-5 shrink-0 border-b border-gray-700">
        <button
          v-for="tab in TABS" :key="tab.id"
          class="px-4 py-2.5 text-xs font-medium transition-colors border-b-2 -mb-px"
          :class="activeTab === tab.id
            ? 'text-blue-400 border-blue-500'
            : 'text-gray-500 border-transparent hover:text-gray-300'"
          @click="switchTab(tab.id)"
        >
          <i :class="tab.icon + ' mr-1.5'"></i>{{ tab.label }}
        </button>
      </div>

      <!-- Body -->
      <div class="flex-1 overflow-y-auto px-5 py-4 space-y-4">

        <!-- ── Target Date (shared between tabs) ─────────────────────── -->
        <div class="flex items-center gap-3 bg-gray-900/50 rounded-lg px-4 py-2.5">
          <i class="fa-solid fa-calendar-day text-blue-400 text-xs"></i>
          <span class="text-xs text-gray-400 shrink-0">匯入目標日期</span>
          <input
            type="date"
            v-model="targetDate"
            class="bg-gray-800 border border-gray-700 rounded px-2 py-0.5 text-xs text-blue-300 font-mono outline-none focus:border-blue-500 [color-scheme:dark]"
          />
          <span v-if="targetDate !== todayStr" class="text-[10px] text-amber-400">
            <i class="fa-solid fa-circle-info mr-1"></i>排程至非今日
          </span>
        </div>

        <!-- ── CSV TAB ─────────────────────────────────────────────────── -->
        <template v-if="activeTab === 'csv'">
          <div
            v-if="!csvParsed.length"
            class="border-2 border-dashed border-gray-600 rounded-lg p-8 text-center cursor-pointer hover:border-blue-500 transition-colors"
            :class="csvDragging ? 'border-blue-400 bg-blue-900/10' : ''"
            @dragover.prevent="csvDragging = true"
            @dragleave="csvDragging = false"
            @drop.prevent="onCsvDrop"
            @click="csvInput?.click()"
          >
            <i class="fa-solid fa-table text-3xl text-gray-500 mb-3"></i>
            <p class="text-sm text-gray-400">拖放 CSV 檔案至此，或點擊選擇</p>
            <p class="text-xs text-gray-600 mt-1">支援 .csv（UTF-8 編碼）</p>
            <input ref="csvInput" type="file" accept=".csv,.txt" class="hidden" @change="onCsvFileChange" />
          </div>

          <div v-if="!csvParsed.length" class="bg-gray-900/50 rounded-lg p-4">
            <div class="text-xs text-gray-500 font-semibold mb-2">CSV 欄位說明（第一行為欄位名稱）</div>
            <div class="grid grid-cols-2 gap-x-6 gap-y-1 text-xs text-gray-600">
              <span v-for="col in CSV_COLS" :key="col.key">
                <span class="text-gray-400 font-mono">{{ col.key }}</span> / {{ col.alias }}
              </span>
            </div>
            <div class="mt-3 text-xs text-gray-600">
              urgency 可填：<span class="text-yellow-500">Trauma | Level1 | Level2 | Level3 | Normal</span>
            </div>
          </div>

          <div v-if="csvParsed.length">
            <div class="flex items-center justify-between mb-2">
              <span class="text-sm font-semibold text-gray-300">預覽（共 {{ csvParsed.length }} 筆）</span>
              <button class="text-xs text-gray-500 hover:text-gray-300" @click="resetCsv">
                <i class="fa-solid fa-rotate-left mr-1"></i>重新選擇
              </button>
            </div>
            <div v-if="csvErrors.length" class="mb-3 bg-red-900/30 border border-red-700 rounded p-3 text-xs text-red-300 space-y-0.5">
              <div class="font-semibold mb-1">⚠ 以下列有問題（將略過）：</div>
              <div v-for="e in csvErrors" :key="e">{{ e }}</div>
            </div>
            <div class="overflow-x-auto rounded-lg border border-gray-700">
              <table class="w-full text-xs">
                <thead class="bg-gray-900/60">
                  <tr>
                    <th class="px-3 py-2 text-left text-gray-400 font-medium w-14">狀態</th>
                    <th class="px-3 py-2 text-left text-gray-400 font-medium">#</th>
                    <th class="px-3 py-2 text-left text-gray-400 font-medium">姓名</th>
                    <th class="px-3 py-2 text-left text-gray-400 font-medium">病歷號</th>
                    <th class="px-3 py-2 text-left text-gray-400 font-medium">緊急程度</th>
                    <th class="px-3 py-2 text-left text-gray-400 font-medium">科別</th>
                    <th class="px-3 py-2 text-left text-gray-400 font-medium">手術名稱</th>
                    <th class="px-3 py-2 text-left text-gray-400 font-medium">時間(分)</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-gray-700/50">
                  <tr v-for="(t, i) in csvParsed.slice(0, 10)" :key="i" class="hover:bg-gray-700/30">
                    <td class="px-3 py-2">
                      <span v-if="csvDupFlags[i] === 'dup'" class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-orange-800 text-orange-200">重複</span>
                      <span v-else-if="csvDupFlags[i] === 'dual'" class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-blue-800 text-blue-200">兩科</span>
                    </td>
                    <td class="px-3 py-2 text-gray-600">{{ i + 1 }}</td>
                    <td class="px-3 py-2 text-gray-200">{{ t.patient_name }}</td>
                    <td class="px-3 py-2 text-gray-400">{{ t.chart_no }}</td>
                    <td class="px-3 py-2">
                      <span class="px-1.5 py-0.5 rounded text-[10px] font-bold" :class="urgencyColor(t.urgency)">
                        {{ t.urgency }}
                      </span>
                    </td>
                    <td class="px-3 py-2 text-gray-400">{{ t.dept }}</td>
                    <td class="px-3 py-2 text-gray-400 max-w-[120px] truncate">{{ t.procedure }}</td>
                    <td class="px-3 py-2 text-gray-400">{{ t.est_time_mins }}</td>
                  </tr>
                </tbody>
              </table>
              <div v-if="csvParsed.length > 10" class="px-3 py-2 text-xs text-gray-600 bg-gray-900/30">
                ...以及 {{ csvParsed.length - 10 }} 筆未顯示
              </div>
            </div>
          </div>
        </template>

        <!-- ── PDF TAB ─────────────────────────────────────────────────── -->
        <template v-else-if="activeTab === 'pdf'">

          <!-- Drop zone -->
          <div
            v-if="pdfStep === 'drop'"
            class="border-2 border-dashed border-gray-600 rounded-lg p-10 text-center cursor-pointer hover:border-red-500 transition-colors"
            :class="pdfDragging ? 'border-red-400 bg-red-900/10' : ''"
            @dragover.prevent="pdfDragging = true"
            @dragleave="pdfDragging = false"
            @drop.prevent="onPdfDrop"
            @click="pdfInput?.click()"
          >
            <i class="fa-solid fa-file-pdf text-4xl text-red-500/60 mb-3"></i>
            <p class="text-sm text-gray-400">拖放 HIS 匯出的手術排程 PDF，或點擊選擇</p>
            <p class="text-xs text-gray-600 mt-1">支援格式：手術紀錄表（橫式，含房號/序號/姓名…）</p>
            <input ref="pdfInput" type="file" accept=".pdf" class="hidden" @change="onPdfFileChange" />
          </div>

          <!-- Parsing -->
          <div v-else-if="pdfStep === 'parsing'" class="flex flex-col items-center justify-center py-16 gap-4">
            <i class="fa-solid fa-spinner animate-spin text-3xl text-blue-400"></i>
            <span class="text-sm text-gray-400">正在解析 PDF…</span>
          </div>

          <!-- Parse error -->
          <div v-else-if="pdfStep === 'error'" class="flex flex-col items-center gap-3 py-10">
            <i class="fa-solid fa-circle-exclamation text-3xl text-red-400"></i>
            <div class="text-sm text-red-300">{{ pdfError }}</div>
            <button class="btn-secondary text-xs mt-2" @click="resetPdf">重新選擇</button>
          </div>

          <!-- Preview -->
          <div v-else-if="pdfStep === 'preview'">
            <!-- Summary bar -->
            <div class="flex items-center gap-4 mb-3 bg-gray-900/50 rounded-lg px-4 py-2.5">
              <div class="text-xs">
                <span class="text-gray-500">排程日期</span>
                <span class="text-blue-300 font-mono ml-2">{{ pdfDate }}</span>
              </div>
              <div class="h-3 w-px bg-gray-700" />
              <div class="text-xs">
                <span class="text-green-400 font-bold">{{ pdfScheduled.length }}</span>
                <span class="text-gray-500 ml-1">已排房間</span>
              </div>
              <div class="text-xs">
                <span class="text-yellow-400 font-bold">{{ pdfWaiting.length }}</span>
                <span class="text-gray-500 ml-1">待排(WT)</span>
              </div>
              <div class="flex-1" />
              <button class="text-xs text-gray-500 hover:text-gray-300" @click="resetPdf">
                <i class="fa-solid fa-rotate-left mr-1"></i>重新選擇
              </button>
            </div>

            <!-- Tab between scheduled / waiting -->
            <div class="flex gap-0 mb-3 text-xs">
              <button
                class="px-3 py-1.5 rounded-l border transition-colors"
                :class="pdfPreviewFilter === 'all'
                  ? 'bg-gray-700 border-gray-600 text-gray-200'
                  : 'bg-transparent border-gray-700 text-gray-500 hover:text-gray-300'"
                @click="pdfPreviewFilter = 'all'"
              >全部 {{ pdfTasks.length }}</button>
              <button
                class="px-3 py-1.5 border-t border-b transition-colors"
                :class="pdfPreviewFilter === 'scheduled'
                  ? 'bg-gray-700 border-gray-600 text-green-300'
                  : 'bg-transparent border-gray-700 text-gray-500 hover:text-gray-300'"
                @click="pdfPreviewFilter = 'scheduled'"
              >已排 {{ pdfScheduled.length }}</button>
              <button
                class="px-3 py-1.5 rounded-r border transition-colors"
                :class="pdfPreviewFilter === 'waiting'
                  ? 'bg-gray-700 border-gray-600 text-yellow-300'
                  : 'bg-transparent border-gray-700 text-gray-500 hover:text-gray-300'"
                @click="pdfPreviewFilter = 'waiting'"
              >待排 {{ pdfWaiting.length }}</button>
            </div>

            <div class="overflow-x-auto rounded-lg border border-gray-700">
              <table class="w-full text-xs">
                <thead class="bg-gray-900/60">
                  <tr>
                    <th class="px-2 py-2 text-left text-gray-400 font-medium w-14">偵測</th>
                    <th class="px-3 py-2 text-left text-gray-400 font-medium w-10">房</th>
                    <th class="px-3 py-2 text-left text-gray-400 font-medium w-7">#</th>
                    <th class="px-3 py-2 text-left text-gray-400 font-medium">姓名</th>
                    <th class="px-3 py-2 text-left text-gray-400 font-medium">科別</th>
                    <th class="px-3 py-2 text-left text-gray-400 font-medium">手術者</th>
                    <th class="px-3 py-2 text-left text-gray-400 font-medium">麻醉</th>
                    <th class="px-3 py-2 text-left text-gray-400 font-medium">預起</th>
                    <th class="px-3 py-2 text-left text-gray-400 font-medium">狀態</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-gray-700/50">
                  <tr
                    v-for="(t, ti) in previewFiltered.slice(0, 40)"
                    :key="t.seq_no + '-' + t.patient_name"
                    class="hover:bg-gray-700/20"
                    :class="t.status === 'waiting' ? 'opacity-70' : ''"
                  >
                    <td class="px-2 py-1.5">
                      <span v-if="pdfDupFlags[previewFilteredIdx(ti)] === 'dup'" class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-orange-800 text-orange-200">重複</span>
                      <span v-else-if="pdfDupFlags[previewFilteredIdx(ti)] === 'dual'" class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-blue-800 text-blue-200">兩科</span>
                    </td>
                    <td class="px-3 py-1.5 font-mono text-blue-300">
                      {{ t.expected_room || 'WT' }}
                    </td>
                    <td class="px-3 py-1.5 text-gray-600">{{ t.seq_no }}</td>
                    <td class="px-3 py-1.5 text-gray-200">{{ t.patient_name }}</td>
                    <td class="px-3 py-1.5 text-gray-400">{{ t.dept }}</td>
                    <td class="px-3 py-1.5 text-gray-400">{{ t.surgeon }}</td>
                    <td class="px-3 py-1.5 text-gray-500">{{ t.anesthesia }}</td>
                    <td class="px-3 py-1.5 text-gray-500 font-mono">{{ fmtScheduledAt(t.scheduled_at) }}</td>
                    <td class="px-3 py-1.5">
                      <span
                        class="px-1.5 py-0.5 rounded text-[10px] font-bold"
                        :class="t.status === 'waiting' ? 'bg-yellow-900/60 text-yellow-300' : 'bg-green-900/60 text-green-300'"
                      >{{ t.status === 'waiting' ? '待排' : '已排' }}</span>
                    </td>
                  </tr>
                </tbody>
              </table>
              <div v-if="previewFiltered.length > 40" class="px-3 py-2 text-xs text-gray-600 bg-gray-900/30">
                ...以及 {{ previewFiltered.length - 40 }} 筆未顯示
              </div>
            </div>
          </div>
        </template>

      </div>

      <!-- Footer -->
      <div class="flex items-center justify-between px-5 py-4 border-t border-gray-700 shrink-0">
        <span v-if="importResult" class="text-sm" :class="importResult.ok ? 'text-green-400' : 'text-red-400'">
          {{ importResult.message }}
        </span>
        <span v-else class="text-xs text-gray-600">
          <template v-if="activeTab === 'csv' && csvValid.length">{{ csvValid.length }} 筆將被匯入</template>
          <template v-else-if="activeTab === 'pdf' && pdfStep === 'preview'">{{ pdfTasks.length }} 筆將被匯入</template>
        </span>
        <div class="flex gap-3">
          <button class="btn-secondary" @click="$emit('close')">關閉</button>
          <!-- CSV import button -->
          <button
            v-if="activeTab === 'csv' && csvValid.length && !importResult"
            class="btn-primary"
            :disabled="importing"
            @click="doCsvImport"
          >
            <i v-if="importing" class="fa-solid fa-spinner animate-spin mr-1.5"></i>
            匯入 {{ csvValid.length }} 筆
          </button>
          <!-- PDF import button -->
          <button
            v-if="activeTab === 'pdf' && pdfStep === 'preview' && !importResult"
            class="btn-primary"
            :disabled="importing"
            @click="doPdfImport"
          >
            <i v-if="importing" class="fa-solid fa-spinner animate-spin mr-1.5"></i>
            匯入 {{ pdfTasks.length }} 筆
          </button>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useTasksStore } from "../stores/tasks";
import { useTasksDb } from "../composables/useDatabase";
import type { SurgeryTask, UrgencyLevel } from "../types";
import pdfWorkerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";

const emit = defineEmits<{ close: []; imported: [count: number] }>();
const tasksStore = useTasksStore();
const tasksDb = useTasksDb();

// ── Tab ───────────────────────────────────────────────────────────────────────
type TabId = "csv" | "pdf";
const TABS = [
  { id: "csv" as TabId, label: "CSV 病患清單", icon: "fa-solid fa-table" },
  { id: "pdf" as TabId, label: "PDF 排程表",   icon: "fa-solid fa-file-pdf" },
];
const activeTab = ref<TabId>("pdf");

function switchTab(id: TabId) {
  activeTab.value = id;
  importResult.value = null;
}

// ── Target date (P2) ──────────────────────────────────────────────────────────
function getTodayStr(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}
const todayStr = getTodayStr();
const targetDate = ref<string>(todayStr);

function dateToOrStartTs(dateStr: string): number {
  const [y, m, d] = dateStr.split("-").map(Number);
  return Math.floor(new Date(y, m - 1, d, 7, 30, 0).getTime() / 1000);
}

// ── Dup detection (P1) ────────────────────────────────────────────────────────
type DupStatus = "dup" | "dual" | null;

function computeDupFlags(tasks: Partial<SurgeryTask>[]): DupStatus[] {
  const existing = tasksStore.tasks;
  const flags: DupStatus[] = new Array(tasks.length).fill(null);
  for (let i = 0; i < tasks.length; i++) {
    const t = tasks[i];
    if (!t.chart_no) continue;
    const existingMatch = existing.find(e => e.chart_no === t.chart_no);
    if (existingMatch) {
      const newFlag: DupStatus = existingMatch.surgeon === t.surgeon ? "dup" : "dual";
      if (!flags[i] || newFlag === "dual") flags[i] = newFlag;
    }
    for (let j = 0; j < tasks.length; j++) {
      if (i === j) continue;
      const other = tasks[j];
      if (other.chart_no !== t.chart_no) continue;
      const newFlag: DupStatus = other.surgeon === t.surgeon ? "dup" : "dual";
      if (!flags[i] || newFlag === "dual") flags[i] = newFlag;
    }
  }
  return flags;
}

function applyDualAnnotations(tasks: SurgeryTask[], flags: DupStatus[]): SurgeryTask[] {
  return tasks.map((t, i) => {
    if (flags[i] !== "dual") return t;
    const batchPartner = tasks.find((o, j) => j !== i && o.chart_no === t.chart_no && o.surgeon !== t.surgeon);
    const existPartner = !batchPartner ? tasksStore.tasks.find(e => e.chart_no === t.chart_no && e.surgeon !== t.surgeon) : undefined;
    const partnerSurgeon = batchPartner?.surgeon || existPartner?.surgeon || "";
    if (!partnerSurgeon) return t;
    const note = t.vs_note ? `${t.vs_note} [合刀: ${partnerSurgeon}]` : `[合刀: ${partnerSurgeon}]`;
    return { ...t, vs_note: note };
  });
}

async function linkDualTasks() {
  const byChart = new Map<string, SurgeryTask[]>();
  for (const t of tasksStore.tasks) {
    if (!t.chart_no) continue;
    if (!byChart.has(t.chart_no)) byChart.set(t.chart_no, []);
    byChart.get(t.chart_no)!.push(t);
  }
  for (const [, group] of byChart) {
    if (group.length !== 2) continue;
    const [a, b] = group;
    if (a.surgeon === b.surgeon) continue;
    if (a.linked_task_id || b.linked_task_id) continue;
    try {
      await tasksDb.update({ ...a, linked_task_id: b.id } as SurgeryTask);
      await tasksDb.update({ ...b, linked_task_id: a.id } as SurgeryTask);
    } catch (e) {
      console.error("[ImportModal] 兩科連結失敗:", e);
    }
  }
}

// ── Shared ────────────────────────────────────────────────────────────────────
const importing = ref(false);
const importResult = ref<{ ok: boolean; message: string } | null>(null);

function urgencyColor(u?: string) {
  const m: Record<string, string> = {
    Trauma: "bg-red-800 text-red-200",
    Level1: "bg-orange-800 text-orange-200",
    Level2: "bg-yellow-800 text-yellow-200",
    Level3: "bg-blue-800 text-blue-200",
    Normal: "bg-gray-700 text-gray-300",
  };
  return m[u ?? "Normal"] ?? m.Normal;
}

// ── CSV logic ─────────────────────────────────────────────────────────────────
const csvInput = ref<HTMLInputElement>();
const csvDragging = ref(false);
const csvParsed = ref<Partial<SurgeryTask>[]>([]);
const csvErrors = ref<string[]>([]);
const csvDupFlags = ref<DupStatus[]>([]);
const csvValid = computed(() => csvParsed.value.filter((t) => t.patient_name?.trim()));

watch(csvParsed, (tasks) => { csvDupFlags.value = computeDupFlags(tasks); });

const CSV_COLS = [
  { key: "patient_name", alias: "病患姓名" },
  { key: "chart_no", alias: "病歷號" },
  { key: "urgency", alias: "緊急程度" },
  { key: "dept", alias: "科別" },
  { key: "procedure", alias: "手術名稱" },
  { key: "diagnosis", alias: "診斷" },
  { key: "expected_room", alias: "預計手術室" },
  { key: "est_time_mins", alias: "預計時間(分)" },
  { key: "vs_note", alias: "醫師備注" },
];

const COL_ALIAS: Record<string, string> = {
  "病患姓名": "patient_name", "姓名": "patient_name",
  "病歷號": "chart_no", "病歷號碼": "chart_no",
  "緊急程度": "urgency", "優先級": "urgency", urgency: "urgency",
  "科別": "dept", "科": "dept",
  "手術名稱": "procedure", "手術": "procedure",
  "診斷": "diagnosis",
  "預計手術室": "expected_room", "手術室": "expected_room",
  "預計時間(分)": "est_time_mins", "手術時間(分)": "est_time_mins", "預計時間": "est_time_mins",
  "醫師備注": "vs_note", "備注": "vs_note",
};

function normalizeUrgency(s: string): UrgencyLevel {
  const map: Record<string, UrgencyLevel> = {
    trauma: "Trauma", t: "Trauma",
    level1: "Level1", l1: "Level1", "1": "Level1",
    level2: "Level2", l2: "Level2", "2": "Level2",
    level3: "Level3", l3: "Level3", "3": "Level3",
    normal: "Normal", n: "Normal", "常規": "Normal",
  };
  return map[s.toLowerCase().trim()] ?? "Normal";
}

function parseCsv(text: string): string[][] {
  return text.split(/\r?\n/).filter((l) => l.trim()).map((line) => {
    const cols: string[] = [];
    let cur = "";
    let inQ = false;
    for (const ch of line) {
      if (ch === '"') { inQ = !inQ; continue; }
      if (ch === "," && !inQ) { cols.push(cur.trim()); cur = ""; continue; }
      cur += ch;
    }
    cols.push(cur.trim());
    return cols;
  });
}

function processCsvText(text: string) {
  csvErrors.value = [];
  csvParsed.value = [];
  const rows = parseCsv(text);
  if (rows.length < 2) { csvErrors.value.push("內容不足（需含標題行與至少一筆資料）"); return; }
  const header = rows[0].map((h) => COL_ALIAS[h.replace(/\s/g, "")] ?? h.replace(/\s/g, ""));
  const now = Math.floor(Date.now() / 1000);
  rows.slice(1).forEach((row, i) => {
    const obj: Record<string, string> = {};
    header.forEach((h, j) => { obj[h] = row[j] ?? ""; });
    if (!obj.patient_name) { csvErrors.value.push(`第 ${i + 2} 行：缺少病患姓名，略過`); return; }
    const importTs = targetDate.value && targetDate.value !== todayStr
      ? dateToOrStartTs(targetDate.value)
      : null;
    csvParsed.value.push({
      id: 0, patient_name: obj.patient_name, chart_no: obj.chart_no ?? "",
      urgency: normalizeUrgency(obj.urgency ?? ""), dept: obj.dept ?? "",
      procedure: obj.procedure ?? "", diagnosis: obj.diagnosis ?? "",
      expected_room: obj.expected_room ?? "",
      est_time_mins: parseInt(obj.est_time_mins ?? "60") || 60,
      vs_note: obj.vs_note ?? "", scheduled_at: importTs, created_at: now, status: "waiting",
      seq_no: 0, gender: "", age: 0, bed_no: "", body_part: "", anesthesia: "", surgeon: "",
      linked_task_id: null,
    });
  });
}

function onCsvDrop(e: DragEvent) {
  csvDragging.value = false;
  const file = e.dataTransfer?.files[0];
  if (file) readCsvFile(file);
}
function onCsvFileChange(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (file) readCsvFile(file);
}
function readCsvFile(file: File) {
  const reader = new FileReader();
  reader.onload = (e) => processCsvText(e.target?.result as string);
  reader.readAsText(file, "utf-8");
}
function resetCsv() {
  csvParsed.value = [];
  csvErrors.value = [];
  importResult.value = null;
  if (csvInput.value) csvInput.value.value = "";
}
async function doCsvImport() {
  importing.value = true;
  let count = 0;
  try {
    const annotated = applyDualAnnotations(csvValid.value as SurgeryTask[], csvDupFlags.value);
    for (const t of annotated) {
      await tasksStore.add(t);
      count++;
    }
    await tasksStore.load();
    await linkDualTasks();
    importResult.value = { ok: true, message: `✓ 成功匯入 ${count} 筆病患` };
    emit("imported", count);
  } catch (e) {
    importResult.value = { ok: false, message: `匯入失敗：${e}` };
    console.error("[ImportModal] CSV 匯入失敗:", e);
  } finally {
    importing.value = false;
  }
}

// ── PDF logic ─────────────────────────────────────────────────────────────────
type PdfStep = "drop" | "parsing" | "error" | "preview";
const pdfInput = ref<HTMLInputElement>();
const pdfDragging = ref(false);
const pdfStep = ref<PdfStep>("drop");
const pdfError = ref("");
const pdfDate = ref("");
const pdfTasks = ref<SurgeryTask[]>([]);
const pdfDupFlags = ref<DupStatus[]>([]);
const pdfPreviewFilter = ref<"all" | "scheduled" | "waiting">("all");

const pdfScheduled = computed(() => pdfTasks.value.filter((t) => t.status === "scheduled"));
const pdfWaiting = computed(() => pdfTasks.value.filter((t) => t.status === "waiting"));
const previewFiltered = computed(() => {
  if (pdfPreviewFilter.value === "scheduled") return pdfScheduled.value;
  if (pdfPreviewFilter.value === "waiting") return pdfWaiting.value;
  return pdfTasks.value;
});

watch(pdfTasks, (tasks) => { pdfDupFlags.value = computeDupFlags(tasks); });

function previewFilteredIdx(localIdx: number): number {
  const t = previewFiltered.value[localIdx];
  return pdfTasks.value.indexOf(t);
}

function shiftTasksToDate(tasks: SurgeryTask[], fromDate: string, toDate: string): SurgeryTask[] {
  if (!fromDate || fromDate === toDate) return tasks;
  const [fy, fm, fd] = fromDate.split("-").map(Number);
  const [ty, tm, td] = toDate.split("-").map(Number);
  const offsetSecs = Math.round((new Date(ty, tm - 1, td).getTime() - new Date(fy, fm - 1, fd).getTime()) / 1000);
  return tasks.map(t => ({
    ...t,
    scheduled_at: t.scheduled_at !== null ? t.scheduled_at + offsetSecs : null,
  }));
}

function fmtScheduledAt(ts: number | null): string {
  if (!ts) return "—";
  const d = new Date(ts * 1000);
  return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
}

function onPdfDrop(e: DragEvent) {
  pdfDragging.value = false;
  const file = e.dataTransfer?.files[0];
  if (file?.type === "application/pdf" || file?.name.endsWith(".pdf")) readPdfFile(file);
}
function onPdfFileChange(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (file) readPdfFile(file);
}
function readPdfFile(file: File) {
  const reader = new FileReader();
  reader.onload = async (e) => {
    pdfStep.value = "parsing";
    try {
      const buffer = e.target?.result as ArrayBuffer;
      const result = await parsePdfBuffer(buffer);
      pdfDate.value = result.date;
      let tasks = result.tasks;
      if (targetDate.value && targetDate.value !== result.date) {
        tasks = shiftTasksToDate(tasks, result.date, targetDate.value);
      }
      pdfTasks.value = tasks;
      if (tasks.length === 0) throw new Error("未解析到任何病患資料，請確認 PDF 格式是否為手術紀錄表");
      pdfStep.value = "preview";
    } catch (err) {
      pdfError.value = String(err);
      pdfStep.value = "error";
      console.error("[ImportModal] PDF 解析失敗:", err);
    }
  };
  reader.readAsArrayBuffer(file);
}
function resetPdf() {
  pdfStep.value = "drop";
  pdfDate.value = "";
  pdfTasks.value = [];
  pdfError.value = "";
  importResult.value = null;
  if (pdfInput.value) pdfInput.value.value = "";
}
async function doPdfImport() {
  importing.value = true;
  try {
    const annotated = applyDualAnnotations(pdfTasks.value, pdfDupFlags.value);
    await tasksDb.batchCreate(annotated);
    await tasksStore.load();
    await linkDualTasks();
    importResult.value = { ok: true, message: `✓ 成功匯入 ${pdfTasks.value.length} 筆（已排 ${pdfScheduled.value.length} / 待排 ${pdfWaiting.value.length}）` };
    emit("imported", pdfTasks.value.length);
  } catch (e) {
    importResult.value = { ok: false, message: `匯入失敗：${e}` };
    console.error("[ImportModal] PDF 匯入失敗:", e);
  } finally {
    importing.value = false;
  }
}

// ── PDF parsing engine ────────────────────────────────────────────────────────
const COL_COUNT = 16;
// 0=房號 1=序號 2=姓名 3=性別 4=AGE 5=科別 6=病歷號碼 7=床號 8=診斷 9=部位
// 10=術式 11=麻醉 12=手術者 13=流動/刷手護士(merged) 14=預起時間 15=備註
const enum Col { Room=0, Seq=1, Name=2, Gender=3, Age=4, Dept=5, Chart=6, Bed=7,
  Diag=8, Body=9, Proc=10, Anes=11, Surgeon=12, Circ=13, ReqTime=14, Note=15 }

interface TItem { str: string; x: number; y: number; }

async function parsePdfBuffer(buffer: ArrayBuffer): Promise<{ date: string; tasks: SurgeryTask[] }> {
  const pdfjsLib = await import("pdfjs-dist");
  pdfjsLib.GlobalWorkerOptions.workerSrc = pdfWorkerUrl;

  const pdf = await pdfjsLib.getDocument({ data: new Uint8Array(buffer) }).promise;
  console.log("[PDF] numPages:", pdf.numPages);

  let colCenters: number[] = [];
  let dateStr = "";
  const tasks: SurgeryTask[] = [];

  for (let pn = 1; pn <= pdf.numPages; pn++) {
    const page = await pdf.getPage(pn);
    const vp = page.getViewport({ scale: 1 });
    const tc = await page.getTextContent();

    // Build item list with screen-Y (top=0)
    const items: TItem[] = (tc.items as any[])
      .filter((it) => "str" in it && it.str.trim().length > 0)
      .map((it) => ({
        str: it.str.trim(),
        x: it.transform[4],
        y: vp.height - it.transform[5],
      }));
    items.sort((a, b) => a.y - b.y || a.x - b.x);

    if (pn === 1) {
      console.log("[PDF] p1 items (first 40):", items.slice(0, 40).map((it) => `"${it.str}"@(${it.x.toFixed(0)},${it.y.toFixed(0)})`));
    }

    // Extract ROC date (first page only) — match ##/##/## or ###/##/##
    if (pn === 1 && !dateStr) {
      for (const it of items) {
        const m = it.str.match(/(\d{2,3})\/(\d{1,2})\/(\d{1,2})/);
        if (m) {
          const roc = parseInt(m[1]);
          const yr = roc < 1000 ? roc + 1911 : roc;
          dateStr = `${yr}-${m[2].padStart(2, "0")}-${m[3].padStart(2, "0")}`;
          break;
        }
      }
      console.log("[PDF] dateStr:", dateStr || "(not found)");
    }

    // Group into lines (y-tolerance = 5)
    const lines: TItem[][] = [];
    for (const it of items) {
      const last = lines[lines.length - 1];
      if (last && Math.abs(it.y - last[0].y) <= 5) {
        last.push(it);
      } else {
        lines.push([it]);
      }
    }

    // Find header line whose joined text contains '姓名'
    const hdrIdx = lines.findIndex((ln) => ln.map((it) => it.str).join("").includes("姓名"));
    console.log(`[PDF] p${pn} hdrIdx:`, hdrIdx, hdrIdx >= 0 ? lines[hdrIdx].map((it) => it.str).join(" | ") : "(not found)");
    if (hdrIdx === -1) continue;

    // Detect column centers from header zone (first page only)
    if (colCenters.length === 0) {
      const hdrY = lines[hdrIdx][0].y;
      const zoneItems = items.filter(
        (it) =>
          Math.abs(it.y - hdrY) <= 12 &&
          !/^[\d\/：]+$/.test(it.str) &&
          it.str !== "手術紀錄表",
      );
      console.log("[PDF] zoneItems:", zoneItems.map((it) => `"${it.str}"@${it.x.toFixed(0)}`));
      const byX = [...zoneItems].sort((a, b) => a.x - b.x);
      const clusters: number[][] = [];
      for (const it of byX) {
        const last = clusters[clusters.length - 1];
        if (last && it.x - last[last.length - 1] <= 12) {
          last.push(it.x);
        } else {
          clusters.push([it.x]);
        }
      }
      colCenters = clusters.map((c) => c.reduce((a, b) => a + b, 0) / c.length);
      console.log("[PDF] colCenters (" + colCenters.length + "):", colCenters.map((x) => x.toFixed(0)));
    }
    if (colCenters.length < 5) {
      console.warn("[PDF] too few colCenters:", colCenters.length, "– skipping page");
      continue;
    }

    // Parse data rows
    let cur: string[] | null = null;
    let rowsFound = 0;

    const flushRow = () => {
      if (!cur) return;
      const t = rowToTask(cur, dateStr);
      if (t) tasks.push(t);
      cur = null;
    };

    for (let li = hdrIdx + 1; li < lines.length; li++) {
      const line = lines[li];

      // Skip short footer/page-number lines
      if (line.length <= 4 && line.every((it) => /^[\d\/頁次：年月日\s]+$/.test(it.str))) continue;

      // Map items to columns using 35% boundary (biases toward right column, fixes 術式 mis-mapping)
      const cells = new Array<string>(COL_COUNT).fill("");
      for (const it of line) {
        let best = 0;
        for (let c = 0; c < colCenters.length - 1 && c < COL_COUNT - 1; c++) {
          const boundary = colCenters[c] + (colCenters[c + 1] - colCenters[c]) * 0.35;
          if (it.x >= boundary) best = c + 1;
          else break;
        }
        if (best >= COL_COUNT) best = COL_COUNT - 1;
        cells[best] = cells[best] ? cells[best] + " " + it.str : it.str;
      }

      let seq = cells[Col.Seq].trim();
      let name = cells[Col.Name].trim();
      // PDF merges seq+name into one text item (e.g. "1 潘萬益") at the seq column position
      if (!name) {
        const m = seq.match(/^(\d+)\s+(.{2,})$/);
        if (m) { seq = m[1]; name = m[2]; }
      }
      const isNewRow = /^\d+$/.test(seq) && name.length >= 2 && /[一-鿿㐀-䶿]/.test(name);

      if (rowsFound < 3) {
        console.log(`[PDF] line ${li}: seq="${seq}" name="${name}" isNewRow=${isNewRow} cells[0..4]=${cells.slice(0, 5)}`);
        if (isNewRow) rowsFound++;
      }

      if (isNewRow) {
        flushRow();
        cells[Col.Seq] = seq;
        cells[Col.Name] = name;
        cur = [...cells];
      } else if (cur) {
        // Continuation: append to text-heavy columns only
        for (let c = Col.Diag; c <= Col.Note; c++) {
          if (cells[c].trim()) {
            cur[c] = cur[c] ? cur[c] + " " + cells[c].trim() : cells[c].trim();
          }
        }
      }
    }
    flushRow();
  }

  console.log("[PDF] total tasks parsed:", tasks.length);
  return { date: dateStr, tasks };
}

function rowToTask(cells: string[], date: string): SurgeryTask | null {
  const name = cells[Col.Name]?.trim() || "";
  const seq = cells[Col.Seq]?.trim() || "";
  const room = cells[Col.Room]?.trim() || "";
  if (!name || !seq) return null;

  const isWT = room.toUpperCase() === "WT";

  let scheduled_at: number | null = null;
  const timeStr = cells[Col.ReqTime]?.trim() ?? "";
  const timeMatch = timeStr.match(/\b(\d{1,2}:\d{2})\b/);
  if (!isWT && timeMatch && date) {
    const [h, m] = timeMatch[1].split(":").map(Number);
    const [y, mo, d] = date.split("-").map(Number);
    scheduled_at = Math.floor(new Date(y, mo - 1, d, h, m, 0).getTime() / 1000);
  }

  return {
    id: 0,
    seq_no: parseInt(seq) || 0,
    patient_name: name,
    gender: cells[Col.Gender]?.trim() || "",
    age: parseInt(cells[Col.Age]?.trim() || "0") || 0,
    chart_no: cells[Col.Chart]?.trim() || "",
    bed_no: cells[Col.Bed]?.trim() || "",
    dept: cells[Col.Dept]?.trim() || "",
    diagnosis: cells[Col.Diag]?.trim() || "",
    body_part: cells[Col.Body]?.trim() || "",
    procedure: cells[Col.Proc]?.trim() || "",
    anesthesia: cells[Col.Anes]?.trim() || "",
    surgeon: cells[Col.Surgeon]?.trim() || "",
    vs_note: cells[Col.Note]?.trim() || "",
    expected_room: isWT ? "" : room,
    urgency: "Normal",
    scheduled_at,
    created_at: 0,
    est_time_mins: 60,
    status: isWT ? "waiting" : "scheduled",
    linked_task_id: null,
  };
}
</script>
