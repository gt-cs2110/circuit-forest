import { computed, ref, watch } from "vue";

export const settings = ref({
    scaleLevel: 0,
    globalBitsize: 1,
});
export const scale = computed(() => {
    return Math.pow(1.2, settings.value.scaleLevel);
});

export const themes = ["light", "light-contrast", "dark", "dark-contrast"] as const;
export type Theme = (typeof themes)[number];
export const theme = ref<Theme>("light");

watch(theme, (newTheme) => {
    document.startViewTransition(() => {
        document.documentElement.dataset.theme = newTheme;
    });
});

export const interfaceScales = [90, 100, 110, 125, 150, 200] as const;
export type InterfaceScale = (typeof interfaceScales)[number];
const currentPercent = window.api.view.getZoomFactor() * 100;
export const interfaceScale = ref<InterfaceScale>(
    interfaceScales.reduce((a, b) =>
        Math.abs(b - currentPercent) < Math.abs(a - currentPercent) ? b : a,
    ),
);

watch(interfaceScale, (percent) => {
    window.api.view.setZoomFactor(percent / 100);
});
