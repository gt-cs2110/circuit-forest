<script setup lang="ts">
import CircuitCanvas from "./components/CircuitCanvas.vue";
import Properties from "./components/Properties.vue";
import { currentCircuit } from "./lib/store";
import CircuitTabs from "./components/CircuitTabs.vue";
import ComponentSelector from "./components/ComponentSelector.vue";
</script>

<template>
    <div class="flex h-screen">
        <ComponentSelector />

        <div class="flex flex-1 flex-col">
            <CircuitTabs />
            <CircuitCanvas :state="currentCircuit" />
        </div>

        <Properties />
    </div>
</template>
