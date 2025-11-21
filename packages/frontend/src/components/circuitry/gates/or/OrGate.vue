<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
</script>

<template>
    <path
        :d="`M 0 0
            A ${4 * GRID_SIZE} ${2 * GRID_SIZE} 180 0 1 ${4 * GRID_SIZE} ${2 * GRID_SIZE}
            A ${4 * GRID_SIZE} ${2 * GRID_SIZE} 180 0 1 0 ${4 * GRID_SIZE}
            A ${GRID_SIZE} ${2 * GRID_SIZE} 0 0 0 0 0 Z`"
        fill="currentColor"
        stroke="gray"
        stroke-width="1"
        class="text-zinc-200"
    />
</template>
