<script setup lang="ts">
import "vue-sonner/style.css";

import CircuitCanvas from "./components/CircuitCanvas.vue";
import Properties from "./components/Properties.vue";
import { currentCircuit, scale, settings } from "./lib/store";
import CircuitTabs from "./components/CircuitTabs.vue";
import ComponentSelector from "./components/ComponentSelector.vue";
import {
    SliderRoot,
    SliderThumb,
    SliderTrack,
    SplitterGroup,
    SplitterPanel,
    SplitterResizeHandle,
} from "reka-ui";
import { Toaster } from "vue-sonner";
import { computed } from "vue";

const sliderValue = computed({
    get() {
        return [settings.scaleLevel];
    },
    set(value) {
        settings.scaleLevel = value[0];
    },
});
</script>

<template>
    <Toaster
        theme="dark"
        :toast-options="{
            style: {
                background: 'var(--color-zinc-900)',
                borderRadius: '0',
                color: 'var(--color-zinc-200)',
                borderColor: 'var(--color-zinc-700)',
            },
        }"
        :gap="6"
        :visible-toasts="3"
    />

    <div class="flex h-screen flex-col">
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
                class="w-px bg-zinc-700 focus-visible:bg-zinc-500 data-[state=drag]:bg-zinc-500 data-[state=hover]:bg-zinc-500"
            />

            <SplitterPanel class="flex flex-1 flex-col">
                <CircuitTabs />
                <CircuitCanvas :state="currentCircuit" />
            </SplitterPanel>

            <SplitterResizeHandle
                class="w-px bg-zinc-700 focus-visible:bg-zinc-500 data-[state=drag]:bg-zinc-500 data-[state=hover]:bg-zinc-500"
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

        <div
            class="flex h-6 items-center border-t border-zinc-700 bg-zinc-800 px-4 text-xs text-zinc-200"
        >
            <SliderRoot
                v-model="sliderValue"
                :min="-5"
                :step="1"
                :max="10"
                class="relative ml-auto flex w-52 touch-none items-center select-none"
            >
                <SliderTrack class="relative h-0.5 flex-1 bg-zinc-500">
                    <div
                        class="absolute top-1/2 left-1/3 h-2 w-0.5 -translate-y-1/2 bg-zinc-500"
                    ></div>
                </SliderTrack>
                <SliderThumb
                    class="block h-2 w-3 bg-zinc-400 hover:bg-zinc-300"
                    aria-label="Zoom level"
                />
            </SliderRoot>
            <span class="w-12 text-right text-zinc-300"> {{ (scale * 100).toFixed(0) }}% </span>
        </div>
    </div>
</template>
