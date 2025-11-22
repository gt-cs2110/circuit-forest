<script setup lang="ts">
import { componentCategories } from "@/lib/types";
import { ChevronDown } from "lucide-vue-next";
import { componentMap } from "./circuitry";
import CircuitComponentPreview from "./CircuitComponentPreview.vue";
import { placingComponent } from "@/lib/store";
import {
    AccordionContent,
    AccordionHeader,
    AccordionItem,
    AccordionRoot,
    AccordionTrigger,
} from "reka-ui";
</script>

<template>
    <input
        type="search"
        class="w-full appearance-none border-b-2 border-zinc-700 bg-zinc-800 px-4 py-3 text-sm text-zinc-200 placeholder:text-zinc-500 focus:outline-none"
        placeholder="Search..."
    />

    <AccordionRoot
        class="overflow-y-auto"
        type="multiple"
        :default-value="Object.keys(componentCategories)"
    >
        <AccordionItem
            v-for="[category, components] in Object.entries(componentCategories)"
            :key="category"
            :value="category"
        >
            <AccordionHeader>
                <AccordionTrigger
                    class="group flex w-full cursor-pointer items-center gap-2 border-b-2 border-zinc-700 bg-zinc-800 p-2 text-left text-sm font-medium text-white capitalize"
                >
                    <ChevronDown
                        class="inline align-middle transition-transform group-data-[state=closed]:-rotate-90"
                        :size="16"
                        absolute-stroke-width
                    />
                    {{ category }}
                </AccordionTrigger>
            </AccordionHeader>

            <AccordionContent
                class="grid grid-cols-fill-20 gap-2 border-b-2 border-zinc-700 p-2 data-[state=closed]:animate-grow-out data-[state=open]:animate-grow-in"
            >
                <button
                    v-for="component in components"
                    :key="component"
                    class="flex aspect-square cursor-pointer flex-col justify-center gap-2 border-2 border-zinc-600 py-1 text-xs"
                    :class="placingComponent === component ? 'bg-zinc-800' : 'bg-zinc-700'"
                    @click="placingComponent = component"
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
