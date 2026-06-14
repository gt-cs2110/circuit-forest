import { ComponentMetadata } from "@/lib/types";

import NotGate from "./NotGate.vue";

export const not: ComponentMetadata = {
    displayName: "NOT",
    component: NotGate,
    getDefaultDimensions: () => ({ width: 3, height: 2 }),
    getDefaultPorts() {
        return [
            { x: 0, y: 1, label: "In" },
            { x: 3, y: 1, label: "Out" },
        ];
    },
    getDimensions: (component)=>{
        return ({width:component.bounds[1].x - component.bounds[0].x,height:component.bounds[1].y - component.bounds[0].y})
    }
};
