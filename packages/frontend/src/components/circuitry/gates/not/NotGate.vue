<script setup lang="ts">
import { BUBBLE_SIZE, GRID_SIZE } from "@/lib/consts";
import { rotateFromComponent } from "@/lib/svg";
import { CircuitComponentProps } from "@/lib/types";

const { component } = defineProps<CircuitComponentProps>();
</script>

<template>
    <g :transform="rotateFromComponent(component)">
        <path
            :d="`M 0 0
                L ${(3 - BUBBLE_SIZE) * GRID_SIZE} ${GRID_SIZE}
                L 0 ${2 * GRID_SIZE} Z`"
            fill="var(--color-component-fill)"
            stroke="var(--color-component-stroke)"
            stroke-linecap="round"
        />
    
        <circle
            :cx="3 * GRID_SIZE - (BUBBLE_SIZE / 2) * GRID_SIZE"
            :cy="1 * GRID_SIZE"
            :r="(BUBBLE_SIZE / 2) * GRID_SIZE"
            fill="var(--color-component-fill)"
            stroke="var(--color-component-stroke)"
            stroke-linecap="round"
        />
    </g>
</template>
