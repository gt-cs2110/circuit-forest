import type { Location } from "@/lib/types";
import type { ComputedRef, WritableComputedRef } from "vue";

export function useZoom(
    offset: WritableComputedRef<Location>,
    mousePosition: { x: number; y: number },
    getScaleLevel: () => number,
    setScaleLevel: (level: number) => void,
    scale: ComputedRef<number>,
) {
    function zoom(newScaleLevel: number) {
        newScaleLevel = Math.min(Math.max(-5, newScaleLevel), 10);

        const oldScale = scale.value;
        const newScale = Math.pow(1.2, newScaleLevel);

        const worldX = (mousePosition.x - offset.value.x) / oldScale;
        const worldY = (mousePosition.y - offset.value.y) / oldScale;

        setScaleLevel(newScaleLevel);
        offset.value = {
            x: mousePosition.x - worldX * newScale,
            y: mousePosition.y - worldY * newScale,
        };
    }

    function wheelZoom(e: WheelEvent) {
        const isTrackpad = Math.abs(e.deltaY) < 50 && e.deltaMode === 0;
        const isPinchZoom = e.ctrlKey || e.metaKey;

        if (isPinchZoom) {
            const delta = isTrackpad ? e.deltaY * -0.03 : e.deltaY * -0.002;
            zoom(getScaleLevel() + delta);
        } else {
            offset.value = {
                x: offset.value.x - e.deltaX,
                y: offset.value.y - e.deltaY,
            };
        }
    }

    /**
     * @param e A global keyboard event
     * @returns true if zoom is triggered; false if not
     */
    function keyboardZoom(e: KeyboardEvent): boolean {
        if (!e.metaKey) return false;
        if (!["-", "=", "+", "0"].includes(e.key)) return false;

        e.preventDefault();
        const newScaleLevel = Math.round(
            e.key === "=" || e.key === "+"
                ? getScaleLevel() + 1
                : e.key === "-"
                  ? getScaleLevel() - 1
                  : 0,
        );
        zoom(newScaleLevel);
        return true;
    }

    return { wheelZoom, keyboardZoom };
}
