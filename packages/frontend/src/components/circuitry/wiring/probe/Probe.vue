<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
import { CircuitComponentProps } from "@/lib/types";
import { computed } from "vue";
import { probe } from ".";

const { component } = defineProps<CircuitComponentProps>();

const width = computed(() => (component ? probe.getDimensions(component).width - 2 : 0));
const height = computed(() => (component ? probe.getDimensions(component).height - 2 : 0));
const textLines = computed(() => {
    if (!component?.componentValue) return [];
    return component.componentValue.match(/.{1,8}/g) || [];
});

const totalLines = computed(() => textLines.value.length);
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
    <text
        :x="(GRID_SIZE * (width + 2)) / 2"
        :y="(GRID_SIZE * (height + 2)) / 2"
        :letter-spacing="GRID_SIZE / 3"
        text-anchor="middle"
        dominant-baseline="central"
        class="pointer-events-none fill-black font-mono text-xs select-none"
    >
        <tspan
            v-for="(line, index) in textLines"
            :key="index"
            :x="((width + 2) * GRID_SIZE) / 2"
            :dy="index === 0 ? `-${(totalLines - 1) * 0.6}em` : '1.2em'"
        >
            {{ line }}
        </tspan>
    </text>
</template>
