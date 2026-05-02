<template>
  <div class="flex flex-col h-full bg-gray-900">

    <!-- Toolbar -->
    <div class="flex items-center gap-3 px-4 py-2.5 border-b border-gray-700 shrink-0 bg-gray-800">
      <button class="flex items-center gap-1.5 text-xs text-gray-400 hover:text-gray-200 transition-colors" @click="$emit('back')">
        <i class="fa-solid fa-arrow-left"></i>返回看板
      </button>
      <div class="h-4 w-px bg-gray-700" />
      <span class="text-sm font-bold text-white">病人清單</span>
      <span class="text-xs text-gray-500">共 {{ filtered.length }} 筆</span>
      <div class="flex-1" />
      <div class="relative">
        <i class="fa-solid fa-magnifying-glass absolute left-2.5 top-1/2 -translate-y-1/2 text-gray-500 text-[10px]"></i>
        <input
          v-model="search"
          placeholder="搜尋姓名 / 病歷號…"
          class="bg-gray-700 border border-gray-600 rounded pl-7 pr-3 py-1 text-xs text-gray-200 outline-none focus:border-blue-500 w-44"
        />
      </div>
      <select v-model="statusFilter" class="bg-gray-700 border border-gray-600 rounded px-2 py-1 text-xs text-gray-200 outline-none focus:border-blue-500">
        <option value="">全部狀態</option>
        <option v-for="s in STATUS_OPTIONS" :key="s.value" :value="s.value">{{ s.label }}</option>
      </select>
      <button class="btn-primary text-xs" @click="$emit('import')">
        <i class="fa-solid fa-file-import mr-1.5"></i>匯入
      </button>
      <button class="btn-secondary text-xs" @click="$emit('add-emergency')">
        <i class="fa-solid fa-plus-circle mr-1.5"></i>新增急診
      </button>
    </div>

    <!-- Batch action bar -->
    <Transition name="slide-down">
      <div v-if="selectedIds.size > 0" class="flex items-center gap-3 px-4 py-2 bg-blue-950/60 border-b border-blue-800/60 shrink-0">
        <i class="fa-solid fa-check-square text-blue-400 text-xs"></i>
        <span class="text-xs text-blue-200 font-medium">已選取 <span class="font-bold text-white">{{ selectedIds.size }}</span> 筆</span>
        <button class="text-xs text-gray-400 hover:text-gray-200 underline" @click="clearSelection">取消選取</button>
        <div class="flex-1" />
        <button
          class="flex items-center gap-1.5 text-xs px-3 py-1.5 rounded bg-red-800/70 border border-red-700 text-red-200 hover:bg-red-700/80 transition-colors"
          @click="promptBatchDelete"
        >
          <i class="fa-solid fa-trash text-[10px]"></i>刪除選取的 {{ selectedIds.size }} 筆
        </button>
      </div>
    </Transition>

    <!-- Table -->
    <div class="flex-1 overflow-auto">
      <table class="w-full text-xs">
        <thead class="bg-gray-800 sticky top-0 z-10">
          <tr>
            <th class="px-3 py-2.5 w-9" @click.stop>
              <input
                type="checkbox"
                class="rounded accent-blue-500 cursor-pointer"
                :checked="allSelected"
                :indeterminate="someSelected"
                @change="toggleAll"
              />
            </th>
            <th class="px-3 py-2.5 text-left text-gray-400 font-medium w-8">#</th>
            <th class="px-3 py-2.5 text-left text-gray-400 font-medium">姓名</th>
            <th class="px-3 py-2.5 text-left text-gray-400 font-medium w-16">性/齡</th>
            <th class="px-3 py-2.5 text-left text-gray-400 font-medium w-24">科別</th>
            <th class="px-3 py-2.5 text-left text-gray-400 font-medium">術式</th>
            <th class="px-3 py-2.5 text-left text-gray-400 font-medium w-12">房間</th>
            <th class="px-3 py-2.5 text-left text-gray-400 font-medium w-16">進線時間</th>
            <th class="px-3 py-2.5 text-left text-gray-400 font-medium w-16">緊急</th>
            <th class="px-3 py-2.5 text-left text-gray-400 font-medium w-20">狀態</th>
            <th class="px-3 py-2.5 text-right text-gray-400 font-medium w-16">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="filtered.length === 0">
            <td colspan="11" class="px-3 py-16 text-center text-gray-600">
              <i class="fa-solid fa-clipboard-list text-3xl mb-3 opacity-30 block"></i>
              {{ search || statusFilter ? '無符合條件的病患' : '目前無病患資料，請先匯入' }}
            </td>
          </tr>
          <tr
            v-for="task in filtered" :key="task.id"
            class="border-t border-gray-800 hover:bg-gray-800/60 transition-colors cursor-pointer"
            :class="selectedIds.has(task.id) ? 'bg-blue-950/30' : ''"
            @click="openDetail(task)"
          >
            <td class="px-3 py-2 w-9" @click.stop>
              <input
                type="checkbox"
                class="rounded accent-blue-500 cursor-pointer"
                :checked="selectedIds.has(task.id)"
                @change="toggleRow(task.id)"
              />
            </td>
            <td class="px-3 py-2 text-gray-600">{{ task.seq_no || '—' }}</td>
            <td class="px-3 py-2">
              <span class="font-semibold text-gray-100">{{ task.patient_name }}</span>
              <span class="ml-1.5 text-gray-600 font-mono text-[10px]">{{ task.chart_no }}</span>
            </td>
            <td class="px-3 py-2 text-gray-400">{{ task.gender }}/{{ task.age }}</td>
            <td class="px-3 py-2 text-gray-300">{{ task.dept }}</td>
            <td class="px-3 py-2 text-gray-400 max-w-[180px] truncate" :title="task.procedure">{{ task.procedure || '—' }}</td>
            <td class="px-3 py-2 text-blue-300 font-mono">{{ task.expected_room || '—' }}</td>
            <td class="px-3 py-2 text-gray-400 font-mono">{{ fmtTime(task.scheduled_at) }}</td>
            <td class="px-3 py-2">
              <span class="text-[10px] font-bold px-1.5 py-0.5 rounded" :class="urgencyBadge(task.urgency)">
                {{ urgencyShort(task.urgency) }}
              </span>
            </td>
            <td class="px-3 py-2">
              <span class="text-[10px] px-1.5 py-0.5 rounded font-medium" :class="statusColor(task.status)">
                {{ statusLabel(task.status) }}
              </span>
            </td>
            <td class="px-3 py-2 text-right" @click.stop>
              <button class="text-gray-500 hover:text-blue-300 mr-2" title="詳細 / 編輯" @click="openDetail(task)">
                <i class="fa-solid fa-pen-to-square"></i>
              </button>
              <button class="text-gray-500 hover:text-red-400" title="刪除" @click="promptDelete(task)">
                <i class="fa-solid fa-trash"></i>
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- PatientDetailModal -->
    <PatientDetailModal
      v-if="detailTask"
      :task="detailTask"
      @close="detailTaskId = null"
      @deleted="detailTaskId = null"
    />

    <!-- Delete / Batch-delete confirm dialog -->
    <Teleport to="body">
      <div v-if="deleteTarget || batchDeletePending" class="fixed inset-0 bg-black/60 z-[9991] flex items-center justify-center">
        <div class="bg-gray-800 rounded-xl border border-red-700/60 p-6 max-w-xs w-full text-center shadow-2xl">
          <i class="fa-solid fa-triangle-exclamation text-2xl text-red-400 mb-3"></i>
          <div class="text-sm font-semibold text-white mb-1">確定刪除？</div>
          <div class="text-xs text-gray-400 mb-4">
            <template v-if="deleteTarget">「{{ deleteTarget.patient_name }}」將永久移除</template>
            <template v-else>已選取的 <span class="text-white font-bold">{{ selectedIds.size }}</span> 筆病患將永久移除</template>
          </div>
          <div class="flex gap-2 justify-center">
            <button class="btn-secondary text-xs" @click="cancelDelete">取消</button>
            <button
              class="text-xs px-4 py-1.5 rounded bg-red-700 hover:bg-red-600 text-white font-bold"
              :disabled="deleting"
              @click="deleteTarget ? doDelete() : doBatchDelete()"
            >
              <i v-if="deleting" class="fa-solid fa-spinner animate-spin mr-1"></i>
              {{ deleteTarget ? '確認刪除' : `刪除 ${selectedIds.size} 筆` }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";
import type { SurgeryTask, UrgencyLevel, TaskStatus } from "../types";
import { TASK_STATUS_LABELS, TASK_STATUS_COLORS } from "../types";
import { useTasksStore } from "../stores/tasks";
import PatientDetailModal from "./PatientDetailModal.vue";

defineEmits<{ back: []; import: []; "add-emergency": [] }>();
const tasksStore = useTasksStore();

const search = ref("");
const statusFilter = ref("");
const detailTaskId = ref<number | null>(null);
const detailTask = computed(() =>
  detailTaskId.value != null ? tasksStore.tasks.find(t => t.id === detailTaskId.value) ?? null : null
);

const deleteTarget = ref<SurgeryTask | null>(null);
const batchDeletePending = ref(false);
const deleting = ref(false);

// ── Selection ─────────────────────────────────────────────────────────────────
const selectedIds = ref(new Set<number>());

const allSelected = computed(() =>
  filtered.value.length > 0 && filtered.value.every(t => selectedIds.value.has(t.id))
);
const someSelected = computed(() =>
  !allSelected.value && filtered.value.some(t => selectedIds.value.has(t.id))
);

function toggleRow(id: number) {
  const s = new Set(selectedIds.value);
  if (s.has(id)) s.delete(id);
  else s.add(id);
  selectedIds.value = s;
}

function toggleAll() {
  if (allSelected.value) {
    clearSelection();
  } else {
    selectedIds.value = new Set(filtered.value.map(t => t.id));
  }
}

function clearSelection() {
  selectedIds.value = new Set();
}

// Clear selection when filters change
watch([search, statusFilter], () => clearSelection());

// ── Status options ────────────────────────────────────────────────────────────
const STATUS_OPTIONS = [
  { value: "waiting",    label: "待排" },
  { value: "scheduled",  label: "已排程" },
  { value: "called",     label: "已叫刀" },
  { value: "in_surgery", label: "手術中" },
  { value: "recovery",   label: "恢復室" },
];

const filtered = computed(() => {
  let list = tasksStore.tasks;
  if (statusFilter.value) list = list.filter(t => t.status === statusFilter.value);
  if (search.value.trim()) {
    const q = search.value.trim().toLowerCase();
    list = list.filter(t =>
      t.patient_name.toLowerCase().includes(q) ||
      t.chart_no.toLowerCase().includes(q)
    );
  }
  return list;
});

// ── Actions ───────────────────────────────────────────────────────────────────
function openDetail(task: SurgeryTask) { detailTaskId.value = task.id; }
function promptDelete(task: SurgeryTask) { deleteTarget.value = task; }
function promptBatchDelete() { batchDeletePending.value = true; }
function cancelDelete() { deleteTarget.value = null; batchDeletePending.value = false; }

async function doDelete() {
  if (!deleteTarget.value) return;
  deleting.value = true;
  try {
    await tasksStore.remove(deleteTarget.value.id);
    selectedIds.value.delete(deleteTarget.value.id);
  } catch (e) {
    console.error("[PatientListPanel] 刪除失敗:", e);
  } finally {
    deleting.value = false;
    deleteTarget.value = null;
  }
}

async function doBatchDelete() {
  deleting.value = true;
  const ids = [...selectedIds.value];
  try {
    for (const id of ids) {
      await tasksStore.remove(id);
    }
    clearSelection();
  } catch (e) {
    console.error("[PatientListPanel] 批量刪除失敗:", e);
  } finally {
    deleting.value = false;
    batchDeletePending.value = false;
  }
}

// ── Formatters ────────────────────────────────────────────────────────────────
function fmtTime(ts: number | null): string {
  if (!ts) return "—";
  const d = new Date(ts * 1000);
  return `${String(d.getHours()).padStart(2,"0")}:${String(d.getMinutes()).padStart(2,"0")}`;
}

function statusLabel(s: string) { return TASK_STATUS_LABELS[s as TaskStatus] ?? s; }
function statusColor(s: string) { return TASK_STATUS_COLORS[s as TaskStatus] ?? "bg-gray-700 text-gray-300"; }

function urgencyShort(u: UrgencyLevel) {
  return { Trauma: "T", Level1: "L1", Level2: "L2", Level3: "L3", Normal: "N" }[u] ?? "?";
}
function urgencyBadge(u: UrgencyLevel) {
  return { Trauma: "bg-red-600 text-white", Level1: "bg-orange-600 text-white",
    Level2: "bg-yellow-600 text-white", Level3: "bg-blue-600 text-white", Normal: "bg-gray-600 text-gray-300" }[u] ?? "bg-gray-700";
}
</script>

<style scoped>
.slide-down-enter-active, .slide-down-leave-active { transition: max-height 0.15s ease, opacity 0.15s ease; overflow: hidden; }
.slide-down-enter-from, .slide-down-leave-to { max-height: 0; opacity: 0; }
.slide-down-enter-to, .slide-down-leave-from { max-height: 48px; opacity: 1; }
</style>
