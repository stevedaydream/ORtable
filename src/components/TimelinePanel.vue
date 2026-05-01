<template>
  <div class="flex flex-col h-full">

    <!-- Header toolbar -->
    <div class="flex items-center gap-3 px-3 py-2 border-b border-gray-700 shrink-0">
      <span class="text-xs font-semibold text-gray-400">房間總覽</span>
      <span class="text-xs text-gray-600">{{ todayLabel }}</span>
      <div class="flex-1" />
      <button class="text-xs text-gray-500 hover:text-gray-300 flex items-center gap-1" @click="reload">
        <i class="fa-solid fa-rotate text-xs"></i>重整
      </button>
    </div>

    <!-- Empty state -->
    <div v-if="rooms.length === 0" class="flex-1 flex flex-col items-center justify-center text-center px-6 text-gray-600">
      <i class="fa-solid fa-door-open text-4xl mb-3 opacity-30"></i>
      <div class="text-sm font-medium">尚未設定手術室</div>
      <div class="text-xs mt-1">請至「設定 → 房間管理」新增房間，或匯入月排班 Excel</div>
    </div>

    <!-- Timeline grid -->
    <div v-else class="flex flex-1 overflow-hidden">

      <!-- Time axis (sticky left) -->
      <div class="w-14 shrink-0 border-r border-gray-700 overflow-hidden relative select-none" :style="{ height: TOTAL_PX + 'px' }">
        <div
          v-for="h in HOUR_LABELS" :key="h.label"
          class="absolute right-1 text-[10px] text-gray-600 leading-none"
          :style="{ top: h.y - 7 + 'px' }"
        >{{ h.label }}</div>
        <!-- now indicator line marker -->
        <div
          v-if="nowY !== null"
          class="absolute left-0 right-0 border-t border-red-500/60"
          :style="{ top: nowY + 'px' }"
        />
      </div>

      <!-- Rooms columns (scrollable) -->
      <div class="flex-1 overflow-auto" ref="scrollEl">
        <div class="flex min-w-max">

          <!-- Per-room column -->
          <div
            v-for="room in rooms" :key="room.id"
            class="w-36 shrink-0 border-r border-gray-700/50 flex flex-col"
          >
            <!-- Room header -->
            <div
              class="px-2 py-1.5 border-b text-center text-xs font-semibold sticky top-0 z-10 shrink-0"
              :class="room.is_backup
                ? 'bg-orange-900/40 border-orange-700/50 text-orange-300'
                : 'bg-gray-800 border-gray-700 text-gray-200'"
            >
              <i v-if="room.is_backup" class="fa-solid fa-circle-exclamation mr-1 text-orange-400"></i>
              {{ room.name }}
              <span v-if="room.is_backup" class="ml-1 text-[9px] text-orange-500">備用</span>
            </div>

            <!-- Timeline column body -->
            <div class="relative" :style="{ height: TOTAL_PX + 'px' }">

              <!-- Hour grid lines -->
              <div
                v-for="h in HOUR_LABELS" :key="h.label"
                class="absolute left-0 right-0 border-t border-gray-700/30"
                :style="{ top: h.y + 'px' }"
              />

              <!-- Now indicator -->
              <div
                v-if="nowY !== null"
                class="absolute left-0 right-0 border-t border-red-500/50 z-10"
                :style="{ top: nowY + 'px' }"
              />

              <!-- Dept blocks -->
              <div
                v-for="shift in shiftsForRoom(room.name)" :key="shift.id"
                class="absolute left-1 right-1 rounded px-1.5 py-0.5 text-[10px] font-medium overflow-hidden cursor-default"
                :class="deptColor(shift.dept)"
                :style="blockStyle(shift)"
                :title="`${shift.dept}  ${fmtTs(shift.start_time)} – ${fmtTs(shift.end_time)}`"
              >
                <div class="truncate leading-tight">{{ shift.dept }}</div>
                <div class="text-[9px] opacity-70 leading-tight">{{ fmtTs(shift.start_time) }}–{{ fmtTs(shift.end_time) }}</div>
              </div>

            </div>
          </div>

        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import type { Room, RoomScheduleEntry } from "../types";
import { useRoomsStore } from "../stores/rooms";
import { useRoomShiftsDb } from "../composables/useDatabase";

// ── Constants ─────────────────────────────────────────────────────────────────
const PX_PER_HOUR = 60;
const START_HOUR = 7.5;   // 07:30
const TOTAL_HOURS = 24;
const TOTAL_PX = TOTAL_HOURS * PX_PER_HOUR;  // 1440px

const DEPT_COLORS = [
  "bg-blue-800/70 text-blue-200",
  "bg-green-800/70 text-green-200",
  "bg-purple-800/70 text-purple-200",
  "bg-yellow-800/70 text-yellow-200",
  "bg-pink-800/70 text-pink-200",
  "bg-teal-800/70 text-teal-200",
  "bg-orange-800/70 text-orange-200",
  "bg-cyan-800/70 text-cyan-200",
];
const deptColorMap = new Map<string, string>();
let colorIdx = 0;

function deptColor(dept: string): string {
  if (!deptColorMap.has(dept)) {
    deptColorMap.set(dept, DEPT_COLORS[colorIdx % DEPT_COLORS.length]);
    colorIdx++;
  }
  return deptColorMap.get(dept)!;
}

// ── Hour labels ───────────────────────────────────────────────────────────────
const HOUR_LABELS = Array.from({ length: TOTAL_HOURS + 1 }, (_, i) => {
  const hour = (Math.floor(START_HOUR) + i) % 24;
  const isHalf = i === 0 && START_HOUR % 1 !== 0;
  const label = isHalf ? `${hour}:30` : `${String(hour).padStart(2, "0")}:00`;
  return { label, y: i * PX_PER_HOUR };
}).filter((_, i) => i % 1 === 0);  // every hour

// ── Data ──────────────────────────────────────────────────────────────────────
const roomsStore = useRoomsStore();
const roomShiftsDb = useRoomShiftsDb();

const rooms = computed<Room[]>(() => roomsStore.rooms);
const shifts = ref<RoomScheduleEntry[]>([]);

const todayStr = computed(() => {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
});

const todayLabel = computed(() => {
  const d = new Date();
  return `${d.getFullYear()}/${d.getMonth() + 1}/${d.getDate()} (${["日","一","二","三","四","五","六"][d.getDay()]})`;
});

// unix timestamp of today's 07:30
const dayStart = computed(() => {
  const d = new Date();
  const midnight = new Date(d.getFullYear(), d.getMonth(), d.getDate());
  return Math.floor(midnight.getTime() / 1000) + Math.floor(START_HOUR * 3600);
});

function shiftsForRoom(roomName: string): RoomScheduleEntry[] {
  return shifts.value.filter((s) => s.room_name === roomName);
}

function timeToY(unixTs: number): number {
  const offsetSecs = unixTs - dayStart.value;
  // handle times after midnight (0-7:30 range → add 24h)
  const adjusted = offsetSecs < 0 ? offsetSecs + 86400 : offsetSecs;
  return (adjusted / 3600) * PX_PER_HOUR;
}

function blockStyle(shift: RoomScheduleEntry) {
  const top = timeToY(shift.start_time);
  const height = Math.max(((shift.end_time - shift.start_time) / 3600) * PX_PER_HOUR, 14);
  return { top: `${top}px`, height: `${height}px` };
}

function fmtTs(ts: number): string {
  const d = new Date(ts * 1000);
  return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
}

// ── Now indicator ─────────────────────────────────────────────────────────────
const nowY = ref<number | null>(null);
let nowTimer: ReturnType<typeof setInterval> | null = null;

function updateNow() {
  const now = Math.floor(Date.now() / 1000);
  const offset = now - dayStart.value;
  if (offset >= 0 && offset < TOTAL_HOURS * 3600) {
    nowY.value = (offset / 3600) * PX_PER_HOUR;
  } else {
    nowY.value = null;
  }
}

// ── Scroll to current time on mount ──────────────────────────────────────────
const scrollEl = ref<HTMLElement | null>(null);

function scrollToNow() {
  if (nowY.value !== null && scrollEl.value) {
    const target = Math.max(0, nowY.value - 100);
    scrollEl.value.scrollTop = target;
  }
}

async function reload() {
  shifts.value = await roomShiftsDb.getByDate(todayStr.value);
  updateNow();
}

onMounted(async () => {
  await roomsStore.load();
  shifts.value = await roomShiftsDb.getByDate(todayStr.value);
  updateNow();
  scrollToNow();
  nowTimer = setInterval(updateNow, 60000);
});

onUnmounted(() => {
  if (nowTimer) clearInterval(nowTimer);
});
</script>
