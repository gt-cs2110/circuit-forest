import { ComponentMetadata } from "@/lib/types";

import Demux from "./Demux.vue";
export const demux: ComponentMetadata = {
    displayName: "DEMUX",
    component: Demux,
    getDefaultDimensions: () => ({ width: 3, height: 4 }),
    getDefaultPorts() {
        return [
            { x: 3, y: 1, label: "0" },
            { x: 3, y: 3, label: "1" },
            { x: 2, y: 4, label: "selector" },

            { x: 0, y: 2, label: "Out" },
        ];
    },
    getDimensions: (component) => {
        return {
            width: component.bounds[1].x - component.bounds[0].x,
            height: component.bounds[1].y - component.bounds[0].y,
        };
    },
    getOriginToFixedPortOffset: (component) => {
        // In the Demux The Fied Port is the input and is on the same x position as origin, and is Math.pow(2,selBit) +2/2
        return { x: 0, y: Math.pow(2, component.selsize + 1) / 2 };
    },
};
