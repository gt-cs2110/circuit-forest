import { componentMap } from "@/components/circuitry";
import type { Location, Subcircuit } from "@/lib/types";
import { ComputedRef, Reactive, reactive } from "vue";

type Rect = { left: number; top: number; right: number; bottom: number };
type Selectable = { id: number; bounds: Rect };

export function useMarquee(subcircuit: Reactive<Subcircuit>, selection: ComputedRef<Set<number>>) {
    const marquee = reactive({
        active: false,
        start: { x: 0, y: 0 },
        current: { x: 0, y: 0 },
    });

    function startMarquee(worldX: number, worldY: number, additive: boolean) {
        if (!additive) selection.value.clear();

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

        for (const { id, bounds } of getSelectables(subcircuit)) {
            if (rectsIntersect(rect, bounds)) {
                selection.value.add(id);
            }
        }
    }

    return { marquee, startMarquee, updateMarquee, finalizeMarquee };
}

function getSelectables(subcircuit: Subcircuit): Selectable[] {
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
