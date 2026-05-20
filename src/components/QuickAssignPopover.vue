<template>
  <Teleport to="body">
    <!-- Click-away backdrop -->
    <div class="fixed inset-0 z-[9990]" @mousedown.self="$emit('close')" />

    <div
      class="fixed z-[9991] bg-gray-800 border border-gray-700 rounded-xl shadow-2xl w-80"
      :style="popoverStyle"
    >
      <!-- Header -->
      <div class="flex items-center gap-2 px-3 py-2 border-b border-gray-700">
        <i class="fa-solid fa-wand-magic-sparkles text-violet-400 text-xs shrink-0"></i>
        <span class="text-xs font-bold text-white flex-1">快速派房</span>
        <button class="text-gray-500 hover:text-gray-300 text-xs" @click="$emit('close')">
          <i class="fa-solid fa-xmark"></i>
        </button>
      </div>

      <!-- Patient summary (read-only) -->
      <div class="flex items-center gap-1.5 flex-wrap px-3 py-2 border-b border-gray-700/60">
        <span class="text-[10px] font-bold px-1.5 py-0.5 rounded" :class="urgencyClass">{{ task.urgency }}</span>
        <span class="text-[10px] text-gray-300">{{ task.dept || '—' }}</span>
        <span v-if="task.anesthesia" class="text-[10px] text-gray-500">· {{ task.anesthesia }}</span>
        <span class="text-[10px] text-gray-500">· ~{{ task.est_time_mins }}min</span>
        <span v-if="task.patient_name" class="text-[10px] text-gray-600 ml-auto truncate max-w-[90px]">{{ task.patient_name }}</span>
      </div>

      <!-- Recommendations -->
      <div class="px-2 py-2 max-h-64 overflow-y-auto space-y-1">
        <div v-if="loading" class="text-xs text-gray-500 text-center py-6">
          <i class="fa-solid fa-spinner animate-spin mr-1"></i>計算推薦中…
        </div>
        <div v-else-if="recs.length === 0" class="text-xs text-gray-600 text-center py-6">
          無可用手術室，請先設定房間
        </div>
        <div
          v-for="(rec, i) in recs" :key="rec.room_name"
          class="flex items-center gap-2 rounded-lg border px-2.5 py-2 transition-colors"
          :class="i === 0 ? 'border-green-700/50 bg-green-900/10' : 'border-gray-700/50 bg-gray-900/20'"
        >
          <!-- Rank -->
          <span class="shrink-0 text-sm w-4 text-center select-none">
            {{ i === 0 ? '🏆' : i === 1 ? '🥈' : i === 2 ? '🥉' : '·' }}
          </span>

          <!-- Info -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-1 flex-wrap">
              <span class="text-xs font-bold" :class="i === 0 ? 'text-gray-100' : 'text-gray-300'">{{ rec.room_name }}</span>
              <span v-if="rec.is_emergency_room"
                class="text-[8px] font-bold bg-red-700/60 text-red-200 border border-red-600/50 px-1 rounded">急診房</span>
              <span v-if="rec.dept_match"
                class="text-[8px] bg-teal-900/60 text-teal-300 border border-teal-700/50 px-1 rounded">科別符合</span>
              <span v-if="rec.anes_conflict"
                class="text-[8px] bg-orange-900/60 text-orange-300 border border-orange-700/50 px-1 rounded">
                <i class="fa-solid fa-triangle-exclamation text-[7px] mr-0.5"></i>麻醉不符</span>
              <span v-if="rec.has_staff"
                class="text-[8px] bg-blue-900/60 text-blue-300 border border-blue-700/50 px-1 rounded">人員就位</span>
              <span v-else
                class="text-[8px] bg-gray-800 text-gray-600 border border-gray-700 px-1 rounded">未排人員</span>
            </div>
            <div class="text-[9px] mt-0.5 flex items-center gap-1">
              <span :class="rec.is_available ? 'text-green-400' : rec.within_deadline ? 'text-yellow-400' : 'text-red-400'">
                {{ rec.is_available ? '立即可用' : `+${rec.est_available_mins}m 後可用` }}
              </span>
              <span v-if="!rec.within_deadline" class="text-red-400">⚠ 超截止</span>
            </div>
          </div>

          <!-- Assign button -->
          <button
            class="shrink-0 text-[10px] font-bold px-2.5 py-1 rounded transition-colors"
            :class="rec.anes_conflict
              ? 'bg-orange-900/60 text-orange-300 hover:bg-orange-800/60 border border-orange-700/50'
              : 'bg-violet-700 hover:bg-violet-600 text-white'"
            :title="rec.anes_conflict ? '注意：麻醉方式不符，確認仍要派入？' : `派入 ${rec.room_name}`"
            @click="assign(rec.room_name)"
          >派入</button>
        </div>
      </div>

      <!-- Time picker -->
      <div class="flex items-center gap-2 px-3 py-2 border-t border-gray-700/60">
        <span class="text-[10px] text-gray-500 shrink-0">進刀時間</span>
        <input
          v-model="timeInput"
          type="time"
          class="flex-1 bg-gray-900 border border-gray-600 rounded px-2 py-0.5 text-xs text-blue-300 outline-none focus:border-blue-500 [color-scheme:dark]"
        />
        <span class="text-[9px] text-gray-600 shrink-0">選填</span>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRoomRecommendationDb } from "../composables/useDatabase";
import { useTasksStore } from "../stores/tasks";
import type { RoomRecommendation, SurgeryTask } from "../types";

const props = defineProps<{ task: SurgeryTask; x: number; y: number }>();
const emit  = defineEmits<{ close: [] }>();

const tasksStore = useTasksStore();
const recDb      = useRoomRecommendationDb();

const loading = ref(true);
const recs    = ref<RoomRecommendation[]>([]);

function getTodayStr(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

function nowTimeStr(): string {
  const d = new Date();
  return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
}

const timeInput = ref(nowTimeStr());

// Clamp to viewport
const popoverStyle = computed(() => {
  const W = 320, H = 420;
  const left = Math.min(props.x + 4, window.innerWidth  - W - 8);
  const top  = Math.min(props.y,     window.innerHeight - H - 8);
  return { left: `${Math.max(8, left)}px`, top: `${Math.max(8, top)}px` };
});

const URGENCY_COLORS: Record<string, string> = {
  Trauma: "bg-red-700 text-white",
  Level1: "bg-orange-700 text-white",
  Level2: "bg-yellow-700 text-white",
  Level3: "bg-blue-700 text-white",
  Normal: "bg-gray-600 text-white",
};
const urgencyClass = computed(() => URGENCY_COLORS[props.task.urgency] ?? "bg-gray-700 text-gray-300");

async function assign(roomName: string) {
  let scheduled_at: number | null = null;
  if (timeInput.value) {
    const [h, m] = timeInput.value.split(":").map(Number);
    const d = new Date();
    d.setHours(h, m, 0, 0);
    scheduled_at = Math.floor(d.getTime() / 1000);
  }
  try {
    await tasksStore.edit({
      ...props.task,
      expected_room: roomName,
      status: "scheduled",
      scheduled_at,
    });
  } catch (e) {
    console.error("[QuickAssignPopover] 派房失敗:", e);
  }
  emit("close");
}

onMounted(async () => {
  try {
    recs.value = await recDb.get(
      props.task.urgency,
      props.task.dept,
      props.task.anesthesia,
      props.task.est_time_mins,
      getTodayStr()
    );
  } catch (e) {
    console.error("[QuickAssignPopover] 推薦計算失敗:", e);
  } finally {
    loading.value = false;
  }
});
</script>
