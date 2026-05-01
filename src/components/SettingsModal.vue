<template>
  <div class="fixed inset-0 bg-black/70 z-50 flex items-center justify-center p-4" @mousedown.self="$emit('close')">
    <div class="bg-gray-800 rounded-xl shadow-2xl w-full max-w-3xl flex flex-col max-h-[90vh]">

      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-4 border-b border-gray-700 shrink-0">
        <h2 class="text-base font-bold text-white"><i class="fa-solid fa-gear text-gray-400 mr-2"></i>設定</h2>
        <button class="text-gray-400 hover:text-white" @click="$emit('close')">
          <i class="fa-solid fa-xmark"></i>
        </button>
      </div>

      <!-- Tabs -->
      <div class="flex border-b border-gray-700 shrink-0 px-5">
        <button
          v-for="t in TABS" :key="t.key"
          class="px-4 py-2.5 text-sm font-medium border-b-2 transition-colors"
          :class="tab === t.key
            ? 'border-blue-500 text-blue-400'
            : 'border-transparent text-gray-400 hover:text-gray-200'"
          @click="tab = t.key"
        >
          <i :class="`fa-solid ${t.icon} mr-1.5`"></i>{{ t.label }}
        </button>
      </div>

      <!-- Body -->
      <div class="flex-1 overflow-y-auto p-5">

        <!-- ── 房間管理 ──────────────────────────────────────────── -->
        <template v-if="tab === 'rooms'">
          <div class="flex items-center justify-between mb-4">
            <span class="text-sm text-gray-400">共 {{ roomsStore.rooms.length }} 間手術室</span>
            <div class="flex gap-2">
              <button class="btn-secondary text-xs flex items-center gap-1.5" @click="triggerXlsx">
                <i class="fa-solid fa-file-excel text-green-400"></i>匯入月排班 (.xlsx)
              </button>
              <input ref="xlsxInput" type="file" accept=".xlsx,.xls" class="hidden" @change="onXlsxFile" />
              <button class="btn-primary text-xs" @click="startAddRoom">
                <i class="fa-solid fa-plus mr-1"></i>新增房間
              </button>
            </div>
          </div>

          <!-- Room list -->
          <div class="space-y-1.5">
            <div
              v-for="(room, idx) in roomsStore.rooms" :key="room.id"
              class="flex items-center gap-3 bg-gray-700/50 rounded-lg px-3 py-2.5"
            >
              <span class="text-gray-500 text-xs w-5 text-center">{{ idx + 1 }}</span>
              <i v-if="room.is_backup" class="fa-solid fa-circle-exclamation text-orange-400 text-xs" title="備用刀房"></i>
              <i v-else class="fa-solid fa-door-open text-blue-400 text-xs"></i>
              <span class="flex-1 text-sm text-gray-100 font-medium">{{ room.name }}</span>
              <span v-if="room.is_backup" class="text-[10px] text-orange-400 bg-orange-900/40 px-1.5 py-0.5 rounded">備用</span>
              <button class="text-gray-500 hover:text-blue-400 text-xs px-1" @click="startEditRoom(room)">
                <i class="fa-solid fa-pen"></i>
              </button>
              <button class="text-gray-500 hover:text-red-400 text-xs px-1" @click="deleteRoom(room.id)">
                <i class="fa-solid fa-trash"></i>
              </button>
            </div>
            <div v-if="roomsStore.rooms.length === 0" class="text-center text-gray-600 py-8 text-sm">
              尚未設定任何房間，請點擊「新增房間」或匯入月排班 Excel
            </div>
          </div>

          <!-- Room add/edit inline form -->
          <div v-if="roomForm.open" class="mt-4 bg-gray-700/60 rounded-xl p-4 border border-gray-600">
            <div class="text-sm font-semibold text-gray-300 mb-3">
              {{ roomForm.id ? '編輯房間' : '新增房間' }}
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="form-label">房間名稱 <span class="text-red-400">*</span></label>
                <input v-model="roomForm.name" class="form-input" placeholder="OR1" />
              </div>
              <div>
                <label class="form-label">顯示順序</label>
                <input v-model.number="roomForm.display_order" type="number" min="0" class="form-input" />
              </div>
            </div>
            <label class="flex items-center gap-2 mt-3 cursor-pointer">
              <input type="checkbox" v-model="roomForm.is_backup" class="rounded" />
              <span class="text-sm text-gray-300">備用刀房（二線機制才啟用）</span>
            </label>
            <div class="flex justify-end gap-2 mt-3">
              <button class="btn-secondary text-xs" @click="roomForm.open = false">取消</button>
              <button class="btn-primary text-xs" :disabled="!roomForm.name.trim()" @click="saveRoom">
                {{ roomForm.id ? '儲存' : '新增' }}
              </button>
            </div>
          </div>

          <!-- XLSX preview -->
          <div v-if="xlsxPreview.length > 0" class="mt-4 bg-gray-900/60 rounded-xl p-4 border border-green-700/50">
            <div class="flex items-center justify-between mb-3">
              <div class="text-sm font-semibold text-green-300">
                <i class="fa-solid fa-table mr-1.5"></i>預覽匯入 — {{ xlsxMonth }}（共 {{ xlsxPreview.length }} 筆）
              </div>
              <div class="flex gap-2">
                <button class="btn-secondary text-xs" @click="xlsxPreview = []">取消</button>
                <button class="btn-primary text-xs" :disabled="importSaving" @click="confirmXlsxImport">
                  <i v-if="importSaving" class="fa-solid fa-spinner animate-spin mr-1"></i>
                  確認匯入
                </button>
              </div>
            </div>
            <div v-if="xlsxErrors.length" class="mb-2 text-xs text-red-400">
              略過 {{ xlsxErrors.length }} 筆無效資料
            </div>
            <div class="overflow-x-auto max-h-52">
              <table class="w-full text-xs text-gray-300 border-collapse">
                <thead>
                  <tr class="text-gray-500 border-b border-gray-700">
                    <th class="text-left py-1 px-2">日期</th>
                    <th class="text-left py-1 px-2">房間</th>
                    <th class="text-left py-1 px-2">科別</th>
                    <th class="text-left py-1 px-2">開始</th>
                    <th class="text-left py-1 px-2">結束</th>
                    <th class="text-left py-1 px-2">備注</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="(e, i) in xlsxPreview.slice(0, 10)" :key="i" class="border-b border-gray-800">
                    <td class="py-1 px-2">{{ e.date }}</td>
                    <td class="py-1 px-2">{{ e.room_name }}</td>
                    <td class="py-1 px-2">{{ e.dept }}</td>
                    <td class="py-1 px-2">{{ fmtTs(e.start_time) }}</td>
                    <td class="py-1 px-2">{{ fmtTs(e.end_time) }}</td>
                    <td class="py-1 px-2 text-gray-500">{{ e.notes }}</td>
                  </tr>
                </tbody>
              </table>
              <div v-if="xlsxPreview.length > 10" class="text-center text-gray-600 text-xs py-1">
                … 還有 {{ xlsxPreview.length - 10 }} 筆
              </div>
            </div>
          </div>
        </template>

        <!-- ── 人員管理 ──────────────────────────────────────────── -->
        <template v-if="tab === 'staff'">
          <div class="flex items-center justify-between mb-4">
            <span class="text-sm text-gray-400">共 {{ staffStore.staffList.length }} 名人員</span>
            <button class="btn-primary text-xs" @click="startAddStaff">
              <i class="fa-solid fa-plus mr-1"></i>新增人員
            </button>
          </div>

          <!-- Staff add/edit inline form -->
          <div v-if="staffForm.open" class="mb-4 bg-gray-700/60 rounded-xl p-4 border border-gray-600">
            <div class="text-sm font-semibold text-gray-300 mb-3">
              {{ staffForm.id ? '編輯人員' : '新增人員' }}
            </div>
            <div class="grid grid-cols-2 gap-3">
              <div>
                <label class="form-label">姓名 <span class="text-red-400">*</span></label>
                <input v-model="staffForm.name" class="form-input" placeholder="王小明" />
              </div>
              <div>
                <label class="form-label">類別 <span class="text-red-400">*</span></label>
                <select v-model="staffForm.staff_category" class="form-input">
                  <option v-for="(label, key) in CATEGORY_LABELS" :key="key" :value="key">{{ label }}</option>
                </select>
              </div>
              <div v-if="staffForm.staff_category === 'cross_train'">
                <label class="form-label">所屬單位 <span class="text-red-400">*</span></label>
                <input v-model="staffForm.unit" class="form-input" placeholder="心臟外科加護病房" />
              </div>
              <div>
                <label class="form-label">手術室角色</label>
                <select v-model="staffForm.role" class="form-input">
                  <option value="Scrub">刷手 (Scrub)</option>
                  <option value="Circ">流動 (Circ)</option>
                  <option value="R">住院醫師 (R)</option>
                  <option value="VS">主治醫師 (VS)</option>
                </select>
              </div>
            </div>
            <div class="flex gap-4 mt-3">
              <label class="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" v-model="staffForm.is_on_call" class="rounded" />
                <span class="text-sm text-gray-300">今日值班 (二線待命)</span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer">
                <input type="checkbox" v-model="staffForm.is_volunteer_extra" class="rounded" />
                <span class="text-sm text-gray-300">可加 Extra</span>
              </label>
            </div>
            <div class="flex justify-end gap-2 mt-3">
              <button class="btn-secondary text-xs" @click="staffForm.open = false">取消</button>
              <button
                class="btn-primary text-xs"
                :disabled="!staffForm.name.trim() || (staffForm.staff_category === 'cross_train' && !staffForm.unit.trim())"
                @click="saveStaff"
              >
                {{ staffForm.id ? '儲存' : '新增' }}
              </button>
            </div>
          </div>

          <!-- Staff list grouped by category -->
          <div v-for="cat in CATEGORIES" :key="cat">
            <div v-if="staffByCategory[cat]?.length">
              <div class="flex items-center gap-2 mb-2 mt-3">
                <span class="text-xs font-semibold uppercase tracking-widest" :class="categoryColor(cat)">
                  {{ CATEGORY_LABELS[cat] }}
                </span>
                <div class="flex-1 border-t border-gray-700/60"></div>
                <span class="text-[10px] text-gray-600">{{ staffByCategory[cat].length }} 人</span>
              </div>
              <div class="space-y-1">
                <div
                  v-for="s in staffByCategory[cat]" :key="s.id"
                  class="flex items-center gap-3 bg-gray-700/40 rounded-lg px-3 py-2"
                >
                  <i class="fa-solid fa-user text-gray-500 text-xs w-4 text-center"></i>
                  <span class="flex-1 text-sm text-gray-100">{{ s.name }}</span>
                  <span v-if="s.unit" class="text-[10px] text-gray-500 truncate max-w-[100px]">{{ s.unit }}</span>
                  <span class="text-[10px] text-gray-500 bg-gray-800 px-1.5 py-0.5 rounded">{{ s.role }}</span>
                  <span v-if="s.is_on_call" class="text-[10px] text-yellow-400 bg-yellow-900/40 px-1.5 py-0.5 rounded">值班</span>
                  <span v-if="s.is_volunteer_extra" class="text-[10px] text-purple-400 bg-purple-900/40 px-1.5 py-0.5 rounded">Extra</span>
                  <button class="text-gray-500 hover:text-blue-400 text-xs px-1" @click="startEditStaff(s)">
                    <i class="fa-solid fa-pen"></i>
                  </button>
                  <button class="text-gray-500 hover:text-red-400 text-xs px-1" @click="deleteStaff(s.id)">
                    <i class="fa-solid fa-trash"></i>
                  </button>
                </div>
              </div>
            </div>
          </div>
          <div v-if="staffStore.staffList.length === 0" class="text-center text-gray-600 py-8 text-sm">
            尚未設定人員，請點擊「新增人員」
          </div>
        </template>

        <!-- ── 雲端設定 ──────────────────────────────────────────── -->
        <template v-if="tab === 'cloud'">
          <div class="max-w-lg space-y-4">
            <div>
              <label class="form-label">Google Apps Script Web App URL</label>
              <input
                v-model="gasUrl"
                class="form-input"
                placeholder="https://script.google.com/macros/s/.../exec"
              />
              <p class="text-xs text-gray-600 mt-1">部署 GAS 後取得的 Web App URL，用於雲端同步（Google Sheets）</p>
            </div>
            <button class="btn-primary text-sm" :disabled="cloudSaved" @click="saveGasUrl">
              <i v-if="cloudSaved" class="fa-solid fa-check mr-1.5 text-green-300"></i>
              {{ cloudSaved ? '已儲存' : '儲存' }}
            </button>
          </div>
        </template>

      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from "vue";
import * as XLSX from "xlsx";
import { useRoomsStore } from "../stores/rooms";
import { useStaffStore } from "../stores/staff";
import { useRoomShiftsDb, useSettings } from "../composables/useDatabase";
import type { Room, Staff, RoomScheduleEntry, StaffCategory } from "../types";
import { STAFF_CATEGORY_LABELS } from "../types";

defineEmits<{ close: [] }>();

const tab = ref<"rooms" | "staff" | "cloud">("rooms");
const TABS = [
  { key: "rooms", label: "房間管理", icon: "fa-door-open" },
  { key: "staff", label: "人員管理", icon: "fa-users" },
  { key: "cloud", label: "雲端設定", icon: "fa-cloud" },
] as const;

const CATEGORY_LABELS = STAFF_CATEGORY_LABELS;
const CATEGORIES: StaffCategory[] = ["sa", "or_nurse", "intern", "cross_train"];

const roomsStore = useRoomsStore();
const staffStore = useStaffStore();
const roomShiftsDb = useRoomShiftsDb();
const { getGasUrl, setGasUrl } = useSettings();

// ── Room form ─────────────────────────────────────────────────────────────────
const roomForm = reactive({
  open: false,
  id: 0,
  name: "",
  display_order: 0,
  is_backup: false,
});

function startAddRoom() {
  Object.assign(roomForm, { open: true, id: 0, name: "", display_order: roomsStore.rooms.length, is_backup: false });
}

function startEditRoom(r: Room) {
  Object.assign(roomForm, { open: true, id: r.id, name: r.name, display_order: r.display_order, is_backup: r.is_backup });
}

async function saveRoom() {
  if (!roomForm.name.trim()) return;
  const payload: Room = {
    id: roomForm.id,
    name: roomForm.name.trim(),
    display_order: roomForm.display_order,
    is_backup: roomForm.is_backup,
  };
  if (roomForm.id) {
    await roomsStore.edit(payload);
  } else {
    await roomsStore.add(payload);
  }
  roomForm.open = false;
}

async function deleteRoom(id: number) {
  if (!confirm("確定刪除此房間？")) return;
  await roomsStore.remove(id);
}

// ── XLSX import ───────────────────────────────────────────────────────────────
const xlsxInput = ref<HTMLInputElement | null>(null);
const xlsxPreview = ref<RoomScheduleEntry[]>([]);
const xlsxErrors = ref<string[]>([]);
const xlsxMonth = ref("");
const importSaving = ref(false);

function triggerXlsx() {
  xlsxInput.value?.click();
}

function onXlsxFile(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (!file) return;
  const reader = new FileReader();
  reader.onload = (ev) => {
    const data = new Uint8Array(ev.target!.result as ArrayBuffer);
    parseXlsx(data);
    (e.target as HTMLInputElement).value = "";
  };
  reader.readAsArrayBuffer(file);
}

const COL_ALIASES: Record<string, string> = {
  date: "date", 日期: "date", 手術日期: "date",
  room: "room", 房間: "room", 刀房: "room", 手術室: "room",
  dept: "dept", 科別: "dept", 科室: "dept", 使用科別: "dept",
  start: "start", 開始: "start", 開始時間: "start", 起始時間: "start",
  end: "end", 結束: "end", 結束時間: "end",
  notes: "notes", 備注: "notes", 備忘: "notes", 說明: "notes",
};

function parseXlsx(data: Uint8Array) {
  const wb = XLSX.read(data, { type: "array", cellDates: true });
  const ws = wb.Sheets[wb.SheetNames[0]];
  const rows: Record<string, unknown>[] = XLSX.utils.sheet_to_json(ws, { raw: false, defval: "" });

  const entries: RoomScheduleEntry[] = [];
  const errors: string[] = [];

  for (const row of rows) {
    // Normalize keys
    const norm: Record<string, string> = {};
    for (const [k, v] of Object.entries(row)) {
      const mapped = COL_ALIASES[k.trim()];
      if (mapped) norm[mapped] = String(v ?? "").trim();
    }

    const dateStr = parseExcelDate(norm["date"]);
    const roomName = norm["room"]?.trim();
    const dept = norm["dept"]?.trim();
    const startSecs = parseExcelTime(norm["start"]);
    const endSecs = parseExcelTime(norm["end"]);

    if (!dateStr || !roomName || !dept || startSecs === null || endSecs === null) {
      errors.push(`略過: ${JSON.stringify(norm)}`);
      continue;
    }

    const midnight = dateToMidnightUnix(dateStr);
    entries.push({
      id: 0,
      room_name: roomName,
      dept,
      date: dateStr,
      start_time: midnight + startSecs,
      end_time: midnight + endSecs,
      notes: norm["notes"] ?? "",
    });
  }

  if (entries.length === 0) {
    errors.push("未解析到任何有效資料，請確認欄位名稱（日期/房間/科別/開始/結束）");
  }

  // Detect month from first entry
  xlsxMonth.value = entries[0]?.date?.slice(0, 7) ?? "";
  xlsxPreview.value = entries;
  xlsxErrors.value = errors;
}

function parseExcelDate(val: string | undefined): string | null {
  if (!val) return null;
  // YYYY/MM/DD or YYYY-MM-DD
  let m = val.match(/(\d{4})[\/\-](\d{1,2})[\/\-](\d{1,2})/);
  if (m) return `${m[1]}-${m[2].padStart(2, "0")}-${m[3].padStart(2, "0")}`;
  // MM/DD/YYYY
  m = val.match(/(\d{1,2})\/(\d{1,2})\/(\d{4})/);
  if (m) return `${m[3]}-${m[1].padStart(2, "0")}-${m[2].padStart(2, "0")}`;
  return null;
}

function parseExcelTime(val: string | undefined): number | null {
  if (!val) return null;
  // HH:MM or HH:MM:SS
  const m = val.match(/^(\d{1,2}):(\d{2})/);
  if (m) return parseInt(m[1]) * 3600 + parseInt(m[2]) * 60;
  return null;
}

function dateToMidnightUnix(dateStr: string): number {
  return Math.floor(new Date(dateStr + "T00:00:00").getTime() / 1000);
}

function fmtTs(ts: number): string {
  const d = new Date(ts * 1000);
  return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
}

async function confirmXlsxImport() {
  if (!xlsxMonth.value || xlsxPreview.value.length === 0) return;
  importSaving.value = true;
  try {
    await roomShiftsDb.replaceByMonth(xlsxMonth.value, xlsxPreview.value);
    // Auto-create missing rooms
    for (const e of xlsxPreview.value) {
      if (!roomsStore.rooms.find((r) => r.name === e.room_name)) {
        await roomsStore.add({ id: 0, name: e.room_name, display_order: roomsStore.rooms.length, is_backup: false });
      }
    }
    xlsxPreview.value = [];
    xlsxErrors.value = [];
  } finally {
    importSaving.value = false;
  }
}

// ── Staff form ────────────────────────────────────────────────────────────────
const staffForm = reactive({
  open: false,
  id: 0,
  name: "",
  role: "Circ",
  staff_category: "or_nurse" as StaffCategory,
  unit: "",
  is_on_call: false,
  is_volunteer_extra: false,
});

const staffByCategory = computed(() => {
  const map: Partial<Record<StaffCategory, Staff[]>> = {};
  for (const cat of CATEGORIES) {
    map[cat] = staffStore.staffList.filter((s) => s.staff_category === cat);
  }
  return map;
});

function categoryColor(cat: StaffCategory): string {
  return { sa: "text-blue-400", or_nurse: "text-green-400", intern: "text-yellow-400", cross_train: "text-purple-400" }[cat];
}

function startAddStaff() {
  Object.assign(staffForm, { open: true, id: 0, name: "", role: "Circ", staff_category: "or_nurse", unit: "", is_on_call: false, is_volunteer_extra: false });
}

function startEditStaff(s: Staff) {
  Object.assign(staffForm, { open: true, id: s.id, name: s.name, role: s.role, staff_category: s.staff_category, unit: s.unit, is_on_call: s.is_on_call, is_volunteer_extra: s.is_volunteer_extra });
}

async function saveStaff() {
  if (!staffForm.name.trim()) return;
  const now = Math.floor(Date.now() / 1000);
  const payload: Staff = {
    id: staffForm.id,
    name: staffForm.name.trim(),
    role: staffForm.role as Staff["role"],
    type: "nur",
    staff_category: staffForm.staff_category,
    unit: staffForm.unit.trim(),
    is_on_call: staffForm.is_on_call,
    is_volunteer_extra: staffForm.is_volunteer_extra,
    today_shift_start: now,
    next_day_shift_start: now + 86400,
  };
  if (staffForm.id) {
    await staffStore.edit(payload);
  } else {
    await staffStore.add(payload);
  }
  staffForm.open = false;
}

async function deleteStaff(id: number) {
  if (!confirm("確定刪除此人員？")) return;
  await staffStore.remove(id);
}

// ── Cloud settings ────────────────────────────────────────────────────────────
const gasUrl = ref("");
const cloudSaved = ref(false);

async function saveGasUrl() {
  await setGasUrl(gasUrl.value.trim());
  cloudSaved.value = true;
  setTimeout(() => (cloudSaved.value = false), 2000);
}

onMounted(async () => {
  await Promise.all([roomsStore.load(), staffStore.load()]);
  const url = await getGasUrl();
  if (url) gasUrl.value = url;
});
</script>
