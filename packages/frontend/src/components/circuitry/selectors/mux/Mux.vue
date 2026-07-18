<script setup lang="ts">
import { getDimensions } from "@/lib/bounds";
import { trapezoid } from "@/lib/svg";
import { CircuitComponentProps } from "@/lib/types";
import { computed } from "vue";

const { component } = defineProps<CircuitComponentProps>();
// FIXME: Don't hardcode default?
const dim = computed(() => (component ? getDimensions(component.bounds) : { width: 3, height: 4 }));
</script>

<template>
    <g>
        <path
            :d="trapezoid(component?.orientation ?? 'East', dim.width, dim.height)"
            fill="var(--color-component-fill)"
            stroke="var(--color-component-stroke)"
            stroke-linecap="round"
        />
    </g>
</template>
