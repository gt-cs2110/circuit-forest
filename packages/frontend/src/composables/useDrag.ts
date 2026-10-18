import { moveSelection } from "@/lib/store/circuit";
import type { Subcircuit } from "@/lib/types";
import { wireId } from "@/lib/wire";
import { Location } from "circuitsim-glue";
import { Ref, ref } from "vue";

export function useDrag(
    subcircuit: Subcircuit,
    componentSelection: Ref<Set<number>>,
    wireSelection: Ref<Set<string>>,
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
            const wires = subcircuit.wires
                .map((wire) => wire.endpoints)
                .filter((endpoints) => wireSelection.value.has(wireId(endpoints)));
            const moved = moveSelection(Array.from(componentSelection.value), wires, delta);

            // recalculate new ids for moved wires
            if (moved) {
                wireSelection.value.clear();
                for (const [a, b] of wires) {
                    wireSelection.value.add(
                        wireId([
                            { x: a.x + delta.x, y: a.y + delta.y },
                            { x: b.x + delta.x, y: b.y + delta.y },
                        ]),
                    );
                }
            }
        }

        drag.value.active = false;
    }

    return { drag, startDrag, updateDrag, stopDrag };
}
