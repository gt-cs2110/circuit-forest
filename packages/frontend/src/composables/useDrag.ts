import { moveSelection } from "@/lib/store/circuit";
import type { Subcircuit } from "@/lib/types";
import { Location } from "circuitsim-glue";
import { Ref, ref } from "vue";

export function useDrag(
    subcircuit: Subcircuit,
    componentSelection: Ref<Set<number>>,
    wireSelection: Ref<Set<number>>,
) {
    const drag = ref({
        active: false,
        initialMouse: { x: 0, y: 0 } as Location,
        delta: { x: 0, y: 0 } as Location,
    });

    function startDrag(worldX: number, worldY: number) {
        drag.value = {
            active: true,
            initialMouse: { x: worldX, y: worldY },
            delta: { x: 0, y: 0 },
        };
    }

    function updateDrag(worldX: number, worldY: number) {
        if (!drag.value.active) return;

        drag.value.delta = {
            x: Math.round(worldX - drag.value.initialMouse.x),
            y: Math.round(worldY - drag.value.initialMouse.y),
        };
    }

    function stopDrag() {
        if (!drag.value.active) return;

        const { delta } = drag.value;
        if (delta.x != 0 || delta.y != 0) {
            // Update wires:
            const wires = Array.from(wireSelection.value, (id) => subcircuit.wires[id].endpoints);
            moveSelection(Array.from(componentSelection.value), wires, delta);
        }

        drag.value.active = false;
    }

    return { drag, startDrag, updateDrag, stopDrag };
}
