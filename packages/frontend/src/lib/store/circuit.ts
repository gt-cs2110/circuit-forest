import { computed, reactive, ref, watch } from "vue";

import { ComponentType, Location, Subcircuit } from "../types";
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



/**
 * [
            "circuit1",
            {
                subcircuit: {
                    name: "Main Circuit",
                    components: new Map([
                        [
                            1,
                            {
                                id: 1,
                                type: "nand",
                                x: 1,
                                y: 1,
                                label: "Component A",
                                bitsize: 1,
                            },
                        ],
                        [
                            2,
                            {
                                id: 2,
                                type: "constant",
                                x: 6,
                                y: 7,
                                label: "Component B",
                                bitsize: 1,
                            },
                        ],
                        [
                            3,
                            {
                                id: 3,
                                type: "or",
                                x: 17,
                                y: 9,
                                label: "Component C",
                                bitsize: 1,
                            },
                        ],
                    ]),
                    wires: [{ x: 5, y: 3, direction: "H", length: 5 }],
                },
                selectedComponentId: null,
                offset: { x: 0, y: 0 },
            },
        ],
        [
            "circuit2",
            {
                subcircuit: {
                    name: "Second Circuit",
                    components: new Map([
                        [
                            1,
                            {
                                id: 1,
                                type: "and",
                                x: 1,
                                y: 10,
                                label: "Component A",
                                bitsize: 1,
                            },
                        ],
                        [
                            2,
                            {
                                id: 2,
                                type: "or",
                                x: 7,
                                y: 6,
                                label: "Component B",
                                bitsize: 1,
                            },
                        ],
                        [
                            3,
                            {
                                id: 3,
                                type: "constant",
                                x: 9,
                                y: 13,
                                label: "Component C",
                                bitsize: 1,
                            },
                        ],
                    ]),
                    wires: [],
                },
                selectedComponentId: null,
                offset: { x: 0, y: 0 },
            },
        ],
 */

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
    const backendKey = window.api.glue.addComponent(currentCircuit.value.subcircuit.backendkey, {componentType:type.toUpperCase(), label:"",bitsize:settings.globalBitsize, inputs: 2, x:x, y:y} );
    

    const id = randomId();
    currentCircuit.value.subcircuit.components.set(id, {
        id,
        backendkey: backendKey,
        bitsize: settings.globalBitsize,
        label: "",
        type,
        x,
        y,
    });
    selectedComponentId.value = id;

    placingComponent.value = null;
}
function createSubcircuit(name: string) {
    const id = randomId().toString();
    const key:bigint = window.api.glue.createCircuit();
    return [
    id,
    {
      subcircuit: {
        name:name,
        backendkey:key,
        components: new Map(),
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
