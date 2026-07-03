import { computed, reactive, ref } from "vue";

import { ComponentType, Location } from "../types";
import { currentSubcircuitId } from "./circuit";

type CircuitViewState = {
    selection: Set<number>;
    offset: Location; // screen coords
};

const viewStates = reactive<Map<number, CircuitViewState>>(new Map());

export function getViewState(circuitFrontendId: number): CircuitViewState {
    if (!viewStates.has(circuitFrontendId)) {
        viewStates.set(circuitFrontendId, {
            selection: new Set(),
            offset: { x: 0, y: 0 },
        });
    }
    return viewStates.get(circuitFrontendId)!;
}

export function deleteViewState(circuitFrontendId: number) {
    viewStates.delete(circuitFrontendId);
}

export const currentViewState = computed(() => getViewState(currentSubcircuitId.value));
export const selection = computed(() => currentViewState.value.selection);

// SELECTION

export function selectComponent(componentFrontendId: number, additive: boolean) {
    if (!additive) selection.value.clear();
    selection.value.add(componentFrontendId);
}

export function deselectComponent(componentFrontendId: number) {
    selection.value.delete(componentFrontendId);
}

export function clearSelection() {
    selection.value.clear();
}

export function isSelected(id: number) {
    return selection.value.has(id);
}

// PLACING

export const placingComponent = ref<ComponentType | null>(null);
