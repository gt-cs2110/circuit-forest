<script setup lang="ts">
import { ChevronDown } from "lucide-vue-next";
import { AccordionHeader, AccordionTrigger } from "reka-ui";
</script>

<template>
    <AccordionHeader>
        <AccordionTrigger
            class="group flex w-full cursor-pointer items-center gap-2 border-b border-zinc-700 bg-zinc-800 px-4 py-2 text-left text-sm font-medium text-zinc-200"
        >
            <slot />
            <ChevronDown
                class="ml-auto transition-transform group-data-[state=closed]:-rotate-90"
                :size="16"
                absolute-stroke-width
            />
        </AccordionTrigger>
    </AccordionHeader>
</template>
