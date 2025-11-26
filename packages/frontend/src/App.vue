<script setup lang="ts">
import "vue-sonner/style.css";

import CircuitCanvas from "./components/CircuitCanvas.vue";
import Properties from "./components/Properties.vue";
import { currentCircuit } from "./lib/store/circuit";
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
import { computed, onBeforeMount, onMounted, onUnmounted, ref } from "vue";
import { scale, settings, theme } from "./lib/store/settings";
import { Moon, Sun } from "lucide-vue-next";

onBeforeMount(() => {
    if (theme.value == "dark") {
        document.documentElement.classList.toggle("dark", true);
    }
});

const sliderValue = computed({
    get() {
        return [settings.scaleLevel];
    },
    set(value) {
        settings.scaleLevel = value[0];
    },
});

const windowWidth = ref(window.innerWidth);
function toPercentage(px: number) {
    return (100 / windowWidth.value) * px;
}
function fromPercentage(percent: number) {
    return (percent / 100) * windowWidth.value;
}

function updateWindowWidth() {
    windowWidth.value = window.innerWidth;
}
onMounted(() => {
    window.addEventListener("resize", updateWindowWidth);
});
onUnmounted(() => {
    window.removeEventListener("resize", updateWindowWidth);
});

const leftWidth = ref(72 * 4);
const rightWidth = ref(72 * 4);
</script>

<template>
    <Toaster
        theme="dark"
        :toast-options="{
            style: {
                background: 'var(--color-panel-dark)',
                borderRadius: '0',
                color: 'var(--color-foreground)',
                borderColor: 'var(--color-border)',
            },
        }"
        :gap="6"
        :visible-toasts="3"
    />

    <div class="flex h-screen flex-col">
        <SplitterGroup direction="horizontal" :keyboard-resize-by="toPercentage(16)">
            <SplitterPanel
                class="bg-panel-dark"
                :min-size="toPercentage(48 * 4)"
                :default-size="toPercentage(leftWidth)"
                :max-size="Math.min(toPercentage(96 * 4), 50)"
                @resize="leftWidth = fromPercentage($event)"
            >
                <ComponentSelector />
            </SplitterPanel>

            <SplitterResizeHandle
                class="w-px bg-border outline-none focus-visible:bg-blue-500 data-[state=drag]:bg-foreground-muted data-[state=hover]:bg-foreground-muted"
            />

            <SplitterPanel class="flex flex-1 flex-col">
                <CircuitTabs />
                <CircuitCanvas :state="currentCircuit" />
            </SplitterPanel>

            <SplitterResizeHandle
                class="w-px bg-border outline-none focus-visible:bg-blue-500 data-[state=drag]:bg-foreground-muted data-[state=hover]:bg-foreground-muted"
            />

            <SplitterPanel
                class="bg-panel-dark"
                :min-size="toPercentage(60 * 4)"
                :default-size="toPercentage(rightWidth)"
                :max-size="Math.min(toPercentage(96 * 4), 50)"
                @resize="rightWidth = fromPercentage($event)"
            >
                <Properties />
            </SplitterPanel>
        </SplitterGroup>

        <div class="flex h-6 items-center border-t bg-panel-light px-4 text-xs">
            <button
                @click="
                    if (theme == 'dark') {
                        theme = 'light';
                    } else {
                        theme = 'dark';
                    }
                "
            >
                <component
                    :is="theme === 'light' ? Moon : Sun"
                    :size="12"
                    absolute-stroke-width
                    class="text-foreground-muted"
                />
            </button>

            <SliderRoot
                v-model="sliderValue"
                :min="-5"
                :step="1"
                :max="10"
                class="relative ml-auto flex w-52 touch-none items-center select-none"
            >
                <SliderTrack class="relative h-0.5 flex-1 bg-foreground-muted">
                    <div
                        class="absolute top-1/2 left-1/3 h-2 w-0.5 -translate-y-1/2 bg-foreground-muted"
                    ></div>
                </SliderTrack>
                <SliderThumb
                    class="block h-2 w-3 bg-foreground-muted hover:bg-foreground"
                    aria-label="Zoom level"
                />
            </SliderRoot>
            <span class="w-12 text-right text-foreground-muted">
                {{ (scale * 100).toFixed(0) }}%
            </span>
        </div>
    </div>
</template>
