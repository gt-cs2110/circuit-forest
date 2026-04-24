import type { Location, Subcircuit } from "@/lib/types";
import { ComputedRef, Reactive, reactive } from "vue";

export function useDrag(subcircuit: Reactive<Subcircuit>, selection: ComputedRef<Set<number>>) {
    const drag = reactive({
        active: false,
        initialMouse: { x: 0, y: 0 } as Location,
        initialPositions: new Map<number, Location>(),
    });

    function startDrag(worldX: number, worldY: number) {
        drag.active = true;
        drag.initialMouse = { x: worldX, y: worldY };
        drag.initialPositions.clear();
        for (const id of selection.value) {
            const comp = subcircuit.components.get(id);
            if (comp) drag.initialPositions.set(id, { x: comp.x, y: comp.y });
        }
    }

    function updateDrag(worldX: number, worldY: number) {
        if (!drag.active) return;

        const deltaX = Math.round(worldX - drag.initialMouse.x);
        const deltaY = Math.round(worldY - drag.initialMouse.y);

        for (const [id, initial] of drag.initialPositions) {
            const comp = subcircuit.components.get(id);
            if (!comp) continue;
            comp.x = Math.max(initial.x + deltaX, 0);
            comp.y = Math.max(initial.y + deltaY, 0);
        }
    }

    function stopDrag() {
        drag.active = false;
    }

    return { drag, startDrag, updateDrag, stopDrag };
}
