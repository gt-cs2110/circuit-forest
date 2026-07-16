<script setup lang="ts">
import ConstantText from "@/components/ConstantText.vue";
import { GRID_SIZE } from "@/lib/consts";
import { CircuitComponentProps } from "@/lib/types";
import { computed } from "vue";
import { ROW_MAX_SIZE } from ".";

const { component } = defineProps<CircuitComponentProps>();

const width = computed(() => (component ? Math.min(Math.max(component.bitsize, 2), ROW_MAX_SIZE) : 2));
const height = computed(() => (component ? Math.max(Math.floor((component.bitsize - 1) / ROW_MAX_SIZE), 0) * 2 + 2 : 2));
</script>

<template>
    <path :d="`M 0 0
            H ${width * GRID_SIZE}
            L ${width * GRID_SIZE} ${height * GRID_SIZE} 
            L ${0} ${height * GRID_SIZE} Z`" fill="var(--color-component-fill)" stroke="var(--color-component-stroke)"
        stroke-linecap="round" />
    <ConstantText :value="component?.constantValue" />
</template>
