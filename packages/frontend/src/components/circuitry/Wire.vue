<script setup lang="ts">
import { DOT_RADIUS, GRID_SIZE } from "@/lib/consts";
import { deselectWire, isWireSelected, selectWire } from "@/lib/store/view";
import type { DragState, Wire } from "@/lib/types";
import { wireColor, wireId } from "@/lib/wire";
import type { Location } from "circuitsim-glue";
import { computed, inject, ref, useId } from "vue";

const { wire } = defineProps<{ wire: Wire }>();
const emit = defineEmits<{
    dragstart: [e: MouseEvent];
    wiredrag: [e: MouseEvent];
}>();
const id = computed(() => wireId(wire.endpoints));
const baseId = useId();
// ~~~ DRAG HANDLERS ~~~
const dragState = inject<DragState>("dragState");
const isDragging = computed(() => dragState?.value.active && isWireSelected(id.value));
const dragTransform = computed(() => {
    if (!dragState || !isDragging.value) return "";
    const { delta } = dragState.value;
    return `translate(${delta.x * GRID_SIZE}, ${delta.y * GRID_SIZE})`;
});

function handleMouseDown(e: MouseEvent) {
    if (e.button !== 0) return;

    // dragging a selected wire should just move the selection instead of extending
    if (isWireSelected(id.value)) {
        e.stopPropagation();
        emit("dragstart", e);
    } else {
        emit("wiredrag", e);
    }
}
function handleClick(e: MouseEvent) {
    if (e.button !== 0) return;
    e.stopPropagation();

    const additive = e.shiftKey || e.metaKey;

    if (additive && isWireSelected(id.value)) {
        deselectWire(id.value);
        return;
    }

    if (!isWireSelected(id.value)) {
        selectWire(id.value, additive);
        let displayKey = `${wire.backendKey.kind}:${wire.backendKey.id[0]}v${wire.backendKey.id[1]}`;
        console.log("Wire value:", wire.value, displayKey);
    }
}

const hoverPoint = ref<Location | null>(null);

function handleMouseMove(e: MouseEvent) {
    const line = e.currentTarget as SVGLineElement;
    const ctm = line.getScreenCTM();
    if (!ctm) return;
    const local = new DOMPoint(e.clientX, e.clientY).matrixTransform(ctm.inverse());

    const [a, b] = wire.endpoints;
    const clamp = (v: number, p: number, q: number) =>
        Math.min(Math.max(Math.round(v / GRID_SIZE), Math.min(p, q)), Math.max(p, q));
    hoverPoint.value = { x: clamp(local.x, a.x, b.x), y: clamp(local.y, a.y, b.y) };
}
function handleMouseLeave() {
    hoverPoint.value = null;
}
</script>

<template>
    <!-- Drag items, which move alongside a drag -->
    <g :transform="dragTransform">
        <!-- Selected outline -->
        <line
            v-if="isWireSelected(id)"
            :x1="wire.endpoints[0].x * GRID_SIZE"
            :y1="wire.endpoints[0].y * GRID_SIZE"
            :x2="wire.endpoints[1].x * GRID_SIZE"
            :y2="wire.endpoints[1].y * GRID_SIZE"
            stroke-width="5"
            stroke-linecap="round"
            stroke="var(--color-primary)"
        />

        <g v-if="isDragging">
            <use :href="`#${baseId}`" />
        </g>
    </g>

    <!-- The original wire -->
    <!-- On drag, it becomes translucent to indicate a move is occurring -->
    <g :class="[{ 'opacity-50': isDragging }]">
        <line
            :x1="wire.endpoints[0].x * GRID_SIZE"
            :y1="wire.endpoints[0].y * GRID_SIZE"
            :x2="wire.endpoints[1].x * GRID_SIZE"
            :y2="wire.endpoints[1].y * GRID_SIZE"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            :id="baseId"
            :color="wireColor(wire.value)"
        />
    </g>

    <!-- Interaction wire -->
    <line
        :x1="wire.endpoints[0].x * GRID_SIZE"
        :y1="wire.endpoints[0].y * GRID_SIZE"
        :x2="wire.endpoints[1].x * GRID_SIZE"
        :y2="wire.endpoints[1].y * GRID_SIZE"
        stroke="transparent"
        stroke-width="6"
        stroke-linecap="round"
        @mousedown="handleMouseDown"
        @click="handleClick"
        @mousemove="handleMouseMove"
        @mouseleave="handleMouseLeave"
    />

    <circle
        v-if="hoverPoint && !isWireSelected(id)"
        :cx="hoverPoint.x * GRID_SIZE"
        :cy="hoverPoint.y * GRID_SIZE"
        :r="DOT_RADIUS"
        fill="transparent"
        stroke="transparent"
        stroke-width="4"
        class="pointer-events-none rounded-full outline-2 outline-port"
    />
</template>
