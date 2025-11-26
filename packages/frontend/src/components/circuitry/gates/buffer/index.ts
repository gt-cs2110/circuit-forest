import { ComponentMetadata } from "@/lib/types";

import BufferGate from "./BufferGate.vue";

export const buffer: ComponentMetadata = {
    displayName: "BUFFER",
    component: BufferGate,
    getDimensions: () => ({ width: 2, height: 2 }),
    getPorts() {
        return [
            { x: 0, y: 1, label: "In" },
            { x: 1, y: 2, label: "Enable" },
            { x: 2, y: 1, label: "Out" },
        ];
    },
};
