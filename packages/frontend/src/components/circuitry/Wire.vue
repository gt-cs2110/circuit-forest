<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
import { currentSubcircuit } from "@/lib/store/circuit";
import { deselectWire, isWireSelected, selectWire } from "@/lib/store/view";
import { DragState, Wire } from "@/lib/types";
import { computed, inject, useId } from "vue";

const { wire } = defineProps<{ wire: Wire }>();
const emit = defineEmits<{
    wiredrag: [e: MouseEvent];
}>();
const index = computed(() => {
    return currentSubcircuit.value.wires.indexOf(wire);
});
const baseId = useId();
// ~~~ DRAG HANDLERS ~~~
const dragState = inject<DragState>("dragState");
const isDragging = computed(() => dragState?.value.active && isWireSelected(index.value));
const dragTransform = computed(() => {
    if (!dragState || !isDragging.value) return "";
    const { delta } = dragState.value;
    return `translate(${delta.x * GRID_SIZE}, ${delta.y * GRID_SIZE})`;
});

function handleMouseDown(e: MouseEvent) {
    if (!isWireSelected(index.value)) emit("wiredrag", e);
}
function handleClick(e: MouseEvent) {
    if (e.button !== 0) return;
    e.stopPropagation();

    const additive = e.shiftKey || e.metaKey;

    if (additive && isWireSelected(index.value)) {
        deselectWire(index.value);
        return;
    }

    if (!isWireSelected(index.value)) {
        selectWire(index.value, additive);
    }
}
</script>

<template>
    <!-- Drag items, which move alongside a drag -->
    <g :transform="dragTransform">
        <!-- Selected outline -->
        <line
            v-if="isWireSelected(index)"
            :x1="wire.endpoints[0].x * GRID_SIZE"
            :y1="wire.endpoints[0].y * GRID_SIZE"
            :x2="wire.endpoints[1].x * GRID_SIZE"
            :y2="wire.endpoints[1].y * GRID_SIZE"
            stroke-width="5"
            stroke-linecap="round"
            stroke="#3b82f6"
        />

        <g v-if="isDragging">
            <use :href="`#${baseId}`" />
        </g>
    </g>

    <!-- The original wire -->
    <!-- On drag, it becomes translucent to indicate a move is occurring -->
    <g :class="[{ 'opacity-50': isDragging }]">
        <line
            :x1="wire.endpoints[0].x * GRID_SIZE"
            :y1="wire.endpoints[0].y * GRID_SIZE"
            :x2="wire.endpoints[1].x * GRID_SIZE"
            :y2="wire.endpoints[1].y * GRID_SIZE"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            :id="baseId"
            :color="
                wire.value.includes('X')
                    ? 'rgb(255,0,0)'
                    : wire.value.includes('Z')
                      ? 'rgb(0,0,255)'
                      : wire.value.includes('1')
                        ? 'rgb(0,255,0)'
                        : '#006400'
            "
        />
    </g>

    <!-- Interaction wire -->
    <line
        :x1="wire.endpoints[0].x * GRID_SIZE"
        :y1="wire.endpoints[0].y * GRID_SIZE"
        :x2="wire.endpoints[1].x * GRID_SIZE"
        :y2="wire.endpoints[1].y * GRID_SIZE"
        stroke="transparent"
        stroke-width="6"
        stroke-linecap="round"
        @mousedown="handleMouseDown"
        @click="handleClick"
    />
</template>
