<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
import { selectComponent, deselectComponent, isSelected } from "@/lib/store/view";
import { CircuitComponent } from "@/lib/types";

import { componentMap } from ".";
import { computed } from "vue";

const props = defineProps<{ component: CircuitComponent }>();
const emit = defineEmits<{
    dragstart: [e: MouseEvent];
}>();

function handleMouseDown(e: MouseEvent) {
    if (e.button !== 0) return;
    e.stopPropagation();

    const additive = e.shiftKey || e.metaKey;

    if (additive && isSelected(props.component.frontendId)) {
        deselectComponent(props.component.frontendId);
        return;
    }

    if (!isSelected(props.component.frontendId)) {
        selectComponent(props.component.frontendId, additive);
    }
    emit("dragstart", e);
}
console.log(props.component)

const metadata = computed(() => componentMap[props.component.type]);
const dimensions = computed(() => ({
  width: props.component.bounds[1].x - props.component.bounds[0].x,
  height: props.component.bounds[1].y - props.component.bounds[0].y,
}));
const topLeft = computed(() => props.component.bounds[0]);

const ports = computed(() =>props.component.ports);
const rotation = computed(() => {
  switch (props.component.orientation) {
    case 0: return 270;//n
    case 1: return 90;//s
    case 2: return 0;//e
    default: return 180;; // w
  }
});
const rotationTransform = computed(() => {
  const cx =
    (topLeft.value.x + dimensions.value.width / 2) * GRID_SIZE;
  const cy =
    (topLeft.value.y + dimensions.value.height / 2) * GRID_SIZE;

  return `rotate(${rotation.value}, ${cx}, ${cy})`;
});
</script>

<template>
    <g
        :transform="rotationTransform"
        @mousedown="handleMouseDown"
    >
    <g
        :transform="`translate(${topLeft.x * GRID_SIZE}, ${topLeft.y * GRID_SIZE})`"
    >
        <component :is="metadata.component" :component="props.component" />

       

        <rect
            v-if="isSelected(props.component.frontendId)"
            class="pointer-events-none outline outline-offset-1 outline-blue-500"
            :width="dimensions.width * GRID_SIZE"
            :height="dimensions.height * GRID_SIZE"
            fill="transparent"
        ></rect>
        
    </g>
     <!-- transparent stroke enlarges hitbox -->
        
        </g>
        <circle
            v-for="(point,index) in ports"
            :key="`${index}`"
            :cx="point.x  * GRID_SIZE"
      :cy="(point.y) * GRID_SIZE"
            r="2"
            fill="currentColor"
            stroke="transparent"
            stroke-width="4"
            class="rounded-full text-orange-500 outline-orange-500 hover:outline-2"
        />
</template>
