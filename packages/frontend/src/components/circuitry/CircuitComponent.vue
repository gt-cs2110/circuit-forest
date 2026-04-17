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

    if (additive && isSelected(props.component.id)) {
        deselectComponent(props.component.id);
        return;
    }

    if (!isSelected(props.component.id)) {
        selectComponent(props.component.id, additive);
    }
    emit("dragstart", e);
}

const metadata = computed(() => componentMap[props.component.type]);
const dimensions = computed(() => metadata.value.getDimensions(props.component));
const ports = computed(() => metadata.value.getPorts(props.component));
</script>

<template>
    <g
        :transform="`translate(${props.component.x * GRID_SIZE}, ${props.component.y * GRID_SIZE})`"
        @mousedown="handleMouseDown"
    >
        <component :is="metadata.component" :component="props.component" />

        <!-- transparent stroke enlarges hitbox -->
        <circle
            v-for="(port, i) in ports"
            :key="i"
            :cx="port.x * GRID_SIZE"
            :cy="port.y * GRID_SIZE"
            r="2"
            fill="currentColor"
            stroke="transparent"
            stroke-width="4"
            class="rounded-full text-orange-500 outline-orange-500 hover:outline-2"
            :data-tooltip="port.label"
        />

        <rect
            v-if="isSelected(props.component.id)"
            class="pointer-events-none outline outline-offset-1 outline-blue-500"
            :width="dimensions.width * GRID_SIZE"
            :height="dimensions.height * GRID_SIZE"
            fill="transparent"
        ></rect>
    </g>
</template>
