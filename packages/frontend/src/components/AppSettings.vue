<script setup lang="ts">
import { Theme, theme, themes } from "@/lib/store/settings";
import { SettingsIcon } from "@lucide/vue";
import { PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger } from "reka-ui";

const themeNames: Record<Theme, string> = {
    light: "Light",
    "light-contrast": "Light · High Contrast",
    dark: "Dark",
    "dark-contrast": "Dark · High Contrast",
};
</script>

<template>
    <PopoverRoot>
        <PopoverTrigger aria-label="Settings" title="Settings" class="cursor-pointer">
            <SettingsIcon
                :size="16"
                absolute-stroke-width
                class="text-foreground-muted transition hover:text-foreground"
            />
        </PopoverTrigger>

        <PopoverPortal>
            <PopoverContent
                align="start"
                side="bottom"
                :side-offset="8"
                class="w-64 border bg-panel-dark p-4 data-[state=closed]:animate-fade-out data-[state=open]:animate-fade-in"
            >
                <fieldset class="text-xs">
                    <legend class="font-medium">Theme</legend>
                    <label
                        v-for="newTheme in themes"
                        :key="newTheme"
                        class="mt-1 block w-full cursor-pointer border px-1 py-1 transition-colors focus-within:outline"
                        :class="
                            newTheme === theme
                                ? 'border-blue-500 bg-blue-500/10'
                                : 'bg-panel-light hover:border-blue-500'
                        "
                    >
                        {{ themeNames[newTheme] }}
                        <input
                            v-model="theme"
                            type="radio"
                            name="theme"
                            :value="newTheme"
                            class="sr-only"
                        />
                    </label>
                </fieldset>
            </PopoverContent>
        </PopoverPortal>
    </PopoverRoot>
</template>
