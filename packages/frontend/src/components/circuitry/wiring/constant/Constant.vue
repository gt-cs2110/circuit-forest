<script setup lang="ts">
import ConstantText from "@/components/ConstantText.vue";
import { GRID_SIZE } from "@/lib/consts";
import { CircuitComponentProps } from "@/lib/types";
import { computed } from "vue";
import { getDimensions } from "@/lib/bounds";

const { component } = defineProps<CircuitComponentProps>();

// FIXME: Don't hardcode default?
const dim = computed(() => (component ? getDimensions(component.bounds) : { width: 2, height: 2 }));
</script>

<template>
    <path
        :d="`M 0 0
            H ${dim.width * GRID_SIZE}
            L ${dim.width * GRID_SIZE} ${dim.height * GRID_SIZE} 
            L ${0} ${dim.height * GRID_SIZE} Z`"
        fill="var(--color-component-fill)"
        stroke="var(--color-component-stroke)"
        stroke-linecap="round"
    />
    <ConstantText :value="component?.constantValue" :width="dim.width" />
</template>
