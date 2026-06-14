<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
import { CircuitComponent } from "@/lib/types";

const props = defineProps<{component:CircuitComponent}>();


</script>


<template>
    <g>
        <path
            :d="`M 0 0
                L ${GRID_SIZE*3} ${GRID_SIZE }
                L ${GRID_SIZE*3} ${(Math.pow(2,(component?component.selsize+1:2))-1) * GRID_SIZE} 
                L 0 ${Math.pow(2,(component?component.selsize+1:2))*GRID_SIZE }
                Z`"
            fill="var(--color-component-fill)"
            stroke="var(--color-component-stroke)"
            stroke-linecap="round"
        />
    </g>
</template>
