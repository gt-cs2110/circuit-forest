import { ComponentMetadata } from "@/lib/types";

import Mux from "./Mux.vue";
export const mux: ComponentMetadata = {
    displayName: "MUX",
    component: Mux,
    getDefaultDimensions: () => ({ width: 3, height: 4 }),
    getDefaultPorts() {
        return [
            { x: 0, y: 1, label: "0" },
            { x: 0, y: 3, label: "1" },
            { x: 1, y: 4, label: "selector" },

            { x: 3, y: 2, label: "Out" },
        ];
    },
    getDimensions: (component) => {
        return {
            width: component.bounds[1].x - component.bounds[0].x,
            height: component.bounds[1].y - component.bounds[0].y,
        };
    },
    getOriginToFixedPortOffset: (component) => {
        // In the mux the origin is alwys 3 away from the fixed point in the x, and is Math.pow(2,selsize+1)/2
        return { x: 3, y: Math.pow(2, component.selsize + 1) / 2 };
    },
};
