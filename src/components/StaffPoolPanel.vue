<template>
  <div class="flex flex-col h-full overflow-hidden">
    <!-- Header -->
    <div class="flex items-center gap-2 px-3 py-2 border-b border-gray-700 shrink-0">
      <span class="text-xs font-semibold text-gray-400">可分配人員</span>
      <div class="flex-1" />
      <span class="text-[10px] text-gray-600">{{ activeCount }} 人</span>
    </div>

    <!-- Empty state -->
    <div v-if="staffStore.staffList.length === 0" class="flex-1 flex flex-col items-center justify-center gap-2 text-gray-500 text-xs">
      <i class="fa-solid fa-users text-gray-700 text-xl"></i>
      <span>尚未設定人員</span>
      <span class="text-gray-600 text-[10px]">請至設定 → 人員管理 新增</span>
    </div>

    <!-- Staff by category -->
    <div
      v-else
      class="flex-1 overflow-y-auto px-3 py-2 space-y-2.5 transition-colors"
      :class="isDragging && dragType === 'assignment' ? 'bg-red-500/10 ring-2 ring-inset ring-red-500/30' : ''"
      @dragover.prevent
      @drop="onDropAssignment"
    >
      <div v-for="cat in CATEGORIES" :key="cat" v-show="staffByCategory[cat]!.length > 0">
        <!-- Category toggle header -->
        <button class="flex items-center gap-1.5 w-full text-left mb-1" @click="activeCategories[cat] = !activeCategories[cat]">
          <i class="fa-solid text-[10px] transition-colors"
            :class="activeCategories[cat] ? ['fa-toggle-on', categoryColor(cat)] : 'fa-toggle-off text-gray-600'"
          ></i>
          <span class="text-[10px] font-semibold uppercase tracking-wider transition-colors"
            :class="activeCategories[cat] ? categoryColor(cat) : 'text-gray-600'"
          >{{ CATEGORY_LABELS[cat] }}</span>
          <div class="flex-1 border-t border-gray-700/40 ml-1" />
          <span class="text-[10px] text-gray-700 ml-1">{{ staffByCategory[cat]!.length }}</span>
        </button>

        <!-- Staff badges -->
        <div v-if="activeCategories[cat]" class="flex flex-wrap gap-1.5 pl-1">
          <span
            v-for="s in staffByCategory[cat]" :key="s.id"
            class="flex items-center gap-0.5 px-2 py-0.5 rounded-full text-[10px] leading-tight cursor-grab active:cursor-grabbing select-none"
            :class="categoryBadge(cat)"
            :title="`拖拉或右鍵 → 分配至房間`"
            draggable="true"
            @dragstart="onDragStart($event, s)"
            @dragend="onDragEnd"
            @contextmenu.prevent="onRightClick($event, s)"
          >
            {{ s.name }}
            <span v-if="s.is_on_call" class="text-yellow-400 ml-0.5" title="值班">★</span>
          </span>
        </div>
      </div>
      
      <!-- Drag-back hint -->
      <div v-if="isDragging && dragType === 'assignment'" class="absolute inset-0 flex items-center justify-center pointer-events-none">
        <div class="bg-red-900/90 border border-red-500 px-4 py-2 rounded-lg text-xs text-red-200 shadow-2xl">
          <i class="fa-solid fa-user-minus mr-2"></i>放開以取消指派
        </div>
      </div>
    </div>

    <!-- Right-click floating menu -->
    <ContextMenu
      :show="menu.open"
      :x="menu.rawX"
      :y="menu.rawY"
      :width="menu.step === 'room' ? 200 : 220"
      @close="closeMenu"
    >
      <!-- Step 1: choose room -->
      <template v-if="menu.step === 'room'">
        <div class="px-3 py-1.5 text-[10px] text-gray-500 font-semibold border-b border-gray-700">
          指派 <span class="text-gray-200">{{ menu.staffName }}</span> 至…
        </div>
        <div v-if="roomsStore.rooms.length === 0" class="px-3 py-2 text-xs text-gray-600">尚未設定房間</div>
        <button
          v-for="r in roomsStore.rooms" :key="r.id"
          class="flex items-center gap-2 w-full px-3 py-1.5 text-xs text-gray-300 hover:bg-gray-700 transition-colors text-left"
          @click="selectRoom(r.name)"
        >
          <i class="fa-solid fa-door-open text-gray-500 text-[10px]"></i>
          {{ r.name }}
          <span v-if="r.is_backup" class="text-[9px] text-orange-400 ml-auto">備用</span>
        </button>
      </template>

      <!-- Step 2: choose role -->
      <template v-else>
        <div class="px-3 py-1.5 text-[10px] text-gray-500 font-semibold border-b border-gray-700">
          <button class="mr-1 text-gray-600 hover:text-gray-300" @click="menu.step = 'room'">
            <i class="fa-solid fa-chevron-left text-[9px]"></i>
          </button>
          <span class="text-gray-200">{{ menu.staffName }}</span> → {{ menu.roomName }}
        </div>
        <div class="grid grid-cols-3 gap-1 p-2">
          <button
            v-for="r in ROLES" :key="r.value"
            class="py-1 rounded text-[10px] font-bold border-2 transition-all"
            :class="selectedRole === r.value ? r.activeClass : r.inactiveClass"
            @click="selectedRole = r.value"
          >{{ r.label }}</button>
        </div>
        <div class="flex gap-2 px-2 pb-2">
          <button class="flex-1 py-1 text-[10px] rounded bg-gray-700 hover:bg-gray-600 text-gray-300" @click="closeMenu">取消</button>
          <button class="flex-1 py-1 text-[10px] rounded bg-blue-700 hover:bg-blue-600 text-white font-bold" @click="confirmAssignment">確認</button>
        </div>
      </template>
    </ContextMenu>
  </div>
</template>

<script setup lang="ts">
import { reactive, computed, ref, watch } from "vue";
import { useStaffStore } from "../stores/staff";
import { useRoomsStore } from "../stores/rooms";
import { useAssignmentsStore } from "../stores/assignments";
import { useDragState } from "../composables/useDragState";
import { useStaffRosterDb } from "../composables/useDatabase";
import type { Staff, StaffAssignment, StaffCategory } from "../types";
import { STAFF_CATEGORY_LABELS } from "../types";
import ContextMenu from "./ContextMenu.vue";

const props = defineProps<{ date?: string }>();
const emit  = defineEmits(["assigned"]);

const staffStore       = useStaffStore();
const roomsStore       = useRoomsStore();
const assignmentsStore = useAssignmentsStore();
const staffRosterDb    = useStaffRosterDb();
const { isDragging, dragType } = useDragState();

// 當日有班表的人名 Set（空 = 尚無班表資料，顯示全員）
const rosterNames = ref(new Set<string>());

watch(() => props.date, async (date) => {
  if (!date) { rosterNames.value = new Set(); return; }
  const entries = await staffRosterDb.getByDate(date).catch(() => []);
  rosterNames.value = new Set(entries.map(e => e.staff_name));
}, { immediate: true });

const CATEGORY_LABELS = STAFF_CATEGORY_LABELS;
const CATEGORIES: StaffCategory[] = ["vs", "r", "sa", "or_nurse", "intern", "cross_train"];

const ROLES = [
  { value: "Scrub", label: "刷手",   activeClass: "bg-blue-700 border-blue-500 text-white",   inactiveClass: "border-blue-800 text-blue-400 hover:bg-blue-900/30" },
  { value: "Circ",  label: "流動",   activeClass: "bg-green-700 border-green-500 text-white",  inactiveClass: "border-green-800 text-green-400 hover:bg-green-900/30" },
  { value: "SA",    label: "助手",   activeClass: "bg-purple-700 border-purple-500 text-white", inactiveClass: "border-purple-800 text-purple-400 hover:bg-purple-900/30" },
  { value: "R",     label: "住院師", activeClass: "bg-gray-600 border-gray-400 text-white",    inactiveClass: "border-gray-600 text-gray-400 hover:bg-gray-700/40" },
  { value: "VS",    label: "主治師", activeClass: "bg-red-700 border-red-500 text-white",      inactiveClass: "border-red-800 text-red-400 hover:bg-red-900/30" },
] as const;

const activeCategories = reactive<Record<StaffCategory, boolean>>({
  vs: true, r: true, sa: true, or_nurse: true, intern: false, cross_train: false,
});

const staffByCategory = computed(() => {
  const hasRoster = rosterNames.value.size > 0;
  const map = {} as Record<StaffCategory, Staff[]>;
  for (const cat of CATEGORIES) {
    let list = staffStore.staffList.filter(s => s.staff_category === cat);
    if (hasRoster) list = list.filter(s => rosterNames.value.has(s.name));
    map[cat] = list;
  }
  return map;
});

const activeCount = computed(() =>
  CATEGORIES.filter((c) => activeCategories[c]).reduce((n, c) => n + staffByCategory.value[c].length, 0)
);

// ── Drag-and-drop ─────────────────────────────────────────────────────────────
function onDragStart(e: DragEvent, s: Staff) {
  isDragging.value = true;
  dragType.value   = "staff";
  e.dataTransfer!.effectAllowed = "copy";
  e.dataTransfer!.setData("text/plain", JSON.stringify({ type: "staff", name: s.name }));
}
function onDragEnd() {
  isDragging.value = false;
  dragType.value   = null;
}

async function onDropAssignment(e: DragEvent) {
  const raw = e.dataTransfer?.getData("text/plain");
  if (!raw) return;
  try {
    const data = JSON.parse(raw);
    if (data.type === "assignment" && data.id != null) {
      await assignmentsStore.remove(data.id);
    }
  } catch (err) {
    console.error("[StaffPool] 取消指派失敗:", err);
  }
}

// ── Right-click menu ──────────────────────────────────────────────────────────
const menu = reactive({
  open: false,
  staffName: "",
  step: "room" as "room" | "role",
  roomName: "",
  rawX: 0,
  rawY: 0,
});
const selectedRole = ref("Circ");

function onRightClick(e: MouseEvent, s: Staff) {
  menu.open     = true;
  menu.staffName = s.name;
  menu.step      = "room";
  menu.roomName  = "";
  menu.rawX      = e.clientX;
  menu.rawY      = e.clientY;
  selectedRole.value = "Circ";
}

function selectRoom(roomName: string) {
  menu.roomName = roomName;
  menu.step     = "role";
}

function closeMenu() {
  menu.open = false;
}

function todayStr(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,"0")}-${String(d.getDate()).padStart(2,"0")}`;
}

async function confirmAssignment() {
  const entry: StaffAssignment = {
    id: 0,
    staff_name: menu.staffName,
    room_name:  menu.roomName,
    date:       todayStr(),
    role:       selectedRole.value,
  };
  try {
    await assignmentsStore.add(entry);
  } catch (e) {
    console.error("[StaffPool] 指派失敗:", e);
  }
  closeMenu();
}

// ── Styles ────────────────────────────────────────────────────────────────────
function categoryColor(cat: StaffCategory): string {
  return ({
    vs: "text-red-400", r: "text-blue-400", sa: "text-purple-400",
    or_nurse: "text-green-400", intern: "text-yellow-400", cross_train: "text-orange-400",
  } as Record<StaffCategory, string>)[cat] ?? "text-gray-400";
}

function categoryBadge(cat: StaffCategory): string {
  return ({
    vs: "bg-red-900/50 text-red-200", r: "bg-blue-900/50 text-blue-200",
    sa: "bg-purple-900/50 text-purple-200", or_nurse: "bg-green-900/50 text-green-200",
    intern: "bg-yellow-900/50 text-yellow-200", cross_train: "bg-orange-900/50 text-orange-200",
  } as Record<StaffCategory, string>)[cat] ?? "bg-gray-700 text-gray-300";
}
</script>
