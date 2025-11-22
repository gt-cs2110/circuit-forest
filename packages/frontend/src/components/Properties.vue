<script setup lang="ts">
import { computed } from "vue";
import { currentCircuit, selectedComponentId, settings } from "@/lib/store";
import { componentMap } from "./circuitry";

const selectedComponent = computed(() =>
    currentCircuit.value.subcircuit.components.get(selectedComponentId.value),
);
</script>

<template>
    <h2 class="border-b-2 border-zinc-700 bg-zinc-800 px-4 py-3 text-sm font-semibold text-white">
        Properties
    </h2>

    <label class="block px-4 py-3 text-xs">
        <span class="flex justify-between">
            <span class="font-semibold">Global Bitsize</span>
            <span>{{ settings.globalBitsize }}</span>
        </span>
        <input
            v-model="settings.globalBitsize"
            type="range"
            min="1"
            step="1"
            max="16"
            class="my-3 block h-1 w-full appearance-none rounded bg-zinc-700 accent-blue-500"
        />
    </label>

    <hr class="mx-4 border-zinc-700" />

    <label class="block px-4 py-3 text-xs">
        <span class="flex justify-between">
            <span class="font-semibold">Zoom</span>
            <span>{{ (Math.pow(1.2, settings.scaleLevel) * 100).toFixed(0) }}%</span>
        </span>
        <input
            v-model="settings.scaleLevel"
            type="range"
            min="-5"
            step="1"
            max="10"
            class="my-3 block h-1 w-full appearance-none rounded bg-zinc-700 accent-blue-500"
        />
    </label>

    <template v-if="selectedComponentId !== null">
        <h3
            class="border-y-2 border-zinc-700 bg-zinc-800 px-4 py-3 text-sm font-semibold text-white"
        >
            {{ componentMap[selectedComponent.type].displayName }}
        </h3>

        <label class="block px-4 py-3 text-xs">
            <span class="flex justify-between">
                <span class="font-semibold"> Bitsize</span>
                <span>{{ selectedComponent.bitsize }}</span>
            </span>
            <input
                v-model="selectedComponent.bitsize"
                type="range"
                min="1"
                step="1"
                max="16"
                class="my-3 block h-1 w-full appearance-none rounded bg-zinc-700 accent-blue-500"
            />
        </label>
    </template>
</template>
