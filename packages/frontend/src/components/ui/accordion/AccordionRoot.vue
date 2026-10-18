<script setup lang="ts">
import { AccordionRoot } from "reka-ui";

defineProps<{ defaultValue: string[] }>();
</script>

<template>
    <AccordionRoot
        class="min-h-0 flex-1 overflow-y-auto"
        type="multiple"
        :default-value="defaultValue"
    >
        <slot />
    </AccordionRoot>
</template>
