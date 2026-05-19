<template>
  <div class="flex h-screen bg-gray-900 text-gray-100 overflow-hidden">
    <Sidebar
      :backup-active="backupActive"
      @import="modal = 'import'"
      @add-emergency="modal = 'emergency'"
      @toggle-backup="backupActive = !backupActive"
      @request-extra="modal = 'extra'"
      @open-settings="modal = 'settings'"
      @open-dept-room="modal = 'dept_room'"
      @open-patient-list="view = 'patient_list'"
      @quick-assign="modal = 'quick_assign'"
    />

    <div class="flex flex-col flex-1 overflow-hidden">
      <Transition name="banner">
        <div v-if="backupActive" class="shrink-0 bg-red-900/80 border-b border-red-700 px-4 py-2 flex items-center gap-3 text-sm">
          <i class="fa-solid fa-bolt text-red-400 animate-pulse"></i>
          <span class="font-semibold text-red-200">二線機制已啟動</span>
          <span class="text-red-400 text-xs">備用刀房與二線人員已解鎖，所有 Trauma/Level1 賦予絕對優先</span>
          <button class="ml-auto text-xs bg-red-800 hover:bg-red-700 px-3 py-1 rounded" @click="backupActive = false">解除</button>
        </div>
      </Transition>

      <!-- Board view -->
      <div v-if="view === 'board'" class="flex flex-1 overflow-hidden">
        <section class="flex-[2] overflow-hidden border-r border-gray-700 flex flex-col">
          <TimelinePanel ref="timelineRef" @date-changed="selectedDate = $event" />
        </section>
        <section class="flex-[1] flex flex-col overflow-hidden">
          <div class="flex-[2] overflow-hidden border-b border-gray-700">
            <StaffPoolPanel :date="selectedDate" @assigned="onStaffAssigned" />
          </div>
          <div class="flex-[3] overflow-hidden">
            <PendingQueuePanel :selected-date="selectedDate" />
          </div>
        </section>
      </div>

      <!-- Patient list view -->
      <PatientListPanel
        v-else-if="view === 'patient_list'"
        class="flex-1 overflow-hidden"
        @back="view = 'board'"
        @import="modal = 'import'; view = 'board'"
        @add-emergency="modal = 'emergency'; view = 'board'"
      />
    </div>

    <!-- P4-1: Quick-start guide -->
    <Transition name="banner">
      <div v-if="showGuide" class="fixed top-4 right-4 z-40 bg-gray-800 border border-blue-700/50 rounded-xl shadow-xl p-4 max-w-xs">
        <div class="flex items-center gap-2 mb-2">
          <i class="fa-solid fa-circle-info text-blue-400"></i>
          <span class="text-sm font-semibold text-gray-200">今日無任何排程</span>
          <button class="ml-auto text-gray-500 hover:text-gray-300 text-xs" @click="showGuide = false">✕</button>
        </div>
        <div class="text-xs text-gray-400 space-y-1.5">
          <div class="flex items-center gap-2 cursor-pointer hover:text-gray-200" @click="modal = 'import'; showGuide = false">
            <i class="fa-solid fa-file-import w-4 text-center text-green-400"></i>匯入 PDF 排程表
          </div>
          <div class="flex items-center gap-2 cursor-pointer hover:text-gray-200" @click="modal = 'dept_room'; showGuide = false">
            <i class="fa-solid fa-calendar-days w-4 text-center text-teal-400"></i>設定科別房間分配
          </div>
          <div class="flex items-center gap-2 cursor-pointer hover:text-gray-200" @click="modal = 'settings'; showGuide = false">
            <i class="fa-solid fa-gear w-4 text-center text-gray-400"></i>確認人員出勤設定
          </div>
        </div>
      </div>
    </Transition>

    <UpdateModal v-if="updater.status.value === 'available'" />

    <TaskFormModal   v-if="modal === 'emergency'"    :prefill="emergencyPrefill" @close="modal = null; emergencyPrefill = undefined" />
    <ImportModal     v-if="modal === 'import'"       @close="modal = null" />
    <ExtraLineModal  v-if="modal === 'extra'"        @close="modal = null" />
    <SettingsModal   v-if="modal === 'settings'"     @close="modal = null" />
    <DeptRoomModal   v-if="modal === 'dept_room'"    @close="modal = null" />
    <QuickAssignModal v-if="modal === 'quick_assign'" @close="modal = null" @confirmed="onQuickAssignConfirmed" />
    <DebugPanel v-if="debugOpen" @close="debugOpen = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import Sidebar from "./components/Sidebar.vue";
import DebugPanel from "./components/DebugPanel.vue";
import TaskFormModal from "./components/TaskFormModal.vue";
import ImportModal from "./components/ImportModal.vue";
import ExtraLineModal from "./components/ExtraLineModal.vue";
import SettingsModal from "./components/SettingsModal.vue";
import DeptRoomModal from "./components/DeptRoomModal.vue";
import TimelinePanel from "./components/TimelinePanel.vue";
import StaffPoolPanel from "./components/StaffPoolPanel.vue";
import PendingQueuePanel from "./components/PendingQueuePanel.vue";
import PatientListPanel from "./components/PatientListPanel.vue";
import UpdateModal from "./components/UpdateModal.vue";
import QuickAssignModal from "./components/QuickAssignModal.vue";
import { useLogger } from "./composables/useLogger";
import { useUpdater } from "./composables/useUpdater";
import { useTasksStore } from "./stores/tasks";
import { useStaffStore } from "./stores/staff";
import { useAssignmentsStore } from "./stores/assignments";
import type { UrgencyLevel } from "./types";

interface QuickAssignPrefill {
  expected_room: string;
  urgency: UrgencyLevel;
  dept: string;
  anesthesia: string;
  est_time_mins: number;
}

const modal = ref<"emergency" | "import" | "extra" | "settings" | "dept_room" | "quick_assign" | null>(null);
const view = ref<"board" | "patient_list">("board");
const debugOpen = ref(false);
const backupActive = ref(false);
const timelineRef = ref<any>(null);
const selectedDate = ref<string>("");
const showGuide = ref(false);
const emergencyPrefill = ref<QuickAssignPrefill | undefined>();

const tasksStore = useTasksStore();
const staffStore = useStaffStore();
const assignmentsStore = useAssignmentsStore();
const updater = useUpdater();

function onStaffAssigned() {
  timelineRef.value?.reload();
}

function onQuickAssignConfirmed(prefill: QuickAssignPrefill) {
  emergencyPrefill.value = prefill;
  modal.value = "emergency";
}

function handleKeydown(e: KeyboardEvent) {
  const key = e.key.toLowerCase();
  if (e.ctrlKey && e.shiftKey && key === "d") {
    e.preventDefault();
    debugOpen.value = !debugOpen.value;
  }
  if (e.key === "Escape") {
    debugOpen.value = false;
    modal.value = null;
  }
}

onMounted(async () => {
  useLogger().initLogger();
  window.addEventListener("keydown", handleKeydown);
  const d = new Date();
  const todayStr = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
  
  // 循序載入，確保最重要的人員池最先被正確初始化
  try {
    await staffStore.load();
    await tasksStore.load();
    await assignmentsStore.load(todayStr);
    console.log("[App] All data loaded");
    if (tasksStore.tasks.length === 0) showGuide.value = true;
  } catch (err) {
    console.error("[App] Load error:", err);
  }
  
  setTimeout(() => updater.checkForUpdate(), 3000);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
});
</script>

<style scoped>
.banner-enter-active, .banner-leave-active { transition: max-height 0.2s ease, opacity 0.2s ease; overflow: hidden; }
.banner-enter-from, .banner-leave-to { max-height: 0; opacity: 0; }
.banner-enter-to, .banner-leave-from { max-height: 80px; opacity: 1; }
</style>
