import { ComponentMetadata } from "@/lib/types";

import Decoder from "./Decoder.vue";
export const decoder: ComponentMetadata = {
    displayName: "DECODER",
    component: Decoder,
    getDefaultDimensions: () => ({ width: 3, height: 4 }),
    getDefaultPorts() {
        return [
            { x: 3, y: 1, label: "0" },
            { x: 3, y: 3, label: "1" },
            { x: 1, y: 4, label: "selector" },
        ];
    },
    getOriginToFixedPortOffset: (component) => {
        // In the Decoder the fixed port is the selector which by default is at the bottom middle
        return { x: 1, y: Math.pow(2, component ? component.selsize + 1 : 2) };
    },
};
