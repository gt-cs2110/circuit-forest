import { computed, reactive, ref } from "vue";

export const settings = reactive({
    scaleLevel: 0,
    globalBitsize: 1,
});
export const scale = computed(() => {
    return Math.pow(1.2, settings.scaleLevel);
});

export type CircuitComponent = {
    id: number;
    name: string;
    bitsize: number;
    x: number;
    y: number;
};

export const selectedComponentId = ref<number | null>(null);
export const components = reactive<Map<number, CircuitComponent>>(
    new Map([
        [1, { id: 1, x: 1, y: 1, name: "Component A", bitsize: 1 }],
        [2, { id: 2, x: 6, y: 7, name: "Component B", bitsize: 1 }],
    ]),
);
