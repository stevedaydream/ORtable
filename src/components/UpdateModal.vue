<template>
  <Teleport to="body">
    <div class="fixed inset-0 bg-black/60 z-[9995] flex items-center justify-center p-4">
      <div class="bg-gray-800 rounded-xl border border-blue-700/60 shadow-2xl w-full max-w-md">

        <!-- Header -->
        <div class="flex items-center gap-3 px-5 py-4 border-b border-gray-700">
          <div class="w-9 h-9 rounded-full bg-blue-600/20 border border-blue-500/40 flex items-center justify-center shrink-0">
            <i class="fa-solid fa-circle-up text-blue-400 text-base"></i>
          </div>
          <div>
            <div class="text-sm font-bold text-white">發現新版本</div>
            <div class="text-xs text-blue-300 font-mono mt-0.5">v{{ updateVersion }}</div>
          </div>
          <button class="ml-auto text-gray-500 hover:text-gray-300" @click="dismiss">
            <i class="fa-solid fa-xmark"></i>
          </button>
        </div>

        <!-- Release notes -->
        <div v-if="updateBody" class="px-5 py-4 max-h-56 overflow-y-auto">
          <div class="text-xs text-gray-400 font-medium mb-2 uppercase tracking-wide">更新內容</div>
          <pre class="text-xs text-gray-300 whitespace-pre-wrap font-sans leading-relaxed">{{ updateBody }}</pre>
        </div>
        <div v-else class="px-5 py-4 text-xs text-gray-500">請前往 GitHub Releases 查看更新內容。</div>

        <!-- Actions -->
        <div class="px-5 py-4 border-t border-gray-700 flex items-center gap-3 justify-end">
          <button class="text-xs text-gray-400 hover:text-gray-200" @click="dismiss">稍後提醒</button>
          <button
            class="flex items-center gap-2 text-xs px-4 py-2 rounded-lg bg-blue-600 hover:bg-blue-500 text-white font-semibold transition-colors"
            :disabled="downloading"
            @click="install"
          >
            <i v-if="downloading" class="fa-solid fa-spinner animate-spin"></i>
            <i v-else class="fa-solid fa-download"></i>
            {{ downloading ? '下載中…' : '立即更新' }}
          </button>
        </div>

      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useUpdater } from "../composables/useUpdater";

const { status, updateVersion, updateBody, installUpdate } = useUpdater();

const downloading = computed(() => status.value === "downloading");

function dismiss() {
  status.value = "idle";
}

async function install() {
  await installUpdate();
}
</script>
