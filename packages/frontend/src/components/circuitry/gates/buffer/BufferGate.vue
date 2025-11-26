<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
</script>

<template>
    <path
        :d="`M 0 0
            L ${2 * GRID_SIZE} ${GRID_SIZE}
            L 0 ${2 * GRID_SIZE} Z`"
        fill="var(--color-component-fill)"
        stroke="var(--color-component-stroke)"
        stroke-linecap="round"
    />
</template>
