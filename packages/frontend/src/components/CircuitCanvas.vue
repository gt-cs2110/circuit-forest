<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import { GRID_SIZE, ORIGIN_OFFSET } from "@/lib/consts";
import { placeComponent } from "@/lib/store/circuit";
import {
    clearSelection,
    drag,
    getViewState,
    marquee,
    placingComponent,
    containerToWorld,
    selection,
    worldToScreen,
} from "@/lib/store/view";
import CircuitComponent from "./circuitry/CircuitComponent.vue";
import CircuitComponentPreview from "./circuitry/CircuitComponentPreview.vue";
import { componentMap } from "./circuitry";
import Wire from "./circuitry/Wire.vue";
import { scale, settings } from "@/lib/store/settings";
import { Subcircuit } from "@/lib/types";

const props = defineProps<{
    subcircuit: Subcircuit;
}>();
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

const isPanning = ref(false);
const panningStart = reactive({ x: 0, y: 0 });

const mousePosition = reactive({
    x: 0,
    y: 0,
});
const placingComponentPosition = reactive({
    x: 0,
    y: 0,
});
watch(placingComponent, () => {
    placingComponentPosition.x = null;
    placingComponentPosition.y = null;
});
watch(mousePosition, (mouse) => {
    const metadata = componentMap[placingComponent.value];
    const dimensions = metadata?.getDimensions() || { width: 1, height: 1 };

    placingComponentPosition.x = Math.floor(
        (mouse.x - offset.value.x) / GRID_SIZE / scale.value - dimensions.width / 2,
    );
    placingComponentPosition.y = Math.floor(
        (mouse.y - offset.value.y) / GRID_SIZE / scale.value - dimensions.height / 2,
    );
});

const tooltip = reactive({
    value: null as null | string,
    x: 0,
    y: 0,
});

function handleMouseDown(e: MouseEvent) {
    if (e.button === 0 && e.metaKey) {
        isPanning.value = true;
        panningStart.x = e.clientX - offset.value.x;
        panningStart.y = e.clientY - offset.value.y;
        return;
    }
    if (e.button !== 0) return;

    if (!e.shiftKey && !e.metaKey) {
        clearSelection();
    }
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const world = containerToWorld(e.clientX - rect.left, e.clientY - rect.top);
    marquee.active = true;
    marquee.start.x = world.x;
    marquee.start.y = world.y;
    marquee.current.x = world.x;
    marquee.current.y = world.y;
}

function handleMouseMove(e: MouseEvent) {
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    mousePosition.x = e.clientX - rect.left;
    mousePosition.y = e.clientY - rect.top;

    handleCanvasMove(e);
    handleComponentMove(e);
    handleMarqueeMove(e);
    handleTooltip(e.target);
}

function handleCanvasMove(e: MouseEvent) {
    if (!isPanning.value) return;
    offset.value = {
        x: e.clientX - panningStart.x,
        y: e.clientY - panningStart.y,
    };
}

function handleComponentMove(e: MouseEvent) {
    if (!drag.active) return;

    const containerRect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const world = containerToWorld(e.clientX - containerRect.left, e.clientY - containerRect.top);

    const deltaX = Math.round(world.x - drag.initialMouse.x);
    const deltaY = Math.round(world.y - drag.initialMouse.y);

    for (const [id, initial] of drag.initialPositions) {
        const comp = props.subcircuit.components.get(id);
        if (!comp) continue;

        comp.x = Math.max(initial.x + deltaX, 0);
        comp.y = Math.max(initial.y + deltaY, 0);
    }
}

function handleMarqueeMove(e: MouseEvent) {
    if (!marquee.active) return;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const world = containerToWorld(e.clientX - rect.left, e.clientY - rect.top);
    marquee.current.x = world.x;
    marquee.current.y = world.y;
}

function handleTooltip(target: EventTarget) {
    if (!("dataset" in target) || !target.dataset || typeof target.dataset !== "object") {
        return;
    }

    const element = target as SVGCircleElement;

    if (element.dataset.tooltip) {
        const rect = element.getBoundingClientRect();
        tooltip.x = rect.x + rect.width / 2;
        tooltip.y = rect.y + rect.height / 2;
        tooltip.value = element.dataset.tooltip;
    } else {
        tooltip.value = null;
    }
}

function handleMouseUp(e: MouseEvent) {
    isPanning.value = false;
    drag.active = false;
    if (marquee.active) {
        finalizeMarquee(e);
        marquee.active = false;
    }
}

function finalizeMarquee(e: MouseEvent) {
    const additive = e.shiftKey || e.metaKey;

    const left = Math.min(marquee.start.x, marquee.current.x);
    const top = Math.min(marquee.start.y, marquee.current.y);
    const right = Math.max(marquee.start.x, marquee.current.x);
    const bottom = Math.max(marquee.start.y, marquee.current.y);

    if (right - left < 1 && bottom - top < 1 && !additive) {
        clearSelection();
        return;
    }

    for (const [id, comp] of props.subcircuit.components) {
        const meta = componentMap[comp.type];
        const dims = meta.getDimensions(comp);

        if (
            rectsIntersect(
                { left, top, right, bottom },
                {
                    left: comp.x,
                    top: comp.y,
                    right: comp.x + dims.width,
                    bottom: comp.y + dims.height,
                },
            )
        ) {
            selection.value.add(id);
        }
    }
}

function rectsIntersect(
    a: { left: number; top: number; right: number; bottom: number },
    b: { left: number; top: number; right: number; bottom: number },
) {
    return a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top;
}

const marqueeStyle = computed(() => {
    const a = worldToScreen(marquee.start.x, marquee.start.y);
    const b = worldToScreen(marquee.current.x, marquee.current.y);
    return {
        left: Math.min(a.x, b.x) + "px",
        top: Math.min(a.y, b.y) + "px",
        width: Math.abs(a.x - b.x) + "px",
        height: Math.abs(a.y - b.y) + "px",
    };
});

function handleWheel(e: WheelEvent) {
    const isTrackpad = Math.abs(e.deltaY) < 50 && e.deltaMode === 0;
    const isPinchZoom = e.ctrlKey || e.metaKey;

    if (isPinchZoom) {
        // trackpad pinch sends larger deltaY values, normalize them
        const delta = isTrackpad ? e.deltaY * -0.03 : e.deltaY * -0.002;
        zoom(settings.scaleLevel + delta);
    } else {
        offset.value = {
            x: offset.value.x - e.deltaX,
            y: offset.value.y - e.deltaY,
        };
    }

    nextTick().then(() => {
        handleTooltip(e.target);
    });
}

function handleKeyDown(e: KeyboardEvent) {
    if (e.metaKey && (e.key === "-" || e.key === "=" || e.key === "+" || e.key === "0")) {
        e.preventDefault();

        const newScaleLevel = Math.round(
            e.key === "=" || e.key === "+"
                ? settings.scaleLevel + 1
                : e.key === "-"
                  ? settings.scaleLevel - 1
                  : 0,
        );

        zoom(newScaleLevel);
    } else if (e.key === "Escape") {
        placingComponent.value = null;
        clearSelection();
    }
}

function handleComponentDragStart(e: MouseEvent) {
    const rect = (e.currentTarget as HTMLElement)
        .closest("#circuit-canvas")!
        .getBoundingClientRect();
    const world = containerToWorld(e.clientX - rect.left, e.clientY - rect.top);
    drag.active = true;
    drag.initialMouse = { x: world.x, y: world.y };
    drag.initialPositions.clear();
    for (const id of selection.value) {
        const comp = props.subcircuit.components.get(id);
        if (comp) drag.initialPositions.set(id, { x: comp.x, y: comp.y });
    }
}

onMounted(() => {
    document.addEventListener("keydown", handleKeyDown);
});

onUnmounted(() => {
    document.removeEventListener("keydown", handleKeyDown);
});

function zoom(newScaleLevel: number) {
    newScaleLevel = Math.min(Math.max(-5, newScaleLevel), 10);

    const oldScale = scale.value;
    const newScale = Math.pow(1.2, newScaleLevel);

    const worldX = (mousePosition.x - offset.value.x) / oldScale;
    const worldY = (mousePosition.y - offset.value.y) / oldScale;

    settings.scaleLevel = newScaleLevel;
    offset.value = {
        x: mousePosition.x - worldX * newScale,
        y: mousePosition.y - worldY * newScale,
    };
}
</script>

<template>
    <div
        id="circuit-canvas"
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
                v-if="placingComponent && placingComponentPosition.x !== null"
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
            :style="{
                left: tooltip.x + 'px',
                top: tooltip.y + 'px',
            }"
        >
            {{ tooltip.value }}
        </div>
    </div>
</template>
