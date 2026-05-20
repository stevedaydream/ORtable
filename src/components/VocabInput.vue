<template>
  <div class="relative" ref="wrapRef">
    <input
      ref="inputRef"
      :value="modelValue"
      :placeholder="placeholder"
      :class="inputClass"
      autocomplete="off"
      @input="onInput"
      @keydown="onKeydown"
      @focus="open = true"
      @blur="onBlur"
    />
    <!-- Dropdown -->
    <div
      v-if="open && filtered.length"
      class="absolute z-50 left-0 right-0 top-full mt-0.5 bg-gray-800 border border-gray-600 rounded-lg shadow-xl overflow-hidden"
    >
      <div class="max-h-48 overflow-y-auto">
        <button
          v-for="(entry, i) in filtered" :key="i"
          type="button"
          class="w-full flex items-center gap-2 px-3 py-2 text-left hover:bg-gray-700/70 transition-colors"
          :class="cursor === i ? 'bg-gray-700/70' : ''"
          @mousedown.prevent="select(entry)"
        >
          <span v-if="entry.abbr" class="shrink-0 text-[10px] font-mono font-bold text-blue-300 bg-blue-900/50 px-1.5 py-0.5 rounded">{{ entry.abbr }}</span>
          <span class="text-xs text-gray-200 truncate">{{ entry.full }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";

export interface VocabEntry { abbr: string; full: string; }

const props = defineProps<{
  modelValue: string;
  vocab: VocabEntry[];
  placeholder?: string;
  inputClass?: string;
}>();
const emit = defineEmits<{ "update:modelValue": [v: string] }>();

const inputRef = ref<HTMLInputElement | null>(null);
const wrapRef  = ref<HTMLElement | null>(null);
const open     = ref(false);
const cursor   = ref(-1);

const filtered = computed(() => {
  const q = props.modelValue.trim().toLowerCase();
  if (!q) return props.vocab.slice(0, 12);
  return props.vocab.filter(e =>
    e.abbr.toLowerCase().startsWith(q) ||
    e.full.toLowerCase().includes(q)
  ).slice(0, 12);
});

function onInput(e: Event) {
  emit("update:modelValue", (e.target as HTMLInputElement).value);
  cursor.value = -1;
  open.value = true;
}

function onKeydown(e: KeyboardEvent) {
  if (!open.value || !filtered.value.length) return;
  if (e.key === "ArrowDown") {
    e.preventDefault();
    cursor.value = Math.min(cursor.value + 1, filtered.value.length - 1);
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    cursor.value = Math.max(cursor.value - 1, -1);
  } else if (e.key === "Enter" && cursor.value >= 0) {
    e.preventDefault();
    select(filtered.value[cursor.value]);
  } else if (e.key === "Escape") {
    open.value = false;
  }
}

function select(entry: VocabEntry) {
  emit("update:modelValue", entry.full);
  open.value = false;
  cursor.value = -1;
}

function onBlur() {
  // small delay so mousedown can fire first
  setTimeout(() => { open.value = false; }, 150);
}
</script>
