<template>
  <div class="fixed inset-0 bg-black/70 z-50 flex items-center justify-center p-4" @mousedown.self="$emit('close')">
    <div class="bg-gray-800 rounded-xl shadow-2xl w-full max-w-lg flex flex-col max-h-[90vh]">

      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-4 border-b border-gray-700 shrink-0">
        <h2 class="text-base font-bold text-white">
          <i class="fa-solid fa-layer-group text-purple-400 mr-2"></i>申請 Extra 加班線
        </h2>
        <button class="text-gray-400 hover:text-white" @click="$emit('close')">
          <i class="fa-solid fa-xmark"></i>
        </button>
      </div>

      <!-- Body -->
      <div class="flex-1 overflow-y-auto px-5 py-4 space-y-4">

        <!-- Extra 結束時間 + 房間 -->
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="form-label">Extra 預計結束時間</label>
            <input
              v-model="endTimeInput"
              type="datetime-local"
              class="form-input"
              @change="checkCompliance"
            />
          </div>
          <div>
            <label class="form-label">指定 Extra 刀房</label>
            <input v-model="room" class="form-input" placeholder="Extra-1" />
          </div>
        </div>

        <!-- 勞基法說明 -->
        <div class="bg-gray-900/50 rounded p-3 text-xs text-gray-500 space-y-0.5">
          <div class="text-gray-400 font-semibold mb-1">勞基法防呆規則</div>
          <div>▸ 當日工時：上班時間 → Extra 結束 ≤ 12 小時</div>
          <div>▸ 班距間隔：Extra 結束 → 明日班 ≥ 12 小時</div>
        </div>

        <!-- 人員合規列表 -->
        <div>
          <label class="form-label">
            當班人員合規狀態
            <span v-if="checking" class="ml-1 text-gray-600">
              <i class="fa-solid fa-spinner animate-spin"></i>
            </span>
          </label>

          <div v-if="!staffStore.staffList.length" class="text-xs text-gray-600 text-center py-4">
            尚無人員資料
          </div>

          <div v-else class="space-y-1.5 max-h-52 overflow-y-auto pr-1">
            <div
              v-for="s in staffStore.staffList"
              :key="s.id"
              class="flex items-center gap-3 px-3 py-2 rounded-lg border transition-colors"
              :class="complianceMap[s.id]?.allowed === false
                ? 'border-red-800/50 bg-red-900/10 opacity-60 cursor-not-allowed'
                : selectedIds.has(s.id)
                  ? 'border-purple-500 bg-purple-900/20 cursor-pointer'
                  : 'border-gray-700 hover:border-gray-500 bg-gray-700/30 cursor-pointer'"
              @click="toggle(s.id, complianceMap[s.id]?.allowed !== false)"
            >
              <!-- 選取框 -->
              <div
                class="w-4 h-4 rounded border-2 shrink-0 flex items-center justify-center"
                :class="complianceMap[s.id]?.allowed === false
                  ? 'border-red-700'
                  : selectedIds.has(s.id) ? 'border-purple-400 bg-purple-500' : 'border-gray-600'"
              >
                <i v-if="selectedIds.has(s.id)" class="fa-solid fa-check text-white text-[9px]"></i>
              </div>

              <!-- 姓名 + 角色 -->
              <div class="flex-1 min-w-0">
                <div class="text-sm text-gray-200">{{ s.name }}</div>
                <div class="text-xs text-gray-500">{{ s.role }} · {{ s.type === 'nur' ? '護理師' : '醫師' }}</div>
              </div>

              <!-- 志願標記 -->
              <span v-if="s.is_volunteer_extra" class="text-[10px] bg-purple-800/50 text-purple-300 px-1.5 py-0.5 rounded">
                自願 Extra
              </span>

              <!-- 合規狀態 -->
              <div class="shrink-0 text-right">
                <template v-if="complianceMap[s.id]">
                  <span v-if="complianceMap[s.id].allowed" class="text-xs text-green-400">
                    <i class="fa-solid fa-circle-check mr-0.5"></i>可加班
                  </span>
                  <span v-else class="text-xs text-red-400 max-w-[100px] text-right leading-tight block">
                    <i class="fa-solid fa-ban mr-0.5"></i>{{ complianceMap[s.id].reason }}
                  </span>
                </template>
                <span v-else class="text-xs text-gray-600">—</span>
              </div>
            </div>
          </div>
        </div>

      </div>

      <!-- Footer -->
      <div class="flex items-center justify-between px-5 py-4 border-t border-gray-700 shrink-0">
        <span class="text-xs text-gray-500">
          已選 {{ selectedIds.size }} 位人員
        </span>
        <div class="flex gap-3">
          <button class="btn-secondary" @click="$emit('close')">取消</button>
          <button
            class="px-4 py-2 bg-purple-700 hover:bg-purple-600 disabled:opacity-40 rounded text-sm font-medium transition-colors"
            :disabled="!selectedIds.size || !room.trim()"
            @click="confirm"
          >
            <i class="fa-solid fa-check mr-1.5"></i>確認開設 Extra 線
          </button>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useStaffStore } from "../stores/staff";
import type { StaffComplianceResult } from "../types";

const emit = defineEmits<{
  close: [];
  confirmed: [payload: { staffIds: number[]; room: string; endTime: number }];
}>();

const staffStore = useStaffStore();
const checking = ref(false);
const room = ref("Extra-1");
const complianceMap = reactive<Record<number, StaffComplianceResult>>({});
const selectedIds = ref(new Set<number>());

// Default: now + 4 hours
const defaultEnd = new Date(Date.now() + 4 * 3600 * 1000);
const endTimeInput = ref(defaultEnd.toISOString().slice(0, 16));

async function checkCompliance() {
  if (!endTimeInput.value || !staffStore.staffList.length) return;
  checking.value = true;
  try {
    const endTs = Math.floor(new Date(endTimeInput.value).getTime() / 1000);
    const results = await invoke<StaffComplianceResult[]>("batch_check_extra_compliance", {
      staffList: staffStore.staffList,
      estimatedExtraEnd: endTs,
    });
    results.forEach((r) => {
      complianceMap[r.staff_id] = r;
      // Auto-deselect if now blocked
      if (!r.allowed) selectedIds.value.delete(r.staff_id);
    });
  } finally {
    checking.value = false;
  }
}

function toggle(id: number, allowed: boolean) {
  if (!allowed) return;
  if (selectedIds.value.has(id)) selectedIds.value.delete(id);
  else selectedIds.value.add(id);
}

function confirm() {
  const endTs = Math.floor(new Date(endTimeInput.value).getTime() / 1000);
  emit("confirmed", { staffIds: [...selectedIds.value], room: room.value, endTime: endTs });
  emit("close");
}

onMounted(checkCompliance);
</script>
