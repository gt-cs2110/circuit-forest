<script setup lang="ts">
import { AccordionContent } from "reka-ui";
</script>

<template>
    <AccordionContent
        class="border-b border-zinc-700 data-[state=closed]:animate-grow-out data-[state=open]:animate-grow-in"
    >
        <slot />
    </AccordionContent>
</template>
