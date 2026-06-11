<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
const props = defineProps<{ bitsize:number }>();

</script>


<template>
    <rect
        :width="Math.max(2,bitsize)*GRID_SIZE"
        :height="2 *GRID_SIZE"

        fill="var(--color-component-fill)"
        stroke="var(--color-component-stroke)"
        stroke-linecap="round"
    />
</template>
