<script setup lang="ts">
import { computed, nextTick, reactive, ref } from "vue";
import { GRID_SIZE } from "@/lib/consts";
import { componentDrag, components, scale, selectedComponentId, settings } from "@/lib/store";
import CircuitComponent from "./CircuitComponent.vue";

const ORIGIN_OFFSET = GRID_SIZE / 2;

const _offset = ref({ x: 0, y: 0 });
const offset = computed({
    get: () => _offset.value,
    set: (val) => {
        _offset.value.x = Math.min(val.x, 0);
        _offset.value.y = Math.min(val.y, 0);
    },
});

const isDragging = ref(false);
const dragStart = reactive({ x: 0, y: 0 });

const tooltip = reactive({
    value: null as null | string,
    x: 0,
    y: 0,
});

function handleMouseDown(e: MouseEvent) {
    if (!((e.button === 0 && (e.shiftKey || e.metaKey)) || e.button === 1)) {
        if (e.button === 0) {
            selectedComponentId.value = null;
        }

        return;
    }

    isDragging.value = true;
    dragStart.x = e.clientX - offset.value.x;
    dragStart.y = e.clientY - offset.value.y;
}

function handleMouseMove(e: MouseEvent) {
    handleCanvasMove(e);
    handleComponentMove(e);
    handleTooltip(e.target);
}

function handleCanvasMove(e: MouseEvent) {
    if (!isDragging.value) return;
    offset.value = {
        x: e.clientX - dragStart.x,
        y: e.clientY - dragStart.y,
    };
}

function handleComponentMove(e: MouseEvent) {
    if (!componentDrag.isDragging) return;

    const deltaX = Math.round((e.clientX - componentDrag.initialMouse.x) / GRID_SIZE / scale.value);
    const newX = Math.max(deltaX + componentDrag.initialPosition.x, 0);
    const deltaY = Math.round((e.clientY - componentDrag.initialMouse.y) / GRID_SIZE / scale.value);
    const newY = Math.max(deltaY + componentDrag.initialPosition.y, 0);

    components.get(componentDrag.componentId).location.x = newX;
    components.get(componentDrag.componentId).location.y = newY;
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

function handleMouseUp() {
    isDragging.value = false;
    componentDrag.isDragging = false;
}

function handleWheel(e: WheelEvent) {
    const isTrackpad = Math.abs(e.deltaY) < 50 && e.deltaMode === 0;
    const isPinchZoom = e.ctrlKey || e.metaKey;

    if (isPinchZoom) {
        e.preventDefault();

        // trackpad pinch sends larger deltaY values, normalize them
        const delta = isTrackpad ? e.deltaY * -0.03 : e.deltaY * -0.002;
        const newScaleLevel = Math.min(Math.max(-5, settings.scaleLevel + delta), 10);

        const oldScale = scale.value;
        const newScale = Math.pow(1.2, newScaleLevel);

        const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;

        const worldX = (mouseX - offset.value.x) / oldScale;
        const worldY = (mouseY - offset.value.y) / oldScale;

        settings.scaleLevel = newScaleLevel;
        offset.value = {
            x: mouseX - worldX * newScale,
            y: mouseY - worldY * newScale,
        };
    } else {
        e.preventDefault();
        offset.value = {
            x: offset.value.x - e.deltaX,
            y: offset.value.y - e.deltaY,
        };
    }

    nextTick().then(() => {
        handleTooltip(e.target);
    });
}
</script>

<template>
    <div
        class="relative flex-1 overflow-hidden bg-zinc-950 text-zinc-200"
        :style="{ cursor: isDragging ? 'grabbing' : 'default' }"
        @mousedown="handleMouseDown"
        @mousemove="handleMouseMove"
        @mouseup="handleMouseUp"
        @mouseleave="handleMouseUp"
        @wheel="handleWheel"
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
                        fill="var(--color-zinc-500)"
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
                v-for="[id, component] in components"
                :key="id"
                :component="component"
            />
        </svg>

        <div
            v-if="tooltip.value"
            class="pointer-events-none fixed z-50 -mt-4 w-max -translate-x-1/2 -translate-y-full border-2 border-blue-800 bg-blue-600 px-2 font-mono text-sm text-white"
            :style="{
                left: tooltip.x + 'px',
                top: tooltip.y + 'px',
            }"
        >
            {{ tooltip.value }}
        </div>
    </div>
</template>
