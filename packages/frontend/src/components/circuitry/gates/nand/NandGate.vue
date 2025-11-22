<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
</script>

<template>
    <path
        :d="`M 0 0
            H ${2 * GRID_SIZE - 0.75 * GRID_SIZE}
            A ${2 * GRID_SIZE} ${2 * GRID_SIZE} 180 1 1 ${2 * GRID_SIZE - 0.75 * GRID_SIZE} ${4 * GRID_SIZE}
            L 0 ${4 * GRID_SIZE} Z`"
        fill="currentColor"
        stroke="gray"
        class="text-zinc-200"
    />

    <circle
        :cx="4 * GRID_SIZE - (0.75 / 2) * GRID_SIZE"
        :cy="2 * GRID_SIZE"
        :r="(0.75 / 2) * GRID_SIZE"
        fill="currentColor"
        stroke="gray"
        class="text-zinc-200"
    />
</template>
