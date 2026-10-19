<script setup lang="ts">
import { ref } from "vue";

/**
 * Labeled slider which emits a `commit` event on change while updating
 * value immediately in the preview.
 */
const props = defineProps<{ label: string; value: number; min: number; max: number }>();
const emit = defineEmits<{ commit: [value: number] }>();

const draft = ref<number | null>(null);

function commit() {
    if (draft.value === null) return;
    const value = draft.value;
    draft.value = null;
    emit("commit", value);
}
</script>

<template>
    <label class="block">
        <span class="flex justify-between">
            <span class="font-medium">{{ label }}</span>
            <span>{{ draft ?? props.value }}</span>
        </span>
        <input
            type="range"
            :value="draft ?? props.value"
            :min="min"
            :step="1"
            :max="max"
            class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-primary"
            @input="draft = ($event.target as HTMLInputElement).valueAsNumber"
            @change="commit"
        />
    </label>
</template>
