<template>
  <Teleport to="body">
    <div v-if="show" class="fixed inset-0 z-[9990]" @mousedown.self="$emit('close')">
      <div
        class="absolute bg-gray-800 border border-gray-600 rounded-xl shadow-2xl py-1 overflow-hidden"
        :style="menuStyle"
        @mousedown.stop
      >
        <slot />
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  show: boolean;
  x: number;
  y: number;
  width?: number;
  height?: number;
}>();

defineEmits<{ close: [] }>();

const menuStyle = computed(() => {
  const GAP = 8;
  const W = props.width || 200;
  const H = props.height || 200;
  
  // Get zoom factor from CSS variable or default to 1
  const zoom = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--app-zoom')) || 1;
  
  // Mouse coordinates (clientX/Y) are viewport-relative and don't account for 'zoom'
  // When applying 'left/top' inside a zoomed element, we must divide by the zoom factor
  let left = (props.x / zoom) + (GAP / zoom);
  let top = (props.y / zoom) + (GAP / zoom);

  // Collision detection (innerWidth/Height are also viewport-relative, so divide them too)
  const viewW = window.innerWidth / zoom;
  const viewH = window.innerHeight / zoom;

  if (left + W > viewW) left = (props.x / zoom) - W - (GAP / zoom);
  if (top + H > viewH) top = (props.y / zoom) - H - (GAP / zoom);
  
  // Ensure it doesn't go off the left/top edges
  left = Math.max(GAP / zoom, left);
  top = Math.max(GAP / zoom, top);

  return {
    left: `${left}px`,
    top: `${top}px`,
    width: props.width ? `${props.width}px` : 'auto',
    minWidth: props.width ? 'none' : '160px',
  };
});
</script>
