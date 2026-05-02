<template>
  <Teleport to="body">
    <div class="fixed inset-0 bg-black/70 z-[9992] flex items-center justify-center p-4" @mousedown.self="$emit('close')">
      <div class="bg-gray-800 rounded-xl shadow-2xl w-full max-w-md flex flex-col max-h-[80vh]">

        <!-- Header -->
        <div class="flex items-center gap-3 px-5 py-3.5 border-b border-gray-700 shrink-0">
          <i class="fa-solid fa-receipt text-yellow-400 text-sm"></i>
          <span class="font-bold text-white text-sm">添加自費備注</span>
          <span class="text-gray-500 text-xs truncate max-w-[120px]">{{ patientName }}</span>
          <div class="flex-1" />
          <button class="text-gray-400 hover:text-white" @click="$emit('close')">
            <i class="fa-solid fa-xmark"></i>
          </button>
        </div>

        <!-- Search -->
        <div class="px-4 py-3 border-b border-gray-700 shrink-0">
          <div class="relative">
            <i class="fa-solid fa-magnifying-glass absolute left-2.5 top-1/2 -translate-y-1/2 text-gray-500 text-[10px]"></i>
            <input
              v-model="search"
              class="w-full bg-gray-700 border border-gray-600 rounded pl-7 pr-3 py-1.5 text-xs text-gray-100 placeholder-gray-500 outline-none focus:border-blue-500"
              placeholder="搜尋自費項目名稱…"
              autofocus
            />
          </div>
        </div>

        <!-- List -->
        <div class="flex-1 overflow-y-auto px-4 py-2">
          <div v-if="loading" class="flex justify-center py-8 text-gray-500 text-xs gap-2">
            <i class="fa-solid fa-spinner animate-spin"></i><span>載入中…</span>
          </div>
          <div v-else-if="filtered.length === 0" class="text-center py-8 text-gray-600 text-xs">
            {{ items.length === 0 ? '尚未設定自費項目（請至設定 → 自費項目新增）' : '無符合的項目' }}
          </div>
          <label
            v-for="item in filtered" :key="item.id"
            class="flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer hover:bg-gray-700/60 transition-colors"
            :class="selected.has(item.id) ? 'bg-yellow-900/20 border border-yellow-800/40' : 'border border-transparent'"
          >
            <input
              type="checkbox"
              :checked="selected.has(item.id)"
              class="rounded accent-yellow-500"
              @change="toggle(item.id)"
            />
            <div class="flex-1 min-w-0">
              <div class="text-xs font-medium text-gray-100 truncate">{{ item.name }}</div>
              <div v-if="item.notes" class="text-[10px] text-gray-500 truncate">{{ item.notes }}</div>
            </div>
            <div v-if="item.price > 0" class="text-[10px] text-yellow-400 shrink-0 font-mono">
              ${{ item.price.toLocaleString() }}
            </div>
          </label>
        </div>

        <!-- Footer -->
        <div class="flex items-center justify-between px-5 py-3.5 border-t border-gray-700 shrink-0">
          <span class="text-xs text-gray-500">已選 {{ selected.size }} 項</span>
          <div class="flex gap-2">
            <button class="btn-secondary text-xs" @click="$emit('close')">取消</button>
            <button
              class="btn-primary text-xs"
              :disabled="selected.size === 0"
              @click="confirm"
            >
              <i class="fa-solid fa-check mr-1.5"></i>套用備注
            </button>
          </div>
        </div>

      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import type { SelfPayItem, SurgeryTask } from "../types";
import { useSelfPayDb } from "../composables/useDatabase";
import { useTasksStore } from "../stores/tasks";

const props = defineProps<{ task: SurgeryTask }>();
const emit = defineEmits<{ close: []; applied: [] }>();

const tasksStore = useTasksStore();
const selfPayDb = useSelfPayDb();

const patientName = computed(() => props.task.patient_name);
const items = ref<SelfPayItem[]>([]);
const loading = ref(true);
const search = ref("");
const selected = ref(new Set<number>());

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return items.value;
  return items.value.filter(i => i.name.toLowerCase().includes(q) || i.notes.toLowerCase().includes(q));
});

function toggle(id: number) {
  if (selected.value.has(id)) selected.value.delete(id);
  else selected.value.add(id);
  selected.value = new Set(selected.value); // trigger reactivity
}

async function confirm() {
  const chosen = items.value.filter(i => selected.value.has(i.id));
  if (chosen.length === 0) return;
  const appendText = chosen.map(i => i.price > 0 ? `${i.name}($${i.price})` : i.name).join("、");
  const newNote = props.task.vs_note
    ? `${props.task.vs_note}、${appendText}`
    : appendText;
  try {
    await tasksStore.edit({ ...props.task, vs_note: newNote });
    emit("applied");
    emit("close");
  } catch (e) {
    console.error("[SelfPayPickerModal] 更新備注失敗:", e);
  }
}

onMounted(async () => {
  try {
    items.value = await selfPayDb.getAll();
  } finally {
    loading.value = false;
  }
});
</script>
