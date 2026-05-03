<template>
  <div
    class="rounded-lg border select-none relative transition-all"
    :class="[
      mode === 'queue' ? queueClasses : timelineClasses,
      isDraggingThis ? 'opacity-40 scale-95' : 'opacity-100',
      (mode === 'queue' && scoredTask?.is_overdue) ? 'animate-pulse ring-1 ring-red-500' : ''
    ]"
    :draggable="true"
    @dragstart="onDragStart"
    @dragend="onDragEnd"
    @contextmenu.prevent="$emit('contextmenu', $event)"
    @click="$emit('click', $event)"
  >
    <!-- Queue Mode Content -->
    <template v-if="mode === 'queue'">
      <div class="p-2.5">
        <!-- Row 1: Urgency + Name + Gender/Age + Overdue -->
        <div class="flex items-center gap-1.5 mb-1">
          <span class="text-[9px] font-bold px-1.5 py-0.5 rounded shrink-0" :class="urgencyBadge(task.urgency)">
            {{ urgencyLabel(task.urgency) }}
          </span>
          <span class="text-sm font-bold text-gray-100 truncate">{{ task.patient_name }}</span>
          <span class="text-[10px] text-gray-400 shrink-0">({{ task.gender }}/{{ task.age }})</span>
          <span v-if="scoredTask?.is_overdue" class="ml-auto shrink-0 text-[9px] text-red-400 font-bold">⚠逾時</span>
        </div>

        <!-- Row 2: Chart No + Bed No + Room -->
        <div class="flex items-center gap-2 text-[10px] mb-1.5">
          <span class="text-blue-400 font-mono">{{ task.chart_no }}</span>
          <span class="text-gray-500">·</span>
          <span class="text-orange-300 font-medium">{{ task.bed_no || '無床號' }}</span>
          <div class="flex-1" />
          <span v-if="task.expected_room" class="bg-blue-900/40 text-blue-300 px-1 rounded border border-blue-800/50">
            <i class="fa-solid fa-door-open text-[8px] mr-1"></i>{{ task.expected_room }}
          </span>
        </div>

        <!-- Row 3: Procedure (Main Focus) -->
        <div class="text-[11px] font-semibold text-gray-200 mb-1 line-clamp-1 border-l-2 border-gray-600 pl-2">
          {{ task.procedure || '（未填手術名稱）' }}
        </div>

        <!-- Row 4: Diagnosis & Surgeon -->
        <div class="flex items-center justify-between text-[10px] text-gray-400">
          <span class="truncate max-w-[65%]" :title="task.diagnosis">
            <i class="fa-solid fa-stethoscope text-[9px] mr-1 text-gray-600"></i>{{ task.diagnosis }}
          </span>
          <span class="shrink-0 text-gray-300">
            <i class="fa-solid fa-user-md text-[9px] mr-1 text-gray-500"></i>{{ task.surgeon }}
          </span>
        </div>

        <!-- Row 5: vs_note (prominent) -->
        <div v-if="task.vs_note" class="mt-1.5 flex items-start gap-1.5 bg-yellow-900/30 border border-yellow-700/50 rounded px-2 py-1">
          <i class="fa-solid fa-triangle-exclamation text-yellow-400 text-[9px] mt-0.5 shrink-0"></i>
          <span class="text-[10px] text-yellow-300 leading-snug break-all">{{ task.vs_note }}</span>
        </div>

        <!-- Progress/Deadline bar -->
        <div v-if="scoredTask?.deadline_elapsed_pct !== null" class="mt-2 h-1 bg-gray-700 rounded-full overflow-hidden">
          <div
            class="h-full rounded-full transition-all"
            :class="scoredTask?.is_overdue ? 'bg-red-500' : (scoredTask?.deadline_elapsed_pct || 0) > 75 ? 'bg-orange-500' : 'bg-blue-500'"
            :style="{ width: Math.min(scoredTask?.deadline_elapsed_pct || 0, 100) + '%' }"
          />
        </div>
      </div>
    </template>

    <!-- Timeline Mode Content -->
    <template v-else>
      <div class="p-1.5 h-full flex flex-col overflow-hidden leading-tight">
        <div class="flex items-center justify-between mb-0.5 shrink-0">
          <div class="flex items-center gap-1 truncate">
            <span class="text-[10px] font-bold text-gray-100">{{ task.patient_name }}</span>
            <span class="text-[9px] text-gray-400">({{ task.bed_no }})</span>
          </div>
          <div class="flex items-center gap-0.5 shrink-0">
            <span v-if="task.linked_task_id" class="text-[8px] px-1 rounded bg-teal-800 text-teal-200" title="兩科合刀">🔗</span>
            <span v-if="task.status === 'called'"     class="text-[8px] font-bold px-1 rounded bg-amber-700 text-amber-100">叫</span>
            <span v-if="task.status === 'in_surgery'" class="text-[8px] font-bold px-1 rounded bg-green-700 text-green-100 animate-pulse">進</span>
            <span v-if="task.status === 'recovery'"   class="text-[8px] font-bold px-1 rounded bg-purple-700 text-purple-100">完</span>
            <span class="text-[8px] font-bold px-1 rounded scale-90" :class="urgencyBadge(task.urgency)">
              {{ urgencyLabel(task.urgency) }}
            </span>
          </div>
        </div>
        <div class="text-[9px] text-blue-200 truncate font-medium shrink-0">{{ task.procedure }}</div>
        <div class="text-[8px] text-gray-400 truncate mt-0.5 shrink-0">
          <i class="fa-solid fa-user-md text-[7px] mr-0.5"></i>{{ task.surgeon }}
        </div>
        <div v-if="task.vs_note" class="text-[8px] text-yellow-500/80 truncate mt-auto italic">
          <i class="fa-solid fa-comment-medical text-[7px] mr-0.5"></i>{{ task.vs_note }}
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import type { SurgeryTask, TaskWithScore, UrgencyLevel } from "../types";
import { useDragState } from "../composables/useDragState";

const props = defineProps<{
  task: SurgeryTask;
  scoredTask?: TaskWithScore;
  mode: "queue" | "timeline";
}>();

defineEmits<{
  contextmenu: [e: MouseEvent];
  click: [e: MouseEvent];
}>();

const { isDragging, dragType } = useDragState();
const isDraggingThis = ref(false);

const queueClasses = computed(() => {
  const u = props.task.urgency;
  return ({
    Trauma: "bg-red-950/40 border-red-800/50 hover:bg-red-900/40 shadow-inner",
    Level1: "bg-orange-950/40 border-orange-800/50 hover:bg-orange-900/40 shadow-inner",
    Level2: "bg-yellow-950/30 border-yellow-800/40 hover:bg-yellow-900/30",
    Level3: "bg-blue-950/30 border-blue-800/40 hover:bg-blue-900/30",
    Normal: "bg-gray-800/50 border-gray-700/50 hover:bg-gray-700/50",
  } as Record<UrgencyLevel, string>)[u] ?? "bg-gray-800/50 border-gray-700/50";
});

const timelineClasses = computed(() => {
  const u = props.task.urgency;
  return ({
    Trauma: "bg-red-900/80 border-red-500 shadow-lg z-10",
    Level1: "bg-orange-900/80 border-orange-500 shadow-md z-10",
    Level2: "bg-yellow-900/70 border-yellow-500 shadow-sm z-10",
    Level3: "bg-blue-900/70 border-blue-500 shadow-sm z-10",
    Normal: "bg-gray-800 border-gray-600 shadow-sm",
  } as Record<UrgencyLevel, string>)[u] ?? "bg-gray-800 border-gray-600";
});

function onDragStart(e: DragEvent) {
  isDragging.value = true;
  dragType.value = "task";
  isDraggingThis.value = true;
  e.dataTransfer!.effectAllowed = "copy";
  e.dataTransfer!.setData("text/plain", JSON.stringify({ type: "task", id: props.task.id }));
}

function onDragEnd() {
  isDragging.value = false;
  dragType.value = null;
  isDraggingThis.value = false;
}

function urgencyLabel(u: UrgencyLevel): string {
  return { Trauma: "Trauma", Level1: "L1", Level2: "L2", Level3: "L3", Normal: "N" }[u] ?? u;
}

function urgencyBadge(u: UrgencyLevel): string {
  return ({
    Trauma: "bg-red-600 text-white shadow-sm",   Level1: "bg-orange-600 text-white shadow-sm",
    Level2: "bg-yellow-600 text-white", Level3: "bg-blue-600 text-white",
    Normal: "bg-gray-600 text-white",
  } as Record<UrgencyLevel, string>)[u] ?? "bg-gray-600 text-white";
}
</script>
