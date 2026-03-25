import { computed, reactive, ref } from "vue";

import { ComponentType, Subcircuit } from "../types";
import { initialCircuit } from "./initialCircuit";
import { settings } from "./settings";
import { deleteViewState, placingComponent, selectComponent } from "./view";

export const circuits = reactive<Map<string, Subcircuit>>(initialCircuit);
export const currentSubcircuitId = ref("circuit1");
export const currentSubcircuit = computed(() => circuits.get(currentSubcircuitId.value)!);

// // place selected components at end of map so that they appear on top
// watch(selectedComponentId, (id) => {
//     if (id === null) return;

//     const component = currentCircuit.value.subcircuit.components.get(id);
//     if (!component) return;

//     currentCircuit.value.subcircuit.components.delete(id);
//     currentCircuit.value.subcircuit.components.set(id, component);
// });

function randomId() {
    return Math.floor(Math.random() * Number.MAX_SAFE_INTEGER);
}

export function placeComponent(type: ComponentType, x: number, y: number) {
    if (x < 0 || y < 0) {
        placingComponent.value = null;
        return;
    }

    const id = randomId();
    currentSubcircuit.value.components.set(id, {
        id,
        bitsize: settings.globalBitsize,
        label: "",
        type,
        x,
        y,
    });
    selectComponent(id, false);

    placingComponent.value = null;
}

export function newSubcircuit() {
    const id = randomId().toString();
    circuits.set(id, {
        id,
        name: "New subcircuit",
        components: new Map(),
        wires: [],
    });
    currentSubcircuitId.value = id;
}

export function deleteSubcircuit(id: string) {
    circuits.delete(id);
    deleteViewState(id);
}
