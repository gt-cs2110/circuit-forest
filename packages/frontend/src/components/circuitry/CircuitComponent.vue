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



const absoluteOutputPortLocation = computed(()=>{
    return {x:props.component.x*GRID_SIZE, y:props.component.y*GRID_SIZE}
})
const localOutputPortLocation = computed(()=>{
    //whicever side is longer is the one hte output port is on
    let longer = Math.max(metadata.value.getDimensions(props.component).width, metadata.value.getDimensions(props.component).height)
    let shorter = Math.min(metadata.value.getDimensions(props.component).width, metadata.value.getDimensions(props.component).height)
    if(props.component.type == "constant"){
         shorter = Math.max(metadata.value.getDimensions(props.component).width, metadata.value.getDimensions(props.component).height)
         longer = Math.min(metadata.value.getDimensions(props.component).width, metadata.value.getDimensions(props.component).height)
    }
    
    
    return {
        x: props.component.type=="constant"?props.component.bitsize*GRID_SIZE:shorter*GRID_SIZE,
        y: longer*GRID_SIZE/2,
    }
}
)


console.log(props.component)
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
computed(()=>console.log(props.component))

</script>

<template>
    ///Translation
    <g 
    :transform="transform"
     @mousedown="handleMouseDown">
    
        <!-- <g :transform="rotate"> -->
            <component :is="metadata.component" :component="props.component" />
            
        <!-- </g> -->
          
    </g>
        <rect
    v-if="isSelected(props.component.frontendId)"
            :x="props.component.bounds[0].x * GRID_SIZE"
            :y="props.component.bounds[0].y * GRID_SIZE"
            :width="metadata.getDimensions(props.component).width * GRID_SIZE"
            :height="metadata.getDimensions(props.component).height * GRID_SIZE"
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
