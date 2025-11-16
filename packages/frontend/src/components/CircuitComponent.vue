<script setup lang="ts">
import { onUnmounted, reactive, ref } from "vue";
import { GRID_SIZE } from "../lib/consts";
import { components, scale, selectedComponentId, type CircuitComponent } from "../lib/store";

const props = defineProps<{ component: CircuitComponent }>();

const isDragging = ref(false);
const dragStartMouse = reactive({ x: 0, y: 0 });
const dragStartPosition = reactive({ x: 0, y: 0 });

function handleMouseDown(e: MouseEvent) {
    if (e.button !== 0) return;

    e.stopPropagation();

    selectedComponentId.value = props.component.id;

    isDragging.value = true;
    dragStartMouse.x = e.clientX;
    dragStartMouse.y = e.clientY;
    dragStartPosition.x = props.component.x;
    dragStartPosition.y = props.component.y;

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
}

function handleMouseMove(e: MouseEvent) {
    if (!isDragging.value) return;

    const deltaX = Math.round((e.clientX - dragStartMouse.x) / GRID_SIZE / scale.value);
    const newX = Math.max(deltaX + dragStartPosition.x, 0);
    const deltaY = Math.round((e.clientY - dragStartMouse.y) / GRID_SIZE / scale.value);
    const newY = Math.max(deltaY + dragStartPosition.y, 0);

    components.get(props.component.id).x = newX;
    components.get(props.component.id).y = newY;
}

function handleMouseUp() {
    isDragging.value = false;

    document.removeEventListener("mousemove", handleMouseMove);
    document.removeEventListener("mouseup", handleMouseUp);
}

onUnmounted(() => {
    document.removeEventListener("mousemove", handleMouseMove);
    document.removeEventListener("mouseup", handleMouseUp);
});
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
