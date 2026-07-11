<script setup lang="ts">
import { GRID_SIZE } from "@/lib/consts";
import { selectComponent, deselectComponent, isComponentSelected } from "@/lib/store/view";
import { CircuitComponent, DragState } from "@/lib/types";
import { componentMap } from ".";
import { computed, inject, useId } from "vue";

const props = defineProps<{ component: CircuitComponent }>();
const emit = defineEmits<{
    dragstart: [e: MouseEvent];
    wiredrag: [e: MouseEvent];
}>();
const baseId = useId();
// ~~~ DRAG HANDLERS ~~~
const dragState = inject<DragState>("dragState");
const isDragging = computed(
    () => dragState?.value.active && isComponentSelected(props.component.frontendId),
);
const dragTransform = computed(() => {
    if (!dragState || !isDragging.value) return "";
    const { delta } = dragState.value;
    return `translate(${delta.x * GRID_SIZE}, ${delta.y * GRID_SIZE})`;
});

function handleMouseDown(e: MouseEvent) {
    console.log(e);
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
    emit("wiredrag", e);
}
const metadata = computed(() => componentMap[props.component.type]);

const ports = computed(() => props.component.ports);
const rotation = computed(() => {
    switch (props.component.orientation) {
        case "North":
            return 270;
        case "South":
            return 90;
        case "East":
            return 0;
        case "West":
        default:
            return 180;
    }
});

const transform = computed(() => {
    const ShiftToWorldCoordinates = {
        x: props.component.pos.x * GRID_SIZE,
        y: props.component.pos.y * GRID_SIZE,
    };

    const OriginRelativeToFixedPortRelative = {
        x: metadata.value.getOriginToFixedPortOffset(props.component).x * GRID_SIZE,
        y: metadata.value.getOriginToFixedPortOffset(props.component).y * GRID_SIZE,
    };
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
</script>

<template>
    <!-- The original component, which is also responsible for interactions -->
    <!-- On drag, it becomes translucent to indicate a move is occurring -->
    <g :class="[{ 'opacity-50': isDragging }]">
        <g :transform="transform" @mousedown="handleMouseDown" :id="baseId">
            <component :is="metadata.component" :component="props.component" />
        </g>
    </g>

    <!-- Drag items, which move alongside a drag -->
    <g :transform="dragTransform">
        <!-- Drag copy of the component -->
        <g v-if="isDragging">
            <use :href="`#${baseId}`" />
        </g>
        <!-- Selected outline -->
        <rect
            v-if="isComponentSelected(props.component.frontendId)"
            :x="props.component.bounds[0].x * GRID_SIZE"
            :y="props.component.bounds[0].y * GRID_SIZE"
            :width="metadata.getDimensions(props.component).width * GRID_SIZE"
            :height="metadata.getDimensions(props.component).height * GRID_SIZE"
            fill="none"
            stroke="#3b82f6"
            stroke-width="2"
        />
        <!-- Ports -->
        <circle
            v-for="(point, index) in ports"
            :key="`${index}`"
            :cx="point.pos.x * GRID_SIZE"
            :cy="point.pos.y * GRID_SIZE"
            r="2"
            :fill="
                point.value.includes('X')
                    ? 'rgb(255,0,0)'
                    : point.value.includes('Z')
                      ? 'rgb(0,0,255)'
                      : point.value.includes('1')
                        ? 'rgb(0,255,0)'
                        : '#006400'
            "
            stroke="transparent"
            stroke-width="4"
            draggable="true"
            @mousedown="onPortDrag"
            @click="console.log('Port value:', point.value)"
            class="rounded-full text-orange-500 outline-orange-500 hover:outline-2"
        />
    </g>
</template>
