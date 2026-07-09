import { ComponentMetadata } from "@/lib/types";

import AndGate from "./AndGate.vue";

export const and: ComponentMetadata = {
    displayName: "AND",
    component: AndGate,
    getDefaultDimensions: () => ({ width: 4, height: 4 }),
    getDefaultPorts() {
        return [
            { x: 0, y: 1, label: "0" },
            { x: 0, y: 3, label: "1" },
            { x: 4, y: 2, label: "Out" },
        ];
    },
    getDimensions: (component) => {
        return {
            width: component.bounds[1].x - component.bounds[0].x,
            height: component.bounds[1].y - component.bounds[0].y,
        };
    },
    getOriginToFixedPortOffset: (component) => {
        // gate is a square, port to origin will always be the full width in the x and half the height in the y bc heigh == width
        return {
            x: component.bounds[1].x - component.bounds[0].x,
            y: (component.bounds[1].y - component.bounds[0].y) / 2,
        };
    },
};
