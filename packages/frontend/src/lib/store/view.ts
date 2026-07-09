import type { Location } from "circuitsim-glue";
import { computed, ref } from "vue";

import type { ComponentType } from "../types";
import { currentSubcircuitId } from "./circuit";

type CircuitViewState = {
    componentSelection: Set<number>;
    wireSelection: Set<number>;
    offset: Location; // screen coords
};

const viewStates = ref<Map<number, CircuitViewState>>(new Map());

export function getViewState(circuitFrontendId: number): CircuitViewState {
    if (!viewStates.value.has(circuitFrontendId)) {
        viewStates.value.set(circuitFrontendId, {
            componentSelection: new Set(),
            wireSelection: new Set(),
            offset: { x: 0, y: 0 },
        });
    }
    return viewStates.value.get(circuitFrontendId)!;
}

export function deleteViewState(circuitFrontendId: number) {
    viewStates.value.delete(circuitFrontendId);
}

export const currentViewState = computed(() => getViewState(currentSubcircuitId.value));
export const componentSelection = computed(() => currentViewState.value.componentSelection);
export const wireSelection = computed(() => currentViewState.value.wireSelection);

// SELECTION

export function selectComponent(componentFrontendId: number, additive: boolean) {
    if (!additive) clearSelection();
    componentSelection.value.add(componentFrontendId);
}
export function selectWire(wireId: number, additive: boolean) {
    if (!additive) clearSelection();
    wireSelection.value.add(wireId);
}

export function deselectComponent(componentFrontendId: number) {
    componentSelection.value.delete(componentFrontendId);
}
export function deselectWire(wireId: number) {
    wireSelection.value.delete(wireId);
}

export function clearSelection() {
    componentSelection.value.clear();
    wireSelection.value.clear();
}

export function isComponentSelected(id: number) {
    return componentSelection.value.has(id);
}
export function isWireSelected(id: number) {
    console.log("checking to see if contains");
    console.log(id);
    console.log(wireSelection);
    return wireSelection.value.has(id);
}

// PLACING

export const placingComponent = ref<ComponentType | null>(null);
