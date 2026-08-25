import { ComponentMetadata } from "@/lib/types";

import Divider from "./Divider.vue";

export const divider: ComponentMetadata = {
    displayName: "Divider",
    component: Divider,
    getDefaultDimensions: () => ({ width: 4, height: 4 }),
    getDefaultPorts() {
        return [
            { x: 0, y: 1, label: "A high" },
            { x: 0, y: 3, label: "B" },
            { x: 2, y: 0, label: "A low" },
            { x: 2, y: 4, label: "Remainder" },

            { x: 4, y: 2, label: "Quotient" },
        ];
    },
};
