import { Key, Location } from "circuitsim-glue";
import { computed, ref, toRaw } from "vue";

import type { CircuitComponent, ComponentType, Subcircuit, Wire } from "../types";
import { createTwoAndGateCircuit } from "./initialCircuit";
import { deleteViewState, placingComponent, selectComponent, wireSelection } from "./view";

export const circuits = ref<Map<number, Subcircuit>>(createTwoAndGateCircuit()); //mapping from frontend id to subcircuit
export const currentSubcircuitId = ref(0);
export const currentSubcircuit = computed(() => circuits.value.get(currentSubcircuitId.value)!);
updateState();
let nextFrontendId = 100;

// // place selected components at end of map so that they appear on top
// watch(selectedComponentId, (id) => {
//     if (id === null) return;

//     const component = currentCircuit.value.subcircuit.components.get(id);
//     if (!component) return;

//     currentCircuit.value.subcircuit.components.delete(id);
//     currentCircuit.value.subcircuit.components.set(id, component);
// });

export function keyEquals(k: Key, j: Key): boolean {
    return k.kind == j.kind && k.id[0] == j.id[0] && k.id[1] == j.id[1];
}

//// Updates component representation in backend and frontend, only succeeds in frontend if succeeds in backend
export function updateComponent(frontendId: number, updates: Partial<CircuitComponent>) {
    //Retreive component from store of current circuit
    const component = currentSubcircuit.value.components.get(frontendId);
    if (!component) return;
    console.log(`Updating component with frontend id ${frontendId}`);
    console.log(updates);

    //Apply the changes in the Partial<Circuit Component>
    Object.assign(component, updates);

    //To Update the component we delete the old and add a new identical component with the changes applied

    //Remove the Old Component with this id
    window.api.core.removeComponent(currentSubcircuit.value.backendKey, component.backendKey);

    ///Add new component with changes applied & update backend key
    const state = currentSubcircuit.value.components.get(frontendId);
    if (state) {
        state.backendKey = window.api.core.addComponent({
            ...toRaw(component),
            circuitKey: currentSubcircuit.value.backendKey,
            componentType: String(component.type).toUpperCase(),
        });
    }
    updateState();
}

export function deleteComponent(frontendId: number) {
    const circuit = circuits.value.get(currentSubcircuitId.value);
    if (!circuit) return;
    const component = circuit?.components.get(frontendId);
    if (!component) return;
    window.api.core.removeComponent(currentSubcircuit.value.backendKey, component.backendKey);
    circuit.components.delete(frontendId);
    updateState();
}
//wires have to be delted in batches
export function deleteWires(ids: number[]) {
    const circuit = circuits.value.get(currentSubcircuitId.value);
    if (!circuit) return;
    ids.forEach((wireId) => {
        const wire = circuit?.wires.at(wireId);
        if (!wire) return;
        window.api.core.removeWire(currentSubcircuit.value.backendKey, ...toRaw(wire.endpoints));
    });
    wireSelection.value.clear();
    updateState();
}
export function addWires(wires: Wire[]) {
    const circuit = circuits.value.get(currentSubcircuitId.value);
    if (!circuit) return;
    wires.forEach((wire) => {
        window.api.core.addWire(currentSubcircuit.value.backendKey, ...wire.endpoints);
    });
    wireSelection.value.clear();
    updateState();
}

/// Adds a new component to the frontend and updates the backend
export function placeComponent(type: ComponentType, x: number, y: number) {
    if (x < 0 || y < 0) {
        placingComponent.value = null;
        return;
    }

    const frontendId = generateFrontendId();
    const new_component: CircuitComponent = {
        backendKey: window.api.core.addComponent({
            circuitKey: currentSubcircuit.value.backendKey,
            componentType: String(type).toUpperCase(),
            x,
            y,
        }),
        frontendId: frontendId,
        type: type,
        label: "",
        bitsize: 1,
        inputs: 2,
        ports: [],
        bounds: [
            { x: 0, y: 0 },
            { x: 0, y: 0 },
        ],
        orientation: "East",
        handedness: "TopLeft",
        labelOrientation: "East",
        x,
        y,
        selsize: 1,
        isInput: false,
        textContent: "",
        constantValue: "0",
    };

    currentSubcircuit.value.components.set(frontendId, new_component);
    selectComponent(frontendId, false);
    updateState(); //will update the ports and bounds values from the backend

    placingComponent.value = null;
}
export function addWire(start: Location, end: Location) {
    console.log("adding");
    console.log(
        window.api.core.addWire(currentSubcircuit.value.backendKey, toRaw(start), toRaw(end)),
    );
    updateState();
}
export function newSubcircuit(name?: string) {
    const frontendId = generateFrontendId();
    name ??= "Circuit" + circuits.value.size;
    const backendKey = window.api.core.createCircuit(name);
    circuits.value.set(frontendId, {
        frontendId,
        backendKey,
        name,
        components: new Map(),
        wires: [],
    });
    currentSubcircuitId.value = frontendId;
}

export function deleteSubcircuit(frontendId: number) {
    //TODO REMOVE on backend. TBH removing circuits is only an issue if you are using subcircuits, but honestly that could be checked and prevented in frontend
    circuits.value.delete(frontendId);
    deleteViewState(frontendId);
    updateState();
}

export function updateState() {
    //call glue functions to propagate changes to backend and update frontend state based on backend state
    window.api.core.propagate(currentSubcircuit.value.backendKey);
    const [transientComponentStates, transientWireState] = window.api.core.getTransientState(
        currentSubcircuit.value.backendKey,
    );
    transientComponentStates.forEach((state) => {
        const corresponding_object = currentSubcircuit.value.components
            .values()
            .find((component) => keyEquals(component.backendKey, state.backendKey));
        if (corresponding_object) {
            Object.assign(corresponding_object, state);
        }
    });
    currentSubcircuit.value.wires = transientWireState;

    console.log("______UPDATING STATE_____");
    console.log(toRaw(currentSubcircuit.value));
    console.log("backend updates");
    console.log(transientComponentStates);
    console.log(transientWireState);
    console.log("____________");
}
//TODO Update Circuit State
export function generateFrontendId() {
    return nextFrontendId++;
}
