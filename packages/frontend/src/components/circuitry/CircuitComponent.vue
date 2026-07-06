<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
import { selectComponent, deselectComponent, isComponentSelected } from "@/lib/store/view";
import { CircuitComponent } from "@/lib/types";
import { componentMap } from ".";
import { computed, inject } from "vue";

const props = defineProps<{ component: CircuitComponent }>();
const emit = defineEmits<{
    dragstart: [e: MouseEvent];
    wiredrag: [e: MouseEvent];
}>();
const dragState = inject("dragState");

function handleMouseDown(e: MouseEvent) {
    console.log(e)
    if (e.button !== 0) return;
    e.stopPropagation();

    const additive = e.shiftKey || e.metaKey;

    if (additive && isComponentSelected(props.component.frontendId)) {
        deselectComponent(props.component.frontendId);
        return;
    }

    if (!isComponentSelected(props.component.frontendId)) {
        selectComponent(props.component.frontendId, additive);
    }
    emit("dragstart", e);
}
function onPortDrag(e: MouseEvent) {
    emit("wiredrag", e)
}
const metadata = computed(() => componentMap[props.component.type]);



const ports = computed(() => props.component.ports);
const rotation = computed(() => {
    switch (props.component.orientation) {
        case 0: return 270;//n
        case 1: return 90;//s
        case 2: return 0;//e
        default: return 180;; // w
    }
});

const transform = computed(() => {
    const ShiftToWorldCoordinates = { x: props.component.x * GRID_SIZE, y: props.component.y * GRID_SIZE };

    const OriginRelativeToFixedPortRelative = { x: metadata.value.getOriginToFixedPortOffset(props.component).x * GRID_SIZE, y: metadata.value.getOriginToFixedPortOffset(props.component).y * GRID_SIZE };
    const angle = rotation.value;

    //Trnaformations are applied right to left, first we shift so output port to be at 0,0 using the fixed to origin offset
    //rotate has to be next bc it rotates about world 0,0, so once we have fixed port on world 0,0 it rotates about ixed port
    //then we shift output port to be at the coordinates x,y
    return `
    translate(${ShiftToWorldCoordinates.x}, ${ShiftToWorldCoordinates.y})
    rotate(${angle})
    translate(${-OriginRelativeToFixedPortRelative.x}, ${-OriginRelativeToFixedPortRelative.y})
  `;
});

//The Ports and bounding box rely on the backend for their positiosning, thus while a draggin is happening they arent being updated until the drag is complete and a backend update is executed
//Thus we need to artifically make it seems like they are moving by applying a transformatino if a drag is actie
const boundingBoxAndPortTransform = computed(() => {
    if (dragState.active && dragState.initialComponentPositions.has(props.component.frontendId)) {
        let start_pos = dragState.initialComponentPositions.get(props.component.frontendId);
        return ` translate(${(props.component.x - start_pos.x) * GRID_SIZE}, ${(props.component.y - start_pos.y) * GRID_SIZE})`;
    }
    else {
        return '';
    }


})
computed(() => console.log(props.component))

</script>

<template>
    <g :transform="transform" @mousedown="handleMouseDown">

        <!-- <g :transform="rotate"> -->
        <component :is="metadata.component" :component="props.component" />

        <!-- </g> -->

    </g>

    <g :transform="boundingBoxAndPortTransform">
        <rect v-if="isComponentSelected(props.component.frontendId)" :x="(props.component.bounds[0].x) * GRID_SIZE"
            :y="(props.component.bounds[0].y) * GRID_SIZE"
            :width="metadata.getDimensions(props.component).width * GRID_SIZE"
            :height="metadata.getDimensions(props.component).height * GRID_SIZE" fill="none" stroke="#3b82f6"
            stroke-width="2" />
        <!-- transparent stroke enlarges hitbox -->
        <circle v-for="(point, index) in ports" :key="`${index}`" :cx="(point.x) * GRID_SIZE"
            :cy="(point.y) * GRID_SIZE" r="2"
            :fill="point.value.includes('Z') ? 'rgb(255,0,0)' : (point.value.includes('X') ? 'rgb(0,0,255)' : (point.value.includes('1') ? 'rgb(0,255,0)' : '#006400'))"
            stroke="transparent" stroke-width="4" draggable="true" @mousedown="onPortDrag"
            class="rounded-full text-orange-500 outline-orange-500 hover:outline-2" />
    </g>
</template>
