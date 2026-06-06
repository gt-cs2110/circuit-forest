<script setup lang="ts">
import { computed, ref } from "vue";
import { circuits, currentSubcircuit, updateComponent } from "@/lib/store/circuit";
import { componentMap } from "./circuitry";
import { AccordionContent, AccordionHeader, AccordionItem, AccordionRoot } from "./ui/accordion";
import { toast } from "vue-sonner";
import { settings } from "@/lib/store/settings";
import { selection } from "@/lib/store/view";

const nameReset = ref(0);
const subcircuitName = computed({
    get() {
        return currentSubcircuit.value.name;
    },
    set(name) {
        if (name === "") {
            toast.error("Subcircuit name is required!");
            nameReset.value++;
            return;
        } else if ([...circuits.values()].some((subcircuit) => subcircuit.name === name)) {
            toast.error("A subcircuit with this name already exists!");
            nameReset.value++;
            return;
        }
        currentSubcircuit.value.name = name;
    },
});
const orientations = [
  { label: "N", value: 0 },
  { label: "S", value: 1 },
  { label: "E", value: 2 },
  { label: "W", value: 3 },
] as const;

const selectedComponent = computed(() =>
    selection.value.values().next().value?currentSubcircuit.value.components.get(selection.value.values().next().value||-1):null
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
            <AccordionHeader> {{ currentSubcircuit.name }} </AccordionHeader>

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

         <AccordionItem v-if="selectedComponent !== null && selectedComponent !=undefined" value="component">
            <AccordionHeader>
                {{ selectedComponent.label }}
            </AccordionHeader>

            <AccordionContent class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between">
                        <span class="font-medium"> Bitsize</span>
                        <span>{{ selectedComponent.bitsize }}</span>
                    </span>
                    <input
                        v-model.number="selectedComponent.bitsize"
                        type="range"
                        min="1"
                        step="1"
                        max="16"
                        @change = "updateComponent(selectedComponent.frontendId, { bitsize: selectedComponent.bitsize })"

                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500"
                    />
                </label>
            </AccordionContent>
            <AccordionContent class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between">
                        <span class="font-medium"> Num Inputs</span>
                        <span>{{ selectedComponent.inputs }}</span>
                    </span>
                    <input
                        v-model.number="selectedComponent.inputs"
                        type="range"
                        min="1"
                        step="1"
                        max="8"
                        @change = "updateComponent(selectedComponent.frontendId, { inputs: selectedComponent.inputs })"

                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500"
                    />
                </label>
            </AccordionContent>
            <AccordionContent class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="font-medium">Orientation</span>

                    <div class="mt-2 flex overflow-hidden rounded border">
                    <button
                        v-for="option in orientations"
                        :key="option.value"
                        type="button"
                        class="flex-1 px-3 py-2 transition-colors"
                        :class="
                        selectedComponent.orientation === option.value
                            ? 'bg-blue-500 text-white'
                            : 'bg-panel-light hover:bg-panel-dark'
                        "
                        @click="
                        updateComponent(selectedComponent.frontendId, {
                            orientation: option.value,
                        })
                        "
                    >
                        {{ option.label }}
                    </button>
                    </div>
                </label>
            </AccordionContent>
        </AccordionItem>
        
    </AccordionRoot>
</template>
