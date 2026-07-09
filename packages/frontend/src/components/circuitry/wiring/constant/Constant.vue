<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
import { CircuitComponentProps } from "@/lib/types";
import { computed } from "vue";

const { component } = defineProps<CircuitComponentProps>();
const width = computed(() => (component ? Math.max(2, component.bitsize) : 2));
</script>

<template>
    <path
        :d="`M 0 0
                H ${width * GRID_SIZE}
                L ${width * GRID_SIZE} ${2 * GRID_SIZE} 
                 L ${0} ${2 * GRID_SIZE} Z`"
        fill="var(--color-component-fill)"
        stroke="var(--color-component-stroke)"
        stroke-linecap="round"
    />
    <!-- <rect
        :width="(component?Math.max(2,component.bitsize):2)*GRID_SIZE"
        :height="2 *GRID_SIZE"

        fill="var(--color-component-fill)"
        stroke="var(--color-component-stroke)"
        stroke-linecap="round"
    /> -->
    <text
        :x="(GRID_SIZE * width) / 2"
        :y="GRID_SIZE * 1"
        :letter-spacing="GRID_SIZE / 3"
        text-anchor="middle"
        dominant-baseline="middle"
        class="pointer-events-none fill-black font-mono text-xs select-none"
    >
        {{ component ? component.constantValue : "" }}
    </text>
</template>
