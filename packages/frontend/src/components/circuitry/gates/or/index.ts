import { ComponentMetadata } from "@/lib/types";

import OrGate from "./OrGate.vue";

export const or: ComponentMetadata = {
    displayName: "OR",
    component: OrGate,
    getDimensions: () => ({ width: 4, height: 4 }),
    getPorts() {
        return [
            { x: 0, y: 1, label: "0" },
            { x: 0, y: 3, label: "1" },
            { x: 4, y: 2, label: "Out" },
        ];
    },
};
