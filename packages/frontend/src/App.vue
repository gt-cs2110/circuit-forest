<script setup lang="ts">
import { ChevronDown } from "lucide-vue-next";

import CircuitCanvas from "./components/CircuitCanvas.vue";
import Properties from "./components/Properties.vue";

const categories = ["Wiring", "Gates", "Arithmetic", "Memory"];
</script>

<template>
    <div class="flex h-screen">
        <div
            class="flex w-72 shrink-0 flex-col border-r-2 border-zinc-700 bg-zinc-900 text-zinc-200"
        >
            <input
                type="search"
                class="appearance-none border-b-2 border-zinc-700 bg-zinc-800 px-4 py-3 text-sm text-zinc-200 placeholder:text-zinc-500 focus:outline-none"
                placeholder="Search..."
            />

            <div class="flex flex-1 flex-col gap-2 overflow-y-auto p-2">
                <div
                    v-for="(category, i) in categories"
                    :key="category"
                    class="h-max shrink-0 rounded-md bg-zinc-800"
                >
                    <button
                        class="w-full cursor-pointer rounded-t-md bg-zinc-700 p-2 text-left text-sm font-semibold text-white"
                    >
                        <ChevronDown class="inline h-4 w-4 align-middle" />
                        {{ category }}
                    </button>
                    <div class="grid grid-cols-3 gap-2 p-2">
                        <button
                            v-for="n in (((i + 1) * 13) % 9) + 3"
                            :key="n"
                            class="aspect-square cursor-pointer rounded-md border-2 border-zinc-500 bg-zinc-600 text-xs"
                        >
                            Item {{ n }}
                        </button>
                    </div>
                </div>
            </div>
        </div>

        <CircuitCanvas />

        <Properties />
    </div>
</template>
