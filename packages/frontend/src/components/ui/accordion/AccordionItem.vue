<script setup lang="ts">
import { AccordionItem } from "reka-ui";

defineProps<{ value: string }>();
</script>

<template>
    <AccordionItem :value="value">
        <slot />
    </AccordionItem>
</template>
