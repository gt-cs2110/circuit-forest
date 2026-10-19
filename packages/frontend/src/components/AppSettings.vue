<script setup lang="ts">
import type { InterfaceScale, Theme } from "@/lib/store/settings";
import { interfaceScale, interfaceScales, theme, themes } from "@/lib/store/settings";
import { SettingsIcon } from "@lucide/vue";
import { PopoverContent, PopoverPortal, PopoverRoot, PopoverTrigger } from "reka-ui";
import { useId } from "vue";
import SegmentedControl from "./ui/SegmentedControl.vue";

const themeNames: Record<Theme, string> = {
    light: "Light",
    "light-contrast": "Light · High Contrast",
    dark: "Dark",
    "dark-contrast": "Dark · High Contrast",
};

const interfaceSizeId = useId();
const interfaceSizeOptions = interfaceScales.map((percent) => ({
    label: `${percent}%`,
    name: `${percent}%`,
    value: String(percent),
}));
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
                                ? 'border-primary bg-primary/10'
                                : 'bg-panel-light hover:border-primary'
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

                <div class="mt-4 text-xs">
                    <div :id="interfaceSizeId" class="font-medium">Interface Scale</div>
                    <SegmentedControl
                        :options="interfaceSizeOptions"
                        :value="String(interfaceScale)"
                        :labelledby="interfaceSizeId"
                        :columns="3"
                        @update:value="interfaceScale = Number($event) as InterfaceScale"
                    />
                </div>
            </PopoverContent>
        </PopoverPortal>
    </PopoverRoot>
</template>
