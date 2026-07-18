import { ComponentMetadata } from "@/lib/types";

import Probe from "./Probe.vue";
export const ROW_MAX_SIZE = 8;
export const probe: ComponentMetadata = {
    displayName: "Probe",
    component: Probe,
    getDefaultDimensions: () => ({ width: 2, height: 2 }),
    getDefaultPorts: () => [{ x: 2, y: 1 }],
};
