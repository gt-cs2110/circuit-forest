<script setup lang="ts">
import { getDimensions } from "@/lib/bounds";
import { GRID_SIZE } from "@/lib/consts";
import { polyline } from "@/lib/svg";
import { CircuitComponentProps } from "@/lib/types";
import { computed } from "vue";

const { component } = defineProps<CircuitComponentProps>();

// FIXME: Don't hardcode
const dim = computed(() => (component ? getDimensions(component.bounds) : { width: 2, height: 2 }));

const orientation = computed(() => component?.orientation ?? "East");
const mainBounds = computed(() => {
    const orient = orientation.value;

    let start_x = 0 + +(orient === "West") / 2;
    let start_y = 0 + +(orient === "North") / 2;
    let end_x = dim.value.width - +(orient === "East") / 2;
    let end_y = dim.value.height - +(orient === "South") / 2;
    return [
        [start_x, start_y],
        [end_x, end_y],
    ] as const;
});
const points = computed<[number, number][]>(() => {
    const orient = orientation.value;
    const [[start_x, start_y], [end_x, end_y]] = mainBounds.value;
    const mid_x = (start_x + end_x) / 2;
    const mid_y = (start_y + end_y) / 2;
    return [
        [start_x, start_y],
        ...(orient === "North" ? [[mid_x, start_y - 0.5]] : []) satisfies [number, number][],
        [end_x, start_y],
        ...(orient === "East" ? [[end_x + 0.5, mid_y]] : []) satisfies [number, number][],
        [end_x, end_y],
        ...(orient === "South" ? [[mid_x, end_y + 0.5]] : []) satisfies [number, number][],
        [start_x, end_y],
        ...(orient === "West" ? [[start_x - 0.5, mid_y]] : []) satisfies [number, number][],
        [start_x, start_y],
    ] as const;
});
</script>

<template>
    <path
        :d="polyline(points)"
        fill="var(--color-component-fill)"
        stroke="var(--color-component-stroke)"
    />
    <text
        :x="((mainBounds[0][0] + mainBounds[1][0]) * GRID_SIZE) / 2"
        :y="((mainBounds[0][1] + mainBounds[1][1]) * GRID_SIZE) / 2"
        :letter-spacing="GRID_SIZE / 3"
        text-anchor="middle"
        dominant-baseline="central"
        class="pointer-events-none fill-black font-mono text-xs select-none"
    >
        {{ component?.label }}
    </text>
</template>
