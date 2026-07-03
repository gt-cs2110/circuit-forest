<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
import { Wire } from "@/lib/types";

const { wire } = defineProps<{ wire: Wire }>();
const emit = defineEmits<{
    wiredrag: [e: MouseEvent];
}>();


</script>

<template>
    <!-- displayed wire -->

    <line :x1="wire.endpoints[0].x * GRID_SIZE" :y1="wire.endpoints[0].y * GRID_SIZE"
        :x2="wire.endpoints[1].x * GRID_SIZE" :y2="wire.endpoints[1].y * GRID_SIZE" stroke="currentColor"
        stroke-width="2" stroke-linecap="round"
        :color="wire.value.includes('Z') ? 'rgb(255,0,0)' : (wire.value.includes('X') ? 'rgb(0,0,255)' : (wire.value.includes('1') ? 'rgb(0,255,0)' : '#006400'))" />

    <!-- hitbox wire (it is larger) -->
    <line :x1="wire.endpoints[0].x * GRID_SIZE" :y1="wire.endpoints[0].y * GRID_SIZE"
        :x2="wire.endpoints[1].x * GRID_SIZE" :y2="wire.endpoints[1].y * GRID_SIZE" stroke="transparent"
        stroke-width="6" stroke-linecap="round" @mousedown="(e) => emit('wiredrag', e)" />



</template>
