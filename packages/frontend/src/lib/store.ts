import { computed, reactive, ref, watch } from "vue";

import { CircuitComponent } from "./types";

export const settings = reactive({
    scaleLevel: 0,
    globalBitsize: 1,
});
export const scale = computed(() => {
    return Math.pow(1.2, settings.scaleLevel);
});

export const selectedComponentId = ref<number | null>(null);
export const components = reactive<Map<number, CircuitComponent>>(
    new Map([
        [1, { id: 1, type: "and", x: 1, y: 1, name: "Component A", bitsize: 1 }],
        [2, { id: 2, type: "constant", x: 6, y: 7, name: "Component B", bitsize: 1 }],
        [3, { id: 3, type: "or", x: 17, y: 9, name: "Component C", bitsize: 1 }],
    ]),
);

// place selected components at end of map so that they appear on top
watch(selectedComponentId, (id) => {
    if (id === null) return;

    const component = components.get(id);
    if (!component) return;

    components.delete(id);
    components.set(id, component);
});

export const componentDrag = reactive({
    componentId: -1,
    isDragging: false,
    initialMouse: { x: 0, y: 0 },
    initialPosition: { x: 0, y: 0 },
});
