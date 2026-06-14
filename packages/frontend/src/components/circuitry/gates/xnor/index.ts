import { ComponentMetadata } from "@/lib/types";

import XnorGate from "./XnorGate.vue";

export const xnor: ComponentMetadata = {
    displayName: "XNOR",
    component: XnorGate,
    getDefaultDimensions: () => ({ width: 4, height: 4 }),
    getDefaultPorts() {
        return [
            { x: 0, y: 1, label: "0" },
            { x: 0, y: 3, label: "1" },
            { x: 4, y: 2, label: "Out" },
        ];
    },
    getDimensions: (component)=>{
        return ({width:component.bounds[1].x - component.bounds[0].x,height:component.bounds[1].y - component.bounds[0].y})
    }
};
