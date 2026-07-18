import { ComponentMetadata } from "@/lib/types";

import Tunnel from "./Tunnel.vue";
export const ROW_MAX_SIZE = 8;
export const tunnel: ComponentMetadata = {
    displayName: "Tunnel",
    component: Tunnel,
    getDefaultDimensions: () => ({ width: 2, height: 2 }),
    getDefaultPorts: () => [{ x: 2, y: 1 }],
};
