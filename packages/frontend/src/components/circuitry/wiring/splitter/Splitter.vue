<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
import { CircuitComponentProps } from "@/lib/types";

const { component } = defineProps<CircuitComponentProps>();
const numLegs = component?.numLegs ?? 2;
let svg = `M ${0 * GRID_SIZE}, ${0 * GRID_SIZE} 
        L ${1 * GRID_SIZE}, ${1 * GRID_SIZE} `;
[...Array(numLegs).keys()].forEach(leg => {
    svg += `L ${1 * GRID_SIZE}, ${2 * (leg + 1) * GRID_SIZE}
            L ${2 * GRID_SIZE}, ${2 * (leg + 1) * GRID_SIZE}
            M ${1 * GRID_SIZE}, ${2 * (leg + 1) * GRID_SIZE} `

})
</script>

<template>>
    <rect x="0" y="0" :width="2 * GRID_SIZE" :height="numLegs ?? 2 * GRID_SIZE" fill="transparent"
        pointer-events="all" />
    <path :d=svg fill="none" stroke="var(--color-component-stroke)" />
</template>
