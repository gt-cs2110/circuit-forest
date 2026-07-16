import { ComponentMetadata } from "@/lib/types";

import BufferGate from "./BufferGate.vue";

export const buffer: ComponentMetadata = {
    displayName: "BUFFER",
    component: BufferGate,
    getDefaultDimensions: () => ({ width: 2, height: 2 }),
    getDefaultPorts() {
        return [
            { x: 0, y: 1, label: "In" },
            { x: 1, y: 2, label: "Enable" },
            { x: 2, y: 1, label: "Out" },
        ];
    },
    getOriginToFixedPortOffset: (component) => {
        // buffer is a square, port to origin will always be the full width in the x and half the height in the y bc heigh == width
        return {
            x: component.bounds[1].x - component.bounds[0].x,
            y: (component.bounds[1].y - component.bounds[0].y) / 2,
        };
    },
};
