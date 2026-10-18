<script setup lang="ts" generic="T extends string">
defineProps<{
    options: readonly { label: string; name: string; value: T }[];
    value?: T;
    labelledby?: string;
}>();
const emit = defineEmits<{ "update:value": [value: T] }>();
</script>

<template>
    <div
        role="group"
        :aria-labelledby="labelledby"
        class="mt-2 flex overflow-hidden rounded border"
    >
        <button
            v-for="option in options"
            :key="option.value"
            type="button"
            :aria-label="option.name"
            :aria-pressed="value === option.value"
            :title="option.name"
            class="flex-1 cursor-pointer px-3 py-2 transition-colors"
            :class="
                value === option.value
                    ? 'bg-blue-500 text-white'
                    : 'bg-panel-light hover:bg-panel-dark'
            "
            @click="emit('update:value', option.value)"
        >
            {{ option.label }}
        </button>
    </div>
</template>
