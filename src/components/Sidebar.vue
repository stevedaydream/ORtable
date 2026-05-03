<script setup lang="ts">
import { onMounted } from "vue";
import { useSync } from "../composables/useSync";

defineProps<{ backupActive?: boolean }>();
const emit = defineEmits(["import", "add-emergency", "toggle-backup", "request-extra", "open-settings", "open-dept-room", "open-patient-list", "quick-assign"]);

const sync = useSync();

async function doPush() { try { await sync.push(); } catch (e) {} }
async function doPull() { try { await sync.pull(); } catch (e) {} }

onMounted(async () => {
  await sync.refreshTimestamps();
});
</script>

<template>
  <aside class="w-52 bg-gray-800 flex flex-col border-r border-gray-700 shrink-0 select-none overflow-y-auto">
    <div class="px-4 py-3 border-b border-gray-700 shrink-0">
      <div class="text-sm font-bold text-white tracking-wide">Smart OR</div>
      <div class="text-xs text-gray-500">手術排程看板</div>
    </div>

    <!-- 主要功能 -->
    <nav class="flex flex-col gap-1 px-2 pt-3 pb-2 shrink-0">
      <button class="flex items-center gap-2 w-full px-2 py-1.5 rounded text-xs text-gray-300 hover:bg-gray-700/60 transition-colors text-left" @click="emit('import')">
        <i class="fa-solid fa-file-import w-4 text-center"></i>
        <span>匯入病患清單</span>
      </button>

      <button class="flex items-center gap-2 w-full px-2 py-1.5 rounded text-xs text-blue-200 hover:bg-blue-700/40 transition-colors text-left" @click="emit('add-emergency')">
        <i class="fa-solid fa-plus-circle w-4 text-center"></i>
        <span>新增急診病患</span>
      </button>

      <div class="my-1.5 border-t border-gray-700/60" />

      <button 
        class="flex items-center gap-2 w-full px-2 py-1.5 rounded text-xs transition-colors text-left"
        :class="backupActive ? 'bg-red-900/40 text-red-300 hover:bg-red-800/50' : 'text-gray-300 hover:bg-orange-700/40 hover:text-orange-300'"
        @click="emit('toggle-backup')"
      >
        <i class="fa-solid fa-bolt w-4 text-center"></i>
        <span>啟動二線機制</span>
        <span v-if="backupActive" class="ml-auto text-[10px] font-bold text-red-400 animate-pulse">ON</span>
      </button>

      <button class="flex items-center gap-2 w-full px-2 py-1.5 rounded text-xs text-gray-300 hover:bg-gray-700/60 transition-colors text-left" @click="emit('request-extra')">
        <i class="fa-solid fa-layer-group w-4 text-center"></i>
        <span>申請 Extra 線</span>
      </button>

      <div class="my-1.5 border-t border-gray-700/60" />

      <button class="flex items-center gap-2 w-full px-2 py-1.5 rounded text-xs text-violet-300 hover:bg-violet-700/30 transition-colors text-left" @click="emit('quick-assign')">
        <i class="fa-solid fa-wand-magic-sparkles w-4 text-center"></i>
        <span>快速派房</span>
      </button>

      <button class="flex items-center gap-2 w-full px-2 py-1.5 rounded text-xs text-teal-300 hover:bg-teal-700/30 transition-colors text-left" @click="emit('open-dept-room')">
        <i class="fa-solid fa-calendar-days w-4 text-center"></i>
        <span>科別房間分配</span>
      </button>

      <button class="flex items-center gap-2 w-full px-2 py-1.5 rounded text-xs text-cyan-300 hover:bg-cyan-700/30 transition-colors text-left" @click="emit('open-patient-list')">
        <i class="fa-solid fa-list-ul w-4 text-center"></i>
        <span>本日病人清單</span>
      </button>
    </nav>

    <div class="mx-3 border-t border-gray-700/60 shrink-0" />

    <!-- 雲端同步 -->
    <div class="px-2 pt-2 pb-1 shrink-0">
      <div class="text-[10px] font-semibold text-gray-500 uppercase tracking-widest px-2 mb-1">雲端同步</div>
      <button 
        class="flex items-center gap-2 w-full px-2 py-1.5 rounded text-xs text-gray-300 hover:bg-gray-700/60 transition-colors text-left disabled:opacity-50" 
        :disabled="sync.status.value === 'pushing'" 
        @click="doPush"
      >
        <i class="fa-solid w-4 text-center" :class="sync.status.value === 'pushing' ? 'fa-spinner animate-spin' : 'fa-cloud-arrow-up'"></i>
        <span>推送至 Sheets</span>
      </button>
      <button 
        class="flex items-center gap-2 w-full px-2 py-1.5 rounded text-xs text-gray-300 hover:bg-gray-700/60 transition-colors text-left disabled:opacity-50" 
        :disabled="sync.status.value === 'pulling'" 
        @click="doPull"
      >
        <i class="fa-solid w-4 text-center" :class="sync.status.value === 'pulling' ? 'fa-spinner animate-spin' : 'fa-cloud-arrow-down'"></i>
        <span>從 Sheets 拉取</span>
      </button>
      
      <div class="px-2 mt-1 space-y-0.5">
        <div class="text-[10px] text-gray-600">↑ {{ sync.formatTs(sync.timestamps.value.last_push_at) }}</div>
        <div class="text-[10px] text-gray-600">↓ {{ sync.formatTs(sync.timestamps.value.last_pull_at) }}</div>
      </div>
    </div>

    <div class="flex-1 min-h-[20px]" />
    <div class="mx-3 border-t border-gray-700/60 shrink-0" />

    <div class="px-2 py-2 shrink-0">
      <button class="flex items-center gap-2 w-full px-2 py-1.5 rounded text-xs text-gray-300 hover:bg-gray-700/60 transition-colors text-left" @click="emit('open-settings')">
        <i class="fa-solid fa-gear w-4 text-center"></i>
        <span>設定</span>
      </button>
    </div>
  </aside>
</template>
