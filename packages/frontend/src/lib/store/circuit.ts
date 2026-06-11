import { computed, reactive, ref } from "vue";

import { CircuitComponent, ComponentType, Handedness, Orientation, Subcircuit } from "../types";
import { createTwoAndGateCircuit } from "./initialCircuit";
import { deleteViewState, placingComponent, selectComponent } from "./view";
import { TransientComponentState } from "circuitsim-glue";

export const circuits = reactive<Map<number, Subcircuit>>(createTwoAndGateCircuit());//mapping from frontend id to subcircuit
export const currentSubcircuitId = ref(0);
export const currentSubcircuit = computed(() => circuits.get(currentSubcircuitId.value)!);
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



//// Updates component representation in backend and frontend, only succeeds in frontend if succeeds in backend
export function updateComponent(frontendId: number, updates: Partial<CircuitComponent>) {
    //Retreive component from store of current circuit
    const component = currentSubcircuit.value.components.get(frontendId);
    if (!component) return;
    console.log(`Updating component with frontend id ${frontendId} to (${updates.x}, ${updates.y}) bitsize: ${updates.bitsize} inputs: ${updates.inputs}`);
   
    //Apply the changes in the Partial<Circuit Component>
    Object.assign(component, updates);
    
    //To Update the component we delete the old and add a new identical component with the changes applied

    //Remove the Old Component with this id
    window.api.core.removeComponent( BigInt(currentSubcircuit.value.backendKey),BigInt(component.backendKey));

    //Add New component with changes applied
    const backendKey = window.api.core.addComponent({ circuitKey: BigInt(currentSubcircuit.value.backendKey), componentType: String(component.type).toUpperCase(), bitsize: component.bitsize, inputs: component.inputs, orientation: component.orientation, label: component.label, x: component.x, y: component.y, labelOrientation: component.labelOrientation, handedness:component.handedness, constantValue:component.componentValue });
    
    //update backend key
    const state = currentSubcircuit.value.components.get(frontendId);
    if(state){
        state.backendKey = String(backendKey);
    }
    updateState();}

export function deleteComponent(frontendId: number){
    const circuit = circuits.get(currentSubcircuitId.value);
    if (!circuit) return;
    const component = circuit?.components.get(frontendId);
    if (!component) return;
    window.api.core.removeComponent( BigInt(currentSubcircuit.value.backendKey),BigInt(component.backendKey));
    circuit.components.delete(frontendId);
    updateState();
}

/// Adds a new component to the frontend and updates the backend
export function placeComponent(type: ComponentType, x: number, y: number) {
    if (x < 0 || y < 0) {
        placingComponent.value = null;
        return;
    }

    const frontendId = generateFrontendId();
    const new_component:CircuitComponent = {
        backendKey: "",
        frontendId:frontendId,
        type: type,
        label: "",
        bitsize: 2,
        inputs:2,
        ports:[],
        bounds:[],
        orientation:Orientation["EAST"],
        handedness:type=="buffer"?Handedness["TOPLEFT"]:Handedness["N/A"],
        labelOrientation:Orientation["EAST"],
        x:x,
        y:y, 
        selsize:2, 
        isInput:false,
        textContent:"",
        constantValue:"0"
        

    }
    const backendKey = window.api.core.addComponent({ circuitKey: BigInt(currentSubcircuit.value.backendKey), componentType:String(type).toUpperCase(),  x: new_component.x, y: new_component.y });
    new_component.backendKey = String(backendKey);

    currentSubcircuit.value.components.set(frontendId, new_component);
    selectComponent(frontendId, false);
    updateState();//will update the ports and bounds values from the backend

    placingComponent.value = null;
}

export function newSubcircuit(name?:string) {
    const frontendId = generateFrontendId();
    const backendKey = String(window.api.core.createCircuit(name ?? ("Circuit" + circuits.size)));
    circuits.set(frontendId, {
        frontendId,
        backendKey,
        name: name ?? ("Circuit" + circuits.size),
        components: new Map(),
        wires: [],
    });
    currentSubcircuitId.value = frontendId;
}

export function deleteSubcircuit(frontendId: number) {
    //TODO REMOVE on backend. TBH removing circuits is only an issue if you are using subcircuits, but honestly that could be checked and prevented in frontend
    circuits.delete(frontendId);
    deleteViewState(frontendId);
}

export function updateState() {
    //call glue functions to propagate changes to backend and update frontend state based on backend state
    window.api.core.propagate(BigInt(currentSubcircuit.value.backendKey));
    const transientStates:TransientComponentState[] = window.api.core.getTransientState(BigInt(currentSubcircuit.value.backendKey));
    transientStates.forEach(state=>{
        const corresponding_object = currentSubcircuit.value.components.values().find(component=>component.backendKey === String(state.backendKey));
        if(corresponding_object){
            Object.assign(corresponding_object, state);
        }
    })
    console.log(circuits)
}
//TODO Update Circuit State
export function generateFrontendId() {
  return nextFrontendId++;
}
