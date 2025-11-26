<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, useTemplateRef } from "vue";

const childWidth = ref(0);
const parentWidth = ref(0);
const scrollAmount = ref(0);
const isScrollable = computed(() => childWidth.value > parentWidth.value);
const isDragging = ref(false);
const dragStartX = ref(0);
const dragStartScrollLeft = ref(0);

const parent = useTemplateRef("parent");
const child = useTemplateRef("child");

let observer: ResizeObserver;

onMounted(() => {
    if (!parent.value || !child.value) return;

    childWidth.value = child.value.scrollWidth;
    parentWidth.value = parent.value.clientWidth;

    observer = new ResizeObserver(() => {
        if (!parent.value || !child.value) return;
        childWidth.value = child.value.scrollWidth;
        parentWidth.value = parent.value.clientWidth;
    });

    observer.observe(parent.value);
    observer.observe(child.value);

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
});

onUnmounted(() => {
    if (observer) {
        observer.disconnect();
    }

    document.removeEventListener("mousemove", handleMouseMove);
    document.removeEventListener("mouseup", handleMouseUp);

    clearTimeout(timeoutId);
});

let timeoutId: ReturnType<typeof setTimeout> | null = null;

function handleWheel(e: WheelEvent) {
    if (!e.deltaY || !parent.value) {
        return;
    }
    e.preventDefault();
    parent.value.scrollLeft += e.deltaY;
}

function scrollChanged(e: Event) {
    const target = e.currentTarget as HTMLElement;
    scrollAmount.value = target.scrollLeft;
    parent.value.parentElement.dataset.active = "true";
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => {
        parent.value.parentElement.dataset.active = "false";
        timeoutId = null;
    }, 700);
}

function handleMouseDown(e: MouseEvent) {
    if (!parent.value) return;
    e.preventDefault();
    isDragging.value = true;
    dragStartX.value = e.clientX;
    dragStartScrollLeft.value = parent.value.scrollLeft;
}

function handleMouseMove(e: MouseEvent) {
    if (!isDragging.value || !parent.value) return;

    const deltaX = e.clientX - dragStartX.value;
    const scrollbarWidth = (parentWidth.value / childWidth.value) * parentWidth.value;
    const maxScrollLeft = childWidth.value - parentWidth.value;
    const maxScrollbarLeft = parentWidth.value - scrollbarWidth;

    const scrollDelta = (deltaX / maxScrollbarLeft) * maxScrollLeft;
    parent.value.scrollLeft = dragStartScrollLeft.value + scrollDelta;
}

function handleMouseUp() {
    isDragging.value = false;
}

function handleTrackClick(e: MouseEvent) {
    if (!parent.value || e.target !== e.currentTarget) return;

    const parentRect = parent.value.getBoundingClientRect();
    const clickX = e.clientX - parentRect.left;
    const scrollbarWidth = (parentWidth.value / childWidth.value) * parentWidth.value;

    const maxScrollLeft = childWidth.value - parentWidth.value;
    const maxScrollbarLeft = parentWidth.value - scrollbarWidth;
    const newScrollbarLeft = Math.max(0, Math.min(clickX - scrollbarWidth / 2, maxScrollbarLeft));

    parent.value.scrollLeft = (newScrollbarLeft / maxScrollbarLeft) * maxScrollLeft;

    handleMouseDown(e);
}

defineExpose({
    parent,
    scrollToEnd() {
        parent.value.scrollLeft = parent.value.scrollWidth;
    },
});
</script>

<template>
    <div class="group/scroll relative">
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
                :style="{
                    width: `max(${(parentWidth / childWidth) * 100}%, 2rem)`,
                    left: `${(scrollAmount / parent.scrollWidth) * 100}%`,
                }"
                @mousedown="handleMouseDown"
            ></div>
        </div>
    </div>
</template>
