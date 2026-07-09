<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
import { CircuitComponentProps } from "@/lib/types";
import { computed } from "vue";

const { component } = defineProps<CircuitComponentProps>();
const mainAxis = computed(() => Math.pow(2, component ? component.selsize + 1 : 2));
</script>

<template>
    <g>
        <path
            :d="`M 0 0
                L ${GRID_SIZE * 3} ${GRID_SIZE}
                L ${GRID_SIZE * 3} ${(mainAxis - 1) * GRID_SIZE} 
                L 0 ${mainAxis * GRID_SIZE}
                Z`"
            fill="var(--color-component-fill)"
            stroke="var(--color-component-stroke)"
            stroke-linecap="round"
        />
    </g>
</template>
