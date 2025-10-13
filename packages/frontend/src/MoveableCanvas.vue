<script setup lang="ts">
import { computed, ref } from "vue";
import { settings } from "./store";

const GRID_SIZE = 40;

let _offset = ref({ x: 0, y: 0 });
let offset = computed({
    get: () => _offset.value,
    set: (val) => {
        _offset.value.x = Math.min(val.x, 0);
        _offset.value.y = Math.min(val.y, 0);
    },
});

let isDragging = ref(false);
let dragStart = ref({ x: 0, y: 0 });

const scale = computed(() => {
    return Math.pow(1.2, settings.scaleLevel);
});

function handleMouseDown(e: MouseEvent) {
    if (!((e.button === 0 && (e.shiftKey || e.metaKey)) || e.button === 1)) return;

    isDragging.value = true;
    dragStart.value = {
        x: e.clientX - offset.value.x,
        y: e.clientY - offset.value.y,
    };
}

function handleMouseMove(e: MouseEvent) {
    if (!isDragging.value) return;
    offset.value = {
        x: e.clientX - dragStart.value.x,
        y: e.clientY - dragStart.value.y,
    };
}

function handleMouseUp() {
    isDragging.value = false;
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
                        :r="2 * scale"
                        fill="var(--color-zinc-600)"
                    />
                </pattern>
            </defs>

            <rect x="0" y="0" width="100%" height="100%" fill="url(#dotPattern)" />
        </svg>

        <div
            class="pointer-events-none absolute inset-0 h-full w-full origin-top-left"
            :style="{
                transform: `translate(${offset.x}px, ${offset.y}px) scale(${scale})`,
            }"
        >
            canvas-relative content?
        </div>
    </div>
</template>
