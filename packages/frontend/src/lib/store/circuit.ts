import { Key, Location } from "circuitsim-glue";
import { computed, ref, toRaw } from "vue";
import { toast } from "vue-sonner";
import type { CircuitComponent, ComponentType, Subcircuit } from "../types";
import { deleteViewState, placingComponent, selectComponent } from "./view";

export const circuits = ref<Map<number, Subcircuit>>(defaultCircuit()); //mapping from frontend id to subcircuit
export const currentSubcircuitId = ref(0);
export const currentSubcircuit = computed(() => circuits.value.get(currentSubcircuitId.value)!);
updateState();
let nextFrontendId = 100;

export function defaultCircuit(): Map<number, Subcircuit> {
    const name = "Circuit";
    const circuitKey = window.api.core.createCircuit(name);

    const subcircuit: Subcircuit = {
        frontendId: 0,
        backendKey: circuitKey,
        name,
        components: new Map<number, CircuitComponent>(),
        wires: [],
    };

    return new Map([[0, subcircuit]]);
}

export function keyEquals(k: Key, j: Key): boolean {
    return k.kind == j.kind && k.id[0] == j.id[0] && k.id[1] == j.id[1];
}

//// Updates component representation in backend and frontend, only succeeds in frontend if succeeds in backend
export function updateComponent(frontendId: number, updates: Partial<CircuitComponent>) {
    //Retreive component from store of current circuit
    const component = currentSubcircuit.value.components.get(frontendId);
    if (!component) return;
    console.log("Update request:", updates);

    // FIXME: If updates wouldn't change the component, don't invoke backend
    // Create object with new changes:
    console.log(`Updating component with frontend id ${frontendId}`);
    const newComponent = Object.assign({}, toRaw(component), updates);

    const args = {
        ...newComponent,
        circuitKey: currentSubcircuit.value.backendKey,
        componentType: String(component.type).toUpperCase(),
    };
    if (window.api.core.validatePlacement(args)) {
        // Remove the original version of this component:
        window.api.core.removeComponent(currentSubcircuit.value.backendKey, component.backendKey);
        // Add new component with changes applied & update backend key.
        currentSubcircuit.value.components.set(frontendId, {
            ...newComponent,
            backendKey: window.api.core.addComponent({
                ...newComponent,
                circuitKey: currentSubcircuit.value.backendKey,
                componentType: String(component.type).toUpperCase(),
            }),
        });
    } else {
        toast.error("Update Unsucessful", {
            description: "Make sure not to place component out of bounds.",
            style: {
                background: "#ef4444",
                color: "#ffffff",
                borderColor: "#dc2626",
            },
            duration: 4000,
        });
        //force referseh properties
        currentSubcircuit.value.components.set(frontendId, { ...component });
    }

    updateState();
}

export function moveSelection(componentFrontendIds: number[], wires: [Location, Location][], delta: Location) {
    const components = componentFrontendIds
        .map(c => currentSubcircuit.value.components.get(c)!.backendKey);
    const move = window.api.core.moveSelection(
        currentSubcircuit.value.backendKey,
        toRaw(components),
        Array.from(wires, ([l, r]) => [toRaw(l), toRaw(r)] as const),
        toRaw(delta),
    );
    if (!move) {
        toast.error("Update Unsucessful", {
            description: "Make sure not to place component out of bounds.",
            style: {
                background: "#ef4444",
                color: "#ffffff",
                borderColor: "#dc2626",
            },
            duration: 4000,
        });
    }

    updateState();
}

export function deleteComponent(frontendId: number) {
    const component = currentSubcircuit.value.components.get(frontendId);
    if (component) {
        window.api.core.removeComponent(currentSubcircuit.value.backendKey, component.backendKey);
        currentSubcircuit.value.components.delete(frontendId);
        updateState();
    }
}
//wires have to be delted in batches
export function deleteWiresFromIds(wires: number[]) {
    deleteWires(wires.map((id) => currentSubcircuit.value.wires[id].endpoints));
}
export function deleteWires(wires: [Location, Location][]) {
    for (const endpoints of wires) {
        window.api.core.removeWire(currentSubcircuit.value.backendKey, ...toRaw(endpoints));
    }
    updateState();
}
export function addWires(wires: (readonly [Location, Location])[]) {
    for (const [start, end] of wires) {
        window.api.core.addWire(currentSubcircuit.value.backendKey, toRaw(start), toRaw(end));
    }
    updateState();
}
export function addPolyWire(points: Location[]) {
    if (points.length <= 1) return;

    const wires = Array.from(
        { length: points.length - 1 },
        (_, i) => [toRaw(points[i]), toRaw(points[i + 1])] as const,
    );
    addWires(wires);
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
            pos: { x, y },
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
        handedness: "DownRight",
        labelOrientation: "East",
        pos: { x, y },
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
