<script setup lang="ts">
import ConstantText from "@/components/ConstantText.vue";
import { GRID_SIZE } from "@/lib/consts";
import { CircuitComponentProps } from "@/lib/types";
import { computed } from "vue";
import { ROW_MAX_SIZE } from ".";

const { component } = defineProps<CircuitComponentProps>();

const width = computed(() => (component ? Math.min(Math.max(component.bitsize - 2, 0), ROW_MAX_SIZE - 2) : 0));
const height = computed(() => (component ? Math.max(Math.floor((component.bitsize - 1) / ROW_MAX_SIZE), 0) * 2 : 0));
</script>

<template>
    <path :d="`M ${GRID_SIZE}, 0 
            a ${GRID_SIZE},${GRID_SIZE} 0 0,0 -${GRID_SIZE},${GRID_SIZE}
            l 0,${height * GRID_SIZE}
            a ${GRID_SIZE},${GRID_SIZE} 0 0,0 ${GRID_SIZE},${GRID_SIZE}
            l ${width * GRID_SIZE},0
            a ${GRID_SIZE},${GRID_SIZE} 0 0,0 ${GRID_SIZE},-${GRID_SIZE}
            l 0,-${height * GRID_SIZE}
            a ${GRID_SIZE},${GRID_SIZE} 0 0,0 -${GRID_SIZE},-${GRID_SIZE}
            z
       `" fill="var(--color-component-fill)" stroke="var(--color-component-stroke)" />
    <ConstantText :value="component?.ports[0]?.value" />
</template>
