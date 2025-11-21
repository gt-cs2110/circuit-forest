import { ComponentMetadata } from "@/lib/types";

import Constant from "./Constant.vue";

export const constant: ComponentMetadata = {
    component: Constant,
    getDimensions: () => ({ width: 2, height: 2 }),
    getPorts: () => [{ x: 2, y: 1 }],
};
