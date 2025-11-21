<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
</script>

<template>
    <rect
        :width="2 * GRID_SIZE"
        :height="2 * GRID_SIZE"
        fill="currentColor"
        stroke="gray"
        class="text-zinc-200"
    />
</template>
