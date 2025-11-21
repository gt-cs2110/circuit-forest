<script setup lang="ts">
import { circuits, currentCircuitId } from "@/lib/store";
</script>

<template>
    <div class="border-b-2 border-zinc-700 bg-zinc-900 text-sm focus:outline-none">
        <button
            v-for="[id, circuit] in circuits"
            :key="id"
            class="relative cursor-pointer border-r-2 border-zinc-700 px-4 py-3"
            :class="[
                currentCircuitId === id ? 'bg-zinc-800 text-white' : 'bg-zinc-900 text-zinc-400',
            ]"
            @click="currentCircuitId = id"
        >
            {{ circuit.subcircuit.name }}

            <div
                v-if="currentCircuitId === id"
                class="absolute inset-x-0 top-full h-1 bg-zinc-800"
            ></div>
        </button>
    </div>
</template>
