<script setup lang="ts" generic="T">
import { onBeforeUnmount, ref, watch } from "vue";

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

const draft = ref(props.value);
const dirty = ref(false);

watch(
    () => props.value,
    (value) => {
        if (!dirty.value) draft.value = value;
    },
);

function commit() {
    if (!dirty.value) return;
    dirty.value = false;
    emit("commit", draft.value.trim(), props.target);
    // update value based on what was actually stored after in case the commit fails or the value is trimmed, etc.
    draft.value = props.value;
}

onBeforeUnmount(commit);
</script>

<template>
    <input v-model="draft" type="text" @input="dirty = true" @change="commit" />
</template>
