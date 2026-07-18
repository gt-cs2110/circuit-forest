import { ComponentMetadata } from "@/lib/types";

import Constant from "./Constant.vue";
export const ROW_MAX_SIZE = 8;

export const constant: ComponentMetadata = {
    displayName: "Constant",
    component: Constant,
    getDefaultDimensions: () => ({ width: 2, height: 2 }),
    getDefaultPorts: () => [{ x: 2, y: 1 }],
};
