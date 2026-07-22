<script setup lang="ts">
import { getDimensions } from "@/lib/bounds";
import { GRID_SIZE } from "@/lib/consts";
import { rotateFromComponent } from "@/lib/svg";
import { CircuitComponentProps } from "@/lib/types";
import { computed } from "vue";

const { component } = defineProps<CircuitComponentProps>();
const numLegs = computed(() => (component ? component.numLegs : 2));
const dimensions = computed(() =>
    component ? getDimensions(component.bounds) : { width: 2, height: 4 },
);
//Maybe to be replaced with something prettier later
const handednessTransform = computed(() => {
    if (component?.orientation === "West" && component.handedness == "TopLeft")
        return `translate(0 ${dimensions.value.height * GRID_SIZE}) scale(1 -1)`;
    if (component?.orientation === "South" && component.handedness == "TopLeft")
        return ` translate(0 ${dimensions.value.width * GRID_SIZE}) scale(1 -1)`;
    if (component?.orientation === "East" && component.handedness == "DownRight")
        return ` translate(0 ${dimensions.value.height * GRID_SIZE}) scale(1 -1)`;
    if (component?.orientation === "North" && component.handedness == "DownRight")
        return ` translate(0 ${dimensions.value.width * GRID_SIZE}) scale(1 -1)`;

    return "";
});
const svgPath = computed(() => {
    let pathString = `M ${0 * GRID_SIZE}, ${0 * GRID_SIZE} L ${1 * GRID_SIZE}, ${1 * GRID_SIZE} `;
    const assignedPorts = new Set(component?.portAssignments);
    // Create the range using a standard array loop
    let legs_created = 0;
    Array.from({ length: numLegs.value }).forEach((_, leg) => {
        if (component && !assignedPorts.has(leg)) return;
        pathString += `L ${1 * GRID_SIZE}, ${2 * (legs_created + 1) * GRID_SIZE}
                       L ${2 * GRID_SIZE}, ${2 * (legs_created + 1) * GRID_SIZE}
                       M ${1 * GRID_SIZE}, ${2 * (legs_created + 1) * GRID_SIZE} `;
        legs_created++;
    });

    return pathString;
});
</script>

<template>
    <g :transform="`${rotateFromComponent(component) ?? ''} ${handednessTransform}`">
        <rect
            x="0"
            y="0"
            :width="dimensions.width * GRID_SIZE"
            :height="dimensions.height * GRID_SIZE"
            fill="white"
            fill-opacity="0"
            pointer-events="all"
        />
        <path :d="svgPath" fill="none" stroke="var(--color-component-stroke)" />
    </g>
</template>
