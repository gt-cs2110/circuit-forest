<script setup lang="ts">
import { circuits, currentCircuitId, newSubcircuit } from "@/lib/store/circuit";
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
            <TabsList class="flex border-b bg-panel-dark text-sm">
                <TabsTrigger
                    v-for="[id, circuit] in circuits"
                    :key="id"
                    :value="id"
                    class="relative cursor-pointer items-stretch border-r px-4 py-3 font-medium"
                    :class="[
                        currentCircuitId === id
                            ? 'bg-panel-light text-foreground-highlight'
                            : 'bg-panel-dark text-foreground-muted hover:bg-panel-light',
                    ]"
                >
                    {{ circuit.subcircuit.name }}

                    <div
                        v-if="currentCircuitId === id"
                        class="absolute inset-x-0 top-full h-0.5 bg-panel-light"
                    ></div>
                </TabsTrigger>

                <button
                    class="grid aspect-square cursor-pointer place-items-center p-3 text-foreground-muted hover:bg-panel-light"
                    @click="createNew"
                >
                    <Plus :size="16" absolute-stroke-width />
                </button>
            </TabsList>
        </HorizontalScroll>
    </TabsRoot>
</template>
