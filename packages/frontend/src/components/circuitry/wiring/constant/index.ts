import { ComponentMetadata } from "@/lib/types";

import Constant from "./Constant.vue";
export const ROW_MAX_SIZE = 8;

export const constant: ComponentMetadata = {
    displayName: "Constant",
    component: Constant,
    getDefaultDimensions: () => ({ width: 2, height: 2 }),
    getDefaultPorts: () => [{ x: 2, y: 1 }],
    getOriginToFixedPortOffset: (component) => {
        // For 1d origin will always be bitsize away in the x direction and 1 in the y direction
        //this changes when we go above bitsize 8 need to implmenet this bc it goes 2d
        const height = Math.max(Math.floor((component.bitsize - 1) / ROW_MAX_SIZE), 0) * 2 + 2;

        return { x: Math.min(Math.max(component.bitsize, 2), ROW_MAX_SIZE), y: height / 2 };
    },
};
