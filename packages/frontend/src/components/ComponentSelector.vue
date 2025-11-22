<script setup lang="ts">
import { componentCategories } from "@/lib/types";
import { ChevronDown } from "lucide-vue-next";
import { componentMap } from "./circuitry";
import CircuitComponentPreview from "./CircuitComponentPreview.vue";
import { reactive } from "vue";
import { placingComponent } from "@/lib/store";

const shownCategories = reactive(
    Object.fromEntries(Object.keys(componentCategories).map((category) => [category, true])),
);
</script>

<template>
    <div class="flex w-72 shrink-0 flex-col border-r-2 border-zinc-700 bg-zinc-900 text-zinc-200">
        <input
            type="search"
            class="appearance-none border-b-2 border-zinc-700 bg-zinc-800 px-4 py-3 text-sm text-zinc-200 placeholder:text-zinc-500 focus:outline-none"
            placeholder="Search..."
        />

        <div class="overflow-y-auto">
            <div
                v-for="[category, components] in Object.entries(componentCategories)"
                :key="category"
            >
                <button
                    class="flex w-full cursor-pointer items-center gap-2 border-b-2 border-zinc-700 bg-zinc-800 p-2 text-left text-sm font-medium text-white capitalize"
                    @click="shownCategories[category] = !shownCategories[category]"
                >
                    <ChevronDown
                        class="inline h-4 w-4 align-middle transition-transform"
                        :class="{ '-rotate-90': !shownCategories[category] }"
                    />
                    {{ category }}
                </button>

                <Transition name="grow">
                    <div
                        v-if="shownCategories[category]"
                        class="grid grid-cols-3 gap-2 border-b-2 border-zinc-700 p-2"
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
                    </div>
                </Transition>
            </div>
        </div>
    </div>
</template>
