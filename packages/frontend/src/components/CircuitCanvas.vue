<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import { GRID_SIZE, ORIGIN_OFFSET } from "@/lib/consts";
import { placeComponent } from "@/lib/store/circuit";
import { clearSelection, getViewState, placingComponent, selection } from "@/lib/store/view";
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
const view = computed(() => getViewState(props.subcircuit.id));

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
const mousePosition = reactive({ x: 0, y: 0 });

const { containerToWorld, worldToContainer } = useCoordinates(offset, scale);
const { isPanning, startPan, updatePan, stopPan } = usePan(offset);
const { wheelZoom, keyboardZoom } = useZoom(
    offset,
    mousePosition,
    () => settings.scaleLevel,
    (level) => (settings.scaleLevel = level),
    scale,
);
const { startDrag, updateDrag, stopDrag } = useDrag(props.subcircuit, selection);
const { marquee, startMarquee, updateMarquee, finalizeMarquee } = useMarquee(
    props.subcircuit,
    selection,
);
const { tooltip, updateTooltip } = useTooltip();

const marqueeStyle = computed(() => {
    const a = worldToContainer(marquee.start.x, marquee.start.y);
    const b = worldToContainer(marquee.current.x, marquee.current.y);
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

const placingComponentPosition = reactive({ x: 0 as number | null, y: 0 as number | null });

watch(placingComponent, () => {
    placingComponentPosition.x = null;
    placingComponentPosition.y = null;
});

watch(mousePosition, (mouse) => {
    if (!placingComponent.value) {
        return;
    }
    const metadata = componentMap[placingComponent.value];
    const dimensions = metadata?.getDimensions() || { width: 1, height: 1 };
    placingComponentPosition.x = Math.floor(
        (mouse.x - offset.value.x) / GRID_SIZE / scale.value - dimensions.width / 2,
    );
    placingComponentPosition.y = Math.floor(
        (mouse.y - offset.value.y) / GRID_SIZE / scale.value - dimensions.height / 2,
    );
});

function handleMouseDown(e: MouseEvent) {
    if ((e.button === 0 && e.metaKey) || e.button === 1) {
        startPan(e.clientX, e.clientY);
        return;
    }
    if (e.button !== 0) return;

    const world = toWorld(e);
    startMarquee(world.x, world.y, e.shiftKey || e.metaKey);
}

function handleMouseMove(e: MouseEvent) {
    const rect = containerRef.value!.getBoundingClientRect();
    mousePosition.x = e.clientX - rect.left;
    mousePosition.y = e.clientY - rect.top;

    const world = toWorld(e);
    updatePan(e.clientX, e.clientY);
    updateDrag(world.x, world.y);
    updateMarquee(world.x, world.y);
    updateTooltip(e.target!);
}

function handleMouseUp() {
    stopPan();
    stopDrag();
    finalizeMarquee();
}

function handleComponentDragStart(e: MouseEvent) {
    const world = toWorld(e);
    startDrag(world.x, world.y);
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
onUnmounted(() => document.removeEventListener("keydown", handleKeyDown));
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
            />

            <g
                v-if="
                    placingComponent &&
                    placingComponentPosition.x !== null &&
                    placingComponentPosition.y !== null
                "
                opacity="0.5"
                :transform="`translate(${placingComponentPosition.x * GRID_SIZE}, ${placingComponentPosition.y * GRID_SIZE})`"
                @click="
                    placeComponent(
                        placingComponent,
                        placingComponentPosition.x,
                        placingComponentPosition.y,
                    )
                "
            >
                <CircuitComponentPreview :type="placingComponent" />
            </g>

            <g v-for="(wire, i) in subcircuit.wires" :key="i">
                <Wire :wire />
            </g>
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
