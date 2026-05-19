import { ref } from "vue";

const isDragging = ref(false);
const dragType   = ref<"staff" | "task" | "assignment" | "shift" | null>(null);

export function useDragState() {
  return { isDragging, dragType };
}
