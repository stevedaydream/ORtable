<template>
  <Teleport to="body">
    <div class="fixed inset-0 bg-black/70 z-[9990] flex items-center justify-center p-4" @mousedown.self="$emit('close')">
      <div class="relative bg-gray-800 rounded-xl shadow-2xl w-full max-w-2xl flex flex-col max-h-[90vh]">

        <!-- Header -->
        <div class="flex items-center gap-3 px-5 py-3.5 border-b border-gray-700 shrink-0">
          <span class="text-xs px-2 py-0.5 rounded font-bold" :class="statusColor(task.status)">
            {{ statusLabel(task.status) }}
          </span>
          <span class="font-bold text-white text-base">{{ task.patient_name }}</span>
          <span class="text-gray-400 text-xs">{{ task.gender }} / {{ task.age }}歲</span>
          <span class="text-gray-500 text-xs font-mono">病歷：{{ task.chart_no }}</span>
          <div class="flex-1" />
          <button class="text-gray-400 hover:text-white" @click="$emit('close')">
            <i class="fa-solid fa-xmark"></i>
          </button>
        </div>

        <!-- Status bar -->
        <div class="flex items-center gap-1.5 px-5 py-2.5 border-b border-gray-700/60 shrink-0 bg-gray-850">
          <span class="text-[10px] text-gray-500 mr-1 shrink-0">狀態切換：</span>
          <button
            v-for="s in STATUS_FLOW" :key="s.value"
            class="text-[11px] px-2.5 py-1 rounded font-medium transition-all border"
            :class="task.status === s.value ? s.activeClass : 'border-gray-700 text-gray-500 hover:border-gray-500 hover:text-gray-300'"
            @click="changeStatus(s.value)"
          >{{ s.label }}</button>
        </div>

        <!-- Body -->
        <div class="flex-1 overflow-y-auto px-5 py-4">

          <!-- View mode -->
          <template v-if="!editMode">
            <div class="grid grid-cols-2 gap-x-10 gap-y-2 text-xs mb-2">
              <div class="col-span-2 text-[10px] font-semibold text-gray-500 uppercase tracking-widest mb-0.5">基本資訊</div>
              <div><span class="text-gray-500">床號：</span><span class="text-gray-200">{{ task.bed_no || '—' }}</span></div>
              <div><span class="text-gray-500">手術室：</span><span class="text-blue-300 font-mono">{{ task.expected_room || '—' }}</span></div>
              <div><span class="text-gray-500">手術者：</span><span class="text-gray-200">{{ task.surgeon || '—' }}</span></div>
              <div><span class="text-gray-500">麻醉：</span><span class="text-gray-200">{{ task.anesthesia || '—' }}</span></div>
              <div><span class="text-gray-500">緊急程度：</span><span :class="urgencyColor(task.urgency)">{{ urgencyLabel(task.urgency) }}</span></div>
              <div><span class="text-gray-500">預計時間：</span><span class="text-gray-200">{{ task.est_time_mins }} 分鐘</span></div>
              <div v-if="task.scheduled_at"><span class="text-gray-500">預計進線：</span><span class="text-gray-200 font-mono">{{ fmtTs(task.scheduled_at) }}</span></div>
              <div v-if="task.seq_no"><span class="text-gray-500">序號：</span><span class="text-gray-400">{{ task.seq_no }}</span></div>

              <div class="col-span-2 text-[10px] font-semibold text-gray-500 uppercase tracking-widest mb-0.5 mt-3">手術資訊</div>
              <div><span class="text-gray-500">科別：</span><span class="text-gray-200">{{ task.dept || '—' }}</span></div>
              <div><span class="text-gray-500">部位：</span><span class="text-gray-200">{{ task.body_part || '—' }}</span></div>
              <div class="col-span-2"><span class="text-gray-500">術式：</span><span class="text-gray-200">{{ task.procedure || '—' }}</span></div>
              <div class="col-span-2"><span class="text-gray-500">診斷：</span><span class="text-gray-200">{{ task.diagnosis || '—' }}</span></div>
              <div v-if="task.vs_note" class="col-span-2"><span class="text-gray-500">備注：</span><span class="text-yellow-300/80">{{ task.vs_note }}</span></div>
            </div>
          </template>

          <!-- Edit mode -->
          <template v-else>
            <div class="grid grid-cols-2 gap-3 text-xs">
              <div><label class="form-label">姓名</label><input v-model="form.patient_name" class="form-input" /></div>
              <div><label class="form-label">科別</label><input v-model="form.dept" class="form-input" /></div>
              <div><label class="form-label">手術者</label><input v-model="form.surgeon" class="form-input" /></div>
              <div><label class="form-label">麻醉</label><input v-model="form.anesthesia" class="form-input" /></div>
              <div><label class="form-label">手術室</label><input v-model="form.expected_room" class="form-input" /></div>
              <div>
                <label class="form-label">緊急程度</label>
                <select v-model="form.urgency" class="form-input">
                  <option v-for="u in URGENCY_OPTIONS" :key="u" :value="u">{{ u }}</option>
                </select>
              </div>
              <div><label class="form-label">預計時間(分)</label><input v-model.number="form.est_time_mins" type="number" min="1" class="form-input" /></div>
              <div><label class="form-label">預計進線時間</label><input v-model="form.scheduledAtStr" type="datetime-local" class="form-input [color-scheme:dark]" /></div>
              <div class="col-span-2"><label class="form-label">術式</label><input v-model="form.procedure" class="form-input" /></div>
              <div class="col-span-2"><label class="form-label">診斷</label><input v-model="form.diagnosis" class="form-input" /></div>
              <div class="col-span-2"><label class="form-label">備注</label><input v-model="form.vs_note" class="form-input" /></div>
            </div>
          </template>
        </div>

        <!-- Footer -->
        <div class="flex items-center justify-between px-5 py-3.5 border-t border-gray-700 shrink-0">
          <div class="flex gap-2">
            <template v-if="!editMode">
              <button class="btn-secondary text-xs" @click="startEdit"><i class="fa-solid fa-pen mr-1.5"></i>編輯</button>
              <button class="text-xs px-3 py-1.5 rounded bg-red-900/40 border border-red-800 text-red-300 hover:bg-red-800/60 transition-colors" @click="showDeleteConfirm = true">
                <i class="fa-solid fa-trash mr-1.5"></i>刪除
              </button>
            </template>
            <template v-else>
              <button class="btn-primary text-xs" :disabled="saving" @click="saveEdit">
                <i v-if="saving" class="fa-solid fa-spinner animate-spin mr-1.5"></i>儲存
              </button>
              <button class="btn-secondary text-xs" @click="cancelEdit">取消</button>
            </template>
          </div>
          <button class="btn-secondary text-xs" @click="$emit('close')">關閉</button>
        </div>

        <!-- Delete confirm overlay -->
        <div v-if="showDeleteConfirm" class="absolute inset-0 bg-gray-900/90 rounded-xl flex items-center justify-center z-10">
          <div class="bg-gray-800 rounded-xl border border-red-700/60 p-6 max-w-xs w-full text-center">
            <i class="fa-solid fa-triangle-exclamation text-2xl text-red-400 mb-3"></i>
            <div class="text-sm font-semibold text-white mb-1">確定刪除？</div>
            <div class="text-xs text-gray-400 mb-4">「{{ task.patient_name }}」將從清單中永久移除</div>
            <div class="flex gap-2 justify-center">
              <button class="btn-secondary text-xs" @click="showDeleteConfirm = false">取消</button>
              <button class="text-xs px-4 py-1.5 rounded bg-red-700 hover:bg-red-600 text-white font-bold" @click="doDelete">確認刪除</button>
            </div>
          </div>
        </div>

      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, reactive } from "vue";
import type { SurgeryTask, UrgencyLevel, TaskStatus } from "../types";
import { TASK_STATUS_LABELS, TASK_STATUS_COLORS } from "../types";
import { useTasksStore } from "../stores/tasks";

const props = defineProps<{ task: SurgeryTask }>();
const emit = defineEmits<{ close: []; deleted: [id: number] }>();

const tasksStore = useTasksStore();
const editMode = ref(false);
const saving = ref(false);
const showDeleteConfirm = ref(false);

// ── Status ────────────────────────────────────────────────────────────────────
const STATUS_FLOW = [
  { value: "waiting",    label: "待排",    activeClass: "bg-gray-700 border-gray-500 text-gray-200" },
  { value: "scheduled",  label: "已排程",  activeClass: "bg-blue-800 border-blue-500 text-blue-100" },
  { value: "called",     label: "已叫刀",  activeClass: "bg-amber-800 border-amber-500 text-amber-100" },
  { value: "in_surgery", label: "手術中",  activeClass: "bg-green-800 border-green-500 text-green-100" },
  { value: "recovery",   label: "恢復室",  activeClass: "bg-purple-800 border-purple-500 text-purple-100" },
] as const;

function statusLabel(s: string) { return TASK_STATUS_LABELS[s as TaskStatus] ?? s; }
function statusColor(s: string) { return TASK_STATUS_COLORS[s as TaskStatus] ?? "bg-gray-700 text-gray-300"; }

async function changeStatus(newStatus: string) {
  if (props.task.status === newStatus) return;
  try { await tasksStore.edit({ ...props.task, status: newStatus }); }
  catch (e) { console.error("[PatientDetailModal] 狀態更新失敗:", e); }
}

// ── Urgency ───────────────────────────────────────────────────────────────────
const URGENCY_OPTIONS: UrgencyLevel[] = ["Trauma", "Level1", "Level2", "Level3", "Normal"];
function urgencyLabel(u: UrgencyLevel) {
  return { Trauma: "Trauma", Level1: "Level 1", Level2: "Level 2", Level3: "Level 3", Normal: "一般" }[u] ?? u;
}
function urgencyColor(u: UrgencyLevel) {
  return { Trauma: "text-red-400", Level1: "text-orange-400", Level2: "text-yellow-400", Level3: "text-blue-400", Normal: "text-gray-400" }[u] ?? "";
}
function fmtTs(ts: number | null): string {
  if (!ts) return "—";
  const d = new Date(ts * 1000);
  return `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,"0")}-${String(d.getDate()).padStart(2,"0")} ${String(d.getHours()).padStart(2,"0")}:${String(d.getMinutes()).padStart(2,"0")}`;
}

// ── Edit ──────────────────────────────────────────────────────────────────────
const form = reactive({
  patient_name: "", dept: "", procedure: "", diagnosis: "",
  expected_room: "", anesthesia: "", surgeon: "",
  est_time_mins: 60, urgency: "Normal" as UrgencyLevel, vs_note: "", scheduledAtStr: "",
});

function tsToLocalDatetime(ts: number | null): string {
  if (!ts) return "";
  const d = new Date(ts * 1000);
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth()+1)}-${p(d.getDate())}T${p(d.getHours())}:${p(d.getMinutes())}`;
}

function startEdit() {
  Object.assign(form, {
    patient_name: props.task.patient_name, dept: props.task.dept,
    procedure: props.task.procedure, diagnosis: props.task.diagnosis,
    expected_room: props.task.expected_room, anesthesia: props.task.anesthesia,
    surgeon: props.task.surgeon, est_time_mins: props.task.est_time_mins,
    urgency: props.task.urgency, vs_note: props.task.vs_note,
    scheduledAtStr: tsToLocalDatetime(props.task.scheduled_at),
  });
  editMode.value = true;
}
function cancelEdit() { editMode.value = false; }

async function saveEdit() {
  saving.value = true;
  try {
    const scheduled_at = form.scheduledAtStr
      ? Math.floor(new Date(form.scheduledAtStr).getTime() / 1000)
      : props.task.scheduled_at;
    await tasksStore.edit({
      ...props.task, patient_name: form.patient_name, dept: form.dept,
      procedure: form.procedure, diagnosis: form.diagnosis, expected_room: form.expected_room,
      anesthesia: form.anesthesia, surgeon: form.surgeon, est_time_mins: form.est_time_mins,
      urgency: form.urgency, vs_note: form.vs_note, scheduled_at,
    });
    editMode.value = false;
  } catch (e) {
    console.error("[PatientDetailModal] 儲存失敗:", e);
  } finally {
    saving.value = false;
  }
}

// ── Delete ────────────────────────────────────────────────────────────────────
async function doDelete() {
  try {
    await tasksStore.remove(props.task.id);
    emit("deleted", props.task.id);
    emit("close");
  } catch (e) {
    console.error("[PatientDetailModal] 刪除失敗:", e);
  }
}
</script>
