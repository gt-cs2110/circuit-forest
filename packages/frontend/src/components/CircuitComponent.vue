<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
import { componentDrag, selectedComponentId } from "@/lib/store";
import { CircuitComponent } from "@/lib/types";

import { componentMap } from "./circuitry";
import { computed } from "vue";

const props = defineProps<{ component: CircuitComponent }>();

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
const dimensions = computed(() => metadata.value.getDimensions(props.component));
</script>

<template>
    <g
        class="cursor-pointer"
        :transform="`translate(${props.component.x * GRID_SIZE}, ${props.component.y * GRID_SIZE})`"
        @mousedown="handleMouseDown"
    >
        <component :is="metadata.component" />

        <rect
            v-if="selectedComponentId === props.component.id"
            class="pointer-events-none outline-2 outline-offset-2 outline-blue-500"
            :width="dimensions.width * GRID_SIZE"
            :height="dimensions.height * GRID_SIZE"
            fill="transparent"
        ></rect>
    </g>
</template>
