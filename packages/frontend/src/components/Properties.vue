<script setup lang="ts">
import { Handedness, Orientation } from "circuitsim-glue";
import { computed, onMounted, onUnmounted, ref, useId, watch, watchEffect } from "vue";
import { toast } from "vue-sonner";

import { circuits, currentSubcircuit, updateComponents } from "@/lib/store/circuit";
import { AccordionContent, AccordionHeader, AccordionItem, AccordionRoot } from "./ui/accordion";
import SegmentedControl from "./ui/SegmentedControl.vue";
import { isEditableTarget } from "@/lib/keyboard";
import { settings } from "@/lib/store/settings";
import {
    componentSelection,
    placingComponent,
    placingHandedness,
    placingOrientation,
} from "@/lib/store/view";
import { CircuitComponent, componentPropertiesMap } from "@/lib/types";

const nameReset = ref(0);
const subcircuitName = computed({
    get() {
        return currentSubcircuit.value.name;
    },
    set(name) {
        const current = currentSubcircuit.value;
        if (name === current.name) return;

        if (name === "") {
            toast.error("Subcircuit name is required!");
            nameReset.value++;
            return;
        } else if (
            [...circuits.value.values()].some(
                (subcircuit) =>
                    subcircuit.frontendId !== current.frontendId && subcircuit.name === name,
            )
        ) {
            toast.error("A subcircuit with this name already exists!");
            nameReset.value++;
            return;
        }
        current.name = name;
    },
});

// FIXME: These two consts is redundant and should be removed.
const orientations = [
    { label: "N", name: "North", value: "North" },
    { label: "E", name: "East", value: "East" },
    { label: "S", name: "South", value: "South" },
    { label: "W", name: "West", value: "West" },
] as const satisfies { label: string; name: string; value: Orientation }[];
const handednesses = [
    { label: "TOP/LEFT", name: "Top/Left", value: "TopLeft" },
    { label: "BOTTOM/RIGHT", name: "Bottom/Right", value: "DownRight" },
] as const satisfies { label: string; name: string; value: Handedness }[];

const labelOrientationId = useId();
const orientationId = useId();
const handednessId = useId();

const selectedComponents = computed<CircuitComponent[]>(() =>
    [...componentSelection.value].map((id) => currentSubcircuit.value.components.get(id)!),
);
// Get properties defined by all selected components.
const selectedProperties = computed(() => {
    if (selectedComponents.value.length == 0) return new Set<string>();
    return selectedComponents.value
        .map((c) => new Set(getComponentProps(c)))
        .reduce((acc, cv) => acc.intersection(cv));
});

const sections = ["global", "circuit", "component"] as const;

/**
 * Gets a value from all selected components, returning it if all components have the same value.
 * Otherwise, this returns the fallback value.
 */
function getFromSelected<T>(query: (c: CircuitComponent) => T, fallback: T) {
    let queries = selectedComponents.value.map((c) => query(c));
    return queries.every((t) => queries[0] == t) ? queries[0] : fallback;
}
/** Updates all selected components with the specified updates. */
function updateAllSelected(updates: Partial<CircuitComponent>) {
    updateComponents(
        selectedComponents.value.map((c) => c.frontendId),
        updates,
    );
}
/** Gets all defined properties for a given component. */
function getComponentProps(component: CircuitComponent) {
    // FIXME: Please use an enum here instead of string[]
    return componentPropertiesMap[component.type.toLowerCase()] ?? [];
}

// TODO: Allow null or "invalid state" cases
//    Should show up when not all inputs are the same
const labelInput = computed({
    get: () => getFromSelected((c) => c.label, ""),
    set: (label) => updateAllSelected({ label }),
});
const bitsizeInput = ref(1);
const selsizeInput = ref(1);
const nInputsInput = ref(1);
watchEffect(() => {
    bitsizeInput.value = getFromSelected((c) => c.bitsize, 1);
    selsizeInput.value = getFromSelected((c) => c.selsize, 1);
    nInputsInput.value = getFromSelected((c) => c.inputs, 1);
});

// Mixed values across the selection show no active option.
const labelOrientationValue = computed(() =>
    getFromSelected<Orientation | undefined>((c) => c.labelOrientation, undefined),
);
const orientationValue = computed(() =>
    getFromSelected<Orientation | undefined>((c) => c.orientation, undefined),
);
const handednessValue = computed(() =>
    getFromSelected<Handedness | undefined>((c) => c.handedness, undefined),
);

function fitToBitsize(bin: string) {
    return bin.padStart(bitsizeInput.value, "0").slice(-bitsizeInput.value);
}

const constantError = ref("");
watch(
    () => [...componentSelection.value].join(),
    () => (constantError.value = ""),
);
const constantValue = {
    dec: computed({
        get: () => {
            let bin = getFromSelected((c) => c.constantValue, "0");
            // BigInt keeps values above 53 bits exact.
            return /^[01]+$/.test(bin) ? BigInt("0b" + bin).toString() : "?";
        },
        set: (value: string) => {
            constantError.value = "";
            if (!/^\d+$/.test(value)) {
                constantError.value = "Enter a non-negative integer";
                return;
            }

            updateAllSelected({ constantValue: fitToBitsize(BigInt(value).toString(2)) });
        },
    }),
    bin: computed({
        get: () => getFromSelected((c) => c.constantValue, "0"),
        set: (value: string) => {
            constantError.value = "";
            if (!/^[01]+$/.test(value)) {
                constantError.value = "Binary digits only (0 and 1)";
                return;
            }

            updateAllSelected({ constantValue: fitToBitsize(value) });
        },
    }),
};

const arrowOrientations: Record<string, Orientation> = {
    ArrowUp: "North",
    ArrowLeft: "West",
    ArrowRight: "East",
    ArrowDown: "South",
};

function handleKeyDown(e: KeyboardEvent) {
    if (isEditableTarget(e.target) || e.metaKey || e.ctrlKey || e.altKey) return;

    const orientation = arrowOrientations[e.key];
    const isHandednessKey = e.key.toLowerCase() == "h";

    if (placingComponent.value) {
        if (orientation) {
            e.preventDefault();
            placingOrientation.value = orientation;
        }
        if (isHandednessKey) {
            placingHandedness.value =
                placingHandedness.value == "TopLeft" ? "DownRight" : "TopLeft";
        }
    } else {
        if (orientation && selectedProperties.value.has("orientation")) {
            e.preventDefault();
            updateAllSelected({ orientation });
        }
        if (isHandednessKey && selectedProperties.value.has("handedness")) {
            if (selectedComponents.value[0].handedness == "TopLeft") {
                updateAllSelected({ handedness: "DownRight" });
            } else {
                updateAllSelected({ handedness: "TopLeft" });
            }
        }
    }
}

onMounted(() => document.addEventListener("keydown", handleKeyDown));
onUnmounted(() => document.removeEventListener("keydown", handleKeyDown));
</script>

<template>
    <h2 class="border-b bg-panel-light px-4 py-3 text-sm font-semibold text-foreground-highlight">
        Properties
    </h2>

    <AccordionRoot :default-value="sections.slice()">
        <AccordionItem value="global">
            <AccordionHeader> Global </AccordionHeader>

            <AccordionContent class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between">
                        <span class="font-medium">Global Bitsize</span>
                        <span>{{ settings.globalBitsize }}</span>
                    </span>
                    <input
                        v-model.number="settings.globalBitsize"
                        type="range"
                        min="1"
                        step="1"
                        max="16"
                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500"
                    />
                </label>
            </AccordionContent>
        </AccordionItem>

        <AccordionItem value="circuit">
            <AccordionHeader> {{ currentSubcircuit.name }} </AccordionHeader>

            <AccordionContent class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="font-medium">Name</span>
                    <input
                        :key="nameReset"
                        v-model.lazy.trim="subcircuitName"
                        type="text"
                        class="mt-1 block w-full border bg-panel-light px-1 py-1"
                    />
                </label>
            </AccordionContent>
        </AccordionItem>

        <AccordionItem
            v-if="selectedComponents.length != 0 && selectedComponents[0] != undefined"
            value="component"
        >
            <AccordionHeader>
                {{
                    selectedComponents.length > 1
                        ? "Component Group"
                        : selectedComponents[0].label != ""
                          ? selectedComponents[0].label
                          : selectedComponents[0].type.toUpperCase()
                }}
            </AccordionHeader>

            <!-- LABEL -->
            <AccordionContent
                v-if="
                    selectedComponents.length == 1 &&
                    getComponentProps(selectedComponents[0]).includes('label')
                "
                class="space-y-3 px-4 py-3 text-xs"
            >
                <label class="block">
                    <span class="font-medium">Label</span>
                    <input
                        v-model.lazy.trim="labelInput"
                        type="text"
                        placeholder="Enter label..."
                        class="mt-1 block w-full border bg-panel-light px-1 py-1 placeholder:text-foreground-muted"
                    />
                </label>

                <div>
                    <span :id="labelOrientationId" class="font-medium">Label Orientation</span>
                    <SegmentedControl
                        :options="orientations"
                        :value="labelOrientationValue"
                        :labelledby="labelOrientationId"
                        @update:value="updateAllSelected({ labelOrientation: $event })"
                    />
                </div>
            </AccordionContent>

            <!-- BITSIZE -->
            <AccordionContent v-if="selectedProperties.has('bitsize')" class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between">
                        <span class="font-medium">Bitsize</span>
                        <span>{{ bitsizeInput }}</span>
                    </span>
                    <input
                        v-model.number="bitsizeInput"
                        type="range"
                        :min="1"
                        :step="1"
                        :max="64"
                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500"
                        @change="updateAllSelected({ bitsize: bitsizeInput })"
                    />
                </label>
            </AccordionContent>

            <!-- SELSIZE -->
            <AccordionContent v-if="selectedProperties.has('selsize')" class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between">
                        <span class="font-medium">Selector Bits</span>
                        <span>{{ selsizeInput }}</span>
                    </span>
                    <input
                        v-model.number="selsizeInput"
                        type="range"
                        :min="1"
                        :step="1"
                        :max="6"
                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500"
                        @change="updateAllSelected({ selsize: selsizeInput })"
                    />
                </label>
            </AccordionContent>
            <!-- INPUTS -->
            <AccordionContent v-if="selectedProperties.has('inputs')" class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between">
                        <span class="font-medium">Num Inputs</span>
                        <span>{{ nInputsInput }}</span>
                    </span>
                    <input
                        v-model.number="nInputsInput"
                        type="range"
                        :min="1"
                        :step="1"
                        :max="8"
                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500"
                        @change="updateAllSelected({ inputs: nInputsInput })"
                    />
                </label>
            </AccordionContent>
            <!-- ORIENTATION -->
            <AccordionContent
                v-if="selectedProperties.has('orientation')"
                class="px-4 py-3 text-xs"
            >
                <span :id="orientationId" class="font-medium">Orientation</span>
                <SegmentedControl
                    :options="orientations"
                    :value="orientationValue"
                    :labelledby="orientationId"
                    @update:value="updateAllSelected({ orientation: $event })"
                />
            </AccordionContent>

            <!-- HANDEDNESS -->
            <AccordionContent v-if="selectedProperties.has('handedness')" class="px-4 py-3 text-xs">
                <span :id="handednessId" class="font-medium">Handedness</span>
                <SegmentedControl
                    :options="handednesses"
                    :value="handednessValue"
                    :labelledby="handednessId"
                    @update:value="updateAllSelected({ handedness: $event })"
                />
            </AccordionContent>
            <!-- CONSTANT VALUE -->
            <AccordionContent
                v-if="selectedProperties.has('constantValue')"
                class="px-4 py-3 text-xs"
            >
                <fieldset class="space-y-3">
                    <legend class="font-medium">Value</legend>

                    <label class="block">
                        <span class="font-medium">Decimal</span>
                        <input
                            v-model.lazy.trim="constantValue.dec.value"
                            type="text"
                            inputmode="numeric"
                            class="mt-1 block w-full border bg-panel-light px-1 py-1 font-mono"
                        />
                    </label>
                    <label class="block">
                        <span class="font-medium">Binary</span>
                        <input
                            v-model.lazy.trim="constantValue.bin.value"
                            type="text"
                            class="mt-1 block w-full border bg-panel-light px-1 py-1 font-mono"
                        />
                    </label>
                    <p v-if="constantError" class="text-xs text-red-500">
                        {{ constantError }}
                    </p>
                </fieldset>
            </AccordionContent>
        </AccordionItem>
    </AccordionRoot>
</template>
