<script setup lang="ts">
import { ComponentType } from "@/lib/types";
import { componentMap } from "./circuitry";
import { GRID_SIZE } from "@/lib/consts";
import { computed } from "vue";

const props = defineProps<{ type: ComponentType }>();

const metadata = computed(() => componentMap[props.type]);
</script>

<template>
    <svg
        xmlns="http://www.w3.org/2000/svg"
        class="mx-auto overflow-visible"
        :style="{
            width: metadata.getDimensions().width * GRID_SIZE + 'px',
            height: metadata.getDimensions().height * GRID_SIZE + 'px',
        }"
    >
        <component :is="metadata.component" />

        <circle
            v-for="(port, i) in metadata.getPorts()"
            :key="i"
            :cx="port.x * GRID_SIZE"
            :cy="port.y * GRID_SIZE"
            r="2"
            fill="currentColor"
            class="rounded-full text-orange-500"
        />
    </svg>
</template>
