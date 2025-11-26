<script setup lang="ts">
import { componentCategories } from "@/lib/types";
import { componentMap } from "./circuitry";
import CircuitComponentPreview from "./circuitry/CircuitComponentPreview.vue";
import { placingComponent } from "@/lib/store/circuit";
import { AccordionContent, AccordionHeader, AccordionItem, AccordionRoot } from "./ui/accordion";
</script>

<template>
    <input
        type="search"
        class="w-full appearance-none border-b bg-panel-light px-4 py-3 text-sm text-foreground-highlight placeholder:text-foreground-muted"
        placeholder="Search..."
    />

    <AccordionRoot :default-value="Object.keys(componentCategories)">
        <AccordionItem
            v-for="[category, components] in Object.entries(componentCategories)"
            :key="category"
            :value="category"
        >
            <AccordionHeader>
                {{
                    category
                        .split("/")
                        .map((w) => w[0].toUpperCase() + w.substring(1))
                        .join("/")
                }}
            </AccordionHeader>

            <AccordionContent class="grid grid-cols-fill-20 gap-2 p-2">
                <button
                    v-for="component in components"
                    :key="component"
                    class="flex aspect-square cursor-pointer flex-col justify-center gap-2 border py-1 text-xs"
                    :class="placingComponent === component ? 'bg-panel-dark' : 'bg-panel-light'"
                    @click="
                        () => {
                            if (placingComponent === component) {
                                placingComponent = null;
                            } else {
                                placingComponent = component;
                            }
                        }
                    "
                >
                    <div class="flex flex-1 flex-col justify-center">
                        <CircuitComponentPreview :type="component" />
                    </div>
                    <span class="mt-auto">
                        {{ componentMap[component].displayName }}
                    </span>
                </button>
            </AccordionContent>
        </AccordionItem>
    </AccordionRoot>
</template>
