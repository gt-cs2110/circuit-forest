import { computed, reactive, ref, watch } from "vue";

import { CircuitComponent, ComponentType, Location, Subcircuit } from "../types";
import { settings } from "./settings";

export type SubcircuitState = {
    subcircuit: Subcircuit;
    selectedComponentId: number | null;
    offset: Location;
};

export const circuits = reactive<Map<string, SubcircuitState>>(
    new Map([createSubcircuit("Circuit 1")]
    ),

);



export const currentCircuitId = ref(circuits.keys().next().value);
export const currentCircuit = computed(() => {
    return circuits.get(currentCircuitId.value)!;
});

export const selectedComponentId = computed({
    get() {
        return currentCircuit.value.selectedComponentId;
    },
    set(id: number) {
        currentCircuit.value.selectedComponentId = id;
    },
});

// place selected components at end of map so that they appear on top
watch(selectedComponentId, (id) => {
    if (id === null) return;
    console.log(`Selected component with id ${id}`);
    const component = currentCircuit.value.subcircuit.components.get(id);
    if (!component) return;

    currentCircuit.value.subcircuit.components.delete(id);
    currentCircuit.value.subcircuit.components.set(id, component);
});

export const componentDrag = reactive({
    componentId: -1,
    isDragging: false,
    initialMouse: { x: 0, y: 0 },
    initialPosition: { x: 0, y: 0 },
});

export const placingComponent = ref<ComponentType | null>(null);

function randomId() {
    return Math.floor(Math.random() * Number.MAX_SAFE_INTEGER);
}

export function placeComponent(type: ComponentType, x: number, y: number) {
    if (x < 0 || y < 0) {
        placingComponent.value = null;
        return;
    }
    console.log(`Placing component of type ${type} at (${x}, ${y})`);
    //when we place a component we remove the old component if it exists and create the new component
    console.log(circuits);
    const backendKey = window.api.core.addComponent(currentCircuit.value.subcircuit.backendkey, {componentType:type.toUpperCase(), label:"",bitsize:settings.globalBitsize, inputs: 2, x:x, y:y} );
    

    const id = randomId();
    currentCircuit.value.subcircuit.components.set(id, {
        id,
        backendkey: backendKey,
        bitsize: settings.globalBitsize,
        inputs:2,
        label: "",
        type,
        x,
        y
    });
    currentCircuit.value.subcircuit.componentStates.set(id, {
        backendKey: backendKey,
        portValues: [],
        bounds: [],
        portLocations: [],
    });
    updateCircuitState();

    selectedComponentId.value = id;

    placingComponent.value = null;
}
export function updateComponent(id: number, updates: Partial<CircuitComponent>) {
    const component = currentCircuit.value.subcircuit.components.get(id);
    if (!component) return;
    console.log(`Updating component with id ${id} to (${updates.x}, ${updates.y}) bitsize: ${updates.bitsize} inputs: ${updates.inputs}`);
    Object.assign(component, updates);
    window.api.core.removeComponent( currentCircuit.value.subcircuit.backendkey,component.backendkey);
    const backendKey = window.api.core.addComponent(currentCircuit.value.subcircuit.backendkey, {componentType:component.type.toUpperCase(), label:component.label,bitsize:component.bitsize, inputs: component.inputs, x:component.x, y:component.y} );
    //update backend key
    const state = currentCircuit.value.subcircuit.componentStates.get(id);
    if(state){
        state.backendKey = backendKey;
    }
    component.backendkey = backendKey;
    updateCircuitState();
}
function createSubcircuit(name: string) {
    const id = randomId().toString();
    const key:bigint = window.api.core.createCircuit();
    return [
    id,
    {
      subcircuit: {
        name:name,
        backendkey:key,
        components: new Map(),
        componentStates: new Map(),
        wires: [],
      },
      selectedComponentId: null,
      offset: { x: 0, y: 0 },
    },
  ] as [string, SubcircuitState];
}
export function newSubcircuit() {
    const [id,circuit] = createSubcircuit("New Subcircuit");
    circuits.set(id, circuit);
    currentCircuitId.value = id;

}

export function updateCircuitState(){
    console.log("Updating circuit state");
    console.log(currentCircuit);
    const state = window.api.core.getCircuitState(currentCircuit.value.subcircuit.backendkey)
    state.components.forEach(component => {
        const {backendKey, portValues, bounds, portLocations} = component;
        const circuitComponent = Array.from(currentCircuit.value.subcircuit.componentStates.values()).find(k => k.backendKey === backendKey);
        if(circuitComponent){
            circuitComponent.portValues = portValues;
            circuitComponent.bounds = bounds;
            circuitComponent.portLocations = portLocations;
        }
    });
    //update wire states
}
