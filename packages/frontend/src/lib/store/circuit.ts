import type { Handedness, Orientation, Key, Location } from "circuitsim-glue";
import { computed, ref, toRaw } from "vue";
import { toast } from "vue-sonner";
import {
    componentPropertiesMap,
    type CircuitComponent,
    type ComponentType,
    type Subcircuit,
} from "../types";
import { settings } from "./settings";
import { deleteViewState, placingComponent, selectComponent } from "./view";
import { wireId } from "../wire";

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
export function keyString(k: Key): string {
    return `${k.kind}:${k.id[0]}v${k.id[1]}`;
}

/// Updates all components with the specified frontend IDs with the specified `updates`.
export function updateComponents(frontendIds: number[], updates: Partial<CircuitComponent>) {
    // FIXME: If updates wouldn't change the component, don't invoke backend
    // Get all of the updated components.
    const components: CircuitComponent[] = [];
    for (const id of frontendIds) {
        const component = currentSubcircuit.value.components.get(id);
        if (!component) return;
        components.push(toRaw(component));
    }
    const newComponents = components.map((c) => ({ ...c, ...updates }));

    const success = window.api.core.updateComponents(
        currentSubcircuit.value.backendKey,
        newComponents.map((c) => {
            const args = {
                ...c,
                componentType: String(c.type).toUpperCase(),
            };

            return [c.backendKey, args] as const;
        }),
    );

    if (success) {
        for (let i = 0; i < frontendIds.length; i++) {
            currentSubcircuit.value.components.set(frontendIds[i], newComponents[i]);
        }
    } else {
        toast.error("Update unsuccessful", {
            description: "Component may not be placed out of bounds.",
            style: {
                background: "var(--color-error)",
                color: "var(--color-error-foreground)",
                borderColor: "var(--color-error-border)",
            },
            duration: 4000,
        });

        // Force refresh properties:
        for (let i = 0; i < frontendIds.length; i++) {
            currentSubcircuit.value.components.set(frontendIds[i], toRaw(components[i]));
        }
    }

    updateState();
}

export function moveSelection(
    componentFrontendIds: number[],
    wires: [Location, Location][],
    delta: Location,
) {
    const components = componentFrontendIds.map(
        (c) => currentSubcircuit.value.components.get(c)!.backendKey,
    );
    const move = window.api.core.moveSelection(
        currentSubcircuit.value.backendKey,
        toRaw(components),
        Array.from(wires, ([l, r]) => [toRaw(l), toRaw(r)] as const),
        toRaw(delta),
    );
    if (!move) {
        toast.error("Update unsuccessful", {
            description: "Components may not be placed out of bounds.",
            style: {
                background: "var(--color-error)",
                color: "var(--color-error-foreground)",
                borderColor: "var(--color-error-border)",
            },
            duration: 4000,
        });
    }

    updateState();
    return move;
}

export function deleteSelection(componentFrontendIds: Iterable<number>, wireIds: Iterable<string>) {
    const { backendKey, components, wires } = currentSubcircuit.value;

    for (const frontendId of componentFrontendIds) {
        const component = components.get(frontendId);
        if (!component) continue;
        window.api.core.removeComponent(backendKey, component.backendKey);
        components.delete(frontendId);
    }

    const ids = new Set(wireIds);
    for (const wire of wires) {
        if (ids.has(wireId(wire.endpoints))) {
            window.api.core.removeWire(backendKey, ...toRaw(wire.endpoints));
        }
    }

    updateState();
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
export function placeComponent(
    type: ComponentType,
    x: number,
    y: number,
    orientation: Orientation,
    handedness: Handedness,
) {
    if (x < 0 || y < 0) {
        placingComponent.value = null;
        return;
    }

    const hasBitsize = componentPropertiesMap[type]?.includes("bitsize") ?? false;
    const bitsize = hasBitsize ? settings.value.globalBitsize : 1;
    const constantValue = "0".repeat(bitsize);

    const frontendId = generateFrontendId();
    const new_component: CircuitComponent = {
        backendKey: window.api.core.addComponent(currentSubcircuit.value.backendKey, {
            componentType: String(type).toUpperCase(),
            pos: { x, y },
            orientation,
            handedness,
            ...(hasBitsize && { bitsize }),
            ...(type === "constant" && { constantValue }),
        }),
        frontendId: frontendId,
        type: type,
        label: "",
        bitsize,
        inputs: 2,
        ports: [],
        bounds: [
            { x: 0, y: 0 },
            { x: 0, y: 0 },
        ],
        orientation,
        handedness,
        labelOrientation: "East",
        pos: { x, y },
        selsize: 1,
        isInput: false,
        textContent: "",
        constantValue,
    };

    currentSubcircuit.value.components.set(frontendId, new_component);
    selectComponent(frontendId, false);
    updateState(); //will update the ports and bounds values from the backend

    placingComponent.value = null;
}
export function newSubcircuit(name?: string) {
    const frontendId = generateFrontendId();
    if (name === undefined) {
        const names = new Set(Array.from(circuits.value.values(), (c) => c.name));
        let n = circuits.value.size;
        while (names.has("New Circuit " + n)) n++;
        name = "New Circuit " + n;
    }
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
    const keyToComponent = new Map(
        Array.from(currentSubcircuit.value.components.values(), (c) => [
            keyString(c.backendKey),
            c,
        ]),
    );
    transientComponentStates.forEach((state) => {
        const corresponding_object = keyToComponent.get(keyString(state.backendKey));
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
