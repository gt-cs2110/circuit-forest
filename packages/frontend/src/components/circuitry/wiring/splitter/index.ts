import { ComponentMetadata } from "@/lib/types";

import Splitter from "./Splitter.vue";
export const splitter: ComponentMetadata = {
    displayName: "Splitter",
    component: Splitter,
    getDefaultDimensions: () => ({ width: 2, height: 4 }),
    getDefaultPorts: () => [
        { x: 2, y: 2 },
        { x: 2, y: 4 },
        { x: 0, y: 0 },
    ],
    getOriginToFixedPortOffset: () => {
        return { x: 0, y: 0 };
    },
};
