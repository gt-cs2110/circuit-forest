import { ComponentMetadata } from "@/lib/types";

import Constant from "./Constant.vue";

export const constant: ComponentMetadata = {
    displayName: "Constant",
    component: Constant,
    getDefaultDimensions: () => ({ width: 2, height: 2 }),
    getDefaultPorts: () => [{ x: 2, y: 1 }],
    getDimensions: (component) => {
        return {
            width: component.bounds[1].x - component.bounds[0].x,
            height: component.bounds[1].y - component.bounds[0].y,
        };
    },
    getOriginToFixedPortOffset: (component) => {
        // For 1d origin will always be bitsize away in the x direction and 1 in the y direction
        //this changes when we go above bitsize 8 need to implmenet this bc it goes 2d
        return { x: Math.max(component.bitsize, 2), y: 1 };
    },
};
