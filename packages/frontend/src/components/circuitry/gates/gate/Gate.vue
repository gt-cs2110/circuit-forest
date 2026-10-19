<script setup lang="ts">
import { BUBBLE_SIZE, GRID_SIZE } from "@/lib/consts";
import { rotateFromComponent } from "@/lib/svg";
import type { CircuitComponentProps } from "@/lib/types";
import { computed } from "vue";
import { gateShapes, isGateKind } from "./index";

const { component } = defineProps<CircuitComponentProps>();
const XOR_OFFSET = 0.5;

const shape = computed(() => {
    const type = component?.type.toLowerCase() ?? "and";
    return gateShapes[isGateKind(type) ? type : "and"];
});

const bubble = computed(() => (shape.value.negated ? BUBBLE_SIZE : 0));

// G is GRID_SIZE, b is right bubble size (0 or BUBBLE_SIZE)
const bodyPaths = {
    and: (G: number, b: number) => `M 0 0
        H ${(2 - b) * G}
        A ${2 * G} ${2 * G} 180 1 1 ${(2 - b) * G} ${4 * G}
        L 0 ${4 * G} Z`,
    or: (G: number, b: number) => `M 0 0
        Q ${(3 - b / 2) * G} 0 ${(4 - b) * G} ${2 * G}
        Q ${(3 - b / 2) * G} ${4 * G} 0 ${4 * G}
        A ${G} ${2 * G} 0 0 0 0 0 Z`,
    xor: (G: number, b: number) => `M ${XOR_OFFSET * G} 0
        Q ${(3 - b / 2) * G} 0 ${(4 - b) * G} ${2 * G}
        Q ${(3 - b / 2) * G} ${4 * G} ${XOR_OFFSET * G} ${4 * G}
        A ${(1 - XOR_OFFSET / 3) * G} ${2 * G} 0 0 0 ${XOR_OFFSET * G} 0 Z`,
} as const;
const bodyPath = computed(() => bodyPaths[shape.value.body](GRID_SIZE, bubble.value));
</script>

<template>
    <g :transform="rotateFromComponent(component)">
        <path
            v-if="shape.body === 'xor'"
            :d="`M 0 ${4 * GRID_SIZE}
                A ${GRID_SIZE} ${2 * GRID_SIZE} 0 0 0 0 0`"
            fill="transparent"
            stroke="var(--color-component-stroke)"
            stroke-linecap="round"
        />

        <path
            :d="bodyPath"
            fill="var(--color-component-fill)"
            stroke="var(--color-component-stroke)"
            stroke-linecap="round"
        />

        <circle
            v-if="shape.negated"
            :cx="(4 - BUBBLE_SIZE / 2) * GRID_SIZE"
            :cy="2 * GRID_SIZE"
            :r="(BUBBLE_SIZE / 2) * GRID_SIZE"
            fill="var(--color-component-fill)"
            stroke="var(--color-component-stroke)"
            stroke-linecap="round"
        />
    </g>
</template>
