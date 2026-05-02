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
          <TimelinePanel ref="timelineRef" />
        </section>
        <section class="flex-[1] flex flex-col overflow-hidden">
          <div class="flex-[2] overflow-hidden border-b border-gray-700">
            <StaffPoolPanel @assigned="onStaffAssigned" />
          </div>
          <div class="flex-[3] overflow-hidden">
            <PendingQueuePanel />
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

    <Transition name="banner">
      <div v-if="updater.status.value === 'available'" class="fixed top-3 left-1/2 -translate-x-1/2 bg-blue-800 border border-blue-600 rounded-lg px-4 py-2 flex items-center gap-3 text-sm z-40 shadow-xl">
        <i class="fa-solid fa-circle-up text-blue-300"></i>
        <span>新版本 <strong>{{ updater.updateVersion.value }}</strong> 可用</span>
        <button class="bg-blue-600 hover:bg-blue-500 px-3 py-1 rounded text-xs" @click="updater.installUpdate()">立即更新</button>
        <button class="text-blue-400 hover:text-white ml-1" @click="updater.status.value = 'idle'"><i class="fa-solid fa-xmark"></i></button>
      </div>
    </Transition>

    <TaskFormModal   v-if="modal === 'emergency'" @close="modal = null" />
    <ImportModal     v-if="modal === 'import'"    @close="modal = null" />
    <ExtraLineModal  v-if="modal === 'extra'"     @close="modal = null" />
    <SettingsModal   v-if="modal === 'settings'"  @close="modal = null" />
    <DeptRoomModal   v-if="modal === 'dept_room'" @close="modal = null" />
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
import { useLogger } from "./composables/useLogger";
import { useUpdater } from "./composables/useUpdater";
import { useTasksStore } from "./stores/tasks";
import { useStaffStore } from "./stores/staff";
import { useAssignmentsStore } from "./stores/assignments";

const modal = ref<"emergency" | "import" | "extra" | "settings" | "dept_room" | null>(null);
const view = ref<"board" | "patient_list">("board");
const debugOpen = ref(false);
const backupActive = ref(false);
const timelineRef = ref<any>(null);

const tasksStore = useTasksStore();
const staffStore = useStaffStore();
const assignmentsStore = useAssignmentsStore();
const updater = useUpdater();

function onStaffAssigned() {
  timelineRef.value?.reload();
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
