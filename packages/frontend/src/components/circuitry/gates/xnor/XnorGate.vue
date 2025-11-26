<script setup lang="ts">
import { BUBBLE_SIZE, GRID_SIZE } from "@/lib/consts";

const XOR_OFFSET = 0.5;
</script>

<template>
    <path
        :d="`M 0 ${4 * GRID_SIZE}
            A ${GRID_SIZE} ${2 * GRID_SIZE} 0 0 0 0 0`"
        fill="transparent"
        stroke="var(--color-component-stroke)"
        stroke-linecap="round"
    />

    <path
        :d="`M ${XOR_OFFSET * GRID_SIZE} 0
            Q ${(3 - BUBBLE_SIZE / 2) * GRID_SIZE} 0 ${(4 - BUBBLE_SIZE) * GRID_SIZE} ${2 * GRID_SIZE}
            Q ${(3 - BUBBLE_SIZE / 2) * GRID_SIZE} ${4 * GRID_SIZE} ${XOR_OFFSET * GRID_SIZE} ${4 * GRID_SIZE}
            A ${(1 - XOR_OFFSET / 3) * GRID_SIZE} ${2 * GRID_SIZE} 0 0 0 ${XOR_OFFSET * GRID_SIZE} 0 Z`"
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
