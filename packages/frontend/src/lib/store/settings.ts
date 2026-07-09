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
