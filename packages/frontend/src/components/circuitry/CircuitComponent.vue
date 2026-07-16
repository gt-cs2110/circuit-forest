<script setup lang="ts">
import { getDimensions } from "@/lib/bounds";
import { GRID_SIZE } from "@/lib/consts";
import { selectComponent, deselectComponent, isComponentSelected } from "@/lib/store/view";
import { CircuitComponent, DragState } from "@/lib/types";
import { componentMap } from ".";
import { computed, inject, useId } from "vue";
import { wireColor } from "@/lib/wire";

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
    const { orientation, handedness, pos } = props.component;

    const ShiftToWorldCoordinates = {
        x: pos.x * GRID_SIZE,
        y: pos.y * GRID_SIZE,
    };

    const OriginRelativeToFixedPortRelative = {
        x: metadata.value.getOriginToFixedPortOffset(props.component).x * GRID_SIZE,
        y: metadata.value.getOriginToFixedPortOffset(props.component).y * GRID_SIZE,
    };
    const angle = rotation.value;

    const flipVert =
        ((orientation === "North" || orientation === "East") && handedness === "TopLeft") ||
        ((orientation === "South" || orientation === "West") && handedness === "DownRight");
    const handednessTransform = flipVert ? "scale(1, -1)" : "";

    // We first shift the fixed port to be at (0,0),
    //   then apply handedness,
    //   then rotate around (0, 0),
    //   and then translate it back.
    // FIXME: This shouldn't actually transform the entire component, just the shape
    return [
        `translate(${ShiftToWorldCoordinates.x}, ${ShiftToWorldCoordinates.y})`,
        `rotate(${angle})`,
        handednessTransform,
        `translate(${-OriginRelativeToFixedPortRelative.x}, ${-OriginRelativeToFixedPortRelative.y})`,
    ].join("\n");
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
            :width="getDimensions(props.component.bounds).width * GRID_SIZE"
            :height="getDimensions(props.component.bounds).height * GRID_SIZE"
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
            :fill="wireColor(point.value)"
            stroke="transparent"
            stroke-width="4"
            draggable="true"
            @mousedown="onPortDrag"
            @click="
                () => {
                    let key = point.backendKey;
                    let displayKey =
                        typeof key !== 'undefined'
                            ? `${key.kind}:${key.id[0]}v${key.id[1]}`
                            : undefined;
                    console.log('Port value:', point.value, displayKey);
                }
            "
            class="rounded-full text-orange-500 outline-orange-500 hover:outline-2"
        />
    </g>
</template>
