<template>
  <div
    class="fixed bottom-4 right-4 w-[480px] max-h-[60vh] flex flex-col bg-gray-900 border border-gray-600 rounded-lg shadow-2xl z-[9999] text-xs"
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-3 py-2 border-b border-gray-700 bg-gray-800 rounded-t-lg">
      <span class="font-bold text-gray-300">Debug Panel ({{ logs.length }})</span>
      <div class="flex gap-2">
        <button class="text-gray-400 hover:text-white px-2 py-0.5 border border-gray-600 rounded" @click="exportLogs">匯出</button>
        <button class="text-gray-400 hover:text-white px-2 py-0.5 border border-gray-600 rounded" @click="clearLogs">清除</button>
        <button class="text-gray-400 hover:text-white px-2 py-0.5 border border-gray-600 rounded" @click="$emit('close')">✕</button>
      </div>
    </div>

    <!-- Log List -->
    <div class="flex-1 overflow-y-auto divide-y divide-gray-800">
      <div
        v-for="entry in [...logs].reverse()"
        :key="entry.id"
        class="px-3 py-1.5 cursor-pointer select-none"
        :class="levelClass(entry.level)"
        @click="entry.detail ? toggle(entry.id) : copy(entry)"
        @dblclick="entry.detail && copy(entry)"
      >
        <div class="flex gap-2 items-start">
          <span class="text-gray-500 shrink-0">{{ entry.time }}</span>
          <span class="uppercase font-bold shrink-0 w-10" :class="badgeClass(entry.level)">{{ entry.level }}</span>
          <span :class="copiedId === entry.id ? 'text-green-400' : ''">
            {{ copiedId === entry.id ? "✓ 已複製" : entry.message }}
          </span>
        </div>
        <pre
          v-if="entry.detail && expanded.has(entry.id)"
          class="mt-1 ml-12 text-gray-400 whitespace-pre-wrap break-all"
          @click.stop="copy(entry)"
        >{{ entry.detail }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useLogger } from "../composables/useLogger";

const { logs, exportLogs, clearLogs } = useLogger();
defineEmits(["close"]);

const expanded = ref(new Set<number>());
const copiedId = ref<number | null>(null);

function toggle(id: number) {
  if (expanded.value.has(id)) expanded.value.delete(id);
  else expanded.value.add(id);
}

async function copy(entry: { id: number; time: string; level: string; message: string; detail?: string }) {
  const text = `[${entry.time}] ${entry.level.toUpperCase()} ${entry.message}${entry.detail ? "\n" + entry.detail : ""}`;
  await navigator.clipboard.writeText(text);
  copiedId.value = entry.id;
  setTimeout(() => (copiedId.value = null), 1500);
}

function levelClass(level: string) {
  return {
    "hover:bg-red-950/40": level === "error",
    "hover:bg-yellow-950/40": level === "warn",
    "hover:bg-gray-800/60": level === "info",
  };
}

function badgeClass(level: string) {
  return {
    "text-red-400": level === "error",
    "text-yellow-400": level === "warn",
    "text-blue-400": level === "info",
  };
}
</script>
