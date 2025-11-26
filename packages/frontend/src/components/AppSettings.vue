<script setup lang="ts">
import { Theme, theme, themes } from "@/lib/store/settings";
import { SettingsIcon } from "lucide-vue-next";
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
        <PopoverTrigger>
            <SettingsIcon :size="16" absolute-stroke-width class="text-foreground-muted" />
        </PopoverTrigger>

        <PopoverPortal>
            <PopoverContent
                align="start"
                side="bottom"
                :side-offset="8"
                class="data-[state=closed]:animate-fade-out data-[state=open]:animate-fade-in left-2 w-64 border bg-panel-dark p-4"
            >
                <fieldset class="text-xs">
                    <span class="flex justify-between font-medium"> Theme </span>
                    <label
                        v-for="newTheme in themes"
                        :key="newTheme"
                        class="mt-1 block w-full appearance-none border px-1 py-1 accent-blue-500 focus-within:outline"
                        :class="{
                            'bg-panel-dark': newTheme === theme,
                            'bg-panel-light': newTheme !== theme,
                        }"
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
