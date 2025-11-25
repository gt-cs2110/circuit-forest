<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
import { Wire } from "@/lib/types";
import { computed } from "vue";

const { wire } = defineProps<{ wire: Wire }>();

const end = computed(() => ({
    x: wire.direction === "H" ? wire.x + wire.length : wire.x,
    y: wire.direction === "V" ? wire.y + wire.length : wire.y,
}));
</script>

<template>
    <!-- displayed wire -->
    <line
        :x1="wire.x * GRID_SIZE"
        :y1="wire.y * GRID_SIZE"
        :x2="end.x * GRID_SIZE"
        :y2="end.y * GRID_SIZE"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        class="text-orange-500"
    />

    <!-- hitbox wire (it is larger) -->
    <line
        :x1="wire.x * GRID_SIZE"
        :y1="wire.y * GRID_SIZE"
        :x2="end.x * GRID_SIZE"
        :y2="end.y * GRID_SIZE"
        stroke="transparent"
        stroke-width="6"
        stroke-linecap="round"
    />
</template>
