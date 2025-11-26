<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
</script>

<template>
    <path
        :d="`M 0 0
            H ${GRID_SIZE * 2}
            A ${2 * GRID_SIZE} ${2 * GRID_SIZE} 180 1 1 ${2 * GRID_SIZE} ${4 * GRID_SIZE}
            L 0 ${4 * GRID_SIZE} Z`"
        fill="var(--color-component-fill)"
        stroke="var(--color-component-stroke)"
        stroke-linecap="round"
    />
</template>
