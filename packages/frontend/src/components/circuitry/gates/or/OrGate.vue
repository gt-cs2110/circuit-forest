<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
import { rotateFromComponent } from "@/lib/svg";
import { CircuitComponentProps } from "@/lib/types";

const { component } = defineProps<CircuitComponentProps>();
</script>

<template>
    <g :transform="rotateFromComponent(component)">
        <path
            :d="`M 0 0
                Q ${3 * GRID_SIZE} 0 ${4 * GRID_SIZE} ${2 * GRID_SIZE}
                Q ${3 * GRID_SIZE} ${4 * GRID_SIZE} 0 ${4 * GRID_SIZE}
                A ${GRID_SIZE} ${2 * GRID_SIZE} 0 0 0 0 0 Z`"
            fill="var(--color-component-fill)"
            stroke="var(--color-component-stroke)"
            stroke-linecap="round"
        />
    </g>
</template>
