<template>
  <div class="flex h-screen bg-gray-900 text-gray-100 overflow-hidden">

    <Sidebar
      :backup-active="backupActive"
      @import="modal = 'import'"
      @add-emergency="modal = 'emergency'"
      @toggle-backup="backupActive = !backupActive"
      @request-extra="modal = 'extra'"
      @open-settings="modal = 'settings'"
    />

    <!-- Main Content: 2:1 layout -->
    <div class="flex flex-col flex-1 overflow-hidden">

      <!-- 二線機制橫幅 -->
      <Transition name="banner">
        <div
          v-if="backupActive"
          class="shrink-0 bg-red-900/80 border-b border-red-700 px-4 py-2 flex items-center gap-3 text-sm"
        >
          <i class="fa-solid fa-bolt text-red-400 animate-pulse"></i>
          <span class="font-semibold text-red-200">二線機制已啟動</span>
          <span class="text-red-400 text-xs">備用刀房與二線人員已解鎖，所有 Trauma/Level1 賦予絕對優先</span>
          <button
            class="ml-auto text-xs bg-red-800 hover:bg-red-700 px-3 py-1 rounded"
            @click="backupActive = false"
          >解除</button>
        </div>
      </Transition>

      <div class="flex flex-1 overflow-hidden">
        <!-- Block 2: Room Timeline (left 2/3) -->
        <section class="flex-[2] overflow-hidden border-r border-gray-700 flex flex-col">
          <TimelinePanel />
        </section>

        <!-- Block 1: Pending Queue (right 1/3) -->
        <section class="flex-[1] overflow-auto p-4">
          <h2 class="text-sm font-semibold text-gray-400 mb-2">待排定區</h2>
        </section>
      </div>
    </div>

    <!-- Update banner -->
    <Transition name="banner">
      <div
        v-if="updater.status.value === 'available'"
        class="fixed top-3 left-1/2 -translate-x-1/2 bg-blue-800 border border-blue-600 rounded-lg px-4 py-2 flex items-center gap-3 text-sm z-40 shadow-xl"
      >
        <i class="fa-solid fa-circle-up text-blue-300"></i>
        <span>新版本 <strong>{{ updater.updateVersion.value }}</strong> 可用</span>
        <button
          class="bg-blue-600 hover:bg-blue-500 px-3 py-1 rounded text-xs"
          @click="updater.installUpdate()"
        >立即更新</button>
        <button class="text-blue-400 hover:text-white ml-1" @click="updater.status.value = 'idle'">
          <i class="fa-solid fa-xmark"></i>
        </button>
      </div>
    </Transition>

    <!-- Modals -->
    <TaskFormModal   v-if="modal === 'emergency'" @close="modal = null" @saved="onTaskSaved" />
    <ImportModal     v-if="modal === 'import'"    @close="modal = null" @imported="onImported" />
    <ExtraLineModal  v-if="modal === 'extra'"     @close="modal = null" @confirmed="onExtraConfirmed" />
    <SettingsModal   v-if="modal === 'settings'"  @close="modal = null" />

    <!-- Debug Panel -->
    <DebugPanel v-if="debugOpen" @close="debugOpen = false" />

  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import Sidebar        from "./components/Sidebar.vue";
import DebugPanel     from "./components/DebugPanel.vue";
import TaskFormModal  from "./components/TaskFormModal.vue";
import ImportModal    from "./components/ImportModal.vue";
import ExtraLineModal from "./components/ExtraLineModal.vue";
import SettingsModal  from "./components/SettingsModal.vue";
import TimelinePanel  from "./components/TimelinePanel.vue";
import { useLogger }     from "./composables/useLogger";
import { useUpdater }    from "./composables/useUpdater";
import { useTasksStore } from "./stores/tasks";
import { useStaffStore } from "./stores/staff";
import type { SurgeryTask } from "./types";

type ModalName = "emergency" | "import" | "extra" | "settings" | null;

const modal       = ref<ModalName>(null);
const debugOpen   = ref(false);
const backupActive = ref(false);

const tasksStore = useTasksStore();
const staffStore = useStaffStore();
const updater    = useUpdater();

// ── Keyboard shortcuts ────────────────────────────────────────────────────────
function handleKeydown(e: KeyboardEvent) {
  if (e.ctrlKey && e.shiftKey && e.key === "D") { e.preventDefault(); debugOpen.value = !debugOpen.value; }
  if (e.key === "Escape") { debugOpen.value = false; modal.value = null; }
}

// ── Event handlers ────────────────────────────────────────────────────────────
function onTaskSaved(_task: SurgeryTask) { /* store already updated */ }
function onImported(count: number) { console.info(`匯入 ${count} 筆`) }
function onExtraConfirmed(payload: { staffIds: number[]; room: string; endTime: number }) {
  console.info("Extra 線確認", payload);
}

onMounted(async () => {
  useLogger().initLogger();
  window.addEventListener("keydown", handleKeydown);
  await Promise.all([tasksStore.load(), staffStore.load()]);
  // Check for updates after a short delay (non-blocking)
  setTimeout(() => updater.checkForUpdate(), 3000);
});

onUnmounted(() => window.removeEventListener("keydown", handleKeydown));
</script>

<style scoped>
.banner-enter-active, .banner-leave-active { transition: max-height 0.2s ease, opacity 0.2s ease; overflow: hidden; }
.banner-enter-from, .banner-leave-to { max-height: 0; opacity: 0; }
.banner-enter-to, .banner-leave-from { max-height: 80px; opacity: 1; }
</style>
