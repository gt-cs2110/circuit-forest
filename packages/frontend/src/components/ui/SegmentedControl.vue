<script setup lang="ts" generic="T extends string">
defineProps<{
    options: readonly { label: string; name: string; value: T }[];
    value?: T;
    labelledby?: string;
    columns?: number;
}>();
const emit = defineEmits<{ "update:value": [value: T] }>();
</script>

<template>
    <div
        role="group"
        :aria-labelledby="labelledby"
        class="mt-2 overflow-hidden rounded border"
        :class="columns ? 'grid' : 'flex'"
        :style="columns ? { gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))` } : undefined"
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
