<script setup lang="ts">
import { BUBBLE_SIZE, GRID_SIZE } from "@/lib/consts";
</script>

<template>
    <path
        :d="`M 0 0
            Q ${(3 - BUBBLE_SIZE / 2) * GRID_SIZE} 0 ${(4 - BUBBLE_SIZE) * GRID_SIZE} ${2 * GRID_SIZE}
            Q ${(3 - BUBBLE_SIZE / 2) * GRID_SIZE} ${4 * GRID_SIZE} 0 ${4 * GRID_SIZE}
            A ${GRID_SIZE} ${2 * GRID_SIZE} 0 0 0 0 0 Z`"
        fill="var(--color-component-fill)"
        stroke="var(--color-component-stroke)"
        stroke-linecap="round"
    />

    <circle
        :cx="4 * GRID_SIZE - (BUBBLE_SIZE / 2) * GRID_SIZE"
        :cy="2 * GRID_SIZE"
        :r="(BUBBLE_SIZE / 2) * GRID_SIZE"
        fill="var(--color-component-fill)"
        stroke="var(--color-component-stroke)"
        stroke-linecap="round"
    />
</template>
