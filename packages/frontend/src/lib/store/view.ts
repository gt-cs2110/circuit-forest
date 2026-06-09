import { computed, reactive, ref } from "vue";

import { ComponentType, Location } from "../types";
import { currentSubcircuitId } from "./circuit";

type CircuitViewState = {
    selection: Set<number>;
    offset: Location; // screen coords
};

const viewStates = reactive<Map<number, CircuitViewState>>(new Map());

export function getViewState(cricuitFrontEndId: number): CircuitViewState {
    if (!viewStates.has(cricuitFrontEndId)) {
        viewStates.set(cricuitFrontEndId, {
            selection: new Set(),
            offset: { x: 0, y: 0 },
        });
    }
    return viewStates.get(cricuitFrontEndId)!;
}

export function deleteViewState(cricuitFrontEndId: number) {
    viewStates.delete(cricuitFrontEndId);
}

export const currentViewState = computed(() => getViewState(currentSubcircuitId.value));
export const selection = computed(() => currentViewState.value.selection);

// SELECTION

export function selectComponent(componentFrontEndId: number, additive: boolean) {
    if (!additive) selection.value.clear();
    selection.value.add(componentFrontEndId);
}

export function deselectComponent(componentFrontEndId: number) {
    selection.value.delete(componentFrontEndId);
}

export function clearSelection() {
    selection.value.clear();
}

export function isSelected(id: number) {
    return selection.value.has(id);
}

// PLACING

export const placingComponent = ref<ComponentType | null>(null);
