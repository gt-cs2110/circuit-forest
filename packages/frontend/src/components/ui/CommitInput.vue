<script setup lang="ts" generic="T">
import { onBeforeUnmount, ref } from "vue";

/**
 * Input that only "commits" its value on `change`. See EditTarget in Properties.vue for the application.
 * This solution is kind of contribed but seemed necessary.
 *
 * Each commit is emitted with the `target` the input had when it was edited, and pending edits are
 * committed **when the input unmounts**. This should be keyed by target so that the edit is committed
 * to the right place even if the target changes before the input is blurred (which happens when
 * clicking on something that changes the target).
 */
const props = defineProps<{ value: string; target: T }>();
const emit = defineEmits<{ commit: [value: string, target: T] }>();

const draft = ref<string | null>(null);

function commit() {
    if (draft.value === null) return;
    const value = draft.value.trim();
    draft.value = null;
    emit("commit", value, props.target);
}

onBeforeUnmount(commit);
</script>

<template>
    <input
        type="text"
        :value="draft ?? props.value"
        @input="draft = ($event.target as HTMLInputElement).value"
        @change="commit"
    />
</template>
