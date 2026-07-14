import { Location } from "circuitsim-glue";
import { Ref, ref } from "vue";

import { getDimensions } from "@/lib/bounds";
import type { Subcircuit } from "@/lib/types";

type Rect = { left: number; top: number; right: number; bottom: number };
type Selectable = { id: number; bounds: Rect };

export function useMarquee(
    subcircuit: Subcircuit,
    componentSelection: Ref<Set<number>>,
    wireSelection: Ref<Set<number>>,
) {
    const marquee = ref({
        active: false,
        start: { x: 0, y: 0 },
        current: { x: 0, y: 0 },
    });

    function startMarquee(worldX: number, worldY: number, additive: boolean) {
        if (!additive) {
            componentSelection.value.clear();
            wireSelection.value.clear();
        }

        marquee.value = {
            active: true,
            start: { x: worldX, y: worldY },
            current: { x: worldX, y: worldY },
        };
    }

    function updateMarquee(worldX: number, worldY: number) {
        if (!marquee.value.active) return;
        marquee.value.current = {
            x: worldX,
            y: worldY,
        };
    }

    function finalizeMarquee() {
        if (!marquee.value.active) return;
        marquee.value.active = false;

        const rect = toBounds(marquee.value.start, marquee.value.current);

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
    return [...subcircuit.components].map(([id, component]) => {
        const dims = getDimensions(component.bounds);
        const { x, y } = component.pos;
        return {
            id,
            bounds: {
                left: x,
                top: y,
                right: x + dims.width,
                bottom: y + dims.height,
            },
        };
    });
    //TO DO add wire seleciton here
}
function getWireSelectables(subcircuit: Subcircuit): Selectable[] {
    return [...subcircuit.wires].map((wire, id) => {
        return {
            id,
            bounds: {
                left: wire.endpoints[0].x,
                right: wire.endpoints[1].x,
                top: wire.endpoints[0].y,
                bottom: wire.endpoints[1].y,
            },
        };
    });
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
