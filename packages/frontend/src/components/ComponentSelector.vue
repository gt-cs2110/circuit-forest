<script setup lang="ts">
import { computed, ref } from "vue";
import { componentCategories } from "@/lib/types";
import { componentMap } from "./circuitry";
import CircuitComponentPreview from "./circuitry/CircuitComponentPreview.vue";
import { placingComponent, placingHandedness, placingOrientation } from "@/lib/store/view";
import { AccordionContent, AccordionHeader, AccordionItem, AccordionRoot } from "./ui/accordion";

const search = ref("");

const filteredCategories = computed(() => {
    const query = search.value.trim().toLowerCase();
    return Object.entries(componentCategories)
        .map(
            ([category, components]) =>
                [
                    category,
                    components.filter(
                        (component) =>
                            component.includes(query) ||
                            componentMap[component].displayName.toLowerCase().includes(query),
                    ),
                ] as const,
        )
        .filter(([, components]) => components.length > 0);
});
</script>

<template>
    <input
        v-model="search"
        type="search"
        aria-label="Search components"
        class="w-full appearance-none border-b bg-panel-light px-4 py-3 text-sm text-foreground-highlight placeholder:text-foreground-muted"
        placeholder="Search..."
    />

    <AccordionRoot :default-value="Object.keys(componentCategories)">
        <AccordionItem
            v-for="[category, components] in filteredCategories"
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
                    type="button"
                    :aria-pressed="placingComponent === component"
                    class="flex aspect-square cursor-pointer flex-col justify-center gap-2 border py-1 text-xs transition-colors"
                    :class="
                        placingComponent === component
                            ? 'border-primary bg-primary/10'
                            : 'bg-panel-light hover:border-primary'
                    "
                    @click="
                        () => {
                            if (placingComponent === component) {
                                placingComponent = null;
                            } else {
                                placingComponent = component;
                                placingOrientation = 'East';
                                placingHandedness = 'DownRight';
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

        <p v-if="filteredCategories.length === 0" class="px-4 py-3 text-xs text-foreground-muted">
            No components match "{{ search.trim() }}".
        </p>
    </AccordionRoot>
</template>
