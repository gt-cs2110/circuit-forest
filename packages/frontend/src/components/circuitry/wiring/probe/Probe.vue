<script setup lang="ts">
import ConstantText from "@/components/ConstantText.vue";
import { getDimensions } from "@/lib/bounds";
import { GRID_SIZE } from "@/lib/consts";
import { CircuitComponentProps } from "@/lib/types";
import { computed } from "vue";

const { component } = defineProps<CircuitComponentProps>();

// FIXME: Don't hardcode default?
const dim = computed(() => (component ? getDimensions(component.bounds) : { width: 2, height: 2 }));
</script>

<template>
    <path
        :d="`M ${GRID_SIZE}, 0 
            a ${GRID_SIZE},${GRID_SIZE} 0 0,0 -${GRID_SIZE},${GRID_SIZE}
            l 0,${(dim.height - 2) * GRID_SIZE}
            a ${GRID_SIZE},${GRID_SIZE} 0 0,0 ${GRID_SIZE},${GRID_SIZE}
            l ${(dim.width - 2) * GRID_SIZE},0
            a ${GRID_SIZE},${GRID_SIZE} 0 0,0 ${GRID_SIZE},-${GRID_SIZE}
            l 0,-${(dim.height - 2) * GRID_SIZE}
            a ${GRID_SIZE},${GRID_SIZE} 0 0,0 -${GRID_SIZE},-${GRID_SIZE}
            z
       `"
        fill="var(--color-component-fill)"
        stroke="var(--color-component-stroke)"
    />
    <ConstantText :value="component?.ports[0]?.value" :width="dim.width" />
</template>
