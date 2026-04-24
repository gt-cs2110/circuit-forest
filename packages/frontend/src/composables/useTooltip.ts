import { reactive } from "vue";

export function useTooltip() {
    const tooltip = reactive({
        value: null as string | null,
        x: 0,
        y: 0,
    });

    function updateTooltip(target: EventTarget) {
        if (!("dataset" in target) || !target.dataset || typeof target.dataset !== "object") {
            return;
        }

        const element = target as SVGElement;

        if (element.dataset.tooltip) {
            const rect = element.getBoundingClientRect();
            tooltip.x = rect.x + rect.width / 2;
            tooltip.y = rect.y + rect.height / 2;
            tooltip.value = element.dataset.tooltip;
        } else {
            tooltip.value = null;
        }
    }

    return { tooltip, updateTooltip };
}
