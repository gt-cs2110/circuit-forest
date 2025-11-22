import { computed, reactive, ref, watch } from "vue";

import { ComponentType, Location, Subcircuit } from "./types";

export const settings = reactive({
    scaleLevel: 0,
    globalBitsize: 1,
});
export const scale = computed(() => {
    return Math.pow(1.2, settings.scaleLevel);
});

export type SubcircuitState = {
    subcircuit: Subcircuit;
    selectedComponentId: number | null;
    offset: Location;
};

export const circuits = reactive<Map<string, SubcircuitState>>(
    new Map([
        [
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
                    wires: [],
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
    ]),
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

    const id = randomId();
    currentCircuit.value.subcircuit.components.set(id, {
        id,
        bitsize: settings.globalBitsize,
        label: "",
        type,
        x,
        y,
    });
    selectedComponentId.value = id;

    placingComponent.value = null;
}

export function newSubcircuit() {
    const id = randomId().toString();
    circuits.set(id, {
        subcircuit: {
            name: "New subcircuit",
            components: new Map(),
            wires: [],
        },
        offset: { x: 0, y: 0 },
        selectedComponentId: null,
    });
    currentCircuitId.value = id;
}
