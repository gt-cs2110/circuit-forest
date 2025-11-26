<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
</script>

<template>
    <rect
        :width="2 * GRID_SIZE"
        :height="2 * GRID_SIZE"
        fill="var(--color-component-fill)"
        stroke="var(--color-component-stroke)"
        stroke-linecap="round"
    />
</template>
