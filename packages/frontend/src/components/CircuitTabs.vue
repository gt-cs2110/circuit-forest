<script setup lang="ts">
import { circuits, currentCircuitId, newSubcircuit } from "@/lib/store";
import { Plus } from "lucide-vue-next";
import { TabsList, TabsRoot, TabsTrigger } from "reka-ui";
import { nextTick, useTemplateRef } from "vue";
import HorizontalScroll from "./ui/HorizontalScroll.vue";

const scroller = useTemplateRef("scroller");

async function createNew() {
    newSubcircuit();
    await nextTick();
    scroller.value.scrollToEnd();
}

async function selectTab() {
    await nextTick();
    const tab = scroller.value.parent.querySelector(`[data-state=active]`);
    if (tab) {
        tab.scrollIntoView();
    }
}
</script>

<template>
    <TabsRoot v-model="currentCircuitId" @update:model-value="selectTab">
        <HorizontalScroll ref="scroller">
            <TabsList class="flex border-b border-zinc-700 bg-zinc-900 text-sm">
                <TabsTrigger
                    v-for="[id, circuit] in circuits"
                    :key="id"
                    :value="id"
                    class="relative cursor-pointer items-stretch border-r border-zinc-700 px-4 py-3 font-medium"
                    :class="[
                        currentCircuitId === id
                            ? 'bg-zinc-800 text-white'
                            : 'bg-zinc-900 text-zinc-400 hover:bg-zinc-800',
                    ]"
                >
                    <!-- @="selectTab($event, id)" -->
                    {{ circuit.subcircuit.name }}

                    <div
                        v-if="currentCircuitId === id"
                        class="absolute inset-x-0 top-full h-0.5 bg-zinc-800"
                    ></div>
                </TabsTrigger>

                <button
                    class="grid aspect-square cursor-pointer place-items-center p-3 text-zinc-600 hover:bg-zinc-800"
                    @click="createNew"
                >
                    <Plus :size="16" absolute-stroke-width />
                </button>
            </TabsList>
        </HorizontalScroll>
    </TabsRoot>
</template>
