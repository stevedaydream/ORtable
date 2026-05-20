<template>
  <div class="fixed inset-0 bg-black/70 z-50 flex items-center justify-center p-4" @mousedown.self="$emit('close')">
    <div class="bg-gray-800 rounded-xl shadow-2xl w-full max-w-2xl flex flex-col max-h-[90vh]">

      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-4 border-b border-gray-700 shrink-0">
        <h2 class="text-base font-bold text-white">
          <i class="fa-solid fa-wand-magic-sparkles text-violet-400 mr-2"></i>快速派房
        </h2>
        <button class="text-gray-400 hover:text-white" @click="$emit('close')">
          <i class="fa-solid fa-xmark"></i>
        </button>
      </div>

      <!-- Body -->
      <div class="flex-1 overflow-y-auto px-5 py-4 space-y-4">

        <!-- Layer 1: Urgency -->
        <div>
          <div class="text-[10px] font-semibold text-gray-500 uppercase tracking-widest mb-2">緊急程度</div>
          <div class="flex gap-2 flex-wrap">
            <button
              v-for="u in URGENCY_OPTIONS" :key="u.value"
              class="px-3 py-1.5 rounded text-xs font-bold border-2 transition-all"
              :class="form.urgency === u.value ? u.activeClass : u.inactiveClass"
              @click="form.urgency = u.value; fetchRecs()"
            >{{ u.label }}</button>
          </div>
        </div>

        <!-- Layer 2: Dept -->
        <div>
          <div class="text-[10px] font-semibold text-gray-500 uppercase tracking-widest mb-2">科別</div>
          <div v-if="availableDepts.length === 0" class="text-[10px] text-amber-500/80 mb-1.5">
            尚未設定科別規則，請直接在下方輸入科別名稱
          </div>
          <div class="flex gap-1.5 flex-wrap">
            <button
              v-for="dept in availableDepts" :key="dept"
              class="px-2.5 py-1 rounded text-xs border transition-all"
              :class="form.dept === dept
                ? 'bg-violet-700 border-violet-500 text-white'
                : 'border-gray-600 text-gray-400 hover:bg-gray-700/60 hover:text-gray-200'"
              @click="form.dept = dept; fetchRecs()"
            >{{ dept }}</button>
            <input
              v-model="customDept"
              class="px-2.5 py-1 rounded text-xs border border-gray-600 bg-gray-900 text-gray-300 outline-none focus:border-violet-500 w-24"
              placeholder="其他…"
              @blur="if (customDept) { form.dept = customDept; fetchRecs(); }"
              @keydown.enter="if (customDept) { form.dept = customDept; fetchRecs(); }"
            />
          </div>
        </div>

        <!-- Layer 3: Anesthesia -->
        <div>
          <div class="text-[10px] font-semibold text-gray-500 uppercase tracking-widest mb-2">麻醉方式</div>
          <div class="flex gap-2">
            <button
              v-for="a in ANES_OPTIONS" :key="a"
              class="px-3 py-1.5 rounded text-xs border transition-all"
              :class="form.anesthesia === a
                ? 'bg-gray-600 border-gray-400 text-white'
                : 'border-gray-700 text-gray-400 hover:bg-gray-700/50'"
              @click="form.anesthesia = a; fetchRecs()"
            >{{ a }}</button>
          </div>
        </div>

        <!-- Layer 4: Duration -->
        <div>
          <div class="text-[10px] font-semibold text-gray-500 uppercase tracking-widest mb-2">預估時長</div>
          <div class="flex gap-2 flex-wrap">
            <button
              v-for="d in DURATION_OPTIONS" :key="d.mins"
              class="px-3 py-1.5 rounded text-xs border transition-all"
              :class="form.est_mins === d.mins
                ? 'bg-gray-600 border-gray-400 text-white'
                : 'border-gray-700 text-gray-400 hover:bg-gray-700/50'"
              @click="form.est_mins = d.mins; fetchRecs()"
            >{{ d.label }}</button>
          </div>
        </div>

        <!-- Recommendations -->
        <div>
          <div class="text-[10px] font-semibold text-gray-500 uppercase tracking-widest mb-2">
            推薦排房
            <span v-if="loading" class="ml-1 text-blue-400 normal-case font-normal">
              <i class="fa-solid fa-spinner animate-spin text-[9px]"></i> 計算中…
            </span>
          </div>

          <div v-if="!form.urgency || !form.dept" class="text-xs text-gray-600 py-4 text-center">
            請先選擇緊急程度與科別
          </div>

          <div v-else-if="recommendations.length === 0 && !loading" class="text-xs text-gray-600 py-4 text-center">
            無可用手術室
            <div class="text-[10px] text-gray-700 mt-1">請確認「設定 → 房間管理」已新增手術室</div>
          </div>

          <div v-else class="space-y-1.5">
            <div
              v-for="(rec, i) in recommendations" :key="rec.room_name"
              class="flex items-center gap-3 rounded-lg border px-3 py-2.5 cursor-pointer transition-all"
              :class="[
                selectedRoom === rec.room_name
                  ? 'border-violet-500 bg-violet-900/20'
                  : 'border-gray-700 bg-gray-900/30 hover:border-gray-600 hover:bg-gray-700/20',
                i === 0 ? 'ring-1 ring-green-600/30' : ''
              ]"
              @click="selectedRoom = rec.room_name"
            >
              <!-- Rank icon -->
              <div class="shrink-0 text-base w-5 text-center">
                {{ i === 0 ? '🏆' : i === 1 ? '🥈' : i === 2 ? '🥉' : '·' }}
              </div>

              <!-- Main info -->
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-1.5 flex-wrap">
                  <span class="text-sm font-bold" :class="i === 0 ? 'text-gray-100' : 'text-gray-300'">
                    {{ rec.room_name }}
                  </span>
                  <span v-if="rec.is_emergency_room"
                    class="text-[9px] font-bold bg-red-700/60 text-red-200 border border-red-600/50 px-1 rounded">
                    急診房
                  </span>
                  <span v-if="rec.dept_match"
                    class="text-[9px] bg-teal-900/60 text-teal-300 border border-teal-700/50 px-1 rounded">
                    科別符合
                  </span>
                  <span v-if="rec.anes_conflict"
                    class="text-[9px] bg-orange-900/60 text-orange-300 border border-orange-700/50 px-1 rounded">
                    <i class="fa-solid fa-triangle-exclamation text-[8px] mr-0.5"></i>麻醉不符
                  </span>
                  <span v-if="rec.has_staff"
                    class="text-[9px] bg-blue-900/60 text-blue-300 border border-blue-700/50 px-1 rounded">
                    <i class="fa-solid fa-user-check text-[8px] mr-0.5"></i>人員就位
                  </span>
                  <span v-else
                    class="text-[9px] bg-gray-800 text-gray-600 border border-gray-700 px-1 rounded">
                    <i class="fa-solid fa-user-slash text-[8px] mr-0.5"></i>未排人員
                  </span>
                </div>
                <div class="text-[10px] text-gray-400 mt-0.5 truncate">{{ rec.reason }}</div>
              </div>

              <!-- Availability -->
              <div class="shrink-0 text-right">
                <div class="text-xs font-bold"
                  :class="rec.is_available ? 'text-green-400' : rec.within_deadline ? 'text-yellow-400' : 'text-red-400'">
                  {{ rec.is_available ? '立即可用' : `+${rec.est_available_mins}m` }}
                </div>
                <div v-if="!rec.within_deadline" class="text-[9px] text-red-400 mt-0.5">⚠ 超出截止</div>
                <div v-else-if="!rec.is_available" class="text-[9px] text-gray-600 mt-0.5">等待中</div>
              </div>

              <!-- Selected check -->
              <div class="shrink-0 w-4">
                <i v-if="selectedRoom === rec.room_name" class="fa-solid fa-circle-check text-violet-400 text-xs"></i>
              </div>
            </div>
          </div>
        </div>

      </div>

      <!-- Footer -->
      <div class="flex items-center justify-between px-5 py-4 border-t border-gray-700 shrink-0">
        <span class="text-xs text-gray-500">
          <span v-if="selectedRoom" class="text-violet-300">已選：{{ selectedRoom }}</span>
          <span v-else>點擊推薦列表選擇刀房</span>
        </span>
        <div class="flex gap-3">
          <button class="btn-secondary" @click="$emit('close')">取消</button>
          <button
            class="btn-primary"
            :disabled="!selectedRoom || !form.urgency"
            @click="confirm"
          >
            <i class="fa-solid fa-wand-magic-sparkles mr-1.5"></i>
            排入 {{ selectedRoom || '—' }}
            <span class="text-xs font-normal opacity-80 ml-1">
              · {{ form.urgency === 'Normal' ? '建立病人資料' : '建立急診單' }}
            </span>
          </button>
        </div>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from "vue";
import { useDeptRulesDb, useRoomRecommendationDb } from "../composables/useDatabase";
import type { RoomRecommendation, UrgencyLevel } from "../types";

const emit = defineEmits<{
  close: [];
  confirmed: [prefill: { expected_room: string; urgency: UrgencyLevel; dept: string; anesthesia: string; est_time_mins: number }];
}>();

const URGENCY_OPTIONS = [
  { value: "Trauma",  label: "Trauma 30m",  activeClass: "bg-red-700    border-red-500    text-white", inactiveClass: "border-red-800    text-red-500    hover:bg-red-900/30" },
  { value: "Level1",  label: "Level1 2hr",  activeClass: "bg-orange-700 border-orange-500 text-white", inactiveClass: "border-orange-800 text-orange-500 hover:bg-orange-900/30" },
  { value: "Level2",  label: "Level2 6hr",  activeClass: "bg-yellow-700 border-yellow-500 text-white", inactiveClass: "border-yellow-800 text-yellow-500 hover:bg-yellow-900/30" },
  { value: "Level3",  label: "Level3 24hr", activeClass: "bg-blue-700   border-blue-500   text-white", inactiveClass: "border-blue-800   text-blue-500   hover:bg-blue-900/30" },
  { value: "Normal",  label: "常規",         activeClass: "bg-gray-600   border-gray-400   text-white", inactiveClass: "border-gray-600   text-gray-500   hover:bg-gray-700/40" },
] as const;

const ANES_OPTIONS = ["全身麻醉", "半身麻醉", "局部麻醉"];
const DURATION_OPTIONS = [
  { label: "< 1hr",  mins: 60  },
  { label: "1~2hr",  mins: 120 },
  { label: "2~4hr",  mins: 240 },
  { label: "> 4hr",  mins: 480 },
];

const form = reactive({
  urgency: "" as UrgencyLevel | "",
  dept: "",
  anesthesia: "",
  est_mins: 60,
});
const customDept = ref("");
const selectedRoom = ref("");
const availableDepts = ref<string[]>([]);
const recommendations = ref<RoomRecommendation[]>([]);
const loading = ref(false);

const deptRulesDb = useDeptRulesDb();
const recDb = useRoomRecommendationDb();

function getTodayStr(): string {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

async function fetchRecs() {
  if (!form.urgency || !form.dept) return;
  loading.value = true;
  selectedRoom.value = "";
  try {
    recommendations.value = await recDb.get(form.urgency, form.dept, form.anesthesia, form.est_mins, getTodayStr());
  } catch (e) {
    console.error("[QuickAssign] 推薦計算失敗:", e);
  } finally {
    loading.value = false;
  }
}

function confirm() {
  if (!selectedRoom.value || !form.urgency) return;
  emit("confirmed", {
    expected_room: selectedRoom.value,
    urgency: form.urgency as UrgencyLevel,
    dept: form.dept,
    anesthesia: form.anesthesia,
    est_time_mins: form.est_mins,
  });
  // 不 emit close — App.vue 會把 modal 換成 "emergency"，QuickAssignModal 的 v-if 自然變 false
}

onMounted(async () => {
  try {
    const rules = await deptRulesDb.getAll();
    availableDepts.value = rules.map(r => r.dept);
  } catch (e) {
    console.error("[QuickAssign] 科別載入失敗:", e);
  }
});
</script>
