<script setup lang="ts">
import { getDimensions } from "@/lib/bounds";
import { GRID_SIZE } from "@/lib/consts";
import { CircuitComponentProps } from "@/lib/types";
import { computed } from "vue";

const { component } = defineProps<CircuitComponentProps>();

const width = computed(() => (component ? Math.max(component.label.length, 2) : 2));
</script>

<template>
    <path :d="`M 0, 0 
            l ${width * GRID_SIZE}, 0
            l ${GRID_SIZE},${GRID_SIZE}
            l ${-1 * GRID_SIZE},${GRID_SIZE}
            l ${-width * GRID_SIZE}, 0
            z
       `" fill="var(--color-component-fill)" stroke="var(--color-component-stroke)" />
    <text :x="(GRID_SIZE * width) / 2" :y="GRID_SIZE" :letter-spacing="GRID_SIZE / 3" text-anchor="middle"
        dominant-baseline="central" class="pointer-events-none fill-black font-mono text-xs select-none">
        {{ component?.label }}
    </text>
</template>
