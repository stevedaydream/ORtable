<template>
  <div class="fixed inset-0 bg-black/70 z-50 flex items-center justify-center p-4" @mousedown.self="$emit('close')">
    <div class="bg-gray-800 rounded-xl shadow-2xl w-full max-w-lg flex flex-col max-h-[90vh]">

      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-4 border-b border-gray-700 shrink-0">
        <h2 class="text-base font-bold text-white">
          <i class="fa-solid fa-plus-circle text-blue-400 mr-2"></i>新增急診病患
        </h2>
        <button class="text-gray-400 hover:text-white" @click="$emit('close')">
          <i class="fa-solid fa-xmark"></i>
        </button>
      </div>

      <!-- Body -->
      <div class="flex-1 overflow-y-auto px-5 py-4 space-y-4">

        <!-- 緊急程度 -->
        <div>
          <label class="form-label">緊急程度 <span class="text-red-400">*</span></label>
          <div class="flex gap-2 mt-1">
            <button
              v-for="u in URGENCY_OPTIONS" :key="u.value"
              type="button"
              class="flex-1 py-1.5 rounded text-xs font-bold border-2 transition-all"
              :class="form.urgency === u.value ? u.activeClass : u.inactiveClass"
              @click="form.urgency = u.value"
            >{{ u.label }}</button>
          </div>
        </div>

        <!-- 姓名 + 病歷號 -->
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="form-label">病患姓名 <span class="text-red-400">*</span></label>
            <input v-model="form.patient_name" class="form-input" placeholder="王小明" />
          </div>
          <div>
            <label class="form-label">病歷號</label>
            <input v-model="form.chart_no" class="form-input" placeholder="A123456" />
          </div>
        </div>

        <!-- 科別 + 預計刀房 -->
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="form-label">科別</label>
            <input v-model="form.dept" class="form-input" placeholder="一般外科" />
          </div>
          <div>
            <label class="form-label">預計手術室</label>
            <input v-model="form.expected_room" class="form-input" placeholder="OR1" />
          </div>
        </div>

        <!-- 手術名稱 -->
        <div>
          <label class="form-label">手術名稱</label>
          <input v-model="form.procedure" class="form-input" placeholder="急診剖腹探查" />
        </div>

        <!-- 診斷 -->
        <div>
          <label class="form-label">診斷</label>
          <input v-model="form.diagnosis" class="form-input" placeholder="腹部創傷" />
        </div>

        <!-- 預計時間 + 預排時間 -->
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="form-label">預計手術時間（分）</label>
            <input v-model.number="form.est_time_mins" type="number" min="1" max="999" class="form-input" />
          </div>
          <div>
            <label class="form-label">預排時間（選填）</label>
            <input v-model="scheduledAtInput" type="datetime-local" class="form-input" />
          </div>
        </div>

        <!-- 醫師備注 -->
        <div>
          <label class="form-label">醫師備注</label>
          <textarea v-model="form.vs_note" rows="2" class="form-input resize-none" placeholder="備注事項..." />
        </div>

      </div>

      <!-- Footer -->
      <div class="flex items-center justify-end gap-3 px-5 py-4 border-t border-gray-700 shrink-0">
        <button class="btn-secondary" @click="$emit('close')">取消</button>
        <button
          class="btn-primary"
          :disabled="!form.patient_name.trim() || saving"
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
import { ref, reactive } from "vue";
import { useTasksStore } from "../stores/tasks";
import type { SurgeryTask } from "../types";

const emit = defineEmits<{ close: []; saved: [task: SurgeryTask] }>();

const tasksStore = useTasksStore();
const saving = ref(false);
const scheduledAtInput = ref("");

const URGENCY_OPTIONS = [
  { value: "Trauma",  label: "Trauma",  activeClass: "bg-red-700    border-red-500    text-white", inactiveClass: "border-red-800    text-red-500    hover:bg-red-900/30" },
  { value: "Level1",  label: "Level 1", activeClass: "bg-orange-700 border-orange-500 text-white", inactiveClass: "border-orange-800 text-orange-500 hover:bg-orange-900/30" },
  { value: "Level2",  label: "Level 2", activeClass: "bg-yellow-700 border-yellow-500 text-white", inactiveClass: "border-yellow-800 text-yellow-500 hover:bg-yellow-900/30" },
  { value: "Level3",  label: "Level 3", activeClass: "bg-blue-700   border-blue-500   text-white", inactiveClass: "border-blue-800   text-blue-500   hover:bg-blue-900/30" },
  { value: "Normal",  label: "Normal",  activeClass: "bg-gray-600   border-gray-400   text-white", inactiveClass: "border-gray-600   text-gray-500   hover:bg-gray-700/40" },
] as const;

const form = reactive<Omit<SurgeryTask, "id" | "created_at">>({
  patient_name: "",
  chart_no: "",
  procedure: "",
  diagnosis: "",
  vs_note: "",
  dept: "",
  expected_room: "",
  urgency: "Trauma",
  scheduled_at: null,
  est_time_mins: 60,
  status: "waiting",
});

async function submit() {
  if (!form.patient_name.trim()) return;
  saving.value = true;
  try {
    const scheduled = scheduledAtInput.value
      ? Math.floor(new Date(scheduledAtInput.value).getTime() / 1000)
      : null;

    const task = await tasksStore.add({
      ...form,
      id: 0,
      created_at: Math.floor(Date.now() / 1000),
      scheduled_at: scheduled,
    });
    emit("saved", task);
    emit("close");
  } finally {
    saving.value = false;
  }
}
</script>
