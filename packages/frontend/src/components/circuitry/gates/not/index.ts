import { ComponentMetadata } from "@/lib/types";

import NotGate from "./NotGate.vue";

export const not: ComponentMetadata = {
    displayName: "NOT",
    component: NotGate,
    getDefaultDimensions: () => ({ width: 3, height: 2 }),
    getDefaultPorts() {
        return [
            { x: 0, y: 1, label: "In" },
            { x: 3, y: 1, label: "Out" },
        ];
    },
};
