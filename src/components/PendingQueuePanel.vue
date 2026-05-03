<template>
  <div class="flex flex-col h-full overflow-hidden">
    <!-- Header -->
    <div class="flex items-center gap-2 px-3 py-2 border-b border-gray-700 shrink-0">
      <span class="text-xs font-semibold text-gray-400">待排定病患</span>
      <span v-if="selectedDate && selectedDate !== todayStr" class="text-[10px] text-amber-400 ml-1">{{ selectedDate }}</span>
      <div class="flex-1" />
      <span class="text-[10px] text-gray-600">{{ waitingTasks.length }} 件</span>
    </div>

    <!-- Empty state -->
    <div v-if="waitingTasks.length === 0"
      class="flex-1 flex flex-col items-center justify-center text-gray-700 text-xs gap-2">
      <i class="fa-solid fa-clipboard-list text-2xl opacity-30"></i>
      <span>目前無待排病患</span>
    </div>

    <!-- Task list -->
    <div v-else class="flex-1 overflow-y-auto px-2 py-2 space-y-1.5">
      <PatientCard
        v-for="ts in waitingTasks" :key="ts.task.id"
        :task="ts.task"
        :scored-task="ts"
        mode="queue"
        @contextmenu="onRightClick($event, ts.task)"
        @click="detailTaskId = ts.task.id"
      />
    </div>

    <!-- Patient Detail Modal -->
    <PatientDetailModal
      v-if="detailTask"
      :task="detailTask"
      @close="detailTaskId = null"
      @deleted="detailTaskId = null"
    />

    <!-- Self-Pay Picker Modal -->
    <SelfPayPickerModal
      v-if="selfPayTask"
      :task="selfPayTask"
      @close="selfPayTaskId = null"
    />

    <!-- Right-click menu: room selection -->
    <ContextMenu
      :show="menu.open"
      :x="menu.rawX"
      :y="menu.rawY"
      @close="menu.open = false"
    >
      <div class="px-3 py-1.5 text-[10px] text-gray-500 font-semibold border-b border-gray-700">
        排入 <span class="text-gray-200">{{ menu.patientName }}</span> 至…
      </div>
      <div v-if="roomsStore.rooms.length === 0" class="px-3 py-2 text-xs text-gray-600">尚未設定房間</div>
      <button
        v-for="r in roomsStore.rooms" :key="r.id"
        class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-gray-300 hover:bg-gray-700 transition-colors text-left"
        @click="assignToRoom(r.name)"
      >
        <i class="fa-solid fa-door-open text-gray-500 text-[10px]"></i>
        {{ r.name }}
        <span v-if="r.is_backup" class="text-[9px] text-orange-400 ml-auto">備用</span>
      </button>
      <div class="border-t border-gray-700/60 my-0.5" />
      <button
        class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-yellow-300 hover:bg-yellow-900/30 transition-colors text-left"
        @click="openSelfPay"
      >
        <i class="fa-solid fa-receipt text-[10px]"></i>添加自費備注
      </button>
    </ContextMenu>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, computed } from "vue";
import { useTasksStore } from "../stores/tasks";
import { useRoomsStore } from "../stores/rooms";
import type { SurgeryTask } from "../types";
import PatientCard from "./PatientCard.vue";
import ContextMenu from "./ContextMenu.vue";
import PatientDetailModal from "./PatientDetailModal.vue";
import SelfPayPickerModal from "./SelfPayPickerModal.vue";

const props = defineProps<{ selectedDate?: string }>();

const tasksStore  = useTasksStore();
const roomsStore  = useRoomsStore();

function getTodayStr(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}
const todayStr = getTodayStr();

function dateOfTs(ts: number): string {
  const d = new Date(ts * 1000);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

const waitingTasks = computed(() => {
  const date = props.selectedDate || todayStr;
  const isToday = date === todayStr;
  return tasksStore.scoredTasks.filter(ts => {
    if (ts.task.status !== "waiting") return false;
    if (!ts.task.scheduled_at) return isToday;
    return dateOfTs(ts.task.scheduled_at) === date;
  });
});

// ── Detail Modal (reactive via store lookup) ──────────────────────────────────
const detailTaskId = ref<number | null>(null);
const detailTask = computed(() =>
  detailTaskId.value != null ? tasksStore.tasks.find(t => t.id === detailTaskId.value) ?? null : null
);

// ── Self-Pay Picker ───────────────────────────────────────────────────────────
const selfPayTaskId = ref<number | null>(null);
const selfPayTask = computed(() =>
  selfPayTaskId.value != null ? tasksStore.tasks.find(t => t.id === selfPayTaskId.value) ?? null : null
);

// ── Right-click menu ──────────────────────────────────────────────────────────
const menu = reactive({ open: false, taskId: 0, patientName: "", rawX: 0, rawY: 0 });

function onRightClick(e: MouseEvent, task: SurgeryTask) {
  menu.open        = true;
  menu.taskId      = task.id;
  menu.patientName = task.patient_name;
  menu.rawX        = e.clientX;
  menu.rawY        = e.clientY;
}

function openSelfPay() {
  selfPayTaskId.value = menu.taskId;
  menu.open = false;
}

async function assignToRoom(roomName: string) {
  const ts = tasksStore.tasks.find(t => t.id === menu.taskId);
  if (ts) {
    try {
      await tasksStore.edit({ ...ts, expected_room: roomName, status: "scheduled" });
    } catch (e) {
      console.error("[Queue] 指派房間失敗:", e);
    }
  }
  menu.open = false;
}
</script>
