<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
import { currentSubcircuit } from "@/lib/store/circuit";
import { deselectWire, isWireSelected, selectWire } from "@/lib/store/view";
import { Wire } from "@/lib/types";
import { computed } from "vue";

const { wire } = defineProps<{ wire: Wire }>();
const emit = defineEmits<{
    wiredrag: [e: MouseEvent];
}>();
let index = computed(() => {
    return currentSubcircuit.value.wires.indexOf(wire)
})
function handleMouseDown(e: MouseEvent) {

    if (!isWireSelected(index.value)) emit('wiredrag', e)
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
    <line v-if="isWireSelected(index)" :x1="wire.endpoints[0].x * GRID_SIZE" :y1="wire.endpoints[0].y * GRID_SIZE"
        :x2="wire.endpoints[1].x * GRID_SIZE" :y2="wire.endpoints[1].y * GRID_SIZE" stroke-width="5"
        stroke-linecap="round" stroke="#3b82f6" />

    <!-- displayed wire -->
    <line :x1="wire.endpoints[0].x * GRID_SIZE" :y1="wire.endpoints[0].y * GRID_SIZE"
        :x2="wire.endpoints[1].x * GRID_SIZE" :y2="wire.endpoints[1].y * GRID_SIZE" stroke="currentColor"
        stroke-width="2" stroke-linecap="round"
        :color="wire.value.includes('Z') ? 'rgb(255,0,0)' : (wire.value.includes('X') ? 'rgb(0,0,255)' : (wire.value.includes('1') ? 'rgb(0,255,0)' : '#006400'))" />

    <!-- hitbox wire (it is larger) -->
    <line :x1="wire.endpoints[0].x * GRID_SIZE" :y1="wire.endpoints[0].y * GRID_SIZE"
        :x2="wire.endpoints[1].x * GRID_SIZE" :y2="wire.endpoints[1].y * GRID_SIZE" stroke="transparent"
        stroke-width="6" stroke-linecap="round" @mousedown="handleMouseDown" @click="handleClick" />

</template>
