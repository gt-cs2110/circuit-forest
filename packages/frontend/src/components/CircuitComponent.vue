<script setup lang="ts">
import { GRID_SIZE } from "../lib/consts";
import { componentDrag, selectedComponentId, type CircuitComponent } from "../lib/store";

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
</script>

<template>
    <rect
        class="cursor-pointer text-zinc-200 outline-offset-2 outline-blue-500"
        :class="{
            'z-50 outline-2': selectedComponentId === props.component.id,
        }"
        :width="2 * GRID_SIZE"
        :height="2 * GRID_SIZE"
        :x="props.component.x * GRID_SIZE"
        :y="props.component.y * GRID_SIZE"
        fill="currentColor"
        @mousedown="handleMouseDown"
    ></rect>
</template>
