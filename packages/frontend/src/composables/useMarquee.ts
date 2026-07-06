import { Location } from "circuitsim-glue";
import { ComputedRef, Reactive, reactive } from "vue";

import { componentMap } from "@/components/circuitry";
import type { Subcircuit } from "@/lib/types";

type Rect = { left: number; top: number; right: number; bottom: number };
type Selectable = { id: number; bounds: Rect };

export function useMarquee(subcircuit: Reactive<Subcircuit>, componentSelection: ComputedRef<Set<number>>, wireSelection: ComputedRef<Set<number>>) {
    const marquee = reactive({
        active: false,
        start: { x: 0, y: 0 },
        current: { x: 0, y: 0 },
    });

    function startMarquee(worldX: number, worldY: number, additive: boolean) {
        if (!additive) {componentSelection.value.clear();wireSelection.value.clear();}

        marquee.active = true;
        marquee.start.x = worldX;
        marquee.start.y = worldY;
        marquee.current.x = worldX;
        marquee.current.y = worldY;
    }

    function updateMarquee(worldX: number, worldY: number) {
        if (!marquee.active) return;
        marquee.current.x = worldX;
        marquee.current.y = worldY;
    }

    function finalizeMarquee() {
        if (!marquee.active) return;
        marquee.active = false;

        const rect = toBounds(marquee.start, marquee.current);

        // only if a drag actually happened
        if (rect.right - rect.left < 1 && rect.bottom - rect.top < 1) return;

        for (const { id, bounds } of getComponentSelectables(subcircuit)) {
            if (rectsIntersect(rect, bounds)) {
                componentSelection.value.add(id);
            }
        }
        for (const { id, bounds } of getWireSelectables(subcircuit)) {
            if (rectsIntersect(rect, bounds)) {
                wireSelection.value.add(id);
            }
        }
    }

    return { marquee, startMarquee, updateMarquee, finalizeMarquee };
}

function getComponentSelectables(subcircuit: Subcircuit): Selectable[] {
    return [...subcircuit.components].map(([id, comp]) => {
        const dims = componentMap[comp.type].getDimensions(comp);
        return {
            id,
            bounds: {
                left: comp.x,
                top: comp.y,
                right: comp.x + dims.width,
                bottom: comp.y + dims.height,
            },
        };
    });
    //TO DO add wire seleciton here
}
function getWireSelectables(subcircuit: Subcircuit): Selectable[]{
    return [...subcircuit.wires].map((wire, id)=>{
        return {
            id, 
            bounds: {
                left:wire.endpoints[0].x,
                right: wire.endpoints[1].x,
                top:wire.endpoints[0].y,
                bottom: wire.endpoints[1].y
            }

        }
    })
}

function toBounds(a: Location, b: Location): Rect {
    return {
        left: Math.min(a.x, b.x),
        top: Math.min(a.y, b.y),
        right: Math.max(a.x, b.x),
        bottom: Math.max(a.y, b.y),
    };
}

function rectsIntersect(a: Rect, b: Rect): boolean {
    return a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top;
}
