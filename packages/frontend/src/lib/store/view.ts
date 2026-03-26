import { computed, reactive, ref } from "vue";

import { GRID_SIZE, ORIGIN_OFFSET } from "../consts";
import { ComponentType, Location } from "../types";
import { currentSubcircuitId } from "./circuit";
import { scale } from "./settings";

type CircuitViewState = {
    selection: Set<number>;
    offset: Location; // screen coords
};

const viewStates = reactive<Map<string, CircuitViewState>>(new Map());

export function getViewState(circuitId: string): CircuitViewState {
    if (!viewStates.has(circuitId)) {
        viewStates.set(circuitId, {
            selection: new Set(),
            offset: { x: 0, y: 0 },
        });
    }
    return viewStates.get(circuitId)!;
}

export function deleteViewState(circuitId: string) {
    viewStates.delete(circuitId);
}

export const currentViewState = computed(() => getViewState(currentSubcircuitId.value));
export const selection = computed(() => currentViewState.value.selection);
export const currentOffset = computed(() => currentViewState.value.offset);

export function containerToWorld(containerX: number, containerY: number) {
    return {
        x:
            (containerX - currentOffset.value.x - ORIGIN_OFFSET * scale.value) /
            scale.value /
            GRID_SIZE,
        y:
            (containerY - currentOffset.value.y - ORIGIN_OFFSET * scale.value) /
            scale.value /
            GRID_SIZE,
    };
}

export function worldToScreen(wx: number, wy: number) {
    return {
        x: wx * GRID_SIZE * scale.value + currentOffset.value.x + ORIGIN_OFFSET * scale.value,
        y: wy * GRID_SIZE * scale.value + currentOffset.value.y + ORIGIN_OFFSET * scale.value,
    };
}

// SELECTION

export function selectComponent(id: number, additive: boolean) {
    if (!additive) selection.value.clear();
    selection.value.add(id);
}

export function deselectComponent(id: number) {
    selection.value.delete(id);
}

export function clearSelection() {
    selection.value.clear();
}

export function isSelected(id: number) {
    return selection.value.has(id);
}

// DRAG

export const drag = reactive({
    active: false,
    initialMouse: { x: 0, y: 0 }, // world coords
    initialPositions: new Map<number, Location>(), // snapshot at drag start
});

export const marquee = reactive({
    active: false,
    start: { x: 0, y: 0 }, // world coords
    current: { x: 0, y: 0 }, // world coords
});

// PLACING

export const placingComponent = ref<ComponentType | null>(null);
