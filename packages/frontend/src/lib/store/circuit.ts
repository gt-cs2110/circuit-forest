import { Handedness, Orientation,Key, Location, TransientWireState } from "circuitsim-glue";
import { computed, ref, toRaw } from "vue";
import { toast } from "vue-sonner";
import type { CircuitComponent, ComponentType, Subcircuit } from "../types";
import { clearSelection, componentSelection, deleteViewState, placingComponent, selectComponent, wireSelection } from "./view";
import { Rect, rectsIntersect, toBounds } from "@/composables/useMarquee";

export const circuits = ref<Map<number, [Subcircuit, Subcircuit[], Subcircuit[]]>>(defaultCircuit()); //mapping from frontend id to subcircuit
export const currentSubcircuitId = ref(0);
export const currentSubcircuit = computed(() => circuits.value.get(currentSubcircuitId.value)![0]);
//Ctr+C/V State saving
export const undoStack = computed(()=>circuits.value.get(currentSubcircuitId.value)![1])
export const redoStack = computed(()=>circuits.value.get(currentSubcircuitId.value)![2])

//we need to save a list of Circuit Components and a list of wiresbased on the selection copied
export const copiedComponents: CircuitComponent[] = [];
export const copiedWires: TransientWireState[] = [];

updateState();
let nextFrontendId = 100;



export function saveState(){
   
    redoStack.value.length = 0
    undoStack.value.push(structuredClone(toRaw(currentSubcircuit.value!)))
}
export function copy(){
    copiedComponents.length = 0;
    copiedWires.length = 0;
    //go through the component and wire selection and copy these circuit components and wires into opied componets and wires variables
    componentSelection.value.forEach(frontendId=>{
        copiedComponents.push(structuredClone(toRaw(currentSubcircuit.value.components.get(frontendId))) as CircuitComponent);
    })
    wireSelection.value.forEach(wireIndex=>{
        copiedWires.push(currentSubcircuit.value.wires[wireIndex]);
    })
}

export function boxCollidesWithComponents(bounds: Rect){

    for ( const comp of currentSubcircuit.value.components){

        if (rectsIntersect(toBounds(comp[1].bounds[0],comp[1].bounds[1]), bounds))return true;

    }
    for ( const wire of currentSubcircuit.value.wires){
        if (rectsIntersect(toBounds(wire.endpoints[0],wire.endpoints[1]), bounds))return true;

    }
    
    return false;

}

// takes mouse position which is world coords of x and y
export function paste(mousePosition: Location){
    if(copiedComponents.length==0&&copiedWires.length==0)return;
    //when you paste it create the components at a displacment and grays it out, it is basically auto selected and you can drag it
    clearSelection();

    //Determine the offset we want to put the pasted component at

    //First we find the offset btwn the mouse position and the position of the top left component/wire
    let minx = copiedComponents.length>0?copiedComponents[0].bounds[0].x:copiedWires[0].endpoints[0].x;
    let miny = copiedComponents.length>0?copiedComponents[0].bounds[0].y:copiedWires[0].endpoints[0].y;
    let maxx = 0;
    let maxy = 0;
    copiedComponents.forEach(comp=>{
        minx = Math.min(comp.bounds[0].x,minx);
        miny = Math.min(comp.bounds[0].y, miny);
        maxx = Math.max(comp.bounds[1].x,maxx);
        maxy = Math.max(comp.bounds[1].y, maxy);
    })
    copiedWires.forEach(wire=>{
        minx = Math.min(wire.endpoints[0].x,minx);
        miny = Math.min(wire.endpoints[0].y, miny);
        maxx = Math.max(wire.endpoints[1].x,maxx);
        maxy = Math.max(wire.endpoints[1].y, maxy);
    })

const displacment = {
  x: Math.round(mousePosition.x - minx),
  y: Math.round(mousePosition.y - miny),
};
    const displacedWireEndpoints:Location[][] = [];


        //Keep tryign to paste diagonally downwards until we find a free space     
        while(boxCollidesWithComponents(toBounds({x:minx+displacment.x, y:miny+displacment.y}, {x:maxx+displacment.x, y:maxy+displacment.y}))){
            displacment.x+=1;
            displacment.y+=1;
        }
    


    

     //Readd all the wires
    for (const [start, end] of copiedWires.map(wire=>wire.endpoints)) {
        window.api.core.addWire(currentSubcircuit.value.backendKey, toRaw({ x: start.x + displacment.x, y: start.y + displacment.y }), 
            toRaw({ x: end.x + displacment.x, y: end.y + displacment.y }));
            displacedWireEndpoints.push([{ x: start.x + displacment.x, y: start.y + displacment.y },{ x: end.x + displacment.x, y: end.y + displacment.y }])
    }

    //Then we readd all components
    copiedComponents.forEach(component=>{
        const componentCopy = structuredClone(toRaw(component));
        componentCopy.pos.x+=displacment.x;
        componentCopy.pos.y+=displacment.y;
        componentCopy.backendKey= window.api.core.addComponent(currentSubcircuit.value.backendKey,{...toRaw(componentCopy), componentType: String(componentCopy.type).toUpperCase()});
        componentCopy.frontendId = generateFrontendId();
        currentSubcircuit.value.components.set(componentCopy.frontendId, componentCopy)
        componentSelection.value.add(componentCopy.frontendId);

    });
    saveState();
    updateState();

    //add wires to the selection
    
    displacedWireEndpoints.forEach((endpoints)=>{
        wireSelection.value.add(currentSubcircuit.value.wires.findIndex(w=>w.endpoints.every((pt, i) => pt.x === endpoints[i].x && pt.y === endpoints[i].y)));
    })
   
}

export function undo(){
    //every time update components, move selectionplace component, add wires, delete wires, delete compoentn, etc is called we push a subcircuit to the stack
    //when undo is called we push the top of stack to the redo stack and restore the subcircuit state of the new top of stack
    
    if(undoStack.value.length<=1)return;

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

    const currSub = circuits.value.get(currentSubcircuitId.value)![0];
    currSub.components = subcircuitState.components;
    currSub.wires = subcircuitState.wires;
    currSub.name = subcircuitState.name;    
    clearSelection();
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
    if(wires.length==0)return;
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
/// Batch delete function for removing components and wires in a single action
export function batchDelete(componentSelection:number[], wireSelection:number[]){
    componentSelection.forEach(frontendId=>{
         const component = currentSubcircuit.value.components.get(frontendId);
        if (component) {
            window.api.core.removeComponent(currentSubcircuit.value.backendKey, component.backendKey);
            currentSubcircuit.value.components.delete(frontendId);
        }

    })
    wireSelection.forEach(wire_id=>{
        const wire = currentSubcircuit.value.wires[wire_id];
        if (wire)window.api.core.removeWire(currentSubcircuit.value.backendKey, ...toRaw(wire.endpoints));
    })
    updateState();
    saveState();



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
    console.log("backend updates");
    console.log(transientComponentStates);
    console.log(transientWireState);
    console.log(toRaw(currentSubcircuit.value));
    console.log("____________");
}
//TODO Update Circuit State
export function generateFrontendId() {
    return nextFrontendId++;
}
