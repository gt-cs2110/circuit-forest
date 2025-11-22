<script setup lang="ts">
import CircuitCanvas from "./components/CircuitCanvas.vue";
import Properties from "./components/Properties.vue";
import { currentCircuit } from "./lib/store";
import CircuitTabs from "./components/CircuitTabs.vue";
import ComponentSelector from "./components/ComponentSelector.vue";
import { SplitterGroup, SplitterPanel, SplitterResizeHandle } from "reka-ui";
</script>

<template>
    <div class="h-screen">
        <SplitterGroup direction="horizontal">
            <SplitterPanel
                class="bg-zinc-900 text-zinc-200"
                :min-size="20"
                :default-size="20"
                :max-size="30"
            >
                <ComponentSelector />
            </SplitterPanel>

            <SplitterResizeHandle
                class="w-0.5 bg-zinc-700 transition focus-visible:bg-zinc-500 data-[state=drag]:bg-zinc-500 data-[state=hover]:bg-zinc-500"
            />

            <SplitterPanel class="flex flex-1 flex-col">
                <CircuitTabs />
                <CircuitCanvas :state="currentCircuit" />
            </SplitterPanel>

            <SplitterResizeHandle
                class="w-0.5 bg-zinc-700 transition focus-visible:bg-zinc-500 data-[state=drag]:bg-zinc-500 data-[state=hover]:bg-zinc-500"
            />

            <SplitterPanel
                class="bg-zinc-900 text-zinc-200"
                :min-size="20"
                :default-size="20"
                :max-size="30"
            >
                <Properties />
            </SplitterPanel>
        </SplitterGroup>
    </div>
</template>
