<script setup lang="ts">
import { computed, ref } from "vue";
import { circuits, currentCircuit, selectedComponentId } from "@/lib/store/circuit";
import { componentMap } from "./circuitry";
import { AccordionContent, AccordionHeader, AccordionItem, AccordionRoot } from "./ui/accordion";
import { toast } from "vue-sonner";
import { settings } from "@/lib/store/settings";

const nameReset = ref(0);
const subcircuitName = computed({
    get() {
        return currentCircuit.value.subcircuit.name;
    },
    set(name) {
        if (name === "") {
            toast.error("Subcircuit name is required!");
            nameReset.value++;
            return;
        } else if ([...circuits.values()].some(({ subcircuit }) => subcircuit.name === name)) {
            toast.error("A subcircuit with this name already exists!");
            nameReset.value++;
            return;
        }
        currentCircuit.value.subcircuit.name = name;
    },
});

const selectedComponent = computed(() =>
    currentCircuit.value.subcircuit.components.get(selectedComponentId.value),
);

const sections = ["global", "circuit", "component"] as const;
</script>

<template>
    <h2 class="border-b bg-panel-light px-4 py-3 text-sm font-semibold text-foreground-highlight">
        Properties
    </h2>

    <AccordionRoot :default-value="sections.slice()">
        <AccordionItem value="global">
            <AccordionHeader> Global </AccordionHeader>

            <AccordionContent class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between">
                        <span class="font-medium">Global Bitsize</span>
                        <span>{{ settings.globalBitsize }}</span>
                    </span>
                    <input
                        v-model="settings.globalBitsize"
                        type="range"
                        min="1"
                        step="1"
                        max="16"
                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500"
                    />
                </label>
            </AccordionContent>
        </AccordionItem>

        <AccordionItem value="circuit">
            <AccordionHeader> {{ currentCircuit.subcircuit.name }} </AccordionHeader>

            <AccordionContent class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between font-medium"> Name </span>
                    <input
                        :key="nameReset"
                        v-model.lazy.trim="subcircuitName"
                        type="text"
                        min="1"
                        step="1"
                        max="16"
                        class="mt-1 block w-full appearance-none border bg-panel-light px-1 py-1 accent-blue-500"
                    />
                </label>
            </AccordionContent>
        </AccordionItem>

        <AccordionItem v-if="selectedComponentId !== null" value="component">
            <AccordionHeader>
                {{ componentMap[selectedComponent.type].displayName }}
            </AccordionHeader>

            <AccordionContent class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between">
                        <span class="font-medium"> Bitsize</span>
                        <span>{{ selectedComponent.bitsize }}</span>
                    </span>
                    <input
                        v-model="selectedComponent.bitsize"
                        type="range"
                        min="1"
                        step="1"
                        max="16"
                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500"
                    />
                </label>
            </AccordionContent>
        </AccordionItem>
    </AccordionRoot>
</template>
