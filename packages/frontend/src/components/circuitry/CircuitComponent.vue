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

const metadata = computed(() => componentMap[props.component.type]);
const dimensions = computed(() => ({
  width: props.component.bounds[1].x - props.component.bounds[0].x,
  height: props.component.bounds[1].y - props.component.bounds[0].y,
}));


const absoluteOutputPortLocation = computed(()=>{
    return {x:props.component.x*GRID_SIZE, y:props.component.y*GRID_SIZE}
})
const localOutputPortLocation = computed(()=>{
    return {
        x: metadata.value.getDimensions().width*GRID_SIZE,
        y: metadata.value.getDimensions().height*GRID_SIZE/2,
    }
}
)



const ports = computed(() =>props.component.ports);
const rotation = computed(() => {
  switch (props.component.orientation) {
    case 0: return 270;//n
    case 1: return 90;//s
    case 2: return 0;//e
    default: return 180;; // w
  }
});

const transform = computed(() => {
  const world = absoluteOutputPortLocation.value;
  const local = localOutputPortLocation.value;
  const angle = rotation.value;

  return `
    translate(${world.x}, ${world.y})
    rotate(${angle})
    translate(${-local.x}, ${-local.y})
  `;
});
</script>

<template>
    ///Translation
    <g 
    :transform="transform"
     @mousedown="handleMouseDown">
    
        <!-- <g :transform="rotate"> -->
            <component :is="metadata.component" :component="props.component" />
            <text
                v-if="props.component.type === 'probe' || props.component.type === 'Constant'"
                :x="(dimensions.width * GRID_SIZE) / 2"
                :y="(dimensions.height * GRID_SIZE) / 2"
                text-anchor="middle"
                dominant-baseline="middle"
                class="pointer-events-none fill-black text-xs"
            >{{ props.component.componentValue }}</text>
        <!-- </g> -->
          
    </g>
        <rect
    v-if="isSelected(props.component.frontendId)"
            :x="props.component.bounds[0].x * GRID_SIZE"
            :y="props.component.bounds[0].y * GRID_SIZE"
            :width="(props.component.bounds[1].x - props.component.bounds[0].x) * GRID_SIZE"
            :height="(props.component.bounds[1].y - props.component.bounds[0].y) * GRID_SIZE"
            fill="none"
            stroke="#3b82f6"
            stroke-width="2"
    />
     <!-- transparent stroke enlarges hitbox -->
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
