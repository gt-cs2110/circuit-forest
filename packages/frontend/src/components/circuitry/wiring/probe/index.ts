import { ComponentMetadata } from "@/lib/types";

import Probe from "./Probe.vue";
export const ROW_MAX_SIZE = 8;
export const probe: ComponentMetadata = {
    displayName: "Probe",
    component: Probe,
    getDefaultDimensions: () => ({ width: 2, height: 2 }),
    getDefaultPorts: () => [{ x: 2, y: 1 }],
    getDimensions: (component) => {
        return {
            width: component.bounds[1].x - component.bounds[0].x,
            height: component.bounds[1].y - component.bounds[0].y,
        };
    },
    getOriginToFixedPortOffset: (component) => {
        const width =
            component.bitsize <= ROW_MAX_SIZE
                ? Math.max(component.bitsize - 2, 0) + 2
                : ROW_MAX_SIZE;
        const height = Math.max(Math.floor((component.bitsize - 1) / ROW_MAX_SIZE), 0) * 2 + 2;
        return { x: width, y: height / 2 };
    },
};
