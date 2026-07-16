<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
import { CircuitComponentProps } from "@/lib/types";
import { computed } from "vue";

const { component } = defineProps<CircuitComponentProps>();

</script>

<template>>
    <rect x="0" y="0" :width="2 * GRID_SIZE" :height="4 * GRID_SIZE" fill="transparent" pointer-events="all" />
    <path :d="`
  M ${0 * GRID_SIZE}, ${0 * GRID_SIZE} 
  L ${1 * GRID_SIZE}, ${1 * GRID_SIZE} 
  L ${1 * GRID_SIZE}, ${2 * GRID_SIZE} 
  L ${2 * GRID_SIZE}, ${2 * GRID_SIZE}
M ${1 * GRID_SIZE}, ${2 * GRID_SIZE} 
  L ${1 * GRID_SIZE}, ${4 * GRID_SIZE}
  L ${2 * GRID_SIZE}, ${4 * GRID_SIZE}


`" fill="none" stroke="var(--color-component-stroke)" />
</template>
