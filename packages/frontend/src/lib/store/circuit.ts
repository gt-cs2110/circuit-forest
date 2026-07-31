import { Handedness, Orientation, Key, Location } from "circuitsim-glue";
import { computed, ref, toRaw } from "vue";
import { toast } from "vue-sonner";
import type { CircuitComponent, ComponentType, Subcircuit } from "../types";
import { deleteViewState, placingComponent, selectComponent } from "./view";

export const circuits = ref<Map<number, [Subcircuit, Subcircuit[], Subcircuit[]]>>(defaultCircuit()); //mapping from frontend id to subcircuit
export const currentSubcircuitId = ref(0);
export const currentSubcircuit = computed(() => circuits.value.get(currentSubcircuitId.value)![0]);
//Ctr+C/V State saving
export const undoStack = computed(()=>circuits.value.get(currentSubcircuitId.value)![1])
export const redoStack = computed(()=>circuits.value.get(currentSubcircuitId.value)![2])

updateState();
let nextFrontendId = 100;



export function saveState(){
   
    redoStack.value.length = 0
    undoStack.value.push(structuredClone(toRaw(currentSubcircuit.value!)))
    

}

export function undo(){
    //every time update components, move selectionplace component, add wires, delete wires, delete compoentn, etc is called we push a subcircuit to the stack
    //when undo is called we push the top of stack to the redo stack and restore the subcircuit state of the new top of stack
    
    if(undoStack.value.length<=1)return;
     console.log('undo snapshot:', [...undoStack.value]);

    redoStack.value.push(undoStack.value.pop()!);

    restoreSubcircuitState(structuredClone(toRaw(undoStack.value[undoStack.value.length-1])))

}
export function redo(){

    if(redoStack.value.length ==0)return;
    undoStack.value.push(redoStack.value.pop()!);
     
    restoreSubcircuitState(structuredClone(toRaw(undoStack.value[undoStack.value.length-1])))

}
export function restoreSubcircuitState(subcircuitState:Subcircuit){

    //to restore a state we cleare the circuit
    window.api.core.clearCircuit(currentSubcircuit.value.backendKey);

    //Then we readd all wires
    for (const [start, end] of subcircuitState.wires.map(wire=>wire.endpoints)) {
        window.api.core.addWire(currentSubcircuit.value.backendKey, toRaw(start), toRaw(end));
    }

    //Then we readd all components
    subcircuitState.components.forEach(component=>{
        component.backendKey= window.api.core.addComponent(currentSubcircuit.value.backendKey,{...toRaw(component), componentType: String(component.type).toUpperCase()});

    });

    circuits.value.get(currentSubcircuitId.value)![0] = subcircuitState;
    updateState();

}

export function defaultCircuit(): Map<number, [Subcircuit, Subcircuit[],Subcircuit[]]> {
    const name = "Circuit";
    const circuitKey = window.api.core.createCircuit(name);

    const subcircuit: Subcircuit = {
        frontendId: 0,
        backendKey: circuitKey,
        name,
        components: new Map<number, CircuitComponent>(),
        wires: [],
    };

    return new Map([[0, [subcircuit,[structuredClone(toRaw(subcircuit))],[]]]]);
}

export function keyEquals(k: Key, j: Key): boolean {
    return k.kind == j.kind && k.id[0] == j.id[0] && k.id[1] == j.id[1];
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
        toast.error("Update Unsucessful", {
            description: "Make sure updates do not move components out of bounds.",
            style: {
                background: "#ef4444",
                color: "#ffffff",
                borderColor: "#dc2626",
            },
            duration: 4000,
        });

        // Force refresh properties:
        for (let i = 0; i < frontendIds.length; i++) {
            currentSubcircuit.value.components.set(frontendIds[i], toRaw(components[i]));
        }
    }
    if(success)        saveState();

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
    if(move)saveState();
}

export function deleteComponent(frontendId: number) {
    const component = currentSubcircuit.value.components.get(frontendId);
    if (component) {
        window.api.core.removeComponent(currentSubcircuit.value.backendKey, component.backendKey);
        currentSubcircuit.value.components.delete(frontendId);
        updateState();
        saveState();

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
    saveState();

}
export function addWires(wires: (readonly [Location, Location])[]) {
    for (const [start, end] of wires) {
        window.api.core.addWire(currentSubcircuit.value.backendKey, toRaw(start), toRaw(end));
    }
    updateState();
    saveState();

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

    const frontendId = generateFrontendId();
    const new_component: CircuitComponent = {
        backendKey: window.api.core.addComponent(currentSubcircuit.value.backendKey, {
            componentType: String(type).toUpperCase(),
            pos: { x, y },
            orientation,
            handedness,
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
        orientation,
        handedness,
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
    saveState();

    placingComponent.value = null;
}
export function newSubcircuit(name?: string) {
    const frontendId = generateFrontendId();
    name ??= "Circuit" + circuits.value.size;
    const backendKey = window.api.core.createCircuit(name);
    const subcircuit ={
        frontendId,
        backendKey,
        name,
        components: new Map(),
        wires: [],
    };
    circuits.value.set(frontendId, [subcircuit,[structuredClone(toRaw(subcircuit))],[]]);
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
