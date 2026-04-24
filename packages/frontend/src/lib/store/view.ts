import { computed, reactive, ref } from "vue";

import { ComponentType, Location } from "../types";
import { currentSubcircuitId } from "./circuit";

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

// PLACING

export const placingComponent = ref<ComponentType | null>(null);
