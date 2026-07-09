import { addWires, deleteWires, updateComponent } from "@/lib/store/circuit";
import type { Subcircuit } from "@/lib/types";
import { Location } from "circuitsim-glue";
import { Ref, ref, toRaw } from "vue";

export function useDrag(
    subcircuit: Subcircuit,
    componentSelection: Ref<Set<number>>,
    wireSelection: Ref<Set<number>>,
) {
    const drag = ref({
        active: false,
        initialMouse: { x: 0, y: 0 } as Location,
        initialComponentPositions: new Map<number, Location>(),
        initialWirePositions: new Map<number, Location>(),
    });

    function startDrag(worldX: number, worldY: number) {
        drag.value.active = true;
        drag.value.initialMouse = { x: worldX, y: worldY };
        drag.value.initialComponentPositions.clear();
        drag.value.initialWirePositions.clear();
        for (const id of componentSelection.value) {
            const comp = subcircuit.components.get(id);
            if (comp) {
                drag.value.initialComponentPositions.set(id, { x: comp.x, y: comp.y });
            }
        }
        for (const id of wireSelection.value) {
            const wire = subcircuit.wires.at(id);
            if (wire)
                drag.value.initialWirePositions.set(id, {
                    x: wire.endpoints[0].x,
                    y: wire.endpoints[0].y,
                });
        }
    }

    function updateDrag(worldX: number, worldY: number) {
        if (!drag.value.active) return;

        const deltaX = Math.round(worldX - drag.value.initialMouse.x);
        const deltaY = Math.round(worldY - drag.value.initialMouse.y);

        for (const [id, initial] of drag.value.initialComponentPositions) {
            //Update Frontend
            const comp = subcircuit.components.get(id);
            if (!comp) continue;
            comp.x = Math.max(initial.x + deltaX, 0);
            comp.y = Math.max(initial.y + deltaY, 0);
            //updateComponent(comp.frontendId, {x:comp.x, y:comp.y})
        }
        for (const [id, initial] of drag.value.initialWirePositions) {
            //Update Frontend
            const wire = subcircuit.wires.at(id);
            if (!wire) continue;
            wire.endpoints[0].x = Math.max(initial.x + deltaX, 0);
            wire.endpoints[0].y = Math.max(initial.y + deltaY, 0);

            if (wire.isHorizontal) {
                wire.endpoints[1].x = wire.endpoints[0].x + wire.length;
                wire.endpoints[1].y = wire.endpoints[0].y;
            } else {
                wire.endpoints[1].x = wire.endpoints[0].x;
                wire.endpoints[1].y = wire.endpoints[0].y + wire.length;
            }
        }
    }

    function stopDrag() {
        if (!drag.value.active) return;
        const newWires = drag.value.initialWirePositions
            .keys()
            .map((index) => toRaw(subcircuit.wires[index]))
            .toArray();

        //return old wires to old positions
        drag.value.initialWirePositions.forEach((start, index) => {
            const wire = subcircuit.wires.at(index);
            if (!wire) return;
            wire.endpoints[0].x = start.x;
            wire.endpoints[0].y = start.y;
            if (wire.isHorizontal) {
                wire.endpoints[1].x = wire.endpoints[0].x + wire.length;
                wire.endpoints[1].y = wire.endpoints[0].y;
            } else {
                wire.endpoints[1].x = wire.endpoints[0].x;
                wire.endpoints[1].y = wire.endpoints[0].y + wire.length;
            }
        });
        deleteWires(drag.value.initialWirePositions.keys().toArray());
        console.log("Adding Wires: ", newWires);
        addWires(newWires);

        drag.value.active = false;
        for (const [id] of drag.value.initialComponentPositions) {
            //Update Frontend
            const comp = subcircuit.components.get(id);
            if (!comp) continue;

            updateComponent(comp.frontendId, { x: comp.x, y: comp.y });
        }
    }

    return { drag, startDrag, updateDrag, stopDrag };
}
