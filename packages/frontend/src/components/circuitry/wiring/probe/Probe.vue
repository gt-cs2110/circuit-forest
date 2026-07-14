<script setup lang="ts">
import ConstantText from "@/components/ConstantText.vue";
import { getDimensions } from "@/lib/bounds";
import { GRID_SIZE } from "@/lib/consts";
import { CircuitComponentProps } from "@/lib/types";
import { computed } from "vue";

const { component } = defineProps<CircuitComponentProps>();

const width = computed(() => (component ? getDimensions(component.bounds).width - 2 : 0));
const height = computed(() => (component ? getDimensions(component.bounds).height - 2 : 0));
</script>

<template>
    <path
        :d="`M ${GRID_SIZE}, 0 
            a ${GRID_SIZE},${GRID_SIZE} 0 0,0 -${GRID_SIZE},${GRID_SIZE}
            l 0,${height * GRID_SIZE}
            a ${GRID_SIZE},${GRID_SIZE} 0 0,0 ${GRID_SIZE},${GRID_SIZE}
            l ${width * GRID_SIZE},0
            a ${GRID_SIZE},${GRID_SIZE} 0 0,0 ${GRID_SIZE},-${GRID_SIZE}
            l 0,-${height * GRID_SIZE}
            a ${GRID_SIZE},${GRID_SIZE} 0 0,0 -${GRID_SIZE},-${GRID_SIZE}
            z
       `"
        fill="var(--color-component-fill)"
        stroke="var(--color-component-stroke)"
    />
    <ConstantText :value="component?.ports[0]?.value" />
</template>
