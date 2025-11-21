import { ComponentMetadata } from "@/lib/types";

import OrGate from "./OrGate.vue";

export const or: ComponentMetadata = {
    component: OrGate,
    getDimensions: () => ({ width: 4, height: 4 }),
};
