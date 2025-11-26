<script setup lang="ts">
import { BUBBLE_SIZE, GRID_SIZE } from "@/lib/consts";
</script>

<template>
    <path
        :d="`M 0 0
            L ${(3 - BUBBLE_SIZE) * GRID_SIZE} ${GRID_SIZE}
            L 0 ${2 * GRID_SIZE} Z`"
        fill="currentColor"
        stroke="gray"
        class="text-zinc-200"
    />

    <circle
        :cx="3 * GRID_SIZE - (BUBBLE_SIZE / 2) * GRID_SIZE"
        :cy="1 * GRID_SIZE"
        :r="(BUBBLE_SIZE / 2) * GRID_SIZE"
        fill="currentColor"
        stroke="gray"
        class="text-zinc-200"
    />
</template>
