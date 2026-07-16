import type { Location } from "circuitsim-glue";
import { ref, type Ref } from "vue";

export function usePan(offset: Ref<Location>) {
    const isPanning = ref(false);
    const panStart = ref({ x: 0, y: 0 });

    function startPan(clientX: number, clientY: number) {
        isPanning.value = true;
        panStart.value = {
            x: clientX - offset.value.x,
            y: clientY - offset.value.y,
        };
    }

    function updatePan(clientX: number, clientY: number) {
        if (!isPanning.value) return;
        offset.value = {
            x: clientX - panStart.value.x,
            y: clientY - panStart.value.y,
        };
    }

    function stopPan() {
        isPanning.value = false;
    }

    return { isPanning, startPan, updatePan, stopPan };
}
