<script setup lang="ts">
import { CircuitComponent, ComponentType } from "@/lib/types";
import { componentMap } from ".";
import { GRID_SIZE } from "@/lib/consts";
import { computed } from "vue";
import { Handedness, Orientation } from "circuitsim-glue";
import { getPreviewGeometry } from "@/lib/preview";

const props = defineProps<{
    type: ComponentType;
    orientation?: Orientation;
    handedness?: Handedness;
}>();

const metadata = computed(() => componentMap[props.type]);

const geometry = computed(() =>
    getPreviewGeometry(props.type, props.orientation ?? "East", props.handedness ?? "DownRight"),
);
const dummyComponent = computed(
    () =>
        ({
            bounds: geometry.value.bounds,
            ports: geometry.value.ports,
            orientation: props.orientation ?? "East",
            handedness: props.handedness ?? "DownRight",
            type: props.type,

            numLegs: 2,
            portAssignments: [0, 1],
            inputs: 2,
            selsize: 1,
            bitsize: 1,
        }) as unknown as CircuitComponent,
);
</script>

<template>
    <svg
        xmlns="http://www.w3.org/2000/svg"
        class="mx-auto overflow-visible"
        :style="{
            width: geometry.dimensions.width * GRID_SIZE + 'px',
            height: geometry.dimensions.height * GRID_SIZE + 'px',
        }"
    >
        <component :is="metadata.component" :component="dummyComponent" />

        <circle
            v-for="(port, i) in dummyComponent.ports"
            :key="i"
            :cx="port.pos.x * GRID_SIZE"
            :cy="port.pos.y * GRID_SIZE"
            r="2"
            fill="currentColor"
            class="rounded-full text-orange-500"
        />
    </svg>
</template>
