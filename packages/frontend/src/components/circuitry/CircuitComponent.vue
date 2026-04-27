<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
import { componentDrag, selectedComponentId } from "@/lib/store/circuit";
import { CircuitComponent } from "@/lib/types";
import {  ComponentState } from "circuitsim-glue";
import { componentMap } from ".";
import { computed } from "vue";

const props = defineProps<{ component: CircuitComponent, state: ComponentState }>();

function handleMouseDown(e: MouseEvent) {
    if (e.button !== 0) return;

    e.stopPropagation();

    selectedComponentId.value = props.component.id;
    componentDrag.componentId = props.component.id;
    componentDrag.isDragging = true;
    componentDrag.initialMouse.x = e.clientX;
    componentDrag.initialMouse.y = e.clientY;
    componentDrag.initialPosition.x = props.component.x;
    componentDrag.initialPosition.y = props.component.y;
}

const metadata = computed(() => componentMap[props.component.type]);
const dimensions = computed(() => ({
  width: props.state.bounds[1][0] - props.state.bounds[0][0],
  height: props.state.bounds[1][1] - props.state.bounds[0][1],
}));const topLeft = computed(() => props.state.bounds[0]);

const ports = computed(() =>props.state.portLocations);
</script>

<template>
    <g
        :transform="`translate(${topLeft[0] * GRID_SIZE}, ${topLeft[1] * GRID_SIZE})`"
        @mousedown="handleMouseDown"
    >
        <component :is="metadata.component" :component="props.component" />

        
                    <!-- :data-tooltip="port.label" -->


        <rect
            v-if="selectedComponentId === props.component.id"
            class="pointer-events-none outline outline-offset-1 outline-blue-500"
            :width="dimensions.width * GRID_SIZE"
            :height="dimensions.height * GRID_SIZE"
            fill="transparent"
        ></rect>
    </g>
    <!-- transparent stroke enlarges hitbox -->
        <circle
            v-for="(point,index) in ports"
            :key="`${index}`"
            :cx="point[0] * GRID_SIZE"
            :cy="point[1] * GRID_SIZE"
            r="2"
            fill="currentColor"
            stroke="transparent"
            stroke-width="4"
            class="rounded-full text-orange-500 outline-orange-500 hover:outline-2"
        />
</template>
