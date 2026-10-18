<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, useTemplateRef } from "vue";

const MIN_THUMB_WIDTH = 32;

const childWidth = ref(0);
const parentWidth = ref(0);
const scrollAmount = ref(0);
const isScrollable = computed(() => childWidth.value > parentWidth.value);
const isActive = ref(false);
const isDragging = ref(false);
const dragStartX = ref(0);
const dragStartScrollLeft = ref(0);

const parent = useTemplateRef("parent");
const child = useTemplateRef("child");

const thumbWidth = computed(() =>
    Math.max((parentWidth.value / childWidth.value) * parentWidth.value, MIN_THUMB_WIDTH),
);
const maxScrollLeft = computed(() => childWidth.value - parentWidth.value);
const maxThumbLeft = computed(() => parentWidth.value - thumbWidth.value);
const thumbLeft = computed(() =>
    maxScrollLeft.value > 0 ? (scrollAmount.value / maxScrollLeft.value) * maxThumbLeft.value : 0,
);

let observer: ResizeObserver | undefined;
let timeoutId: ReturnType<typeof setTimeout> | undefined;

function measure() {
    if (!parent.value || !child.value) return;
    childWidth.value = child.value.scrollWidth;
    parentWidth.value = parent.value.clientWidth;
    scrollAmount.value = parent.value.scrollLeft;
}

onMounted(() => {
    if (!parent.value || !child.value) return;

    measure();
    observer = new ResizeObserver(measure);
    observer.observe(parent.value);
    observer.observe(child.value);

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
});

onUnmounted(() => {
    observer?.disconnect();

    document.removeEventListener("mousemove", handleMouseMove);
    document.removeEventListener("mouseup", handleMouseUp);

    clearTimeout(timeoutId);
});

function handleWheel(e: WheelEvent) {
    if (!parent.value || Math.abs(e.deltaX) >= Math.abs(e.deltaY)) return;
    e.preventDefault();
    parent.value.scrollLeft += e.deltaY;
}

function scrollChanged() {
    if (!parent.value) return;
    scrollAmount.value = parent.value.scrollLeft;
    isActive.value = true;
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => (isActive.value = false), 700);
}

function handleMouseDown(e: MouseEvent) {
    if (!parent.value) return;
    e.preventDefault();
    isDragging.value = true;
    dragStartX.value = e.clientX;
    dragStartScrollLeft.value = parent.value.scrollLeft;
}

function handleMouseMove(e: MouseEvent) {
    if (!isDragging.value || !parent.value || maxThumbLeft.value <= 0) return;

    const deltaX = e.clientX - dragStartX.value;
    const scrollDelta = (deltaX / maxThumbLeft.value) * maxScrollLeft.value;
    parent.value.scrollLeft = dragStartScrollLeft.value + scrollDelta;
}

function handleMouseUp() {
    isDragging.value = false;
}

function handleTrackClick(e: MouseEvent) {
    if (!parent.value || e.target !== e.currentTarget || maxThumbLeft.value <= 0) return;

    const clickX = e.clientX - parent.value.getBoundingClientRect().left;
    const newThumbLeft = Math.max(0, Math.min(clickX - thumbWidth.value / 2, maxThumbLeft.value));
    parent.value.scrollLeft = (newThumbLeft / maxThumbLeft.value) * maxScrollLeft.value;

    handleMouseDown(e);
}

defineExpose({
    parent,
    scrollToEnd() {
        if (parent.value) parent.value.scrollLeft = parent.value.scrollWidth;
    },
});
</script>

<template>
    <div class="group/scroll relative" :data-active="isActive">
        <div
            ref="parent"
            class="flex overflow-x-auto overflow-y-hidden"
            @wheel="handleWheel"
            @scroll="scrollChanged"
        >
            <div ref="child" class="w-max shrink-0 grow">
                <slot />
            </div>
        </div>

        <div
            v-if="isScrollable"
            class="group/scrollbar absolute inset-x-0 bottom-0 h-1 cursor-default bg-transparent"
            @mousedown.self="handleTrackClick"
        >
            <div
                class="absolute top-0 h-full bg-transparent opacity-25 transition-colors group-hover/scroll:bg-foreground group-active/scrollbar:bg-foreground group-active/scrollbar:opacity-50 group-data-[active=true]/scroll:bg-foreground hover:opacity-35 active:bg-foreground active:opacity-50"
                :style="{ width: `${thumbWidth}px`, left: `${thumbLeft}px` }"
                @mousedown="handleMouseDown"
            ></div>
        </div>
    </div>
</template>
