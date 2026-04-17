import { GRID_SIZE, ORIGIN_OFFSET } from "@/lib/consts";
import type { Location } from "@/lib/types";
import type { ComputedRef } from "vue";

export function useCoordinates(offset: ComputedRef<Location>, scale: ComputedRef<number>) {
    function containerToWorld(cx: number, cy: number): Location {
        return {
            x: (cx - offset.value.x - ORIGIN_OFFSET * scale.value) / scale.value / GRID_SIZE,
            y: (cy - offset.value.y - ORIGIN_OFFSET * scale.value) / scale.value / GRID_SIZE,
        };
    }

    function worldToContainer(wx: number, wy: number): Location {
        return {
            x: wx * GRID_SIZE * scale.value + offset.value.x + ORIGIN_OFFSET * scale.value,
            y: wy * GRID_SIZE * scale.value + offset.value.y + ORIGIN_OFFSET * scale.value,
        };
    }

    return { containerToWorld, worldToContainer };
}
