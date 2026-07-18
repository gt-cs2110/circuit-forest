<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
import { computed } from "vue";

const { value, width } = defineProps<{ value?: string; width: number }>();

const bitsize = computed(() => value?.length ?? 0);
const fontSize = computed(() => (bitsize.value > 1 ? "1.2em" : "1.3em"));

const getX = (i: number) =>
    bitsize.value > 1 ? ((i % width) + 0.5) * GRID_SIZE : ((i % width) + 0.5) * 2 * GRID_SIZE;
const getY = (i: number) => (Math.floor(i / width) + 0.5) * 2 * GRID_SIZE;
</script>

<template>
    <text
        text-anchor="middle"
        dominant-baseline="central"
        class="pointer-events-none fill-black font-mono text-xs select-none"
    >
        <tspan
            v-for="(char, i) in value ?? ''"
            :key="i"
            :x="getX(i)"
            :y="getY(i)"
            :font-size="fontSize"
        >
            {{ char }}
        </tspan>
    </text>
</template>
