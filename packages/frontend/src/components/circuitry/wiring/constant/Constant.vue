<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
import { CircuitComponentProps } from "@/lib/types";
import { computed } from "vue";
import { constant } from ".";
const { component } = defineProps<CircuitComponentProps>();
const width = computed(() => (component ? constant.getDimensions(component).width : 2));
const height = computed(() => (component ? constant.getDimensions(component).height : 2));

const textLines = computed(() => {
    if (!component?.constantValue) return [];
    return component.constantValue.match(/.{1,8}/g) || [];
});

const totalLines = computed(() => textLines.value.length);
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
    <text
        :x="(GRID_SIZE * width) / 2"
        :y="(GRID_SIZE * height) / 2"
        :letter-spacing="GRID_SIZE / 3"
        text-anchor="middle"
        dominant-baseline="central"
        class="pointer-events-none fill-black font-mono text-xs select-none"
    >
        <tspan
            v-for="(line, index) in textLines"
            :key="index"
            :x="(width * GRID_SIZE) / 2"
            :dy="index === 0 ? `-${(totalLines - 1) * 0.6}em` : '1.2em'"
        >
            {{ line }}
        </tspan>
    </text>
</template>
