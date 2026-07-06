import { addWires, deleteWires, updateComponent } from "@/lib/store/circuit";
import type { Subcircuit } from "@/lib/types";
import { Location, TransientWireState } from "circuitsim-glue";
import { ComputedRef, Reactive, reactive } from "vue";

export function useDrag(subcircuit: Reactive<Subcircuit>, componentSelection: ComputedRef<Set<number>>,wireSelection: ComputedRef<Set<number>>) {
    const drag = reactive({
        active: false,
        initialMouse: { x: 0, y: 0 } as Location,
        initialComponentPositions: new Map<number, Location>(),
        initialWirePositions: new Map<number, Location>(),

    });

    function startDrag(worldX: number, worldY: number) {
        drag.active = true;
        drag.initialMouse = { x: worldX, y: worldY };
        drag.initialComponentPositions.clear();
        drag.initialWirePositions.clear()
        for (const id of componentSelection.value) {
            const comp = subcircuit.components.get(id);
            if (comp) drag.initialComponentPositions.set(id, { x: comp.x, y: comp.y });
        }
        for(const id of wireSelection.value) {
            const wire = subcircuit.wires.at(id);
            if (wire) drag.initialWirePositions.set(id, { x: wire.endpoints[0].x, y: wire.endpoints[0].y })
        }
    }

    function updateDrag(worldX: number, worldY: number) {

        if (!drag.active) return;

        const deltaX = Math.round(worldX - drag.initialMouse.x);
        const deltaY = Math.round(worldY - drag.initialMouse.y);

        for (const [id, initial] of drag.initialComponentPositions) {
            //Update Frontend
            const comp = subcircuit.components.get(id);
            if (!comp) continue;
            comp.x = Math.max(initial.x + deltaX, 0);
            comp.y = Math.max(initial.y + deltaY, 0);
            //updateComponent(comp.frontendId, {x:comp.x, y:comp.y})

        }
        for (const [id, initial] of drag.initialWirePositions) {
            //Update Frontend
            const wire = subcircuit.wires.at(id);
            if (!wire) continue;
            wire.endpoints[0].x= Math.max(initial.x + deltaX, 0);
            wire.endpoints[0].y= Math.max(initial.y + deltaY, 0);

            if(wire.isHorizontal){
                wire.endpoints[1].x=wire.endpoints[0].x+wire.length;
                wire.endpoints[1].y = wire.endpoints[0].y
            }
            else {
                wire.endpoints[1].x=wire.endpoints[0].x
                wire.endpoints[1].y = wire.endpoints[0].y+wire.length;
            }

        }
    }

    function stopDrag() {
        if(!drag.active) return;
        const newWires = drag.initialWirePositions.keys().map(index=>{
            const wireData = JSON.parse(JSON.stringify(subcircuit.wires.at(index)));
            return wireData as TransientWireState;
        }).toArray();

        //return old wires to old positions
        drag.initialWirePositions.forEach((start, index)=>{
            const wire = subcircuit.wires.at(index);
            if(!wire)return;
            wire.endpoints[0].x=start.x;
            wire.endpoints[0].y=start.y;
            if(wire.isHorizontal){
                wire.endpoints[1].x=wire.endpoints[0].x+wire.length;
                wire.endpoints[1].y = wire.endpoints[0].y
            }
            else {
                wire.endpoints[1].x=wire.endpoints[0].x
                wire.endpoints[1].y = wire.endpoints[0].y+wire.length;
            }
        })
        deleteWires(drag.initialWirePositions.keys().toArray());
        console.log("Adding Wires: ", newWires);
        addWires(newWires);


        drag.active = false;
        for (const [id, _] of drag.initialComponentPositions) {
            //Update Frontend
            const comp = subcircuit.components.get(id);
            if (!comp) continue;
            
            updateComponent(comp.frontendId, {x:comp.x, y:comp.y})
        }
        
        
        
    }

    return { drag, startDrag, updateDrag, stopDrag };
}
