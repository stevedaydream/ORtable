<template>
  <div class="fixed inset-0 bg-black/70 z-50 flex items-center justify-center p-4" @mousedown.self="$emit('close')">
    <div class="bg-gray-800 rounded-xl shadow-2xl w-full max-w-lg flex flex-col max-h-[90vh]">

      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-3 border-b border-gray-700 shrink-0">
        <h2 class="text-sm font-bold text-white">
          <i class="fa-solid fa-plus-circle text-blue-400 mr-2"></i>新增急診病患
        </h2>
        <button class="text-gray-400 hover:text-white" @click="$emit('close')">
          <i class="fa-solid fa-xmark"></i>
        </button>
      </div>

      <!-- Body -->
      <div class="flex-1 overflow-y-auto px-5 py-4 space-y-3">

        <!-- 緊急程度 -->
        <div class="flex gap-1.5">
          <button
            v-for="u in URGENCY_OPTIONS" :key="u.value"
            type="button"
            class="flex-1 py-1.5 rounded text-xs font-bold border-2 transition-all"
            :class="form.urgency === u.value ? u.activeClass : u.inactiveClass"
            @click="form.urgency = u.value"
          >{{ u.label }}</button>
        </div>

        <!-- 主刀醫師 + 進刀時間 -->
        <div class="flex gap-3">
          <div class="flex-1">
            <label class="form-label">主刀醫師</label>
            <input
              v-model="form.surgeon"
              list="surgeon-list"
              class="form-input"
              placeholder="醫師姓名"
              autocomplete="off"
            />
            <datalist id="surgeon-list">
              <option v-for="name in surgeonOptions" :key="name" :value="name" />
            </datalist>
          </div>
          <div class="w-28">
            <label class="form-label">進刀時間</label>
            <input v-model="timeInput" type="time" class="form-input [color-scheme:dark]" />
          </div>
        </div>

        <!-- 診斷 + 術式 -->
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="form-label">診斷</label>
            <VocabInput v-model="form.diagnosis" :vocab="diagVocab" input-class="form-input" placeholder="腹部創傷" />
          </div>
          <div>
            <label class="form-label">術式</label>
            <VocabInput v-model="form.procedure" :vocab="procVocab" input-class="form-input" placeholder="急診剖腹探查" />
          </div>
        </div>

        <!-- 病患姓名 + 病歷號 -->
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="form-label">病患姓名</label>
            <input v-model="form.patient_name" class="form-input" placeholder="王小明" />
          </div>
          <div>
            <label class="form-label">病歷號</label>
            <input v-model="form.chart_no" class="form-input" placeholder="A123456" />
          </div>
        </div>

        <!-- 科別 + 預計刀房 + 預估時長 -->
        <div class="grid grid-cols-3 gap-2">
          <div>
            <label class="form-label">科別</label>
            <input v-model="form.dept" class="form-input" placeholder="外科" />
          </div>
          <div>
            <label class="form-label">預計刀房</label>
            <input v-model="form.expected_room" list="room-datalist" class="form-input" placeholder="OR1" />
            <datalist id="room-datalist">
              <option v-for="r in roomsStore.rooms" :key="r.id" :value="r.name" />
            </datalist>
          </div>
          <div>
            <label class="form-label">預估時長(分)</label>
            <input v-model.number="form.est_time_mins" type="number" min="1" class="form-input" />
          </div>
        </div>

        <!-- 自費項目 -->
        <div class="border border-gray-700 rounded-lg p-3">
          <label class="flex items-center gap-2 cursor-pointer">
            <input type="checkbox" v-model="hasSelfPay" class="rounded accent-yellow-500" />
            <span class="text-sm text-gray-300 font-medium">自費項目</span>
            <span v-if="selectedSelfPay.length" class="text-xs text-yellow-400">
              已選 {{ selectedSelfPay.length }} 項
            </span>
          </label>

          <div v-if="hasSelfPay" class="mt-2 space-y-2">
            <input
              v-model="selfPaySearch"
              class="form-input text-xs"
              placeholder="搜尋自費項目…"
            />
            <div class="max-h-36 overflow-y-auto space-y-0.5">
              <label
                v-for="item in filteredSelfPayItems" :key="item.id"
                class="flex items-center gap-2 px-2 py-1 rounded cursor-pointer hover:bg-gray-700/50 transition-colors"
              >
                <input
                  type="checkbox"
                  :value="item"
                  v-model="selectedSelfPay"
                  class="accent-yellow-500 shrink-0"
                />
                <span class="text-xs text-gray-200 flex-1">{{ item.name }}</span>
                <span v-if="item.price > 0" class="text-[10px] text-yellow-400 font-mono shrink-0">
                  ${{ item.price.toLocaleString() }}
                </span>
              </label>
              <div v-if="filteredSelfPayItems.length === 0" class="text-xs text-gray-600 py-2 text-center">
                無符合項目
              </div>
            </div>
            <!-- Selected chips -->
            <div v-if="selectedSelfPay.length" class="flex flex-wrap gap-1 pt-1 border-t border-gray-700/60">
              <span
                v-for="item in selectedSelfPay" :key="item.id"
                class="flex items-center gap-1 text-[10px] bg-yellow-900/50 text-yellow-300 border border-yellow-700/50 px-1.5 py-0.5 rounded"
              >
                {{ item.name }}
                <button class="hover:text-white" @click="removeSelfPay(item)">
                  <i class="fa-solid fa-xmark text-[9px]"></i>
                </button>
              </span>
            </div>
          </div>
        </div>

        <!-- 醫師備注 -->
        <div>
          <label class="form-label">醫師備注</label>
          <textarea v-model="form.vs_note" rows="2" class="form-input resize-none" placeholder="備注事項…" />
        </div>

      </div>

      <!-- Footer -->
      <div class="flex items-center justify-end gap-3 px-5 py-3 border-t border-gray-700 shrink-0">
        <button class="btn-secondary" @click="$emit('close')">取消</button>
        <button
          class="btn-primary"
          :disabled="saving"
          @click="submit"
        >
          <i v-if="saving" class="fa-solid fa-spinner animate-spin mr-1.5"></i>
          新增病患
        </button>
      </div>

    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from "vue";
import { useTasksStore } from "../stores/tasks";
import { useStaffStore } from "../stores/staff";
import { useRoomsStore } from "../stores/rooms";
import { useSelfPayDb, useSettings } from "../composables/useDatabase";
import type { SurgeryTask, UrgencyLevel, SelfPayItem } from "../types";
import VocabInput from "./VocabInput.vue";
import type { VocabEntry } from "./VocabInput.vue";

interface QuickAssignPrefill {
  expected_room?: string;
  urgency?: UrgencyLevel;
  dept?: string;
  anesthesia?: string;
  est_time_mins?: number;
}

const props = defineProps<{ prefill?: QuickAssignPrefill }>();
const emit = defineEmits<{ close: []; saved: [task: SurgeryTask] }>();

const tasksStore  = useTasksStore();
const staffStore  = useStaffStore();
const roomsStore  = useRoomsStore();
const selfPayDb   = useSelfPayDb();
const { getDiagnosisVocab, getProcedureVocab } = useSettings();

const saving          = ref(false);
const diagVocab       = ref<VocabEntry[]>([]);
const procVocab       = ref<VocabEntry[]>([]);

function nowTimeStr(): string {
  const d = new Date();
  return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
}
const timeInput = ref(props.prefill?.expected_room ? nowTimeStr() : "");
const selfPaySearch   = ref("");
const hasSelfPay      = ref(false);
const selfPayItems    = ref<SelfPayItem[]>([]);
const selectedSelfPay = ref<SelfPayItem[]>([]);

const URGENCY_OPTIONS = [
  { value: "Trauma", label: "Trauma",  activeClass: "bg-red-700    border-red-500    text-white", inactiveClass: "border-red-800    text-red-500    hover:bg-red-900/30" },
  { value: "Level1", label: "Level 1", activeClass: "bg-orange-700 border-orange-500 text-white", inactiveClass: "border-orange-800 text-orange-500 hover:bg-orange-900/30" },
  { value: "Level2", label: "Level 2", activeClass: "bg-yellow-700 border-yellow-500 text-white", inactiveClass: "border-yellow-800 text-yellow-500 hover:bg-yellow-900/30" },
  { value: "Level3", label: "Level 3", activeClass: "bg-blue-700   border-blue-500   text-white", inactiveClass: "border-blue-800   text-blue-500   hover:bg-blue-900/30" },
  { value: "Normal", label: "Normal",  activeClass: "bg-gray-600   border-gray-400   text-white", inactiveClass: "border-gray-600   text-gray-500   hover:bg-gray-700/40" },
] as const;

const form = reactive<Omit<SurgeryTask, "id" | "created_at">>({
  seq_no: 0,
  patient_name: "",
  gender: "",
  age: 0,
  chart_no: "",
  bed_no: "",
  dept: props.prefill?.dept || "",
  diagnosis: "",
  body_part: "",
  procedure: "",
  anesthesia: props.prefill?.anesthesia || "",
  surgeon: "",
  vs_note: "",
  expected_room: props.prefill?.expected_room || "",
  urgency: props.prefill?.urgency || "Trauma",
  scheduled_at: null,
  est_time_mins: props.prefill?.est_time_mins || 60,
  status: props.prefill?.expected_room ? "scheduled" : "waiting",
  linked_task_id: null,
});

const surgeonOptions = computed(() =>
  staffStore.staffList
    .filter(s => s.staff_category === "vs" || s.staff_category === "r")
    .map(s => s.name)
);

const filteredSelfPayItems = computed(() => {
  const q = selfPaySearch.value.trim().toLowerCase();
  return q ? selfPayItems.value.filter(i => i.name.toLowerCase().includes(q)) : selfPayItems.value;
});

function removeSelfPay(item: SelfPayItem) {
  selectedSelfPay.value = selectedSelfPay.value.filter(i => i.id !== item.id);
}

onMounted(async () => {
  await Promise.all([staffStore.load(), roomsStore.load()]);
  selfPayItems.value = await selfPayDb.getAll();
  try {
    const [dRaw, pRaw] = await Promise.all([getDiagnosisVocab(), getProcedureVocab()]);
    diagVocab.value = JSON.parse(dRaw || "[]");
    procVocab.value = JSON.parse(pRaw || "[]");
  } catch { /* vocab optional */ }
});

async function submit() {
  saving.value = true;
  try {
    let scheduled: number | null = null;
    if (timeInput.value) {
      const [h, m] = timeInput.value.split(":").map(Number);
      const d = new Date();
      d.setHours(h, m, 0, 0);
      scheduled = Math.floor(d.getTime() / 1000);
    }

    let vsNote = form.vs_note.trim();
    if (selectedSelfPay.value.length > 0) {
      const selfPayStr = selectedSelfPay.value
        .map(i => (i.price > 0 ? `${i.name}($${i.price})` : i.name))
        .join("、");
      vsNote = vsNote ? `${vsNote} [自費: ${selfPayStr}]` : `[自費: ${selfPayStr}]`;
    }

    const task = await tasksStore.add({
      ...form,
      id: 0,
      created_at: Math.floor(Date.now() / 1000),
      scheduled_at: scheduled,
      vs_note: vsNote,
    });
    emit("saved", task);
    emit("close");
  } finally {
    saving.value = false;
  }
}
</script>
