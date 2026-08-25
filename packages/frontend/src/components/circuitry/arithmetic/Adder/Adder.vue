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
                H ${4*GRID_SIZE }
                V ${4*GRID_SIZE }
                H ${0}
                Z`"
            fill="var(--color-component-fill)"
            stroke="var(--color-component-stroke)"
            stroke-linecap="round"
        />
        <path
    :d="`
      M ${2 * GRID_SIZE - 0.5 * GRID_SIZE} ${2 * GRID_SIZE} H ${2 * GRID_SIZE + 0.5 * GRID_SIZE}
      M ${2 * GRID_SIZE} ${2 * GRID_SIZE - 0.5 * GRID_SIZE} V ${2 * GRID_SIZE + 0.5 * GRID_SIZE}
    `"
    stroke="var(--color-component-stroke)"
    stroke-width="3"
    stroke-linecap="round"
  />
    </g>
</template>
