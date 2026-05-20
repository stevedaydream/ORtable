<template>
  <div class="fixed inset-0 bg-black/70 z-50 flex items-center justify-center p-4" @mousedown.self="$emit('close')">
    <div class="bg-gray-800 rounded-xl shadow-2xl w-full max-w-3xl flex flex-col max-h-[90vh]">

      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-4 border-b border-gray-700 shrink-0">
        <h2 class="text-base font-bold text-white">
          <i class="fa-solid fa-calendar-days text-blue-400 mr-2"></i>科別房間分配
        </h2>
        <div class="flex items-center gap-3">
          <input
            type="date"
            v-model="selectedDate"
            class="bg-gray-700 border border-gray-600 rounded px-2 py-1 text-xs text-blue-300 font-mono outline-none focus:border-blue-500 [color-scheme:dark]"
            @change="loadForDate"
          />
          <span class="text-[10px] text-gray-500">({{ weekday }})</span>
          <button class="text-gray-400 hover:text-white ml-2" @click="$emit('close')">
            <i class="fa-solid fa-xmark"></i>
          </button>
        </div>
      </div>

      <!-- Body -->
      <div class="flex-1 overflow-y-auto px-5 py-4">
        <!-- Loading -->
        <div v-if="loading" class="flex items-center justify-center py-16 gap-3 text-gray-500">
          <i class="fa-solid fa-spinner animate-spin"></i><span class="text-xs">載入中…</span>
        </div>

        <!-- Table -->
        <div v-else>
          <div class="overflow-x-auto rounded-lg border border-gray-700">
            <table class="w-full text-xs">
              <thead class="bg-gray-900/60">
                <tr>
                  <th class="px-3 py-2 text-left text-gray-400 font-medium w-24">房間</th>
                  <th class="px-3 py-2 text-left text-gray-400 font-medium w-28">科別</th>
                  <th class="px-3 py-2 text-left text-gray-400 font-medium w-20">開始</th>
                  <th class="px-3 py-2 text-left text-gray-400 font-medium w-20">結束</th>
                  <th class="px-3 py-2 text-left text-gray-400 font-medium">備注</th>
                  <th class="px-3 py-2 text-center text-gray-400 font-medium w-14">急診</th>
                  <th class="px-3 py-2 text-right text-gray-400 font-medium w-20">操作</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-700/50">
                <tr v-for="(row, i) in rows" :key="i" class="hover:bg-gray-700/20" :class="row.is_emergency ? 'bg-red-950/20' : ''">
                  <!-- View row -->
                  <template v-if="editIdx !== i">
                    <td class="px-3 py-2 text-blue-300 font-mono">{{ row.room_name }}</td>
                    <td class="px-3 py-2 text-gray-200">{{ row.dept }}</td>
                    <td class="px-3 py-2 text-gray-400 font-mono">{{ tsToTime(row.start_time) }}</td>
                    <td class="px-3 py-2 text-gray-400 font-mono">{{ tsToTime(row.end_time) }}</td>
                    <td class="px-3 py-2 text-gray-500">{{ row.notes }}</td>
                    <td class="px-3 py-2 text-center">
                      <span v-if="row.is_emergency" class="text-[9px] font-bold text-red-300 bg-red-700/60 px-1.5 py-0.5 rounded">急診</span>
                    </td>
                    <td class="px-3 py-2 text-right">
                      <button class="text-gray-500 hover:text-blue-300 mr-2" title="編輯" @click="startRowEdit(i)">
                        <i class="fa-solid fa-pen text-[10px]"></i>
                      </button>
                      <button class="text-gray-500 hover:text-red-400" title="刪除" @click="deleteRow(i)">
                        <i class="fa-solid fa-trash text-[10px]"></i>
                      </button>
                    </td>
                  </template>
                  <!-- Edit row -->
                  <template v-else>
                    <td class="px-2 py-1.5">
                      <input v-model="editForm.room_name" list="room-list" class="form-input w-20" />
                    </td>
                    <td class="px-2 py-1.5">
                      <input v-model="editForm.dept" class="form-input w-24" />
                    </td>
                    <td class="px-2 py-1.5">
                      <input v-model="editForm.startStr" type="time" class="form-input w-20 [color-scheme:dark]" />
                    </td>
                    <td class="px-2 py-1.5">
                      <input v-model="editForm.endStr" type="time" class="form-input w-20 [color-scheme:dark]" />
                    </td>
                    <td class="px-2 py-1.5">
                      <input v-model="editForm.notes" class="form-input" />
                    </td>
                    <td class="px-2 py-1.5 text-center">
                      <input type="checkbox" v-model="editForm.is_emergency" class="accent-red-500" title="標記為急診房" />
                    </td>
                    <td class="px-2 py-1.5 text-right">
                      <button class="text-green-400 hover:text-green-300 mr-2" @click="confirmRowEdit(i)">
                        <i class="fa-solid fa-check text-[10px]"></i>
                      </button>
                      <button class="text-gray-500 hover:text-gray-300" @click="editIdx = -1">
                        <i class="fa-solid fa-xmark text-[10px]"></i>
                      </button>
                    </td>
                  </template>
                </tr>

                <!-- Add new row (always at bottom when addMode) -->
                <tr v-if="addMode" class="bg-blue-950/20">
                  <td class="px-2 py-1.5">
                    <input v-model="addForm.room_name" list="room-list" placeholder="房間名稱" class="form-input w-20" />
                  </td>
                  <td class="px-2 py-1.5">
                    <input v-model="addForm.dept" placeholder="科別" class="form-input w-24" />
                  </td>
                  <td class="px-2 py-1.5">
                    <input v-model="addForm.startStr" type="time" class="form-input w-20 [color-scheme:dark]" />
                  </td>
                  <td class="px-2 py-1.5">
                    <input v-model="addForm.endStr" type="time" class="form-input w-20 [color-scheme:dark]" />
                  </td>
                  <td class="px-2 py-1.5">
                    <input v-model="addForm.notes" placeholder="備注" class="form-input" />
                  </td>
                  <td class="px-2 py-1.5 text-center">
                    <input type="checkbox" v-model="addForm.is_emergency" class="accent-red-500" title="標記為急診房" />
                  </td>
                  <td class="px-2 py-1.5 text-right">
                    <button class="text-green-400 hover:text-green-300 mr-2" @click="confirmAdd">
                      <i class="fa-solid fa-check text-[10px]"></i>
                    </button>
                    <button class="text-gray-500 hover:text-gray-300" @click="addMode = false">
                      <i class="fa-solid fa-xmark text-[10px]"></i>
                    </button>
                  </td>
                </tr>

                <tr v-if="rows.length === 0 && !addMode">
                  <td colspan="6" class="px-3 py-8 text-center text-gray-600 text-xs">
                    此日期尚無排班資料，點「+ 新增」建立
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <!-- Room datalist -->
          <datalist id="room-list">
            <option v-for="r in roomsStore.rooms" :key="r.id" :value="r.name" />
          </datalist>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-between px-5 py-4 border-t border-gray-700 shrink-0">
        <button
          v-if="!addMode"
          class="text-xs px-3 py-1.5 rounded border border-blue-800 text-blue-300 hover:bg-blue-900/30 transition-colors"
          @click="startAdd"
        >
          <i class="fa-solid fa-plus mr-1.5"></i>新增排班
        </button>
        <span v-else class="text-xs text-gray-600">填寫上方欄位後按 ✓ 確認</span>
        <div class="flex gap-2">
          <span v-if="saveResult" class="text-xs mr-2" :class="saveResult.ok ? 'text-green-400' : 'text-red-400'">
            {{ saveResult.message }}
          </span>
          <button class="btn-secondary" @click="$emit('close')">關閉</button>
          <button class="btn-primary" :disabled="saving" @click="saveAll">
            <i v-if="saving" class="fa-solid fa-spinner animate-spin mr-1.5"></i>
            儲存當日排班
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive, onMounted } from "vue";
import type { RoomScheduleEntry } from "../types";
import { useRoomShiftsDb, useSettings } from "../composables/useDatabase";
import { useRoomsStore } from "../stores/rooms";

defineEmits<{ close: [] }>();
const roomShiftsDb = useRoomShiftsDb();
const roomsStore   = useRoomsStore();
const { getRoomGroups } = useSettings();

const roomGroupsMap = ref<Record<string, string[]>>({});

async function loadRoomGroups() {
  try {
    const raw = await getRoomGroups();
    roomGroupsMap.value = JSON.parse(raw || "{}");
  } catch { roomGroupsMap.value = {}; }
}

function expandByGroups(entries: RoomScheduleEntry[]): RoomScheduleEntry[] {
  const groups = roomGroupsMap.value;
  if (!Object.keys(groups).length) return entries;
  const result: RoomScheduleEntry[] = [];
  for (const entry of entries) {
    const physical = groups[entry.room_name];
    if (physical && physical.length) {
      const groupTag = /局麻|局部/.test(entry.room_name) ? '局麻'
        : /全麻|全身/.test(entry.room_name) ? '全麻' : '';
      for (const roomName of physical) {
        const notes = groupTag && !entry.notes.includes(groupTag)
          ? (entry.notes ? `${entry.notes} ${groupTag}` : groupTag)
          : entry.notes;
        result.push({ ...entry, id: 0, room_name: roomName, notes });
      }
    } else {
      result.push(entry);
    }
  }
  return result;
}

// ── State ─────────────────────────────────────────────────────────────────────
const loading = ref(false);
const saving = ref(false);
const saveResult = ref<{ ok: boolean; message: string } | null>(null);

const selectedDate = ref(getTodayStr());
const rows = ref<RoomScheduleEntry[]>([]);
const editIdx = ref(-1);
const addMode = ref(false);

function getTodayStr() {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,"0")}-${String(d.getDate()).padStart(2,"0")}`;
}

const weekday = computed(() => {
  if (!selectedDate.value) return "";
  const [y, m, d] = selectedDate.value.split("-").map(Number);
  return ["日","一","二","三","四","五","六"][new Date(y, m-1, d).getDay()];
});

// ── Time conversion ───────────────────────────────────────────────────────────
function dateToMidnight(dateStr: string): number {
  const [y, m, d] = dateStr.split("-").map(Number);
  return Math.floor(new Date(y, m-1, d).getTime() / 1000);
}
function timeToTs(timeStr: string): number {
  const midnight = dateToMidnight(selectedDate.value);
  const [h, min] = timeStr.split(":").map(Number);
  return midnight + h * 3600 + min * 60;
}
function tsToTime(ts: number): string {
  const d = new Date(ts * 1000);
  return `${String(d.getHours()).padStart(2,"0")}:${String(d.getMinutes()).padStart(2,"0")}`;
}

// ── Load ──────────────────────────────────────────────────────────────────────
async function loadForDate() {
  loading.value = true;
  editIdx.value = -1;
  addMode.value = false;
  saveResult.value = null;
  try {
    rows.value = await roomShiftsDb.getByDate(selectedDate.value);
  } catch (e) {
    console.error("[DeptRoomModal] 載入失敗:", e);
  } finally {
    loading.value = false;
  }
}

onMounted(async () => {
  await roomsStore.load();
  await Promise.all([loadForDate(), loadRoomGroups()]);
});

// ── Edit row ──────────────────────────────────────────────────────────────────
const editForm = reactive({ room_name: "", dept: "", startStr: "", endStr: "", notes: "", is_emergency: false });

function startRowEdit(i: number) {
  const r = rows.value[i];
  Object.assign(editForm, {
    room_name: r.room_name, dept: r.dept,
    startStr: tsToTime(r.start_time), endStr: tsToTime(r.end_time),
    notes: r.notes, is_emergency: r.is_emergency ?? false,
  });
  editIdx.value = i;
  addMode.value = false;
}

function confirmRowEdit(i: number) {
  rows.value[i] = {
    ...rows.value[i],
    room_name: editForm.room_name,
    dept: editForm.dept,
    start_time: timeToTs(editForm.startStr),
    end_time:   timeToTs(editForm.endStr),
    notes: editForm.notes,
    is_emergency: editForm.is_emergency,
    date: selectedDate.value,
  };
  editIdx.value = -1;
}

function deleteRow(i: number) {
  rows.value.splice(i, 1);
  if (editIdx.value === i) editIdx.value = -1;
}

// ── Add row ───────────────────────────────────────────────────────────────────
const addForm = reactive({ room_name: "", dept: "", startStr: "07:30", endStr: "12:30", notes: "", is_emergency: false });

function startAdd() {
  addMode.value = true;
  editIdx.value = -1;
  Object.assign(addForm, { room_name: "", dept: "", startStr: "07:30", endStr: "12:30", notes: "", is_emergency: false });
}

function confirmAdd() {
  if (!addForm.room_name || !addForm.dept) return;
  rows.value.push({
    id: 0,
    room_name: addForm.room_name,
    dept: addForm.dept,
    date: selectedDate.value,
    start_time: timeToTs(addForm.startStr),
    end_time:   timeToTs(addForm.endStr),
    notes: addForm.notes,
    is_emergency: addForm.is_emergency,
  });
  addMode.value = false;
}

// ── Save ──────────────────────────────────────────────────────────────────────
async function saveAll() {
  saving.value = true;
  saveResult.value = null;
  try {
    const toSave = expandByGroups(rows.value);
    await roomShiftsDb.replaceByDate(selectedDate.value, toSave);
    const expandedNote = toSave.length !== rows.value.length ? `（展開後 ${toSave.length} 筆）` : "";
    saveResult.value = { ok: true, message: `✓ 已儲存 ${rows.value.length} 筆排班${expandedNote}` };
  } catch (e) {
    saveResult.value = { ok: false, message: `儲存失敗：${e}` };
    console.error("[DeptRoomModal] 儲存失敗:", e);
  } finally {
    saving.value = false;
  }
}
</script>
