<script setup lang="ts">
import ConstantText from "@/components/ConstantText.vue";
import { getDimensions } from "@/lib/bounds";
import { GRID_SIZE } from "@/lib/consts";
import { CircuitComponentProps } from "@/lib/types";
import { computed } from "vue";

const { component } = defineProps<CircuitComponentProps>();
const width = computed(() => (component ? getDimensions(component.bounds).width : 2));
const height = computed(() => (component ? getDimensions(component.bounds).height : 2));
</script>

<template>
    <path
        :d="`M 0 0
            H ${width * GRID_SIZE}
            L ${width * GRID_SIZE} ${height * GRID_SIZE} 
            L ${0} ${height * GRID_SIZE} Z`"
        fill="var(--color-component-fill)"
        stroke="var(--color-component-stroke)"
        stroke-linecap="round"
    />
    <ConstantText :value="component?.constantValue" />
</template>
