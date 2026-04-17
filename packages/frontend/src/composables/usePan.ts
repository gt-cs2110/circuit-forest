import type { Location } from "@/lib/types";
import { reactive, ref } from "vue";
import type { WritableComputedRef } from "vue";

export function usePan(offset: WritableComputedRef<Location>) {
    const isPanning = ref(false);
    const panStart = reactive({ x: 0, y: 0 });

    function startPan(clientX: number, clientY: number) {
        isPanning.value = true;
        panStart.x = clientX - offset.value.x;
        panStart.y = clientY - offset.value.y;
    }

    function updatePan(clientX: number, clientY: number) {
        if (!isPanning.value) return;
        offset.value = {
            x: clientX - panStart.x,
            y: clientY - panStart.y,
        };
    }

    function stopPan() {
        isPanning.value = false;
    }

    return { isPanning, startPan, updatePan, stopPan };
}
