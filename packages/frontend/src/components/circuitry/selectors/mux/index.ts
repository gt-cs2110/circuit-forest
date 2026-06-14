import { ComponentMetadata } from "@/lib/types";

import MuxGate from "./MuxGate.vue";
export const mux: ComponentMetadata = {
    displayName: "MUX",
    component: MuxGate,
    getDefaultDimensions: () => ({ width: 3, height: 4 }),
    getDefaultPorts() {
        return [
            { x: 0, y: 1, label: "0" },
            { x: 0, y: 3, label: "1" },
            { x: 3, y: 2, label: "Out" },
        ];
    },
    getDimensions: (component)=>{
        return ({width:component.bounds[1].x - component.bounds[0].x,height:component.bounds[1].y - component.bounds[0].y})
    }

};
