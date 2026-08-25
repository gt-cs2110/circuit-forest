import { ComponentMetadata } from "@/lib/types";

import Multiplier from "./Multiplier.vue";

export const multiplier: ComponentMetadata = {
    displayName: "Multiplier",
    component: Multiplier,
    getDefaultDimensions: () => ({ width: 4, height: 4 }),
    getDefaultPorts() {
        return [
            { x: 0, y: 1, label: "A" },
            { x: 0, y: 3, label: "B" },
            { x: 2, y: 0, label: "CarryIn" },
            { x: 2, y: 4, label: "UpperBits" },
            { x: 4, y: 2, label: "Out" },//lower bits
        ];
    },
};
