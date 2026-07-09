<script setup lang="ts">
import type { Location } from "circuitsim-glue";
import { computed, nextTick, onMounted, onUnmounted, provide, ref, toRaw, watch } from "vue";
import { GRID_SIZE, ORIGIN_OFFSET } from "@/lib/consts";
import {
    addWire,
    deleteComponent,
    deleteWires,
    placeComponent,
    updateState,
} from "@/lib/store/circuit";
import {
    clearSelection,
    componentSelection,
    getViewState,
    placingComponent,
    wireSelection,
} from "@/lib/store/view";
import { scale, settings } from "@/lib/store/settings";
import { Subcircuit } from "@/lib/types";
import { componentMap } from "./circuitry";
import CircuitComponent from "./circuitry/CircuitComponent.vue";
import CircuitComponentPreview from "./circuitry/CircuitComponentPreview.vue";
import Wire from "./circuitry/Wire.vue";
import { useCoordinates } from "@/composables/useCoordinates";
import { usePan } from "@/composables/usePan";
import { useDrag } from "@/composables/useDrag";
import { useMarquee } from "@/composables/useMarquee";
import { useZoom } from "@/composables/useZoom";
import { useTooltip } from "@/composables/useTooltip";

const props = defineProps<{
    subcircuit: Subcircuit;
}>();

const containerRef = ref<HTMLDivElement>();
const view = computed(() => getViewState(props.subcircuit.frontendId));

// NOTE: offset should always be assigned to by setting offset.value, not by
// setting offset.value.x/y individually. this is so that the value is always
// clamped. there are probably better ways to ensure this.
const offset = computed({
    get: () => view.value.offset,
    set: (val) => {
        view.value.offset.x = Math.min(val.x, 0);
        view.value.offset.y = Math.min(val.y, 0);
    },
});

// container coordinates
const mousePosition = ref({ x: 0, y: 0 });

const wireDrag = ref({
    active: false,
    multiDimension: false,
    start: { x: 0, y: 0 },
    middle: { x: 0, y: 0 },
    end: { x: 0, y: 0 },
});

const { containerToWorld, worldToContainer } = useCoordinates(offset, scale);
const { isPanning, startPan, updatePan, stopPan } = usePan(offset);
const { wheelZoom, keyboardZoom } = useZoom(
    offset,
    mousePosition,
    () => settings.value.scaleLevel,
    (level) => (settings.value.scaleLevel = level),
    scale,
);
const { drag, startDrag, updateDrag, stopDrag } = useDrag(
    props.subcircuit,
    componentSelection,
    wireSelection,
);
provide("dragState", drag);
const { marquee, startMarquee, updateMarquee, finalizeMarquee } = useMarquee(
    props.subcircuit,
    componentSelection,
    wireSelection,
);
const { tooltip, updateTooltip } = useTooltip();

const marqueeStyle = computed(() => {
    const a = worldToContainer(marquee.value.start.x, marquee.value.start.y);
    const b = worldToContainer(marquee.value.current.x, marquee.value.current.y);
    return {
        left: Math.min(a.x, b.x) + "px",
        top: Math.min(a.y, b.y) + "px",
        width: Math.abs(a.x - b.x) + "px",
        height: Math.abs(a.y - b.y) + "px",
    };
});

function toWorld(e: MouseEvent) {
    const rect = containerRef.value!.getBoundingClientRect();
    return containerToWorld(e.clientX - rect.left, e.clientY - rect.top);
}

const placingComponentPosition = ref<Location | null>(null);

watch(placingComponent, () => {
    placingComponentPosition.value = null;
});

watch(mousePosition, (mouse) => {
    if (!placingComponent.value) {
        return;
    }

    const metadata = componentMap[placingComponent.value];
    const dimensions = metadata?.getDefaultDimensions() || { width: 1, height: 1 };
    placingComponentPosition.value = {
        x: Math.floor((mouse.x - offset.value.x) / GRID_SIZE / scale.value - dimensions.width / 2),
        y: Math.floor((mouse.y - offset.value.y) / GRID_SIZE / scale.value - dimensions.height / 2),
    };
});

function handleMouseDown(e: MouseEvent) {
    if ((e.button === 0 && e.metaKey) || e.button === 1) {
        startPan(e.clientX, e.clientY);
        return;
    }
    if (e.button !== 0) return;

    const world = toWorld(e);
    if (!wireDrag.value.active) {
        startMarquee(world.x, world.y, e.shiftKey || e.metaKey);
    }
}

function handleMouseMove(e: MouseEvent) {
    const rect = containerRef.value!.getBoundingClientRect();
    mousePosition.value = {
        x: e.clientX - rect.left,
        y: e.clientY - rect.top,
    };

    const world = toWorld(e);

    updatePan(e.clientX, e.clientY);
    updateDrag(world.x, world.y);
    updateWireDrag(e);
    updateMarquee(world.x, world.y);
    updateTooltip(e.target!);
}
function handleDelete(e: KeyboardEvent) {
    if (e.key === "Backspace") {
        if (componentSelection.value.size + wireSelection.value.size > 0) {
            //Delete all wires on backend manuall first, bc ids will not persits through updateState

            componentSelection.value.forEach((frontendId) => {
                deleteComponent(frontendId);
            });
            deleteWires(wireSelection.value.keys().toArray());
        }
    }
}

function handleMouseUp(e: MouseEvent) {
    const rect = containerRef.value!.getBoundingClientRect();
    mousePosition.value.x = e.clientX - rect.left;
    mousePosition.value.y = e.clientY - rect.top;
    const world = toWorld(e);

    updatePan(e.clientX, e.clientY);
    updateDrag(world.x, world.y);
    stopPan();
    if (drag.value.active) stopDrag();
    stopWireDrag();
    finalizeMarquee();
}

function handleComponentDragStart(e: MouseEvent) {
    const world = toWorld(e);
    startDrag(world.x, world.y);
}
function handleComponentWireDrag(e: MouseEvent) {
    console.log("starting wire drag");
    const world = toWorld(e);

    if (!wireDrag.value.active) {
        let startPoint: Location = { x: Math.round(world.x), y: Math.round(world.y) };
        wireDrag.value = {
            active: true,
            multiDimension: false,
            start: startPoint,
            middle: startPoint,
            end: startPoint,
        };
    }
}
function stopWireDrag() {
    if (!wireDrag.value.active) return;

    let drag = toRaw(wireDrag);
    let isEmpty = drag.value.end.x == drag.value.start.x && drag.value.end.y == drag.value.start.y;
    if (!isEmpty) {
        if (drag.value.multiDimension) {
            addWire(drag.value.start, drag.value.middle);
            addWire(drag.value.middle, drag.value.end);
        } else {
            addWire(drag.value.start, drag.value.end);
        }
    }

    wireDrag.value = {
        active: false,
        multiDimension: false,
        start: { x: 0, y: 0 },
        middle: { x: 0, y: 0 },
        end: { x: 0, y: 0 },
    };
    updateState();
}
function updateWireDrag(e: MouseEvent) {
    if (!wireDrag.value.active) return;
    const world = toWorld(e);
    world.x = Math.ceil(world.x);
    world.y = Math.floor(world.y);

    if (wireDrag.value.start.x == world.x) {
        // Only changing y direction
        wireDrag.value.multiDimension = false;
        wireDrag.value.end = world;
    } else if (wireDrag.value.start.y == world.y) {
        // Only changing x direction
        wireDrag.value.multiDimension = false;
        wireDrag.value.end = world;
    } else {
        // Line with largest delta becomes first line
        let horizDelta = Math.abs(wireDrag.value.start.x - world.x);
        let vertDelta = Math.abs(wireDrag.value.start.y - world.y);
        let horizFirst = wireDrag.value.multiDimension
            ? wireDrag.value.middle.y == wireDrag.value.start.y
            : horizDelta > vertDelta;

        if (horizFirst) {
            // Horizontal is largest delta, set horizontal as first line
            wireDrag.value = {
                ...wireDrag.value,
                middle: { x: world.x, y: wireDrag.value.start.y },
                end: world,
                multiDimension: true,
            };
        } else {
            // Vertical is largest delta, set vertical as first line
            wireDrag.value = {
                ...wireDrag.value,
                middle: { x: wireDrag.value.start.x, y: world.y },
                end: world,
                multiDimension: true,
            };
        }
    }
}

function handleWheel(e: WheelEvent) {
    wheelZoom(e);
    nextTick().then(() => updateTooltip(e.target!));
}

function handleKeyDown(e: KeyboardEvent) {
    if (keyboardZoom(e)) return;

    if (e.key === "Escape") {
        placingComponent.value = null;
        clearSelection();
    }
}

onMounted(() => document.addEventListener("keydown", handleKeyDown));
onMounted(() => {
    document.addEventListener("keydown", handleDelete);
});
onUnmounted(() => document.removeEventListener("keydown", handleKeyDown));

const metadata = computed(() => componentMap[placingComponent.value || "and"]);
</script>

<template>
    <div
        ref="containerRef"
        class="relative flex-1 overflow-hidden bg-canvas-background"
        :style="{ cursor: isPanning ? 'grabbing' : 'default' }"
        @mousedown="handleMouseDown"
        @mousemove="handleMouseMove"
        @mouseup="handleMouseUp"
        @mouseleave="handleMouseUp"
        @wheel.prevent="handleWheel"
    >
        <svg
            class="pointer-events-none absolute inset-0 h-full w-full"
            xmlns="http://www.w3.org/2000/svg"
        >
            <defs>
                <pattern
                    id="dotPattern"
                    :x="offset.x % (GRID_SIZE * scale)"
                    :y="offset.y % (GRID_SIZE * scale)"
                    :width="GRID_SIZE * scale"
                    :height="GRID_SIZE * scale"
                    patternUnits="userSpaceOnUse"
                >
                    <circle
                        :cx="(GRID_SIZE / 2) * scale"
                        :cy="(GRID_SIZE / 2) * scale"
                        :r="0.5 * scale"
                        fill="var(--color-canvas-dots)"
                    />
                </pattern>
            </defs>

            <rect x="0" y="0" width="100%" height="100%" fill="url(#dotPattern)" />
        </svg>

        <svg
            class="absolute origin-top-left overflow-visible"
            xmlns="http://www.w3.org/2000/svg"
            :style="{
                transform: `translate(${offset.x + ORIGIN_OFFSET * scale}px, ${offset.y + ORIGIN_OFFSET * scale}px) scale(${scale})`,
            }"
        >
            <CircuitComponent
                v-for="[id, component] in subcircuit.components"
                :key="id"
                :component="component"
                @dragstart="handleComponentDragStart"
                @wiredrag="handleComponentWireDrag"
            />

            <g
                v-if="placingComponent && placingComponentPosition !== null"
                opacity="0.5"
                :transform="`translate(${placingComponentPosition.x * GRID_SIZE}, ${placingComponentPosition.y * GRID_SIZE})`"
                @click="
                    placeComponent(
                        placingComponent,
                        placingComponentPosition.x + metadata.getDefaultDimensions().width,
                        placingComponentPosition.y + metadata.getDefaultDimensions().height / 2,
                    )
                "
            >
                <CircuitComponentPreview :type="placingComponent" />
            </g>

            <g v-for="(wire, i) in subcircuit.wires" :key="i">
                <Wire :wire @wiredrag="handleComponentWireDrag" />
            </g>
            <line
                v-if="wireDrag.active && !wireDrag.multiDimension"
                :x1="wireDrag.start.x * GRID_SIZE"
                :y1="wireDrag.start.y * GRID_SIZE"
                :x2="wireDrag.end.x * GRID_SIZE"
                :y2="wireDrag.end.y * GRID_SIZE"
                stroke="black"
                stroke-width="2"
                stroke-linecap="round"
            />
            <line
                v-if="wireDrag.active && wireDrag.multiDimension"
                :x1="wireDrag.start.x * GRID_SIZE"
                :y1="wireDrag.start.y * GRID_SIZE"
                :x2="wireDrag.middle.x * GRID_SIZE"
                :y2="wireDrag.middle.y * GRID_SIZE"
                stroke="black"
                stroke-width="2"
                stroke-linecap="round"
            />
            <line
                v-if="wireDrag.active && wireDrag.multiDimension"
                :x1="wireDrag.middle.x * GRID_SIZE"
                :y1="wireDrag.middle.y * GRID_SIZE"
                :x2="wireDrag.end.x * GRID_SIZE"
                :y2="wireDrag.end.y * GRID_SIZE"
                stroke="black"
                stroke-width="2"
                stroke-linecap="round"
            />
        </svg>

        <div
            v-if="marquee.active"
            class="pointer-events-none absolute border border-blue-500 bg-blue-500/10"
            :style="marqueeStyle"
        />

        <div
            v-if="tooltip.value"
            class="pointer-events-none fixed z-50 -mt-4 w-max -translate-x-1/2 -translate-y-full border border-blue-800 bg-blue-600 px-2 font-mono text-sm text-white"
            :style="{ left: tooltip.x + 'px', top: tooltip.y + 'px' }"
        >
            {{ tooltip.value }}
        </div>
    </div>
</template>
