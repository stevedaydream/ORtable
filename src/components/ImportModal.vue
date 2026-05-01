<template>
  <div class="fixed inset-0 bg-black/70 z-50 flex items-center justify-center p-4" @mousedown.self="$emit('close')">
    <div class="bg-gray-800 rounded-xl shadow-2xl w-full max-w-2xl flex flex-col max-h-[90vh]">

      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-4 border-b border-gray-700 shrink-0">
        <h2 class="text-base font-bold text-white">
          <i class="fa-solid fa-file-import text-green-400 mr-2"></i>匯入病患清單
        </h2>
        <button class="text-gray-400 hover:text-white" @click="$emit('close')">
          <i class="fa-solid fa-xmark"></i>
        </button>
      </div>

      <!-- Body -->
      <div class="flex-1 overflow-y-auto px-5 py-4 space-y-4">

        <!-- Drop zone -->
        <div
          v-if="!parsed.length"
          class="border-2 border-dashed border-gray-600 rounded-lg p-8 text-center cursor-pointer hover:border-blue-500 transition-colors"
          :class="dragging ? 'border-blue-400 bg-blue-900/10' : ''"
          @dragover.prevent="dragging = true"
          @dragleave="dragging = false"
          @drop.prevent="onDrop"
          @click="fileInput?.click()"
        >
          <i class="fa-solid fa-cloud-arrow-up text-3xl text-gray-500 mb-3"></i>
          <p class="text-sm text-gray-400">拖放 CSV 檔案至此，或點擊選擇檔案</p>
          <p class="text-xs text-gray-600 mt-1">支援格式：.csv（UTF-8 編碼）</p>
          <input ref="fileInput" type="file" accept=".csv,.txt" class="hidden" @change="onFileChange" />
        </div>

        <!-- 格式說明 -->
        <div v-if="!parsed.length" class="bg-gray-900/50 rounded-lg p-4">
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

        <!-- 預覽表格 -->
        <div v-if="parsed.length">
          <div class="flex items-center justify-between mb-2">
            <span class="text-sm font-semibold text-gray-300">
              預覽（共 {{ parsed.length }} 筆）
            </span>
            <button class="text-xs text-gray-500 hover:text-gray-300" @click="reset">
              <i class="fa-solid fa-rotate-left mr-1"></i>重新選擇
            </button>
          </div>

          <!-- 錯誤列 -->
          <div v-if="errors.length" class="mb-3 bg-red-900/30 border border-red-700 rounded p-3 text-xs text-red-300 space-y-0.5">
            <div class="font-semibold mb-1">⚠ 以下列有問題（將略過）：</div>
            <div v-for="e in errors" :key="e">{{ e }}</div>
          </div>

          <div class="overflow-x-auto rounded-lg border border-gray-700">
            <table class="w-full text-xs">
              <thead class="bg-gray-900/60">
                <tr>
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
                <tr v-for="(t, i) in parsed.slice(0, 10)" :key="i" class="hover:bg-gray-700/30">
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
            <div v-if="parsed.length > 10" class="px-3 py-2 text-xs text-gray-600 bg-gray-900/30">
              ...以及 {{ parsed.length - 10 }} 筆未顯示
            </div>
          </div>
        </div>

      </div>

      <!-- Footer -->
      <div class="flex items-center justify-between px-5 py-4 border-t border-gray-700 shrink-0">
        <span v-if="importResult" class="text-sm" :class="importResult.ok ? 'text-green-400' : 'text-red-400'">
          {{ importResult.message }}
        </span>
        <span v-else class="text-xs text-gray-600">{{ parsed.length ? `${valid.length} 筆將被匯入` : '' }}</span>
        <div class="flex gap-3">
          <button class="btn-secondary" @click="$emit('close')">關閉</button>
          <button
            v-if="valid.length"
            class="btn-primary"
            :disabled="importing"
            @click="doImport"
          >
            <i v-if="importing" class="fa-solid fa-spinner animate-spin mr-1.5"></i>
            匯入 {{ valid.length }} 筆
          </button>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useTasksStore } from "../stores/tasks";
import type { SurgeryTask, UrgencyLevel } from "../types";

const emit = defineEmits<{ close: []; imported: [count: number] }>();
const tasksStore = useTasksStore();

const fileInput = ref<HTMLInputElement>();
const dragging = ref(false);
const importing = ref(false);
const importResult = ref<{ ok: boolean; message: string } | null>(null);
const parsed = ref<Partial<SurgeryTask>[]>([]);
const errors = ref<string[]>([]);

const valid = computed(() =>
  parsed.value.filter((t) => t.patient_name?.trim())
);

const CSV_COLS = [
  { key: "patient_name",   alias: "病患姓名" },
  { key: "chart_no",       alias: "病歷號" },
  { key: "urgency",        alias: "緊急程度" },
  { key: "dept",           alias: "科別" },
  { key: "procedure",      alias: "手術名稱" },
  { key: "diagnosis",      alias: "診斷" },
  { key: "expected_room",  alias: "預計手術室" },
  { key: "est_time_mins",  alias: "預計時間(分)" },
  { key: "vs_note",        alias: "醫師備注" },
];

// Column alias mapping
const COL_ALIAS: Record<string, string> = {
  "病患姓名": "patient_name", "姓名": "patient_name",
  "病歷號": "chart_no", "病歷號碼": "chart_no",
  "緊急程度": "urgency", "優先級": "urgency", "urgency": "urgency",
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
  return text
    .split(/\r?\n/)
    .filter((l) => l.trim())
    .map((line) => {
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

function processText(text: string) {
  errors.value = [];
  parsed.value = [];

  const rows = parseCsv(text);
  if (rows.length < 2) { errors.value.push("檔案內容不足（需含標題行與至少一筆資料）"); return; }

  const headerRow = rows[0].map((h) => {
    const normalized = h.replace(/\s/g, "");
    return COL_ALIAS[normalized] ?? normalized;
  });

  const now = Math.floor(Date.now() / 1000);

  rows.slice(1).forEach((row, i) => {
    const obj: Record<string, string> = {};
    headerRow.forEach((h, j) => { obj[h] = row[j] ?? ""; });

    if (!obj.patient_name) {
      errors.value.push(`第 ${i + 2} 行：缺少病患姓名，略過`);
      return;
    }

    parsed.value.push({
      id: 0,
      patient_name: obj.patient_name,
      chart_no: obj.chart_no ?? "",
      urgency: normalizeUrgency(obj.urgency ?? ""),
      dept: obj.dept ?? "",
      procedure: obj.procedure ?? "",
      diagnosis: obj.diagnosis ?? "",
      expected_room: obj.expected_room ?? "",
      est_time_mins: parseInt(obj.est_time_mins ?? "60") || 60,
      vs_note: obj.vs_note ?? "",
      scheduled_at: null,
      created_at: now,
      status: "waiting",
    });
  });
}

function onDrop(e: DragEvent) {
  dragging.value = false;
  const file = e.dataTransfer?.files[0];
  if (file) readFile(file);
}

function onFileChange(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (file) readFile(file);
}

function readFile(file: File) {
  const reader = new FileReader();
  reader.onload = (e) => processText(e.target?.result as string);
  reader.readAsText(file, "utf-8");
}

async function doImport() {
  importing.value = true;
  let count = 0;
  try {
    for (const t of valid.value) {
      await tasksStore.add(t as SurgeryTask);
      count++;
    }
    importResult.value = { ok: true, message: `✓ 成功匯入 ${count} 筆病患` };
    emit("imported", count);
  } catch (e) {
    importResult.value = { ok: false, message: `匯入失敗：${e}` };
  } finally {
    importing.value = false;
  }
}

function reset() {
  parsed.value = [];
  errors.value = [];
  importResult.value = null;
  if (fileInput.value) fileInput.value.value = "";
}

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
</script>
