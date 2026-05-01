<template>
  <aside class="w-52 bg-gray-800 flex flex-col border-r border-gray-700 shrink-0 select-none">

    <!-- ── Logo ────────────────────────────────────────────────── -->
    <div class="px-4 py-3 border-b border-gray-700">
      <div class="text-sm font-bold text-white tracking-wide">Smart OR</div>
      <div class="text-xs text-gray-500">手術排程看板</div>
    </div>

    <!-- ── 主要功能 ──────────────────────────────────────────── -->
    <nav class="flex flex-col gap-0.5 px-2 pt-3 pb-2">
      <SbBtn icon="fa-file-import"   label="匯入病患清單"  @click="$emit('import')" />
      <SbBtn icon="fa-plus-circle"   label="新增急診病患"  accent="blue" @click="$emit('addEmergency')" />

      <div class="my-1.5 border-t border-gray-700/60" />

      <SbBtn
        icon="fa-bolt"
        label="啟動二線機制"
        :accent="backupActive ? 'red-active' : 'orange'"
        @click="$emit('toggleBackup')"
      >
        <span v-if="backupActive" class="ml-auto text-[10px] font-bold text-red-400 animate-pulse">ON</span>
      </SbBtn>

      <SbBtn icon="fa-layer-group" label="申請 Extra 線" @click="$emit('requestExtra')" />
    </nav>

    <div class="mx-3 border-t border-gray-700/60" />

    <!-- ── 雲端同步 ──────────────────────────────────────────── -->
    <div class="px-2 pt-2 pb-1">
      <div class="text-[10px] font-semibold text-gray-500 uppercase tracking-widest px-2 mb-1">雲端同步</div>

      <SbBtn
        icon="fa-cloud-arrow-up"
        label="推送至 Sheets"
        :loading="syncStatus === 'pushing'"
        @click="doPush"
      />
      <SbBtn
        icon="fa-cloud-arrow-down"
        label="從 Sheets 拉取"
        :loading="syncStatus === 'pulling'"
        @click="doPull"
      />

      <!-- 時間戳 -->
      <div class="px-2 mt-1 space-y-0.5">
        <div class="text-[10px] text-gray-600">
          ↑ {{ formatTs(timestamps.last_push_at) }}
        </div>
        <div class="text-[10px] text-gray-600">
          ↓ {{ formatTs(timestamps.last_pull_at) }}
        </div>
      </div>

      <!-- 同步狀態訊息 -->
      <div
        v-if="syncMessage"
        class="mx-1 mt-2 px-2 py-1 rounded text-[10px] leading-tight"
        :class="syncStatus === 'error' ? 'bg-red-900/60 text-red-300' : 'bg-green-900/50 text-green-300'"
      >
        {{ syncMessage }}
      </div>
    </div>

    <!-- ── 彈性空間 ──────────────────────────────────────────── -->
    <div class="flex-1" />

    <div class="mx-3 border-t border-gray-700/60" />

    <!-- ── 設定 ─────────────────────────────────────────────── -->
    <div class="px-2 py-2">
      <SbBtn icon="fa-gear" label="設定" @click="$emit('openSettings')" />
    </div>

  </aside>
</template>

<script setup lang="ts">
import { onMounted, defineComponent, h } from "vue";
import { useSync } from "../composables/useSync";

defineProps<{ backupActive?: boolean }>();
defineEmits(["import", "addEmergency", "toggleBackup", "requestExtra", "openSettings"]);

// ── Sync ──────────────────────────────────────────────────────────────────────
const { status: syncStatus, message: syncMessage, timestamps, push, pull, refreshTimestamps, formatTs } = useSync();

async function doPush() {
  try { await push(); } catch (_) {}
}
async function doPull() {
  try { await pull(); } catch (_) {}
}


onMounted(async () => {
  await refreshTimestamps();
});

// ── Internal SbBtn ────────────────────────────────────────────────────────────
const SbBtn = defineComponent({
  props: {
    icon:    { type: String, required: true },
    label:   { type: String, required: true },
    accent:  { type: String, default: "" },
    loading: { type: Boolean, default: false },
  },
  emits: ["click"],
  setup(props, { emit, slots }) {
    const accentClass = (a: string) => ({
      "blue":        "hover:bg-blue-700/40 hover:text-blue-300",
      "orange":      "hover:bg-orange-700/40 hover:text-orange-300",
      "red-active":  "bg-red-900/40 text-red-300 hover:bg-red-800/50",
    }[a] ?? "hover:bg-gray-700/60");

    return () =>
      h("button", {
        class: `flex items-center gap-2 w-full px-2 py-1.5 rounded text-xs text-gray-300 transition-colors ${accentClass(props.accent)}`,
        onClick: () => emit("click"),
      }, [
        h("i", {
          class: `fa-solid ${props.icon} w-4 text-center ${props.loading ? "animate-spin fa-spinner" : ""}`,
        }),
        h("span", { class: "truncate" }, props.label),
        ...(slots.default ? [slots.default()] : []),
      ]);
  },
});
</script>

