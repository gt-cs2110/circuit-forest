import { ComponentMetadata } from "@/lib/types";

import Tunnel from "./Tunnel.vue";
export const ROW_MAX_SIZE = 8;
export const tunnel: ComponentMetadata = {
    displayName: "Tunnel",
    component: Tunnel,
    getDefaultDimensions: () => ({ width: 3, height: 2 }),
    getDefaultPorts: () => [{ x: 3, y: 1 }],
    getOriginToFixedPortOffset: (component) => {
        const width = component.bounds[1].x - component.bounds[0].x;

        return { x: width, y: 1 };
    },
};
