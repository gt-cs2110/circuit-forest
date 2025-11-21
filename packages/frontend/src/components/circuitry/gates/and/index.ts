import { ComponentMetadata } from "@/lib/types";

import AndGate from "./AndGate.vue";

export const and: ComponentMetadata = {
    component: AndGate,
    getDimensions: () => ({ width: 4, height: 4 }),
};
